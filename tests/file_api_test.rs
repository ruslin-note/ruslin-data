use std::ops::Deref;

use ruslin_data::{
    sync::{FileApi, FileApiDriverLocal, SyncResult},
    DateTimeTimestamp,
};
use tempfile::TempDir;

pub struct TestFileApiDriverLocal(FileApi<FileApiDriverLocal>, TempDir);

impl TestFileApiDriverLocal {
    pub async fn temp() -> Self {
        let temp_dir =
            tempfile::TempDir::new().expect(&format!("unwrap error in {}:{}", file!(), line!()));
        let base_dir =
            temp_dir
                .path()
                .to_str()
                .expect(&format!("unwrap error in {}:{}", file!(), line!()));
        let file_api = FileApi::new(base_dir, FileApiDriverLocal::new());
        file_api
            .clear_root()
            .await
            .expect(&format!("unwrap error in {}:{}", file!(), line!()));
        TestFileApiDriverLocal(file_api, temp_dir)
    }
}

impl Deref for TestFileApiDriverLocal {
    type Target = FileApi<FileApiDriverLocal>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tokio::test]
async fn test_create_a_file() -> SyncResult<()> {
    let file_api = TestFileApiDriverLocal::temp().await;
    file_api.put("test.txt", "testing").await?;
    assert_eq!("testing", file_api.get("test.txt").await?);
    Ok(())
}

#[tokio::test]
async fn test_get_a_file_info() -> SyncResult<()> {
    let file_api = TestFileApiDriverLocal::temp().await;
    file_api.put("test1.txt", "testing").await?;
    file_api.mkdir("sub").await?;
    file_api.put("sub/test2.txt", "testing").await?;

    let stat = file_api.stat("test1.txt").await?;
    assert_eq!("test1.txt", stat.path);
    assert!(stat.updated_time > DateTimeTimestamp::zero());
    assert!(!stat.is_dir);

    let stat = file_api.stat("sub/test2.txt").await?;
    assert_eq!("sub/test2.txt", stat.path);
    assert!(stat.updated_time > DateTimeTimestamp::zero());
    assert!(!stat.is_dir);

    Ok(())
}

#[tokio::test]
async fn test_create_a_file_in_a_subdirectory() -> SyncResult<()> {
    let file_api = TestFileApiDriverLocal::temp().await;
    file_api.mkdir("subdir").await?;
    file_api.put("subdir/test.txt", "testing").await?;
    let content = file_api.get("subdir/test.txt").await?;
    assert_eq!("testing", content);
    Ok(())
}

#[tokio::test]
async fn test_list_files() -> SyncResult<()> {
    let file_api = TestFileApiDriverLocal::temp().await;
    file_api.mkdir("subdir").await?;
    file_api.put("subdir/test1.txt", "testing1").await?;
    file_api.put("subdir/test2.txt", "testing2").await?;
    let files = file_api.list("subdir").await?;
    assert_eq!(2, files.items.len());
    let mut paths: Vec<String> = files.items.into_iter().map(|s| s.path).collect();
    paths.sort();
    assert_eq!("test1.txt", paths[0]);
    assert_eq!("test2.txt", paths[1]);
    Ok(())
}

#[tokio::test]
async fn test_delete_a_file() -> SyncResult<()> {
    let file_api = TestFileApiDriverLocal::temp().await;
    file_api.put("test1.txt", "testing1").await?;
    assert_eq!(1, file_api.list("").await?.items.len());
    file_api.delete("test1.txt").await?;
    let files = file_api.list("").await?;
    assert!(files.items.is_empty());
    Ok(())
}
