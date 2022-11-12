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

#[test]
fn test_get_a_file_info() -> Result<()> {
    run_with_file_api(|file_api| {
        file_api.put("test1.txt", "testing")?;
        file_api.mkdir("sub")?;
        file_api.put("sub/test2.txt", "testing")?;

        let stat = file_api.stat("test1.txt")?;
        assert_eq!("test1.txt", stat.path);
        assert!(stat.updated_time > 0);
        assert!(!stat.is_dir);

        let stat = file_api.stat("sub/test2.txt")?;
        assert_eq!("sub/test2.txt", stat.path);
        assert!(stat.updated_time > 0);
        assert!(!stat.is_dir);

        Ok(())
    })
}
