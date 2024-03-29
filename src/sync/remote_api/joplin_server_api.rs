use std::path::Path;

pub use reqwest::StatusCode;
use reqwest::{Body, Client, RequestBuilder, Response};
use reqwest::{Error as ResError, Method};
use serde::{Deserialize, Serialize};

use serde_json::json;
use serde_repr::{Deserialize_repr, Serialize_repr};
use thiserror::Error;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::sync::lock_handler::{Lock, LockClientType, LockList, LockType};
use crate::{sync::SyncError, DateTimeTimestamp};

pub type JoplinServerResult<T> = Result<T, JoplinServerError>;

#[derive(Error, Debug)]
pub enum JoplinServerError {
    #[error("response error {status_code} {text}")]
    ResponseError {
        text: String,
        status_code: StatusCode,
    },
    #[error("{status_code}:{api_error}")]
    APIError {
        status_code: StatusCode,
        api_error: JoplinAPIError,
    },
    #[error("response inner error {0}")]
    ResponseInnerError(#[from] ResError),
    #[error("io {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug, Deserialize)]
#[error("{error}")]
pub struct JoplinAPIError {
    pub code: Option<String>, // enum?
    pub error: String,
}

const RESYNC_REQUIRED_CODE: &str = "resyncRequired";

impl From<JoplinServerError> for SyncError {
    fn from(err: JoplinServerError) -> Self {
        match &err {
            JoplinServerError::ResponseError { text, status_code } => {
                if *status_code == StatusCode::NOT_FOUND {
                    return Self::FileNotExists(text.to_string());
                }
            }
            JoplinServerError::APIError {
                status_code,
                api_error,
            } => {
                if *status_code == StatusCode::NOT_FOUND {
                    return Self::FileNotExists(api_error.error.to_string());
                }
            }
            JoplinServerError::ResponseInnerError(_) | JoplinServerError::IoError(_) => (),
        };
        Self::APIError(Box::new(err))
    }
}

#[derive(Debug, Serialize)]
struct LoginForm<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct LoginResult {
    pub id: String,
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct PutResult {
    pub id: String,
    pub name: String,
    pub updated_time: DateTimeTimestamp,
    pub created_time: Option<DateTimeTimestamp>,
}

#[derive(Debug, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub updated_time: DateTimeTimestamp,
    pub created_time: DateTimeTimestamp,
}

#[derive(Debug, Deserialize_repr, Serialize_repr, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum ChangeType {
    Create = 1,
    Update = 2,
    Delete = 3,
}

impl ChangeType {
    pub fn is_deleted(self) -> bool {
        self == ChangeType::Delete
    }
}

#[derive(Debug, Deserialize)]
pub struct DeltaItem {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub r#type: ChangeType,
    pub updated_time: DateTimeTimestamp,
    pub jop_updated_time: Option<DateTimeTimestamp>,
}

#[derive(Debug, Deserialize)]
pub struct DeltaResult {
    pub items: Vec<DeltaItem>,
    pub cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Deserialize)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub updated_time: DateTimeTimestamp,
}

#[derive(Debug, Deserialize)]
pub struct ListResult {
    pub items: Vec<FileItem>,
    pub cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug)]
pub struct JoplinServerAPI {
    host: String,
    client: Client,
    session_id: String,
}

impl JoplinServerAPI {
    pub fn new(host: &str, session_id: &str) -> Self {
        let client = Client::new();
        Self {
            host: host.to_string(),
            client,
            session_id: session_id.to_string(),
        }
    }

    fn with_path(&self, path: &str) -> String {
        format!("{}/api/items/root:/{}:", self.host, path)
    }

    fn with_api(&self, path: &str) -> String {
        format!("{}/api/{}", self.host, path)
    }

    fn request_builder(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, path)
            .header("X-API-AUTH", &self.session_id)
            .header("X-API-MIN-VERSION", "2.6.0")
    }

    pub async fn login(host: &str, email: &str, password: &str) -> JoplinServerResult<Self> {
        let login_form = LoginForm { email, password };
        let host = host.to_string();
        let client = Client::new();
        let res = client
            .post(format!("{}/{}", host, "api/sessions"))
            .json(&login_form)
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        let login_result = res.json::<LoginResult>().await?;
        Ok(Self {
            host,
            client,
            session_id: login_result.id,
        })
    }

    pub async fn put_bytes(&self, path: &str, bytes: Vec<u8>) -> JoplinServerResult<PutResult> {
        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn put_file(
        &self,
        path: &str,
        local_file_path: &Path,
    ) -> JoplinServerResult<PutResult> {
        // https://stackoverflow.com/questions/65814450/how-to-post-a-file-using-reqwest
        // https://github.com/tokio-rs/tokio/discussions/4264
        let file = File::open(local_file_path).await.unwrap();
        let reader_stream = ReaderStream::new(file);
        let body = Body::wrap_stream(reader_stream);

        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(body)
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn check_response(res: Response) -> JoplinServerResult<Response> {
        let status_code = res.status();
        if status_code.is_success() {
            Ok(res)
        } else {
            let api_error: JoplinAPIError = res.json().await?;
            Err(JoplinServerError::APIError {
                status_code,
                api_error,
            })
        }
    }

    pub async fn put_text(
        &self,
        path: &str,
        s: impl Into<String>,
    ) -> JoplinServerResult<PutResult> {
        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(Into::<String>::into(s))
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn delete(&self, path: &str) -> JoplinServerResult<()> {
        let res = self
            .request_builder(Method::DELETE, &self.with_path(path))
            .send()
            .await?;
        Self::check_response(res).await?;
        Ok(())
    }

    pub async fn get_file(&self, path: &str, destination: &Path) -> JoplinServerResult<()> {
        // https://gist.github.com/giuliano-oliveira/4d11d6b3bb003dba3a1b53f43d81b30d
        // https://users.rust-lang.org/t/async-download-very-large-files/79621
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let res = self
            .request_builder(Method::GET, &format!("{}/content", self.with_path(path)))
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        let mut file = tokio::fs::File::create(destination).await?;

        async fn _get_file(res: Response, file: &mut File) -> JoplinServerResult<()> {
            let mut stream = res.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk).await?;
            }
            file.flush().await?;
            Ok(())
        }

        let result = _get_file(res, &mut file).await;
        if result.is_err() {
            tokio::fs::remove_file(destination).await?;
        }

        result
    }

    pub async fn get_text(&self, path: &str) -> JoplinServerResult<String> {
        let res = self
            .request_builder(Method::GET, &format!("{}/content", self.with_path(path)))
            .send()
            .await?;
        let res = Self::check_response(res).await?;
        Ok(res.text().await?)
    }

    pub async fn metadata(&self, path: &str) -> JoplinServerResult<Option<FileMetadata>> {
        let res = self
            .request_builder(Method::GET, &self.with_path(path))
            .send()
            .await?;
        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn root_delta(&self, cursor: Option<&str>) -> JoplinServerResult<DeltaResult> {
        self.delta("", cursor).await
    }

    async fn _delta(&self, path: &str, cursor: Option<&str>) -> JoplinServerResult<DeltaResult> {
        let mut builder =
            self.request_builder(Method::GET, &format!("{}/delta", self.with_path(path)));
        if let Some(cursor) = cursor {
            builder = builder.query(&[("cursor", cursor)]);
        }
        let res = builder.send().await?;
        let res = Self::check_response(res).await?;
        let mut delta_result: DeltaResult = res.json().await?;
        delta_result.items.retain(|item| {
            if item.item_name.starts_with("locks/") {
                return false;
            }
            if item.item_name.starts_with("temp/") {
                return false;
            }
            if item.item_name.starts_with(".resource/") {
                return false;
            }
            true
        });
        Ok(delta_result)
    }

    pub async fn delta(&self, path: &str, cursor: Option<&str>) -> JoplinServerResult<DeltaResult> {
        let delta_result = self._delta(path, cursor).await;
        match delta_result {
            Ok(delta) => Ok(delta),
            Err(e) => match &e {
                JoplinServerError::APIError { api_error, .. } => {
                    if api_error.code.as_deref() == Some(RESYNC_REQUIRED_CODE) {
                        self._delta(path, None).await
                    } else {
                        Err(e)
                    }
                }
                _ => Err(e),
            },
        }
    }

    pub async fn root_list(&self, cursor: Option<&str>) -> JoplinServerResult<ListResult> {
        self.list("", cursor).await
    }

    pub async fn list(&self, path: &str, cursor: Option<&str>) -> JoplinServerResult<ListResult> {
        let path = if path.is_empty() {
            path.to_string()
        } else {
            format!("{path}/*")
        };
        let mut builder =
            self.request_builder(Method::GET, &format!("{}/children", self.with_path(&path)));
        if let Some(cursor) = cursor {
            builder = builder.query(&[("cursor", cursor)]);
        }
        let res = builder.send().await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn acquire_lock(
        &self,
        r#type: LockType,
        client_type: LockClientType,
        client_id: &str,
    ) -> JoplinServerResult<Lock> {
        let mut builder = self.request_builder(Method::POST, &self.with_api("locks"));
        builder = builder.json(&json!({
            "type": r#type,
            "clientType": client_type,
            "clientId": client_id,
        }));
        let res = builder.send().await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn release_lock(
        &self,
        r#type: LockType,
        client_type: LockClientType,
        client_id: &str,
    ) -> JoplinServerResult<()> {
        let builder = self.request_builder(
            Method::DELETE,
            &self.with_api(&format!(
                "locks/{}_{}_{}",
                r#type as u8, client_type as u8, client_id
            )),
        );
        let res = builder.send().await?;
        Self::check_response(res).await?;
        Ok(())
    }

    pub async fn list_locks(&self) -> JoplinServerResult<LockList> {
        let builder = self.request_builder(Method::GET, &self.with_api("locks"));
        let res = builder.send().await?;
        let res = Self::check_response(res).await?;
        Ok(res.json().await?)
    }

    pub async fn clear_root(&self) -> JoplinServerResult<()> {
        let list = self.root_list(None).await?;
        for item in list.items {
            self.delete(&item.name).await?;
        }
        Ok(())
    }
}

#[cfg(debug_assertions)]
pub mod test_api {
    use crate::sync::SyncConfig;

    use super::JoplinServerAPI;

    #[derive(Debug, Clone, Copy)]
    #[repr(i32)]
    pub enum TestSyncClient {
        Default = 1,
        Basic1,
        Basic2,
        Conflict1,
        SyncTargetInfo,
        Upload1,
        Upload2,
        UpdateNoteItem,
    }

    impl TestSyncClient {
        pub async fn login(&self) -> JoplinServerAPI {
            let host = "http://localhost:22300";
            let user_num = *self as i32;
            let email = format!("user{user_num}@example.com");
            let password = "111111";
            JoplinServerAPI::login(host, &email, password)
                .await
                .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()))
        }

        pub fn sync_config(&self) -> SyncConfig {
            let user_num = *self as i32;
            SyncConfig::JoplinServer {
                host: "http://localhost:22300".to_string(),
                email: format!("user{user_num}@example.com"),
                password: "111111".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        sync::{
            lock_handler::{LockClientType, LockType},
            SerializeForSync,
        },
        Folder, Note,
    };
    use std::{
        fs::{self, File},
        io::Write,
    };

    use super::{test_api::TestSyncClient, JoplinServerResult};

    #[tokio::test]
    async fn test_clear_root() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        api.clear_root().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_login() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        assert!(!api.session_id.is_empty());
        println!("session id: {}", api.session_id);
        Ok(())
    }

    #[tokio::test]
    async fn test_simple() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        let path = "testing.bin";
        let create_result = api.put_text(path, "testing1").await?;
        let create_metadata = api
            .metadata(path)
            .await?
            .unwrap_or_else(|| panic!("unwrap error in {}:{}", file!(), line!()));
        assert_eq!("testing1", api.get_text(path).await?);
        let update_result = api.put_bytes(path, b"testing2".to_vec()).await?;
        assert_eq!("testing2", api.get_text(path).await?);
        let update_metadata = api
            .metadata(path)
            .await?
            .unwrap_or_else(|| panic!("unwrap error in {}:{}", file!(), line!()));
        assert!(update_result.created_time.is_none());
        assert_eq!(create_result.id, update_result.id);
        assert_eq!(create_result.name, update_result.name);
        assert_eq!(create_metadata.id, update_metadata.id);
        assert_eq!(create_metadata.name, update_metadata.name);
        assert_eq!(create_metadata.created_time, update_metadata.created_time);
        assert!(create_metadata.updated_time < update_metadata.updated_time);
        api.delete(path).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_file() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        let remote_path = ".tests-bin/file.bin";

        let temp_dir = tempfile::TempDir::new().unwrap();
        let upload_file_path = temp_dir.path().join("upload_file.bin");
        let mut upload_file = File::create(&upload_file_path).unwrap();
        upload_file.write_all(b"file").unwrap();
        api.put_file(remote_path, &upload_file_path).await?;
        let download_file_path = temp_dir.path().join("download_file.bin");
        api.get_file(remote_path, &download_file_path).await?;
        assert!(download_file_path.exists());
        assert_eq!(b"file" as &[u8], &fs::read(download_file_path).unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn test_delta() -> JoplinServerResult<()> {
        // let test_config = test_env::read_test_env().joplin_server;
        // let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        // let folder_1 = Folder::new("TestFolder1".to_string(), None);
        // let path_1 = format!("{}.md", folder_1.id.as_str());
        // let put_result_1 = api.put(&path_1, folder_1.serialize().expect(&format!("unwrap error in {}:{}", file!(), line!())).into_string())?;
        // println!("put_result_1 {:?}", put_result_1);
        // let folder_2 = Folder::new("TestFolder2".to_string(), None);
        // let path_2 = format!("{}.md", folder_2.id.as_str());
        // let put_result_2 = api.put(&path_2, folder_2.serialize().expect(&format!("unwrap error in {}:{}", file!(), line!())).into_string())?;
        // let delta_result = api.root_delta(Some(&put_result_1.id))?;
        // assert_eq!(1, delta_result.items.len());
        // assert_eq!(put_result_2.id, delta_result.items[0].id);
        // api.delete(&path_1)?;
        // api.delete(&path_2)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_delta_invalid_cursor() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        api.delta("", Some("invalid")).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_list() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        let path = "test/test-list.md";
        api.put_bytes(path, b"testing1".to_vec()).await?;
        let list = api.root_list(None).await?;
        assert!(!list.items.is_empty());
        let list = api.list("test", None).await?;
        assert!(!list.items.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_create_note() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        let test_folder = Folder::new("TestFolder".to_string(), None);
        let test_folder_path = test_folder.md_file_path();
        api.put_text(&test_folder_path, test_folder.serialize().into_string())
            .await?;
        let test_note = Note::new(
            Some(test_folder.id),
            "TestNote".to_string(),
            "# Test Title\n\n Content".to_string(),
        );
        let test_note_path = test_note.md_file_path();
        api.put_text(&test_note_path, test_note.serialize().into_string())
            .await?;
        api.delete(&test_folder_path).await?;
        api.delete(&test_note_path).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_lock() -> JoplinServerResult<()> {
        let api = TestSyncClient::Default.login().await;
        let lock = api
            .acquire_lock(LockType::Sync, LockClientType::Cli, "test")
            .await?;
        assert_eq!("test", lock.client_id);
        let locks = api.list_locks().await?;
        assert!(!locks.items.is_empty());
        assert_eq!(lock, locks.items[0]);
        api.release_lock(lock.r#type, lock.client_type, &lock.client_id)
            .await?;
        let locks = api.list_locks().await?;
        assert!(locks.items.is_empty());
        Ok(())
    }
}
