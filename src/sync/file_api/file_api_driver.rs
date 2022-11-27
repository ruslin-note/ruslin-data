use crate::sync::{SyncError, SyncResult};
use std::fs::Metadata;
use std::time::UNIX_EPOCH;

pub struct Stat {
    pub path: String,
    pub updated_time: i64,
    // jop_updated_time: i64,
    pub is_dir: bool,
    // is_deleted: bool,
}

impl TryFrom<Metadata> for Stat {
    type Error = SyncError;

    fn try_from(metadata: Metadata) -> SyncResult<Self> {
        Ok(Self {
            path: "".to_string(),
            updated_time: metadata
                .modified()?
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as i64,
            is_dir: metadata.is_dir(),
        })
    }
}

pub struct StatList {
    pub items: Vec<Stat>,
    pub has_more: bool,
    // context:
}

pub enum Source {
    File(String),
    Text(String),
}

pub struct PutOptions {
    pub source: Source,
}

pub struct ListOptions;

pub enum GetTarget {
    File(String),
    Text,
}

pub struct GetOptions {
    pub target: GetTarget,
}

pub struct MultiPutItem {
    pub name: String,
    pub body: String,
}

pub trait FileApiDriver {
    fn supports_multi_put(&self) -> bool;
    fn supports_accurate_timestamp(&self) -> bool;
    fn supports_locks(&self) -> bool;
    fn request_repeat_count(&self) -> u32;
    fn stat(&self, path: &str) -> SyncResult<Stat>;
    // public async delta(path: string, options: any)
    fn list(&self, path: &str, options: &ListOptions) -> SyncResult<StatList>;
    fn get(&self, path: &str, options: &GetOptions) -> SyncResult<Option<String>>;
    fn mkdir(&self, path: &str) -> SyncResult<()>;
    fn put(&self, path: &str, options: &PutOptions) -> SyncResult<()>;
    fn multi_put(&self, items: &[MultiPutItem], options: &PutOptions) -> SyncResult<()>;
    fn delete(&self, path: &str) -> SyncResult<()>;
    fn r#move(&self, old_path: &str, new_path: &str) -> SyncResult<()>;
    fn clear_root(&self, base_dir: &str) -> SyncResult<()>;
    // public async acquireLock(type: LockType, clientType: LockClientType, clientId: string): Promise<Lock>
    // public async releaseLock(type: LockType, clientType: LockClientType, clientId: string)
    // public async listLocks()
}
