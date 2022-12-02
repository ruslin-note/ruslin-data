use std::ops::Deref;

use tempfile::TempDir;

use ruslin_data::{Database, DatabaseResult, Folder, UpdateSource};

pub struct TestDatabase(pub Database, TempDir);

impl TestDatabase {
    pub fn temp() -> Self {
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
    Folder::new("folder1".to_string(), None)
}

#[test]
fn test_folder() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let mut folder = get_folder_1();
    db.replace_folder(&folder, UpdateSource::LocalEdit)?;
    let load_folders = db.load_folders()?;
    assert_eq!(1, load_folders.len());
    assert_eq!(folder, load_folders[0]);
    folder.title = "folder1a".to_string();
    db.replace_folder(&folder, UpdateSource::LocalEdit)?;
    let load_folders = db.load_folders()?;
    assert_eq!(1, load_folders.len());
    assert_eq!(folder, load_folders[0]);
    db.delete_folder(&folder.id, UpdateSource::LocalEdit)?;
    let load_folders = db.load_folders()?;
    assert!(load_folders.is_empty());
    Ok(())
}

#[test]
fn test_note() -> DatabaseResult<()> {
    Ok(())
}
