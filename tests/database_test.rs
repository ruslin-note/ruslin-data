use ruslin_data::{Database, DatabaseResult, Folder, Note, SearchBodyOption, UpdateSource};
use std::{ops::Deref, time::Duration};
use tempfile::TempDir;

pub struct TestDatabase(pub Database, TempDir);

impl TestDatabase {
    pub fn temp() -> Self {
        let temp_dir = tempfile::TempDir::new()
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        let filename = "test.sqlite";
        let db = Database::new_with_filename(temp_dir.path(), filename)
            .unwrap_or_else(|e| panic!("unwrap error {e:?} in {}:{}", file!(), line!()));
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
    Folder::new("folder1", None)
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
fn test_folder_order() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    db.insert_root_folder("b")?;
    db.insert_root_folder("a")?;
    let folders = db.load_folders()?;
    let titles: Vec<&str> = folders.iter().map(|x| x.title.as_str()).collect();
    assert_eq!(&vec!["a", "b"], &titles);
    Ok(())
}

#[test]
fn insert_folder_note() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let folder_a = db.insert_root_folder("folder_a")?;
    db.insert_note_with_parent("a", "", &folder_a.id)?;
    let notes = db.load_abbr_notes(Some(&folder_a.id))?;
    assert_eq!(1, notes.len());
    Ok(())
}

#[test]
fn test_delete_folder() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    // a -> b,c
    // c -> d
    // x -> y
    let folder_a = db.insert_root_folder("folder_a")?;
    let folder_b = db.insert_folder_with_parent("folder_b", &folder_a.id)?;
    let folder_c = db.insert_folder_with_parent("folder_c", &folder_a.id)?;
    let folder_d = db.insert_folder_with_parent("folder_d", &folder_c.id)?;
    let folder_x = db.insert_root_folder("folder_x")?;
    let folder_y = db.insert_folder_with_parent("folder_y", &folder_x.id)?;

    let note_a = db.insert_note_with_parent("a", "", &folder_a.id)?;
    let note_b = db.insert_note_with_parent("b", "", &folder_b.id)?;
    let note_c = db.insert_note_with_parent("c", "", &folder_c.id)?;
    let note_d = db.insert_note_with_parent("d", "", &folder_d.id)?;
    let _note_x = db.insert_note_with_parent("x", "", &folder_x.id)?;
    let _note_y = db.insert_note_with_parent("y", "", folder_y.id)?;

    let deleted_item_ids: Vec<&str> = vec![
        &folder_a.id,
        &folder_b.id,
        &folder_c.id,
        &folder_d.id,
        &note_a.id,
        &note_b.id,
        &note_c.id,
        &note_d.id,
    ];

    db.delete_folder(&folder_a.id, UpdateSource::LocalEdit)?;
    let remained_folders = db.load_folders()?;
    assert_eq!(2, remained_folders.len());
    let remained_notes = db.load_abbr_notes(None)?;
    assert_eq!(2, remained_notes.len());
    let deleted_items = db.load_deleted_items()?;
    assert_eq!(deleted_item_ids.len(), deleted_items.len());
    let sync_items = db.load_all_sync_items()?;
    assert_eq!(4, sync_items.len());

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

#[test]
fn test_search_notes() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    db.replace_note(
        &Note::new(None, "abcd efgh".to_string(), "abcd".to_string()),
        UpdateSource::LocalEdit,
    )?;
    db.replace_note(
        &Note::new(None, "abcd".to_string(), "".to_string()),
        UpdateSource::LocalEdit,
    )?;
    db.replace_note(
        &Note::new(None, "efgh".to_string(), "abcd".to_string()),
        UpdateSource::LocalEdit,
    )?;
    db.rebuild_fts()?;
    let notes = db.search_notes("abcd", None)?;
    assert_eq!(3, notes.len());
    let notes = db.search_notes("efgh", None)?;
    assert_eq!(2, notes.len());
    Ok(())
}

#[test]
fn test_search_chinese_notes() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let note = Note::new(None, "我是中国人".to_string(), "中文测试".to_string());
    db.replace_note(&note, UpdateSource::LocalEdit)?;
    db.rebuild_fts()?;
    let notes = db.search_notes("中国", Some(SearchBodyOption::Highlight))?;
    assert_eq!(1, notes.len());
    assert_eq!("我是<b>中国</b>人", notes[0].title);
    let notes = db.search_notes("中国", None)?;
    assert_eq!(1, notes.len());
    assert_eq!("我是中国人", notes[0].title);

    let notes = db.search_notes("我", Some(SearchBodyOption::Highlight))?;
    assert_eq!(1, notes.len());
    let notes = db.search_notes("国人", Some(SearchBodyOption::Highlight))?;
    assert_eq!(0, notes.len());
    let notes = db.search_notes("测试", Some(SearchBodyOption::Highlight))?;
    assert_eq!("中文<b>测试</b>", notes[0].body);
    assert_eq!(1, notes.len());
    Ok(())
}

#[test]
fn test_search_notes_update() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let mut note = Note::new(None, "abcd efgh".to_string(), "abcd".to_string());
    db.replace_note(&note, UpdateSource::LocalEdit)?;
    let notes = db.search_notes("efgh", None)?;
    assert_eq!(1, notes.len());
    note.title = "abcd".to_string();
    db.replace_note(&note, UpdateSource::LocalEdit)?;
    let notes = db.search_notes("efgh", None)?;
    assert_eq!(0, notes.len());
    let notes = db.search_notes("abcd", None)?;
    assert_eq!(1, notes.len());
    db.delete_note(&note.id, UpdateSource::LocalEdit)?;
    let notes = db.search_notes("abcd", None)?;
    assert_eq!(0, notes.len());
    Ok(())
}

#[test]
fn test_search_option() -> DatabaseResult<()> {
    let db = TestDatabase::temp();
    let note = Note::new(None, "lorem ipsum", "Rutrum pellentesque fringilla conubia lacinia efficitur facilisis condimentum faucibus congue. 
    Aliquet duis quisque rhoncus leo donec praesent conubia gravida. Nunc lectus massa cubilia fusce viverra hendrerit ultrices auctor nullam. 
    Massa nec eros nulla porta lacinia bibendum. Magna enim velit commodo venenatis sit nisl ullamcorper. 
    Malesuada donec sodales urna fermentum primis mauris pellentesque.");
    db.replace_note(&note, UpdateSource::LocalEdit)?;
    let notes = db.search_notes("congue", Some(SearchBodyOption::Snippet { max_tokens: 16 }))?;
    assert!(!notes.is_empty());
    let notes = db.search_notes("conubia", Some(SearchBodyOption::Snippet { max_tokens: 8 }))?;
    assert!(!notes.is_empty());
    Ok(())
}
