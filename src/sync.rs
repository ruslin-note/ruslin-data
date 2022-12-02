mod deserialize;
mod error;
mod file_api;
pub mod remote_api;
mod serializer;

use std::sync::Arc;

pub use deserialize::{DeserializeForSync, ForSyncDeserializer};
pub use error::{SyncError, SyncResult};
pub use file_api::*;
pub use serializer::{ForSyncSerializer, SerializeForSync};

use crate::Database;

pub struct Synchronizer {
    _db: Arc<Database>,
    file_api_driver: Box<dyn FileApiDriver>,
}

impl Synchronizer {
    pub fn new(db: Arc<Database>, file_api_driver: Box<dyn FileApiDriver>) -> Self {
        Self {
            _db: db,
            file_api_driver,
        }
    }

    pub async fn start(&self) -> SyncResult<()> {
        self.delete_remote().await;
        self.upload().await;
        self.delta().await?;
        Ok(())
    }

    async fn delete_remote(&self) {}

    async fn upload(&self) {}

    async fn delta(&self) -> SyncResult<()> {
        let mut context: Option<Box<dyn SyncContext>> = None;
        loop {
            let list_result = self.file_api_driver.delta("", context.as_deref()).await?;
            context = list_result.context;
            if !list_result.has_more {
                break;
            }
        }
        Ok(())
    }
}
