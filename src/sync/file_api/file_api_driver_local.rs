use async_trait::async_trait;

use crate::sync::{SyncError, SyncResult};

use super::{
    file_api_driver::{Stat, StatList},
    FileApiDriver, SyncContext,
};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

#[derive(Debug, Default)]
pub struct FileApiDriverLocal {}

impl FileApiDriverLocal {
    pub fn new() -> Self {
        FileApiDriverLocal {}
    }
}

#[async_trait]
impl FileApiDriver for FileApiDriverLocal {
    fn supports_multi_put(&self) -> bool {
        false
    }

    fn supports_accurate_timestamp(&self) -> bool {
        false
    }

    fn supports_locks(&self) -> bool {
        false
    }

    fn request_repeat_count(&self) -> u32 {
        todo!()
    }

    async fn stat(&self, path: &str) -> SyncResult<Option<Stat>> {
        let metadata = fs::metadata(path)?;
        metadata.try_into().map(Some)
    }

    async fn list(&self, path: &str) -> SyncResult<StatList> {
        let mut stats: Vec<Stat> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let mut stat: Stat = entry.metadata()?.try_into()?;
            stat.path = entry
                .file_name()
                .to_str()
                .unwrap_or_else(|| panic!("unwrap error in {}:{}", file!(), line!()))
                .to_string();
            stats.push(stat);
        }
        Ok(StatList {
            items: stats,
            has_more: false,
            context: None,
        })
    }

    async fn get(&self, path: &str) -> SyncResult<String> {
        Ok(fs::read_to_string(path)?)
    }

    async fn mkdir(&self, path: &str) -> SyncResult<()> {
        if Path::new(path).is_dir() {
            return Ok(());
        }
        Ok(fs::create_dir(path)?)
    }

    async fn put(&self, path: &str, content: &str) -> SyncResult<()> {
        let mut file = File::create(path)?;
        write!(&mut file, "{}", content)?;
        Ok(())
    }

    async fn multi_put(&self, _items: &[super::file_api_driver::MultiPutItem]) -> SyncResult<()> {
        unimplemented!()
    }

    async fn delete(&self, path: &str) -> SyncResult<()> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(SyncError::FileNotExists);
        }
        Ok(fs::remove_file(path)?)
    }

    async fn r#move(&self, old_path: &str, new_path: &str) -> SyncResult<()> {
        Ok(fs::rename(old_path, new_path)?)
    }

    async fn delta(
        &self,
        _path: &str,
        _ctx: Option<&dyn SyncContext>,
    ) -> SyncResult<super::file_api_driver::DeltaList> {
        todo!()
    }

    fn deserializer_delta_context(&self, _s: &str) -> SyncResult<Box<dyn SyncContext>> {
        todo!()
    }

    async fn clear_root(&self, base_dir: &str) -> SyncResult<()> {
        fs::remove_dir_all(base_dir)?;
        Ok(fs::create_dir(base_dir)?)
    }

    async fn check_config(&self) -> SyncResult<()> {
        Ok(())
    }
}
