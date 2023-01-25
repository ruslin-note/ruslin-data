use async_trait::async_trait;

use crate::sync::lock_handler::{Lock, LockClientType, LockList, LockType};
use crate::sync::{SyncError, SyncResult};
use crate::DateTimeTimestamp;
use std::fmt::Debug;
use std::fs::Metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::file_api_driver_joplin_server::JoplinServerSyncContext;

pub trait SyncContext: Debug + Send + Sync {
    fn to_joplin_server_sync_context(&self) -> &JoplinServerSyncContext {
        panic!()
    }

    fn to_string(&self) -> String;
}

pub struct Stat {
    pub path: String,
    pub updated_time: DateTimeTimestamp,
    // jop_updated_time: i64,
    pub is_dir: bool,
    // is_deleted: bool,
}

impl TryFrom<Metadata> for Stat {
    type Error = SyncError;

    fn try_from(metadata: Metadata) -> SyncResult<Self> {
        Ok(Self {
            path: "".to_string(),
            updated_time: DateTimeTimestamp::from_timestamp_millis(
                metadata
                    .modified()?
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()))
                    .as_micros() as i64,
            ),
            is_dir: metadata.is_dir(),
        })
    }
}

#[derive(Debug)]
pub struct DataList<Item> {
    pub items: Vec<Item>,
    pub has_more: bool,
    pub context: Option<Box<dyn SyncContext>>,
}

pub type StatList = DataList<Stat>;

#[derive(Debug)]
pub struct RemoteItem {
    // pub id: String,
    pub path: String,
    // pub r#type: ModelType,
    pub is_deleted: bool,
    pub updated_time: DateTimeTimestamp,
    pub jop_updated_time: Option<DateTimeTimestamp>,
}

impl RemoteItem {
    pub fn path_id(&self) -> &str {
        self.path
            .split('/')
            .last()
            .unwrap_or_else(|| panic!("unwrap error in {}:{}", file!(), line!()))
            .split_once('.')
            .unwrap_or_else(|| panic!("unwrap error in {}:{}", file!(), line!()))
            .0
    }
}

pub type DeltaList = DataList<RemoteItem>;

pub struct MultiPutItem {
    pub name: String,
    pub body: String,
}

#[async_trait]
pub trait FileApiDriver: Send + Sync + Debug {
    fn supports_multi_put(&self) -> bool;
    fn supports_accurate_timestamp(&self) -> bool;
    fn supports_locks(&self) -> bool;
    fn request_repeat_count(&self) -> u32;
    async fn stat(&self, path: &str) -> SyncResult<Option<Stat>>;
    async fn delta(&self, path: &str, ctx: Option<&dyn SyncContext>) -> SyncResult<DeltaList>;
    fn deserializer_delta_context(&self, s: &str) -> SyncResult<Box<dyn SyncContext>>;
    async fn list(&self, path: &str) -> SyncResult<StatList>;
    async fn get_text(&self, path: &str) -> SyncResult<String>;
    async fn get_file(&self, path: &str, destination: &Path) -> SyncResult<()>;
    async fn mkdir(&self, path: &str) -> SyncResult<()>;
    async fn put_text(&self, path: &str, content: &str) -> SyncResult<()>;
    async fn put_file(&self, path: &str, local_file_path: &Path) -> SyncResult<()>;
    async fn multi_put(&self, items: &[MultiPutItem]) -> SyncResult<()>;
    async fn delete(&self, path: &str) -> SyncResult<()>;
    async fn r#move(&self, old_path: &str, new_path: &str) -> SyncResult<()>;
    async fn clear_root(&self, base_dir: &str) -> SyncResult<()>;
    async fn check_config(&self) -> SyncResult<()>;
    async fn acquire_lock(
        &self,
        r#type: LockType,
        client_type: LockClientType,
        client_id: &str,
    ) -> SyncResult<Lock>;
    async fn release_lock(
        &self,
        r#type: LockType,
        client_type: LockClientType,
        client_id: &str,
    ) -> SyncResult<()>;
    async fn list_locks(&self) -> SyncResult<LockList>;
}
