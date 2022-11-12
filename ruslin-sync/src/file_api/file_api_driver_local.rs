use crate::{Result, SyncError};

use super::{
    file_api_driver::{self, ListOptions, Source, Stat, StatList},
    FileApiDriver,
};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub struct FileApiDriverLocal {}

impl FileApiDriverLocal {
    pub fn new() -> Self {
        FileApiDriverLocal {}
    }
}

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

    fn stat(&self, path: &str) -> crate::Result<Stat> {
        let metadata = fs::metadata(path)?;
        metadata.try_into()
    }

    fn list(&self, path: &str, _options: &ListOptions) -> Result<StatList> {
        let mut stats: Vec<Stat> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let mut stat: Stat = entry.metadata()?.try_into()?;
            stat.path = entry.file_name().to_str().unwrap().to_string();
            stats.push(stat);
        }
        Ok(StatList {
            items: stats,
            has_more: false,
        })
    }

    fn get(
        &self,
        path: &str,
        options: &super::file_api_driver::GetOptions,
    ) -> crate::Result<Option<String>> {
        use file_api_driver::GetTarget;
        match &options.target {
            GetTarget::File(target_path) => {
                fs::copy(path, target_path)?;
                Ok(None)
            }
            GetTarget::Text => Ok(Some(fs::read_to_string(path)?)),
        }
    }

    fn mkdir(&self, path: &str) -> crate::Result<()> {
        if Path::new(path).is_dir() {
            return Ok(());
        }
        Ok(fs::create_dir(path)?)
    }

    fn put(&self, path: &str, options: &super::file_api_driver::PutOptions) -> crate::Result<()> {
        match &options.source {
            Source::File(from_path) => {
                fs::copy(from_path, path)?;
            }
            Source::Text(content) => {
                let mut file = File::create(path)?;
                write!(&mut file, "{}", content)?;
            }
        }
        Ok(())
    }

    fn multi_put(
        &self,
        _items: &[super::file_api_driver::MultiPutItem],
        _options: &super::file_api_driver::PutOptions,
    ) -> crate::Result<()> {
        unimplemented!()
    }

    fn delete(&self, path: &str) -> crate::Result<()> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(SyncError::FileNotExists);
        }
        Ok(fs::remove_file(path)?)
    }

    fn r#move(&self, old_path: &str, new_path: &str) -> crate::Result<()> {
        Ok(fs::rename(old_path, new_path)?)
    }

    fn clear_root(&self, base_dir: &str) -> crate::Result<()> {
        fs::remove_dir_all(base_dir)?;
        Ok(fs::create_dir(base_dir)?)
    }
}
