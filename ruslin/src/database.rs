mod connection_options;
mod error;

pub use error::DatabaseError;

use std::{
    fs,
    path::{Path, PathBuf},
};

use diesel::{
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::{
    database::connection_options::ConnectionOptions, models::Folder, AbbrNote, FolderID, Note,
    NoteID,
};

pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub struct Database {
    connection_pool: Pool<ConnectionManager<SqliteConnection>>,
    _path: PathBuf,
    _filename: String,
}

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

impl Database {
    fn new_with_filename(data_dir: &Path, filename: &str) -> DatabaseResult<Database> {
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
        let mut conn = self.connection_pool.get()?;
        use crate::schema::folders;
        diesel::replace_into(folders::table)
            .values(folder)
            .execute(&mut conn)?;
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
            ))
            .first(&mut conn)?)
    }

    pub fn replace_note(&self, note: &Note) -> DatabaseResult<()> {
        let mut conn = self.connection_pool.get()?;
        use crate::schema::notes;
        diesel::replace_into(notes::table)
            .values(note)
            .execute(&mut conn)?;
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

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use tempfile::TempDir;

    use super::{Database, DatabaseResult};
    use crate::models::Folder;

    struct TestDatabase(Database, TempDir);

    impl TestDatabase {
        fn temp() -> Self {
            let temp_dir = tempfile::TempDir::new().unwrap();
            let filename = "test.sqlite";
            let db = Database::new_with_filename(temp_dir.path(), filename).unwrap();
            Self(db, temp_dir)
        }
    }

    impl Deref for TestDatabase {
        type Target = Database;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    fn get_folder_1() -> Folder {
        Folder::new("folder1".to_string())
    }

    #[test]
    fn test_folder() -> DatabaseResult<()> {
        let db = TestDatabase::temp();
        let mut folder = get_folder_1();
        db.replace_folder(&folder)?;
        let load_folders = db.load_folders()?;
        assert_eq!(1, load_folders.len());
        assert_eq!(folder, load_folders[0]);
        folder.title = "folder1a".to_string();
        db.replace_folder(&folder)?;
        let load_folders = db.load_folders()?;
        assert_eq!(1, load_folders.len());
        assert_eq!(folder, load_folders[0]);
        db.delete_folder(&folder.id)?;
        let load_folders = db.load_folders()?;
        assert!(load_folders.is_empty());
        Ok(())
    }

    #[test]
    fn test_note() -> DatabaseResult<()> {
        Ok(())
    }
}
