mod file_api_driver;
mod file_api_driver_joplin_server;
mod file_api_driver_local;

use std::path::{Path, PathBuf};

pub use file_api_driver::{FileApiDriver, Stat, StatList, SyncContext};
pub use file_api_driver_joplin_server::FileApiDriverJoplinServer;
pub use file_api_driver_local::FileApiDriverLocal;

use super::SyncResult;

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
    pub async fn put(&self, path: &str, content: &str) -> SyncResult<()> {
        self.driver.put(&self.full_path(path), content).await
    }

    // get(path: string, options: any = null)
    pub async fn get(&self, path: &str) -> SyncResult<String> {
        self.driver.get(&self.full_path(path)).await
    }

    pub async fn mkdir(&self, path: &str) -> SyncResult<()> {
        let path = self.full_path(path);
        self.driver.mkdir(&path).await
    }

    pub async fn stat(&self, path: &str) -> SyncResult<Stat> {
        let mut stat = self.driver.stat(&self.full_path(path)).await?;
        if let Some(stat) = &mut stat {
            stat.path = path.to_string();
        }
        Ok(stat.unwrap())
    }

    pub async fn list(&self, path: &str) -> SyncResult<StatList> {
        self.driver.list(&self.full_path(path)).await
    }

    pub async fn clear_root(&self) -> SyncResult<()> {
        self.driver
            .clear_root(self.base_dir.to_str().unwrap())
            .await
    }

    pub async fn delete(&self, path: &str) -> SyncResult<()> {
        self.driver.delete(&self.full_path(path)).await
    }

    fn full_path(&self, path: &str) -> String {
        self.base_dir.join(path).to_str().unwrap().to_string()
    }
}
