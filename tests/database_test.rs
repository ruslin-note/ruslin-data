use std::{ops::Deref, time::Duration};

use tempfile::TempDir;

use ruslin_data::{Database, DatabaseResult, Folder, Note, UpdateSource};

pub struct TestDatabase(pub Database, TempDir);

impl TestDatabase {
    pub fn temp() -> Self {
        let temp_dir =
            tempfile::TempDir::new().expect(&format!("unwrap error in {}:{}", file!(), line!()));
        let filename = "test.sqlite";
        let db = Database::new_with_filename(temp_dir.path(), filename).expect(&format!(
            "unwrap error in {}:{}",
            file!(),
            line!()
        ));
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
fn test_load_abbr_notes_order() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let note_1 = Note::new(None, "title_1".to_string(), "".to_string());
    db.replace_note(&note_1, UpdateSource::LocalEdit)?;
    std::thread::sleep(Duration::from_millis(200));
    let note_2 = Note::new(None, "title_2".to_string(), "".to_string());
    db.replace_note(&note_2, UpdateSource::LocalEdit)?;
    let notes = db.load_abbr_notes(None)?;
    assert_eq!(note_2.title, notes[0].title);
    assert_eq!(note_1.title, notes[1].title);
    Ok(())
}
