use std::sync::Arc;

use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::DateTimeTimestamp;

use super::{FileApiDriver, SyncResult};

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum LockType {
    None = 0,
    Sync = 1,
    Exclusive = 2,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum LockClientType {
    Desktop = 1,
    Mobile = 2,
    Cli = 3,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Lock {
    pub id: Option<String>,
    pub r#type: LockType,
    pub client_type: LockClientType,
    pub client_id: String,
    pub updated_time: DateTimeTimestamp,
}

impl Lock {
    pub fn is_active(&self, current_date: DateTimeTimestamp, lock_ttl: i64) -> bool {
        current_date.timestamp_millis() - self.updated_time.timestamp_millis() < lock_ttl
    }
}

#[derive(Debug, Deserialize)]
pub struct LockList {
    pub items: Vec<Lock>,
    pub has_more: bool,
}

// pub struct LockHandlerOptions {
//     pub auto_refresh_interval: i32,
//     pub lock_ttl: i32,
// }

pub struct LockHandler {
    file_api_driver: Arc<Box<dyn FileApiDriver>>,
    // options: LockHandlerOptions,
}

impl LockHandler {
    pub fn new(file_api_driver: Arc<Box<dyn FileApiDriver>>) -> Self {
        Self { file_api_driver }
    }

    pub async fn locks(&self) -> SyncResult<Vec<Lock>> {
        Ok(self.file_api_driver.list_locks().await?.items)
    }

    pub async fn acquire_sync_lock(
        &self,
        client_type: LockClientType,
        client_id: &str,
    ) -> SyncResult<Lock> {
        self
            .file_api_driver
            .acquire_lock(LockType::Sync, client_type, client_id)
            .await
    }

    // pub async fn acquire_exclusive_lock(&self, client_type: LockClientType, client_id: &str)

    pub async fn release_lock(
        &self,
        lock_type: LockType,
        client_type: LockClientType,
        client_id: &str,
    ) -> SyncResult<()> {
        self
            .file_api_driver
            .release_lock(lock_type, client_type, client_id)
            .await
    }
}
