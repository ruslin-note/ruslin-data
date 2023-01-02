pub mod database;
mod models;
mod schema;
pub mod sync;

use std::{path::Path, sync::Arc};

pub use database::{Database, DatabaseError, DatabaseResult, UpdateSource};
pub use models::*;
use parking_lot::RwLock;
use sync::{
    remote_api::JoplinServerAPI, FileApiDriver, FileApiDriverJoplinServer, SyncConfig, SyncError,
    SyncInfo, SyncResult, Synchronizer,
};

#[derive(Debug)]
pub struct RuslinData {
    pub db: Arc<Database>,
    pub sync_config: RwLock<Option<SyncConfig>>,
}

impl RuslinData {
    pub fn new(data_dir: &Path) -> SyncResult<Self> {
        let db = Arc::new(Database::new(data_dir)?);
        let sync_config = db.get_setting_value(Setting::FILE_API_SYNC_CONFIG)?;
        let sync_config = RwLock::new(match sync_config {
            Some(c) => serde_json::from_str(&c.value)?,
            None => None,
        });
        Ok(Self { db, sync_config })
    }

    async fn get_file_api_driver(&self) -> SyncResult<Box<dyn FileApiDriver>> {
        let sync_config = self.sync_config.read().clone();
        let sync_config = sync_config.ok_or(SyncError::SyncConfigNotExists)?;
        let file_api_driver = match &sync_config {
            SyncConfig::JoplinServer {
                host,
                email,
                password,
            } => {
                let api = JoplinServerAPI::login(host, email, password).await?;
                Box::new(FileApiDriverJoplinServer::new(api))
            }
        };
        Ok(file_api_driver)
    }

    pub async fn sync(&self) -> SyncResult<SyncInfo> {
        let file_api_driver = self.get_file_api_driver().await?;
        let synchronizer = Synchronizer::new(self.db.clone(), file_api_driver);
        synchronizer.check_target_info_support().await?;
        synchronizer.start().await
    }

    pub fn sync_exists(&self) -> bool {
        self.sync_config.read().is_some()
    }

    pub async fn clear_remote(&self) -> SyncResult<()> {
        let file_api_driver = self.get_file_api_driver().await?;
        file_api_driver.clear_root("").await?;
        Ok(())
    }

    pub fn get_sync_config(&self) -> SyncResult<Option<SyncConfig>> {
        let sync_config = self.db.get_setting_value(Setting::FILE_API_SYNC_CONFIG)?;
        Ok(match sync_config {
            Some(sync_config) => Some(serde_json::from_str(&sync_config.value)?),
            None => None,
        })
    }

    pub async fn save_sync_config(&self, sync_config: SyncConfig) -> SyncResult<()> {
        match &sync_config {
            SyncConfig::JoplinServer {
                host,
                email,
                password,
            } => {
                let api = JoplinServerAPI::login(host, email, password).await?;
                let file_api_driver = Box::new(FileApiDriverJoplinServer::new(api));
                file_api_driver.check_config().await?;
                let synchronizer = Synchronizer::new(self.db.clone(), file_api_driver);
                synchronizer.check_target_info_support().await?;
            }
        };
        self.db.replace_setting(
            Setting::FILE_API_SYNC_CONFIG,
            &serde_json::to_string(&sync_config).expect("sync_config to_string error"),
        )?;
        self.sync_config.write().replace(sync_config);
        Ok(())
    }
}
