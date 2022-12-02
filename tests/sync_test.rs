use std::sync::Arc;

use database_test::TestDatabase;
use ruslin_data::sync::{remote_api::JoplinServerAPI, FileApiDriverJoplinServer, Synchronizer};

mod database_test;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct TestEnv {
    pub joplin_server: TestJoplinServerEnv,
}

#[derive(Deserialize)]
pub struct TestJoplinServerEnv {
    pub host: String,
    pub session_id: String,
    pub email: String,
    pub password: String,
}

pub fn read_test_env() -> TestEnv {
    let env_toml = include_str!("../.test-env.toml");
    toml::from_str(env_toml).unwrap()
}

#[tokio::test]
async fn test_delta() {
    let db = TestDatabase::temp();
    let joplin_server_config = read_test_env().joplin_server;
    let api = JoplinServerAPI::login(
        &joplin_server_config.host,
        &joplin_server_config.email,
        &joplin_server_config.password,
    )
    .await
    .unwrap();
    let file_api_driver = FileApiDriverJoplinServer::new(api);
    let synchronizer = Synchronizer::new(Arc::new(db.0), Box::new(file_api_driver));
    synchronizer.start().await.unwrap();
}
