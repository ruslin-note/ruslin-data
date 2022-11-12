use super::file_api_driver::FileApiDriver;

pub struct FileApiDriverMemory {}

impl FileApiDriver for FileApiDriverMemory {
    fn supports_multi_put(&self) -> bool {
        true
    }

    fn supports_accurate_timestamp(&self) -> bool {
        true
    }

    fn supports_locks(&self) -> bool {
        false
    }

    fn request_repeat_count(&self) -> u32 {
        todo!()
    }

    fn stat(&self, path: &str) -> crate::Result<super::file_api_driver::Stat> {
        todo!()
    }

    fn list(
        &self,
        path: &str,
        options: &super::file_api_driver::PutOptions,
    ) -> crate::Result<super::file_api_driver::StatList> {
        todo!()
    }

    fn get(
        &self,
        path: &str,
        options: &super::file_api_driver::GetOptions,
    ) -> crate::Result<Option<String>> {
        todo!()
    }

    fn mkdir(&self, path: &str) -> crate::Result<()> {
        todo!()
    }

    fn put(&self, path: &str, options: &super::file_api_driver::PutOptions) -> crate::Result<()> {
        todo!()
    }

    fn multi_put(
        &self,
        items: &[super::file_api_driver::MultiPutItem],
        options: &super::file_api_driver::PutOptions,
    ) -> crate::Result<()> {
        todo!()
    }

    fn delete(&self, path: &str) -> crate::Result<()> {
        todo!()
    }

    fn r#move(&self, old_path: &str, new_path: &str) -> crate::Result<()> {
        todo!()
    }

    fn clear_root(&self, base_dir: &str) -> crate::Result<()> {
        todo!()
    }
}
