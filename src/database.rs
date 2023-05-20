mod connection_options;
mod error;
mod jieba_tokenizer;
mod sqlite3_fts5;

pub use error::DatabaseError;

use diesel::{
    dsl::exists,
    r2d2::{ConnectionManager, Pool},
    select, sql_query, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{
    fs,
    path::{Path, PathBuf},
};

use connection_options::ConnectionOptions;

use crate::{
    models::Folder,
    new_id,
    sync::{ForSyncSerializer, SerializeForSync},
    AbbrNote, DateTimeTimestamp, DeletedItem, ModelType, NewDeletedItem, NewSetting, NewSyncItem,
    Note, NoteFts, NoteTag, NoteTagId, Resource, Setting, Status, SyncItem, Tag,
};

pub type DatabaseResult<T> = Result<T, DatabaseError>;

// use diesel::prelude::sql_function;
// use diesel::sql_types::Text;
// how to declare a sql_function?
// sql_function! {
//     fn highlight(table: Text, column: Integer, before: Text, after: Text) -> Text;
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateSource {
    RemoteSync,
    LocalEdit,
}

impl UpdateSource {
    pub fn is_local_edit(&self) -> bool {
        match self {
            UpdateSource::RemoteSync => false,
            UpdateSource::LocalEdit => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchBodyOption {
    Highlight,
    Snippet { max_tokens: u8 },
}

#[derive(Debug)]
pub struct Database {
    connection_pool: Pool<ConnectionManager<SqliteConnection>>,
    _path: PathBuf,
    _filename: String,
    resource_path: PathBuf,
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

impl Database {
    pub fn new_with_filename(
        data_dir: &Path,
        resource_path: &Path,
        filename: &str,
    ) -> DatabaseResult<Database> {
        fs::create_dir_all(data_dir)?;
        let database_url = data_dir.join(filename);
        let database_url = database_url
            .to_str()
            .expect("database url error")
            .to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let connection_pool = Pool::builder()
            .connection_customizer(Box::<ConnectionOptions>::default())
            .max_size(16)
            .build(manager)?;

        let db = Self {
            connection_pool,
            _path: data_dir.into(),
            _filename: filename.into(),
            resource_path: resource_path.to_path_buf(),
        };
        db.init()?;
        Ok(db)
    }

    pub fn new(data_dir: &Path, resource_path: &Path) -> DatabaseResult<Database> {
        Database::new_with_filename(data_dir, resource_path, "database.sqlite")
    }

    fn init(&self) -> DatabaseResult<()> {
        let mut connection = self.connection_pool.get()?;
        connection.run_pending_migrations(MIGRATIONS).map_err(|e| {
            log::error!("Database migration failed: {}", e);
            DatabaseError::Migration(e)
        })?;
        diesel::sql_query("PRAGMA journal_mode = WAL").execute(&mut connection)?;
        Ok(())
    }
}

impl Database {
    pub fn insert_root_folder(&self, title: impl Into<String>) -> DatabaseResult<Folder> {
        let folder = Folder::new_root(title);
        self.replace_folder(&folder, UpdateSource::LocalEdit)?;
        Ok(folder)
    }

    pub fn insert_folder_with_parent(
        &self,
        title: impl Into<String>,
        parent_id: impl Into<String>,
    ) -> DatabaseResult<Folder> {
        let folder = Folder::new_with_parent(title, parent_id);
        self.replace_folder(&folder, UpdateSource::LocalEdit)?;
        Ok(folder)
    }

    pub fn replace_folder(
        &self,
        folder: &Folder,
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let folder = match update_source {
            UpdateSource::RemoteSync => folder.clone(),
            UpdateSource::LocalEdit => folder.updated(),
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        diesel::replace_into(folders::table)
            .values(&folder)
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Folder, folder.id.as_str(), update_source)?;
        Ok(())
    }

    pub fn load_folders(&self) -> DatabaseResult<Vec<Folder>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table
            .select(Folder::SELECTION)
            .order(folders::title.asc())
            .load(&mut conn)?)
    }

    pub fn load_folder(&self, id: &str) -> DatabaseResult<Folder> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table
            .filter(folders::id.eq(id))
            .select(Folder::SELECTION)
            .first(&mut conn)?)
    }

    pub fn load_subfolders(&self, id: &str) -> DatabaseResult<Vec<Folder>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table
            .select(Folder::SELECTION)
            .filter(folders::parent_id.eq(id))
            .load(&mut conn)?)
    }

    pub fn delete_folder(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        if update_source.is_local_edit() {
            self.delete_notes_by_folder_id(id)?;
            let subfolders = self.load_subfolders(id)?;
            for folder in subfolders {
                self.delete_folder(&folder.id, UpdateSource::LocalEdit)?;
            }
        }
        self.delete_sync_item(id)?;
        diesel::delete(folders::table)
            .filter(folders::id.eq(id))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.insert_deleted_item(ModelType::Folder, id)?;
        }
        Ok(())
    }

    pub fn folder_count(&self) -> DatabaseResult<i64> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table.count().get_result(&mut conn)?)
    }
}

impl Database {
    pub fn load_abbr_notes(&self, parent_id: Option<&str>) -> DatabaseResult<Vec<AbbrNote>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        let selection = (
            notes::id,
            notes::parent_id,
            notes::title,
            notes::user_created_time,
            notes::user_updated_time,
        );
        let query_stmt = notes::table
            .select(selection)
            .filter(notes::is_conflict.eq(false))
            .order(notes::user_updated_time.desc());
        Ok(match parent_id {
            Some(parent_id) => query_stmt
                .filter(notes::parent_id.eq(parent_id))
                .load(&mut conn),
            None => query_stmt.load(&mut conn),
        }?)
    }

    pub fn load_abbr_conflict_notes(&self) -> DatabaseResult<Vec<AbbrNote>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        Ok(notes::table
            .select((
                notes::id,
                notes::parent_id,
                notes::title,
                notes::user_created_time,
                notes::user_updated_time,
            ))
            .filter(notes::is_conflict.eq(true))
            .order(notes::user_updated_time.desc())
            .load(&mut conn)?)
    }

    pub fn conflict_note_exists(&self) -> DatabaseResult<bool> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        Ok(
            select(exists(notes::table.filter(notes::is_conflict.eq(true))))
                .get_result(&mut conn)?,
        )
    }

    pub fn load_note(&self, id: &str) -> DatabaseResult<Note> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        Ok(notes::table
            .filter(notes::id.eq(id))
            .select((
                notes::id,
                notes::parent_id,
                notes::title,
                notes::body,
                notes::created_time,
                notes::updated_time,
                notes::is_conflict,
                notes::latitude,
                notes::longitude,
                notes::altitude,
                notes::author,
                notes::source_url,
                notes::is_todo,
                notes::todo_due,
                notes::todo_completed,
                notes::source,
                notes::source_application,
                notes::application_data,
                notes::order,
                notes::user_created_time,
                notes::user_updated_time,
                notes::encryption_cipher_text,
                notes::encryption_applied,
                notes::markup_language,
                notes::is_shared,
                notes::share_id,
                notes::conflict_original_id,
                notes::master_key_id,
            ))
            .first(&mut conn)?)
    }

    pub fn insert_note_with_parent(
        &self,
        title: impl Into<String>,
        body: impl Into<String>,
        parent_id: impl Into<String>,
    ) -> DatabaseResult<Note> {
        let note = Note::new_with_parent(parent_id, title, body);
        self.replace_note(&note, UpdateSource::LocalEdit)?;
        Ok(note)
    }

    pub fn replace_note(&self, note: &Note, update_source: UpdateSource) -> DatabaseResult<()> {
        let note = match update_source {
            UpdateSource::RemoteSync => note.clone(),
            UpdateSource::LocalEdit => note.updated(),
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        let note_exist: bool = select(exists(notes::table.filter(notes::id.eq(note.id.as_str()))))
            .get_result(&mut conn)?;
        if note_exist {
            diesel::update(notes::table)
                .filter(notes::id.eq(note.id.as_str()))
                .set(&note)
                .execute(&mut conn)?;
        } else {
            diesel::insert_into(notes::table)
                .values(&note)
                .execute(&mut conn)?;
        }
        self.replace_sync_item(ModelType::Note, note.id.as_str(), update_source)?;
        Ok(())
    }

    pub fn update_note_body(&self, id: &str, body: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        let dt = DateTimeTimestamp::now();
        diesel::update(notes::table)
            .filter(notes::id.eq(id))
            .set((
                notes::body.eq(body),
                notes::updated_time.eq(dt),
                notes::user_updated_time.eq(dt),
            ))
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Note, id, UpdateSource::LocalEdit)?;
        Ok(())
    }

    pub fn update_note_title(&self, id: &str, title: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        let dt = DateTimeTimestamp::now();
        diesel::update(notes::table)
            .filter(notes::id.eq(id))
            .set((
                notes::title.eq(title),
                notes::updated_time.eq(dt),
                notes::user_updated_time.eq(dt),
            ))
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Note, id, UpdateSource::LocalEdit)?;
        Ok(())
    }

    pub fn delete_note(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        self.delete_sync_item(id)?;
        diesel::delete(notes::table)
            .filter(notes::id.eq(id))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.delete_note_tag_by_note_ids(&[id], UpdateSource::LocalEdit)?;
            self.insert_deleted_item(ModelType::Note, id)?;
        }
        Ok(())
    }

    pub fn delete_notes(&self, notes_id: &[&str]) -> DatabaseResult<()> {
        if notes_id.is_empty() {
            return Ok(());
        }
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        self.delete_sync_items(notes_id)?;
        diesel::delete(notes::table)
            .filter(notes::id.eq_any(notes_id))
            .execute(&mut conn)?;
        self.delete_note_tag_by_note_ids(notes_id, UpdateSource::LocalEdit)?;
        self.insert_deleted_items(ModelType::Note, notes_id)?;
        Ok(())
    }

    fn delete_notes_by_folder_id(&self, folder_id: &str) -> DatabaseResult<()> {
        let notes = self.load_abbr_notes(Some(folder_id))?;
        let note_ids: Vec<&str> = notes.iter().map(|n| n.id.as_str()).collect();
        self.delete_notes(&note_ids)?;
        Ok(())
    }

    pub fn note_count(&self) -> DatabaseResult<i64> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        Ok(notes::table.count().get_result(&mut conn)?)
    }

    pub fn rebuild_fts(&self) -> DatabaseResult<()> {
        log::info!("rebuilding notes_fts");
        let mut conn = self.connection_pool.get()?;
        sql_query("INSERT INTO notes_fts(notes_fts) VALUES('rebuild')").execute(&mut conn)?;
        Ok(())
    }

    pub fn search_notes(
        &self,
        search_term: &str,
        option: Option<SearchBodyOption>,
    ) -> DatabaseResult<Vec<NoteFts>> {
        let mut conn = self.connection_pool.get()?;
        use crate::notes_fts;
        if let Some(option) = option {
            let auxiliary_function = match option {
                SearchBodyOption::Highlight => {
                    "highlight(`notes_fts`, 1, '<b>', '</b>')".to_string()
                }
                SearchBodyOption::Snippet { max_tokens } => {
                    assert!(max_tokens <= 64);
                    format!("snippet(`notes_fts`, 1, '<b>', '</b>', '…', {max_tokens})")
                }
            };
            let query = format!("SELECT `notes_fts`.`id`, highlight(`notes_fts`, 0, '<b>', '</b>') as `title`, {auxiliary_function} as `body` FROM `notes_fts` WHERE notes_fts MATCH '{search_term}' ORDER BY bm25(notes_fts);");
            Ok(sql_query(query).load(&mut conn)?)
        } else {
            // let body = highlight(notes_fts_table, 2, "<b>", "</b>");
            Ok(notes_fts::table
                .select((notes_fts::id, notes_fts::title, notes_fts::body))
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>(&format!(
                    "notes_fts MATCH '{search_term}' ORDER BY bm25(notes_fts)"
                )))
                .load(&mut conn)?)
        }
    }
}

impl Database {
    fn replace_sync_item(
        &self,
        item_type: ModelType,
        item_id: &str,
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        let sync_item: Option<SyncItem> = sync_items::table
            .filter(sync_items::item_id.eq(item_id))
            .select((
                sync_items::id,
                sync_items::sync_target,
                sync_items::sync_time,
                sync_items::update_time,
                sync_items::item_type,
                sync_items::item_id,
            ))
            .first(&mut conn)
            .ok();
        match sync_item {
            Some(mut sync_item) => {
                match update_source {
                    UpdateSource::RemoteSync => sync_item.sync_time = DateTimeTimestamp::now(),
                    UpdateSource::LocalEdit => {
                        sync_item.update_time = DateTimeTimestamp::now();
                        // workaround: When the program runs fast enough, update_time may equal sync_time. May not happen in a real environment
                        if sync_item.update_time <= sync_item.sync_time {
                            sync_item.update_time = DateTimeTimestamp::from_timestamp_millis(
                                sync_item.sync_time.timestamp_millis() + 1,
                            );
                        }
                    }
                }
                diesel::replace_into(sync_items::table)
                    .values(&sync_item)
                    .execute(&mut conn)?;
            }
            None => {
                let sync_item = NewSyncItem::new(item_type, item_id, update_source);
                diesel::insert_into(sync_items::table)
                    .values(&sync_item)
                    .execute(&mut conn)?;
            }
        };
        Ok(())
    }

    pub fn load_sync_item(&self, item_id: &str) -> DatabaseResult<SyncItem> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        Ok(sync_items::table
            .filter(sync_items::item_id.eq(item_id))
            .select((
                sync_items::id,
                sync_items::sync_target,
                sync_items::sync_time,
                sync_items::update_time,
                sync_items::item_type,
                sync_items::item_id,
            ))
            .first(&mut conn)?)
    }

    pub fn set_sync_item_up_to_data(&self, item_id: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        diesel::update(sync_items::table)
            .filter(sync_items::item_id.eq(item_id))
            .set(sync_items::sync_time.eq(DateTimeTimestamp::now()))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn load_sync_items(&self, item_ids: &[&str]) -> DatabaseResult<Vec<SyncItem>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        Ok(sync_items::table
            .filter(sync_items::item_id.eq_any(item_ids))
            .select((
                sync_items::id,
                sync_items::sync_target,
                sync_items::sync_time,
                sync_items::update_time,
                sync_items::item_type,
                sync_items::item_id,
            ))
            .load(&mut conn)?)
    }

    pub fn load_all_sync_items(&self) -> DatabaseResult<Vec<SyncItem>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        Ok(sync_items::table
            .select((
                sync_items::id,
                sync_items::sync_target,
                sync_items::sync_time,
                sync_items::update_time,
                sync_items::item_type,
                sync_items::item_id,
            ))
            .load(&mut conn)?)
    }

    pub fn load_need_upload_sync_items(&self) -> DatabaseResult<Vec<SyncItem>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        Ok(sync_items::table
            .filter(sync_items::sync_time.lt(sync_items::update_time))
            .select((
                sync_items::id,
                sync_items::sync_target,
                sync_items::sync_time,
                sync_items::update_time,
                sync_items::item_type,
                sync_items::item_id,
            ))
            .load(&mut conn)?)
    }

    pub fn load_sync_item_content(
        &self,
        sync_item: &SyncItem,
    ) -> DatabaseResult<ForSyncSerializer> {
        match sync_item.item_type {
            ModelType::Note => self
                .load_note(&sync_item.item_id)
                .map(|note| note.serialize()),
            ModelType::Folder => self
                .load_folder(&sync_item.item_id)
                .map(|folder| folder.serialize()),
            ModelType::Resource => self
                .load_resource(&sync_item.item_id)
                .map(|x| x.serialize()),
            ModelType::Tag => self.load_tag(&sync_item.item_id).map(|x| x.serialize()),
            ModelType::NoteTag => self
                .load_note_tag(&sync_item.item_id)
                .map(|x| x.serialize()),
            ModelType::Unsupported => {
                panic!("cannot load unsupported type");
            }
        }
    }

    pub fn delete_sync_item(&self, item_id: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        diesel::delete(sync_items::table)
            .filter(sync_items::item_id.eq(item_id))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn delete_sync_items(&self, item_id: &[&str]) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;
        diesel::delete(sync_items::table)
            .filter(sync_items::item_id.eq_any(item_id))
            .execute(&mut conn)?;
        Ok(())
    }
}

impl Database {
    fn insert_deleted_item(&self, item_type: ModelType, item_id: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::deleted_items;
        let deleted_item = NewDeletedItem::new(item_type, item_id);
        diesel::insert_into(deleted_items::table)
            .values(&deleted_item)
            .execute(&mut conn)?;
        Ok(())
    }

    fn insert_deleted_items(&self, item_type: ModelType, item_ids: &[&str]) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::deleted_items;
        let deleted_items = NewDeletedItem::new_items(item_type, item_ids);
        diesel::insert_into(deleted_items::table)
            .values(&deleted_items)
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn delete_deleted_item(&self, deleted_item: DeletedItem) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::deleted_items;
        diesel::delete(deleted_items::table)
            .filter(deleted_items::id.eq(deleted_item.id))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn load_deleted_items(&self) -> DatabaseResult<Vec<DeletedItem>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::deleted_items;
        Ok(deleted_items::table
            .select((
                deleted_items::id,
                deleted_items::item_type,
                deleted_items::item_id,
                deleted_items::deleted_time,
            ))
            .load(&mut conn)?)
    }
}

impl Database {
    pub fn get_setting_value(&self, key: &str) -> DatabaseResult<Option<Setting>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::settings;
        Ok(settings::table
            .filter(settings::key.eq(key))
            .select((settings::key, settings::value))
            .first(&mut conn)
            .optional()?)
    }

    pub fn replace_setting(&self, key: &str, value: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::settings;
        let new_setting = NewSetting::new(key, value);
        diesel::replace_into(settings::table)
            .values(&new_setting)
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn delete_setting(&self, key: &str) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::settings;
        diesel::delete(settings::table)
            .filter(settings::key.eq(key))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn get_client_id(&self) -> DatabaseResult<String> {
        let setting = self.get_setting_value(Setting::CLIENT_ID)?;
        match setting {
            Some(s) => Ok(s.value),
            None => {
                let id = new_id();
                self.replace_setting(Setting::CLIENT_ID, &id)?;
                Ok(id)
            }
        }
    }
}

impl Database {
    pub fn load_tag(&self, id: &str) -> DatabaseResult<Tag> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        Ok(tags::table
            .filter(tags::id.eq(id))
            .select(Tag::SELECTION)
            .first(&mut conn)?)
    }

    pub fn load_all_tags(&self) -> DatabaseResult<Vec<Tag>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        Ok(tags::table.select(Tag::SELECTION).load(&mut conn)?)
    }

    pub fn load_tags(&self, ids: &[&str]) -> DatabaseResult<Vec<Tag>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        Ok(tags::table
            .filter(tags::id.eq_any(ids))
            .select(Tag::SELECTION)
            .load(&mut conn)?)
    }

    pub fn replace_tag(&self, tag: &Tag, update_source: UpdateSource) -> DatabaseResult<()> {
        let tag = match update_source {
            UpdateSource::RemoteSync => tag.clone(),
            UpdateSource::LocalEdit => tag.updated(),
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        diesel::replace_into(tags::table)
            .values(&tag)
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Tag, tag.id.as_str(), update_source)?;
        Ok(())
    }

    pub fn delete_tag(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        self.delete_sync_item(id)?;
        diesel::delete(tags::table)
            .filter(tags::id.eq(id))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.insert_deleted_item(ModelType::Tag, id)?;
        }
        Ok(())
    }

    pub fn tag_count(&self) -> DatabaseResult<i64> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::tags;
        Ok(tags::table.count().get_result(&mut conn)?)
    }

    pub fn load_note_tag(&self, id: &str) -> DatabaseResult<NoteTag> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        Ok(note_tags::table
            .filter(note_tags::id.eq(id))
            .select(NoteTag::SELECTION)
            .first(&mut conn)?)
    }

    pub fn load_all_note_tags(&self) -> DatabaseResult<Vec<NoteTag>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        Ok(note_tags::table
            .select(NoteTag::SELECTION)
            .load(&mut conn)?)
    }

    pub fn load_note_tag_on_note(&self, note_id: &str, tag_id: &str) -> DatabaseResult<NoteTag> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        Ok(note_tags::table
            .filter(note_tags::note_id.eq(note_id))
            .filter(note_tags::tag_id.eq(tag_id))
            .select(NoteTag::SELECTION)
            .first(&mut conn)?)
    }

    pub fn get_note_tags(&self, note_id: &str) -> DatabaseResult<Vec<Tag>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        let note_tags: Vec<NoteTag> = note_tags::table
            .filter(note_tags::note_id.eq(note_id))
            .select(NoteTag::SELECTION)
            .load(&mut conn)?;
        self.load_tags(
            &note_tags
                .iter()
                .map(|x| x.tag_id.as_str())
                .collect::<Vec<&str>>(),
        )
    }

    pub fn add_tag_on_note(&self, note_id: &str, tag_id: &str) -> DatabaseResult<()> {
        let note_tag = NoteTag::new(note_id, tag_id);
        self.replace_note_tag(&note_tag, UpdateSource::LocalEdit)?;
        Ok(())
    }

    pub fn replace_note_tag(
        &self,
        note_tag: &NoteTag,
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let note_tag = match update_source {
            UpdateSource::RemoteSync => note_tag.clone(),
            UpdateSource::LocalEdit => note_tag.updated(),
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        diesel::replace_into(note_tags::table)
            .values(&note_tag)
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::NoteTag, note_tag.id.as_str(), update_source)?;
        Ok(())
    }

    pub fn delete_note_tag(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        self.delete_sync_item(id)?;
        diesel::delete(note_tags::table)
            .filter(note_tags::id.eq(id))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.insert_deleted_item(ModelType::NoteTag, id)?;
        }
        Ok(())
    }

    pub fn delete_note_tag_by_note_id_and_tag_id(
        &self,
        note_id: &str,
        tag_id: &str,
    ) -> DatabaseResult<()> {
        let tag_id = self.load_note_tag_on_note(note_id, tag_id)?;
        self.delete_note_tag(&tag_id.id, UpdateSource::LocalEdit)?;
        Ok(())
    }

    pub fn delete_note_tags(
        &self,
        ids: &[&str],
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        self.delete_sync_items(ids)?;
        diesel::delete(note_tags::table)
            .filter(note_tags::id.eq_any(ids))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.insert_deleted_items(ModelType::NoteTag, ids)?;
        }
        Ok(())
    }

    pub fn delete_note_tags_by_note_id(
        &self,
        note_id: &str,
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        let tag_ids: Vec<NoteTagId> = note_tags::table
            .filter(note_tags::note_id.eq(note_id))
            .select((note_tags::id,))
            .load(&mut conn)?;
        self.delete_note_tags(
            &tag_ids.iter().map(|x| x.id.as_str()).collect::<Vec<&str>>(),
            update_source,
        )?;
        Ok(())
    }

    pub fn delete_note_tag_by_note_ids(
        &self,
        note_id: &[&str],
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        let tag_ids: Vec<NoteTagId> = note_tags::table
            .filter(note_tags::note_id.eq_any(note_id))
            .select((note_tags::id,))
            .load(&mut conn)?;
        self.delete_note_tags(
            &tag_ids.iter().map(|x| x.id.as_str()).collect::<Vec<&str>>(),
            update_source,
        )?;
        Ok(())
    }

    pub fn note_tag_count(&self) -> DatabaseResult<i64> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::note_tags;
        Ok(note_tags::table.count().get_result(&mut conn)?)
    }
}

impl Database {
    pub fn load_resource(&self, id: &str) -> DatabaseResult<Resource> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::resources;
        Ok(resources::table
            .filter(resources::id.eq(id))
            .select((
                resources::id,
                resources::title,
                resources::mime,
                resources::filename,
                resources::created_time,
                resources::updated_time,
                resources::user_created_time,
                resources::user_updated_time,
                resources::file_extension,
                resources::encryption_cipher_text,
                resources::encryption_applied,
                resources::encryption_blob_encrypted,
                resources::size,
                resources::is_shared,
                resources::share_id,
                resources::master_key_id,
            ))
            .first(&mut conn)?)
    }

    pub fn replace_resource(
        &self,
        resource: &Resource,
        update_source: UpdateSource,
    ) -> DatabaseResult<()> {
        let resource_file = self
            .resource_path
            .join(&resource.id)
            .with_extension(&resource.file_extension);
        let resource = match update_source {
            UpdateSource::RemoteSync => resource.clone(),
            UpdateSource::LocalEdit => {
                assert!(resource_file.exists());
                resource.updated()
            }
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::resources;
        diesel::replace_into(resources::table)
            .values(&resource)
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Resource, resource.id.as_str(), update_source)?;
        Ok(())
    }

    pub fn delete_resource(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::resources;
        self.delete_sync_item(id)?;
        diesel::delete(resources::table)
            .filter(resources::id.eq(id))
            .execute(&mut conn)?;
        if update_source.is_local_edit() {
            self.insert_deleted_item(ModelType::Resource, id)?;
        }
        Ok(())
    }

    pub fn resource_count(&self) -> DatabaseResult<i64> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::resources;
        Ok(resources::table.count().get_result(&mut conn)?)
    }
}

impl Database {
    pub fn status(&self) -> DatabaseResult<Status> {
        Ok(Status {
            note_count: self.note_count()?,
            folder_count: self.folder_count()?,
            resource_count: self.resource_count()?,
            tag_count: self.tag_count()?,
            note_tag_count: self.note_tag_count()?,
        })
    }
}
