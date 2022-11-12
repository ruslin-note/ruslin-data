use ruslin_sync::{FileApi, FileApiDriver, FileApiDriverLocal, Result};
use tempfile::tempdir;

// fn get_local_file_api() -> FileApi<FileApiDriverLocal> {

// }

fn run_with_file_api(run: impl FnOnce(FileApi<FileApiDriverLocal>) -> Result<()>) -> Result<()> {
    let dir = tempdir().unwrap();
    let base_dir = dir.path().to_str().unwrap();
    let file_api = FileApi::new(base_dir, FileApiDriverLocal::new());
    file_api.clear_root()?;
    run(file_api)
}

#[test]
fn test_create_a_file() -> Result<()> {
    run_with_file_api(|file_api| {
        file_api.put("test.txt", "testing")?;
        assert_eq!("testing", file_api.get("test.txt")?.unwrap());
        Ok(())
    })
}
