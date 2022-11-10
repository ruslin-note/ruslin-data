mod file_api_driver;
mod file_api_driver_local;
mod file_api_driver_memory;

pub use file_api_driver::FileApiDriver;
pub use file_api_driver_local::FileApiDriverLocal;
pub use file_api_driver_memory::FileApiDriverMemory;

pub struct FileApi {}

impl FileApi {}
