use crate::SyncError;

use super::{
    file_api_driver::{self, Source},
    FileApiDriver,
};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub struct FileApiDriverLocal {}

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

    fn stat(path: &str) -> crate::Result<super::file_api_driver::Stat> {
        todo!()
    }

    fn list(
        path: &str,
        options: &super::file_api_driver::Options,
    ) -> crate::Result<super::file_api_driver::StatList> {
        todo!()
    }

    fn get(
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

    fn mkdir(path: &str) -> crate::Result<()> {
        if Path::new(path).is_dir() {
            return Ok(());
        }
        Ok(fs::create_dir(path)?)
    }

    fn put(path: &str, options: &super::file_api_driver::Options) -> crate::Result<()> {
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
        _items: &[super::file_api_driver::MultiPutItem],
        _options: &super::file_api_driver::Options,
    ) -> crate::Result<()> {
        unimplemented!()
    }

    fn delete(path: &str) -> crate::Result<()> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(SyncError::FileNotExists);
        }
        Ok(fs::remove_file(path)?)
    }

    fn r#move(old_path: &str, new_path: &str) -> crate::Result<()> {
        Ok(fs::rename(old_path, new_path)?)
    }

    fn clear_root(base_dir: &str) -> crate::Result<()> {
        fs::remove_dir_all(base_dir)?;
        Ok(fs::create_dir(base_dir)?)
    }
}
