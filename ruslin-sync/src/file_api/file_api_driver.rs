use crate::Result;

pub struct Stat {
    path: String,
    updated_time: i64,
    jop_updated_time: i64,
    is_dir: bool,
    is_deleted: bool,
}

pub struct StatList {
    items: Vec<Stat>,
    has_more: bool,
    // context:
}

pub enum Source {
    File(String),
    Text(String),
}

pub struct PutOptions {
    pub source: Source,
}

pub enum GetTarget {
    File(String),
    Text,
}

pub struct GetOptions {
    pub target: GetTarget,
}

pub struct MultiPutItem {
    name: String,
    body: String,
}

pub trait FileApiDriver {
    fn supports_multi_put(&self) -> bool;
    fn supports_accurate_timestamp(&self) -> bool;
    fn supports_locks(&self) -> bool;
    fn request_repeat_count(&self) -> u32;
    fn stat(&self, path: &str) -> Result<Stat>;
    // public async delta(path: string, options: any)
    fn list(&self, path: &str, options: &PutOptions) -> Result<StatList>;
    fn get(&self, path: &str, options: &GetOptions) -> Result<Option<String>>;
    fn mkdir(&self, path: &str) -> Result<()>;
    fn put(&self, path: &str, options: &PutOptions) -> Result<()>;
    fn multi_put(&self, items: &[MultiPutItem], options: &PutOptions) -> Result<()>;
    fn delete(&self, path: &str) -> Result<()>;
    fn r#move(&self, old_path: &str, new_path: &str) -> Result<()>;
    fn clear_root(&self, base_dir: &str) -> Result<()>;
    // public async acquireLock(type: LockType, clientType: LockClientType, clientId: string): Promise<Lock>
    // public async releaseLock(type: LockType, clientType: LockClientType, clientId: string)
    // public async listLocks()
}
