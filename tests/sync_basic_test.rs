use ruslin_data::sync::{remote_api::joplin_server_api::test_api::TestSyncClient, SyncResult};
use ruslin_data::{Folder, Note, RuslinData, UpdateSource};

mod database_test;

#[tokio::test]
async fn test_basic() -> SyncResult<()> {
    let data_dir_1 =
        tempfile::TempDir::new().expect(&format!("unwrap error in {}:{}", file!(), line!()));
    let client_1 = RuslinData::new(data_dir_1.path())?;
    client_1
        .save_sync_config(TestSyncClient::Basic1.sync_config())
        .await?;

    let data_dir_2 =
        tempfile::TempDir::new().expect(&format!("unwrap error in {}:{}", file!(), line!()));
    let client_2 = RuslinData::new(data_dir_2.path())?;
    client_2
        .save_sync_config(TestSyncClient::Basic2.sync_config())
        .await?;

    // should create remote items
    let folder = Folder::new("folder1".to_string(), None);
    client_1
        .db
        .replace_folder(&folder, UpdateSource::LocalEdit)?;
    let mut note = Note::new(Some(folder.id), "un".to_string(), "".to_string());
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.sync().await?;
    // TODO: check client_1 local remote

    // should create local items
    client_2.sync().await?;
    // TODO: check client_2 local remote

    // should update remote items
    note.set_title("un UPDATE");
    client_1.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_1.sync().await?;
    // TODO: check client_1 local remote

    // should update local items
    client_2.sync().await?;
    note.set_title("Updated on client 2");
    client_2.db.replace_note(&note, UpdateSource::LocalEdit)?;
    client_2.sync().await?;
    client_1.sync().await?;
    // TODO: check client_1 local remote

    // should delete remote notes
    client_2.db.delete_note(&note.id, UpdateSource::LocalEdit)?;
    client_2.sync().await?;
    client_1.sync().await?;
    // TODO: check client_1 & client_2 local remote

    // should not created deleted_items entries for items deleted via sync

    // should delete local notes

    // should delete remote folder

    // should delete local folder

    // should cross delete all folders

    // should not sync deletions that came via sync even when there is a conflict

    Ok(())
}
