mod file_api_driver;
mod file_api_driver_local;
mod file_api_driver_memory;

use std::path::{Path, PathBuf};

pub use file_api_driver::FileApiDriver;
pub use file_api_driver_local::FileApiDriverLocal;
pub use file_api_driver_memory::FileApiDriverMemory;

use self::file_api_driver::{GetOptions, PutOptions, Source, Stat};
use crate::Result;

pub struct FileApi<D: FileApiDriver> {
    pub base_dir: PathBuf,
    pub driver: D,
}

impl<D: FileApiDriver> FileApi<D> {
    pub fn new(base_dir: &str, driver: D) -> Self {
        Self {
            base_dir: Path::new(base_dir).to_path_buf(),
            driver,
        }
    }

    // async put(path: string, content: any, options: any = null)
    pub fn put(&self, path: &str, content: &str) -> Result<()> {
        self.driver.put(
            &self.full_path(path),
            &PutOptions {
                source: Source::Text(content.to_string()),
            },
        )
    }

    /// Returns UTF-8 encoded string by default, or a Response if `options.target = 'file'`
    // get(path: string, options: any = null)
    pub fn get(&self, path: &str) -> Result<Option<String>> {
        self.driver.get(
            &self.full_path(path),
            &GetOptions {
                target: file_api_driver::GetTarget::Text,
            },
        )
    }

    pub fn mkdir(&self, path: &str) -> Result<()> {
        let path = self.full_path(path);
        log::debug!("mkdir {path}");
        self.driver.mkdir(&path)
    }

    pub fn stat(&self, path: &str) -> Result<Stat> {
        let mut stat = self.driver.stat(&self.full_path(path))?;
        stat.path = path.to_string();
        Ok(stat)
    }

    pub fn clear_root(&self) -> Result<()> {
        self.driver.clear_root(self.base_dir.to_str().unwrap())
    }

    fn full_path(&self, path: &str) -> String {
        self.base_dir.join(path).to_str().unwrap().to_string()
    }
}
