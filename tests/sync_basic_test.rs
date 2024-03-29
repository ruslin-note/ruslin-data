use ruslin_data::sync::SyncConfig;
use ruslin_data::sync::{remote_api::joplin_server_api::test_api::TestSyncClient, SyncResult};
use ruslin_data::{Folder, Note, Resource, RuslinData, UpdateSource};

use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use tempfile::TempDir;

mod database_test;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

struct TestClient(TempDir, TempDir, RuslinData);

impl TestClient {
    async fn new(sync_config: SyncConfig) -> SyncResult<Self> {
        let data_dir = tempfile::TempDir::new().unwrap();
        let resource_dir = tempfile::TempDir::new().unwrap();
        let ruslin_data = RuslinData::new(data_dir.path(), resource_dir.path())?;
        ruslin_data.save_sync_config(sync_config).await?;
        // ruslin_data.clear_remote().await?;
        Ok(Self(data_dir, resource_dir, ruslin_data))
    }
}

impl Deref for TestClient {
    type Target = RuslinData;

    fn deref(&self) -> &Self::Target {
        &self.2
    }
}

#[tokio::test]
async fn test_basic() -> SyncResult<()> {
    init();
    let client_1 = TestClient::new(TestSyncClient::Basic1.sync_config()).await?;
    let client_2 = TestClient::new(TestSyncClient::Basic1.sync_config()).await?;

    let mut note = should_create_items(&client_1, &client_2).await?;
    should_update_items(&client_1, &client_2, &mut note).await?;
    should_delete_note(&client_1, &client_2, note).await?;
    Ok(())
}

async fn should_create_items(client_1: &RuslinData, client_2: &RuslinData) -> SyncResult<Note> {
    let folder = Folder::new("folder1".to_string(), None);
    client_1
        .db
        .replace_folder(&folder, UpdateSource::LocalEdit)?;
    let note = Note::new(Some(folder.id), "un".to_string(), "".to_string());
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await?;
    // should create local items
    client_2.synchronize(false).await?;
    client_2.db.load_note(&note.id)?;
    Ok(note)
}

async fn should_update_items(
    client_1: &RuslinData,
    client_2: &RuslinData,
    note: &mut Note,
) -> SyncResult<()> {
    // should update remote items
    note.set_title("un UPDATE");
    client_1.db.replace_note(note, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await?;
    // TODO: check client_1 local remote

    // should update local items
    client_2.synchronize(false).await?;
    note.set_title("Updated on client 2");
    client_2.db.replace_note(note, UpdateSource::LocalEdit)?;
    client_2.synchronize(false).await?;
    client_1.synchronize(false).await?;
    // TODO: check client_1 local remote

    Ok(())
}

async fn should_delete_note(
    client_1: &RuslinData,
    client_2: &RuslinData,
    note: Note,
) -> SyncResult<()> {
    // should delete remote notes
    client_2.db.delete_note(&note.id, UpdateSource::LocalEdit)?;
    client_2.synchronize(false).await?;
    client_1.synchronize(false).await?;
    // TODO: check client_1 & client_2 local remote
    Ok(())
}

#[tokio::test]
async fn should_update_note_item() -> SyncResult<()> {
    let client_1 = TestClient::new(TestSyncClient::UpdateNoteItem.sync_config()).await?;
    let note = Note::new(None, "title".to_string(), "body".to_string());
    let note_id = &note.id;
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await?;
    assert_eq!(client_1.db.load_need_upload_sync_items()?.len(), 0);
    client_1.db.update_note_body(note_id, "body1")?;
    println!("{:?}", client_1.db.load_all_sync_items()?);
    assert_eq!(client_1.db.load_need_upload_sync_items()?.len(), 1);
    client_1.synchronize(false).await?;
    client_1.db.update_note_title(note_id, "title1")?;
    assert_eq!(client_1.db.load_need_upload_sync_items()?.len(), 1);
    let updated_note = client_1.db.load_note(note_id)?;
    assert_eq!("body1", &updated_note.body);
    assert_eq!("title1", &updated_note.title);
    assert_eq!(note.created_time, updated_note.created_time);
    assert!(note.updated_time < updated_note.updated_time);
    Ok(())
}

// should not created deleted_items entries for items deleted via sync

// should delete local notes

// should delete remote folder

// should delete local folder

// should cross delete all folders

#[tokio::test]
async fn test_should_not_sync_deletions_that_came_via_sync_even_when_there_is_a_conflict(
) -> SyncResult<()> {
    init();
    let client_1 = TestClient::new(TestSyncClient::Conflict1.sync_config()).await?;
    let client_2 = TestClient::new(TestSyncClient::Conflict1.sync_config()).await?;
    let mut note = Note::new(None, "title".to_string(), "body".to_string());
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await?;

    client_2.synchronize(false).await?;
    client_2.db.load_note(&note.id)?;
    client_2.db.delete_note(&note.id, UpdateSource::LocalEdit)?;
    client_2.synchronize(false).await?;

    note.title = "title2".to_string();
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await?;
    let abbr_notes = client_1.db.load_abbr_notes(None)?;
    assert!(!abbr_notes.iter().any(|n| n.id == note.id));
    assert!(client_1.db.conflict_note_exists()?);
    let abbr_conflict_notes = client_1.db.load_abbr_conflict_notes()?;
    assert_eq!(1, abbr_conflict_notes.len());
    assert!(client_1.db.load_note(&note.id).is_err());
    let conflict_note = client_1.db.load_note(&abbr_conflict_notes[0].id)?;
    assert!(conflict_note.is_conflict);
    assert!(conflict_note.conflict_original_id.unwrap() == note.id);
    Ok(())
}

#[tokio::test]
async fn test_should_upload_resource() -> SyncResult<()> {
    init();
    let client_1 = TestClient::new(TestSyncClient::Upload1.sync_config()).await?;
    let mut resource = Resource::new("file.txt", "text/plain", "txt", 0);
    let path = resource.resource_file_path(&client_1.resource_dir);
    let mut output = File::create(&path).unwrap();
    write!(output, "Rust\n💖\nFun")?;
    output.sync_all().unwrap();
    let metadata = output.metadata().unwrap();
    resource.size = metadata.len() as i32;

    client_1
        .db
        .replace_resource(&resource, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await.unwrap();

    let client_2 = TestClient::new(TestSyncClient::Upload1.sync_config()).await?;
    client_2.synchronize(false).await.unwrap();
    let resource = client_2.db.load_resource(&resource.id)?;
    let path = resource.resource_file_path(&client_2.resource_dir);
    assert!(path.exists());

    Ok(())
}

#[tokio::test]
async fn test_allow_remote_resource_file_does_not_exists() -> SyncResult<()> {
    init();
    let client_1 = TestClient::new(TestSyncClient::Upload1.sync_config()).await?;
    let mut resource = Resource::new("file.txt", "text/plain", "txt", 0);
    let path = resource.resource_file_path(&client_1.resource_dir);
    let mut output = File::create(&path).unwrap();
    write!(output, "Rust\n💖\nFun")?;
    output.sync_all().unwrap();
    let metadata = output.metadata().unwrap();
    resource.size = metadata.len() as i32;

    client_1
        .db
        .replace_resource(&resource, UpdateSource::LocalEdit)?;
    client_1.synchronize(false).await.unwrap();
    client_1
        .get_file_api_driver()
        .await
        .unwrap()
        .delete(&resource.remote_path())
        .await
        .unwrap();

    let client_2 = TestClient::new(TestSyncClient::Upload1.sync_config()).await?;
    client_2.synchronize(false).await.unwrap();

    Ok(())
}
