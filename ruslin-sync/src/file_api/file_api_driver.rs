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

pub struct Options {
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
    fn stat(path: &str) -> Result<Stat>;
    // public async delta(path: string, options: any)
    fn list(path: &str, options: &Options) -> Result<StatList>;
    fn get(path: &str, options: &GetOptions) -> Result<Option<String>>;
    fn mkdir(path: &str) -> Result<()>;
    fn put(path: &str, options: &Options) -> Result<()>;
    fn multi_put(items: &[MultiPutItem], options: &Options) -> Result<()>;
    fn delete(path: &str) -> Result<()>;
    fn r#move(old_path: &str, new_path: &str) -> Result<()>;
    fn clear_root(base_dir: &str) -> Result<()>;
    // public async acquireLock(type: LockType, clientType: LockClientType, clientId: string): Promise<Lock>
    // public async releaseLock(type: LockType, clientType: LockClientType, clientId: string)
    // public async listLocks()
}
