mod connection_options;
mod error;

pub use error::DatabaseError;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    database::connection_options::ConnectionOptions,
    models::Folder,
    sync::{ForSyncSerializer, SerializeForSync},
    AbbrNote, DateTimeTimestamp, DeletedItem, ModelType, NewDeletedItem, NewSetting, NewSyncItem,
    Note, Setting, SyncItem,
};

pub type DatabaseResult<T> = Result<T, DatabaseError>;

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

#[derive(Debug)]
pub struct Database {
    connection_pool: Pool<ConnectionManager<SqliteConnection>>,
    _path: PathBuf,
    _filename: String,
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

impl Database {
    pub fn new_with_filename(data_dir: &Path, filename: &str) -> DatabaseResult<Database> {
        fs::create_dir_all(data_dir)?;
        let database_url = data_dir.join(filename);
        let database_url = database_url.to_str().unwrap().to_string();
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let connection_pool = Pool::builder()
            .connection_customizer(Box::new(ConnectionOptions::default()))
            .max_size(16)
            .build(manager)?;

        let db = Self {
            connection_pool,
            _path: data_dir.into(),
            _filename: filename.into(),
        };
        db.init()?;
        Ok(db)
    }

    pub fn new(data_dir: &Path) -> DatabaseResult<Database> {
        Database::new_with_filename(data_dir, "database.sqlite")
    }

    fn init(&self) -> DatabaseResult<()> {
        let mut connection = self.connection_pool.get()?;
        connection.run_pending_migrations(MIGRATIONS).map_err(|e| {
            log::error!("Database migration failed: {}", e);
            DatabaseError::Migration
        })?;
        diesel::sql_query("PRAGMA journal_mode = WAL").execute(&mut connection)?;
        Ok(())
    }
}

impl Database {
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
        Ok(folders::table.select(Folder::SELECTION).load(&mut conn)?)
    }

    pub fn load_folder(&self, id: &str) -> DatabaseResult<Folder> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table
            .filter(folders::id.eq(id))
            .select(Folder::SELECTION)
            .first(&mut conn)?)
    }

    pub fn delete_folder(&self, id: &str, update_source: UpdateSource) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        if update_source.is_local_edit() {
            self.delete_notes_by_folder_id(Some(id))?;
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
}

impl Database {
    pub fn load_abbr_notes(&self, parent_id: Option<&str>) -> DatabaseResult<Vec<AbbrNote>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        let selection = (
            notes::id,
            notes::parent_id,
            notes::title,
            notes::created_time,
            notes::updated_time,
        );
        Ok(match parent_id {
            Some(parent_id) => notes::table
                .filter(notes::parent_id.eq(parent_id))
                .select(selection)
                .load(&mut conn),
            None => notes::table.select(selection).load(&mut conn),
        }?)
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

    pub fn replace_note(&self, note: &Note, update_source: UpdateSource) -> DatabaseResult<()> {
        let note = match update_source {
            UpdateSource::RemoteSync => note.clone(),
            UpdateSource::LocalEdit => note.updated(),
        };
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        diesel::replace_into(notes::table)
            .values(&note)
            .execute(&mut conn)?;
        self.replace_sync_item(ModelType::Note, note.id.as_str(), update_source)?;
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
            self.insert_deleted_item(ModelType::Note, id)?;
        }
        Ok(())
    }

    fn delete_notes_by_folder_id(&self, folder_id: Option<&str>) -> DatabaseResult<()> {
        let notes = self.load_abbr_notes(folder_id)?;
        for note in notes {
            self.delete_note(&note.id, UpdateSource::LocalEdit)?;
        }
        // TODO: batch delete
        // let mut conn = self.connection_pool.get()?;
        // use crate::schema::notes;
        // diesel::delete(notes::table)
        //     .filter(notes::parent_id.eq_any(folder_id))
        //     .execute(&mut conn)?;
        Ok(())
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
                    UpdateSource::LocalEdit => sync_item.update_time = DateTimeTimestamp::now(),
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
}
