use ruslin_data::sync::{FileApi, FileApiDriverLocal, SyncResult};
use tempfile::tempdir;

fn run_with_file_api(
    run: impl FnOnce(FileApi<FileApiDriverLocal>) -> SyncResult<()>,
) -> SyncResult<()> {
    let dir = tempdir().unwrap();
    let base_dir = dir.path().to_str().unwrap();
    let file_api = FileApi::new(base_dir, FileApiDriverLocal::new());
    file_api.clear_root()?;
    run(file_api)
}

#[test]
fn test_create_a_file() -> SyncResult<()> {
    run_with_file_api(|file_api| {
        file_api.put("test.txt", "testing")?;
        assert_eq!("testing", file_api.get("test.txt")?.unwrap());
        Ok(())
    })
}

#[test]
fn test_get_a_file_info() -> SyncResult<()> {
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

#[test]
fn test_create_a_file_in_a_subdirectory() -> SyncResult<()> {
    run_with_file_api(|file_api| {
        file_api.mkdir("subdir")?;
        file_api.put("subdir/test.txt", "testing")?;
        let content = file_api.get("subdir/test.txt")?;
        assert_eq!("testing", content.unwrap());
        Ok(())
    })
}

#[test]
fn test_list_files() -> SyncResult<()> {
    run_with_file_api(|file_api| {
        file_api.mkdir("subdir")?;
        file_api.put("subdir/test1.txt", "testing1")?;
        file_api.put("subdir/test2.txt", "testing2")?;
        let files = file_api.list("subdir")?;
        assert_eq!(2, files.items.len());
        let mut paths: Vec<String> = files.items.into_iter().map(|s| s.path).collect();
        paths.sort();
        assert_eq!("test1.txt", paths[0]);
        assert_eq!("test2.txt", paths[1]);
        Ok(())
    })
}

#[test]
fn test_delete_a_file() -> SyncResult<()> {
    run_with_file_api(|file_api| {
        file_api.put("test1.txt", "testing1")?;
        assert_eq!(1, file_api.list("")?.items.len());
        file_api.delete("test1.txt")?;
        let files = file_api.list("")?;
        assert!(files.items.is_empty());
        Ok(())
    })
}
