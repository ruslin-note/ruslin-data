use std::sync::Arc;

use database_test::TestDatabase;
use ruslin_data::sync::{
    remote_api::joplin_server_api::test_api::TestSyncClient, FileApiDriverJoplinServer,
    Synchronizer,
};

mod database_test;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

#[tokio::test]
async fn test_delta() {
    init();
    let db = TestDatabase::temp();
    let api = TestSyncClient::Default.login().await;
    let file_api_driver = FileApiDriverJoplinServer::new(api);
    let temp_dir = tempfile::tempdir().unwrap();
    let synchronizer =
        Synchronizer::new(Arc::new(db.0), temp_dir.path(), Box::new(file_api_driver));
    synchronizer
        .start(false)
        .await
        .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
}
