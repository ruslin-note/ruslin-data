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

    fn stat(&self, _path: &str) -> crate::Result<super::file_api_driver::Stat> {
        todo!()
    }

    fn list(
        &self,
        _path: &str,
        _options: &super::file_api_driver::PutOptions,
    ) -> crate::Result<super::file_api_driver::StatList> {
        todo!()
    }

    fn get(
        &self,
        _path: &str,
        _options: &super::file_api_driver::GetOptions,
    ) -> crate::Result<Option<String>> {
        todo!()
    }

    fn mkdir(&self, _path: &str) -> crate::Result<()> {
        todo!()
    }

    fn put(&self, _path: &str, _options: &super::file_api_driver::PutOptions) -> crate::Result<()> {
        todo!()
    }

    fn multi_put(
        &self,
        _items: &[super::file_api_driver::MultiPutItem],
        _options: &super::file_api_driver::PutOptions,
    ) -> crate::Result<()> {
        todo!()
    }

    fn delete(&self, _path: &str) -> crate::Result<()> {
        todo!()
    }

    fn r#move(&self, _old_path: &str, _new_path: &str) -> crate::Result<()> {
        todo!()
    }

    fn clear_root(&self, _base_dir: &str) -> crate::Result<()> {
        todo!()
    }
}
