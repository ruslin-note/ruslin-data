mod connection_options;
mod error;

pub use error::DatabaseError;

use diesel::{
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    database::connection_options::ConnectionOptions, models::Folder, AbbrNote, DateTimeTimestamp,
    FolderID, ModelType, NewSyncItem, Note, NoteID, SyncTarget,
};

pub type DatabaseResult<T> = Result<T, DatabaseError>;

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
    pub fn replace_folder(&self, folder: &Folder) -> DatabaseResult<()> {
        let folder = folder.updated();
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        diesel::replace_into(folders::table)
            .values(&folder)
            .execute(&mut conn)?;
        self.insert_sync_item(ModelType::Folder, folder.id.as_str())?;
        Ok(())
    }

    pub fn load_folders(&self) -> DatabaseResult<Vec<Folder>> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        Ok(folders::table
            .select((
                folders::id,
                folders::title,
                folders::created_time,
                folders::updated_time,
                folders::user_created_time,
                folders::user_updated_time,
                folders::encryption_cipher_text,
                folders::encryption_applied,
                folders::parent_id,
                folders::is_shared,
                folders::share_id,
                folders::master_key_id,
                folders::icon,
            ))
            .load(&mut conn)?)
    }

    pub fn delete_folder(&self, id: &FolderID) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        diesel::delete(folders::table)
            .filter(folders::id.eq(id))
            .execute(&mut conn)?;
        Ok(())
    }
}

impl Database {
    pub fn load_abbr_notes(&self, parent_id: Option<&FolderID>) -> DatabaseResult<Vec<AbbrNote>> {
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

    pub fn load_note(&self, id: &NoteID) -> DatabaseResult<Note> {
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
                notes::custom_order,
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

    pub fn replace_note(&self, note: &Note) -> DatabaseResult<()> {
        let note = note.updated();
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        diesel::replace_into(notes::table)
            .values(&note)
            .execute(&mut conn)?;
        self.insert_sync_item(ModelType::Note, note.id.as_str())?;
        Ok(())
    }

    pub fn delete_note(&self, id: &NoteID) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        diesel::delete(notes::table)
            .filter(notes::id.eq(id))
            .execute(&mut conn)?;
        Ok(())
    }

    pub fn delete_notes(&self, notes: &[NoteID]) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        diesel::delete(notes::table)
            .filter(notes::id.eq_any(notes))
            .execute(&mut conn)?;
        Ok(())
    }
}

impl Database {
    fn insert_sync_item(
        &self,
        // sync_target: SyncTarget,
        item_type: ModelType,
        item_id: &str,
    ) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::sync_items;

        let sync_item_exists: bool = diesel::select(diesel::dsl::exists(
            sync_items::dsl::sync_items.filter(sync_items::item_id.eq(item_id)),
        ))
        .get_result(&mut conn)?;
        if sync_item_exists {
            return Ok(());
        }

        let sync_item = NewSyncItem {
            sync_target: SyncTarget::FileSystem, // TODO: supports multiple sync targets?
            item_type,
            item_id,
        };
        diesel::insert_into(sync_items::table)
            .values(&sync_item)
            .execute(&mut conn)?;
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
}
