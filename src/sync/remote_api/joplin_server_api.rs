use reqwest::blocking::{Client, RequestBuilder, Response};
pub use reqwest::StatusCode;
use reqwest::{Error as ResError, Method};
use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::{DateTimeTimestamp, ModelType};

#[cfg(test)]
mod test_env {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct TestEnv {
        pub joplin_server: TestJoplinServerEnv,
    }

    #[derive(Deserialize)]
    pub struct TestJoplinServerEnv {
        pub host: String,
        pub session_id: String,
        pub email: String,
        pub password: String,
    }

    pub fn read_test_env() -> TestEnv {
        let env_toml = include_str!("../../../.test-env.toml");
        toml::from_str(env_toml).unwrap()
    }
}

pub type JoplinServerResult<T> = Result<T, JoplinServerError>;

#[derive(Error, Debug)]
pub enum JoplinServerError {
    #[error("response error")]
    ResError {
        text: String,
        status_code: StatusCode,
    },
    #[error("response inner error")]
    ResInnerError(#[from] ResError),
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

#[derive(Debug, Deserialize)]
pub struct DeltaItem {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub r#type: ModelType,
    pub updated_time: DateTimeTimestamp,
    pub jop_updated_time: DateTimeTimestamp,
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

    fn request_builder(&self, method: Method, path: &str) -> RequestBuilder {
        self.client
            .request(method, path)
            .header("X-API-AUTH", &self.session_id)
            .header("X-API-MIN-VERSION", "2.6.0")
    }

    pub fn login(host: &str, email: &str, password: &str) -> JoplinServerResult<Self> {
        let login_form = LoginForm { email, password };
        let host = host.to_string();
        let client = Client::new();
        let res = client
            .post(format!("{}/{}", host, "api/sessions"))
            .json(&login_form)
            .send()?;
        let res = Self::check_response(res)?;
        let login_result = res.json::<LoginResult>()?;
        Ok(Self {
            host,
            client,
            session_id: login_result.id,
        })
    }

    pub fn put_bytes(&self, path: &str, bytes: Vec<u8>) -> JoplinServerResult<PutResult> {
        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()?;
        let res = Self::check_response(res)?;
        Ok(res.json()?)
    }

    pub fn check_response(res: Response) -> JoplinServerResult<Response> {
        let status_code = res.status();
        if status_code.is_success() {
            return Ok(res);
        }
        Err(JoplinServerError::ResError {
            text: res.text().ok().unwrap_or_default(),
            status_code,
        })
    }

    pub fn put(&self, path: &str, s: String) -> JoplinServerResult<PutResult> {
        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(s)
            .send()?;
        let res = Self::check_response(res)?;
        Ok(res.json()?)
    }

    pub fn delete(&self, path: &str) -> JoplinServerResult<()> {
        let res = self
            .request_builder(Method::DELETE, &self.with_path(path))
            .send()?;
        Self::check_response(res)?;
        Ok(())
    }

    pub fn get(&self, path: &str) -> JoplinServerResult<Vec<u8>> {
        let res = self
            .request_builder(Method::GET, &format!("{}/content", self.with_path(path)))
            .send()?;
        let res = Self::check_response(res)?;
        Ok(res.bytes()?.to_vec())
    }

    pub fn metadata(&self, path: &str) -> JoplinServerResult<FileMetadata> {
        let res = self
            .request_builder(Method::GET, &self.with_path(path))
            .send()?;
        let res = Self::check_response(res)?;
        Ok(res.json()?)
    }

    pub fn root_delta(&self, cursor: Option<&str>) -> JoplinServerResult<DeltaResult> {
        self.delta("", cursor)
    }

    pub fn delta(&self, path: &str, cursor: Option<&str>) -> JoplinServerResult<DeltaResult> {
        let mut builder =
            self.request_builder(Method::GET, &format!("{}/delta", self.with_path(path)));
        if let Some(cursor) = cursor {
            builder = builder.query(&[("cursor", cursor)]);
        }
        let res = builder.send()?;
        let res = Self::check_response(res)?;
        Ok(res.json()?)
    }

    pub fn root_list(&self, cursor: Option<&str>) -> JoplinServerResult<ListResult> {
        self.list("", cursor)
    }

    pub fn list(&self, path: &str, cursor: Option<&str>) -> JoplinServerResult<ListResult> {
        let path = if path.is_empty() {
            path.to_string()
        } else {
            format!("{}/*", path)
        };
        let mut builder =
            self.request_builder(Method::GET, &format!("{}/children", self.with_path(&path)));
        if let Some(cursor) = cursor {
            builder = builder.query(&[("cursor", cursor)]);
        }
        let res = builder.send()?;
        let res = Self::check_response(res)?;
        Ok(res.json()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{sync::SerializeForSync, Folder, Note};

    use super::{test_env, JoplinServerAPI, JoplinServerResult};

    #[test]
    fn test_login() -> JoplinServerResult<()> {
        let test_config = test_env::read_test_env().joplin_server;
        let api =
            JoplinServerAPI::login(&test_config.host, &test_config.email, &test_config.password)?;
        assert!(!api.session_id.is_empty());
        println!("session id: {}", api.session_id);
        Ok(())
    }

    #[test]
    fn test_simple() -> JoplinServerResult<()> {
        let test_config = test_env::read_test_env().joplin_server;
        let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        let path = "testing.bin";
        let create_result = api.put_bytes(path, b"testing1".to_vec())?;
        let create_metadata = api.metadata(path)?;
        assert_eq!(b"testing1".to_vec(), api.get(path)?);
        let update_result = api.put_bytes(path, b"testing2".to_vec())?;
        assert_eq!(b"testing2".to_vec(), api.get(path)?);
        let update_metadata = api.metadata(path)?;
        assert!(update_result.created_time.is_none());
        assert_eq!(create_result.id, update_result.id);
        assert_eq!(create_result.name, update_result.name);
        assert_eq!(create_metadata.id, update_metadata.id);
        assert_eq!(create_metadata.name, update_metadata.name);
        assert_eq!(create_metadata.created_time, update_metadata.created_time);
        assert!(create_metadata.updated_time < update_metadata.updated_time);
        api.delete(path)?;
        Ok(())
    }

    #[test]
    fn test_delta() -> JoplinServerResult<()> {
        // let test_config = test_env::read_test_env().joplin_server;
        // let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        // let folder_1 = Folder::new("TestFolder1".to_string(), None);
        // let path_1 = format!("{}.md", folder_1.id.as_str());
        // let put_result_1 = api.put(&path_1, folder_1.serialize().unwrap().into_string())?;
        // println!("put_result_1 {:?}", put_result_1);
        // let folder_2 = Folder::new("TestFolder2".to_string(), None);
        // let path_2 = format!("{}.md", folder_2.id.as_str());
        // let put_result_2 = api.put(&path_2, folder_2.serialize().unwrap().into_string())?;
        // let delta_result = api.root_delta(Some(&put_result_1.id))?;
        // assert_eq!(1, delta_result.items.len());
        // assert_eq!(put_result_2.id, delta_result.items[0].id);
        // api.delete(&path_1)?;
        // api.delete(&path_2)?;
        Ok(())
    }

    #[test]
    fn test_list() -> JoplinServerResult<()> {
        let test_config = test_env::read_test_env().joplin_server;
        let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        let path = "test/test-list.md";
        api.put_bytes(path, b"testing1".to_vec())?;
        let list = api.root_list(None)?;
        assert!(!list.items.is_empty());
        let list = api.list("test", None)?;
        assert!(!list.items.is_empty());
        Ok(())
    }

    #[test]
    fn test_create_note() -> JoplinServerResult<()> {
        let test_config = test_env::read_test_env().joplin_server;
        let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        let test_folder = Folder::new("TestFolder".to_string(), None);
        let test_folder_path = test_folder.md_file_path();
        api.put(
            &test_folder_path,
            test_folder.serialize().unwrap().into_string(),
        )?;
        let test_note = Note::new(
            test_folder.id,
            "TestNote".to_string(),
            "# Test Title\n\n Content".to_string(),
        );
        let test_note_path = test_note.md_file_path();
        api.put(
            &test_note_path,
            test_note.serialize().unwrap().into_string(),
        )?;
        api.delete(&test_folder_path)?;
        api.delete(&test_note_path)?;
        Ok(())
    }
}
