use reqwest::blocking::{Client, RequestBuilder};
use reqwest::{Error as ResError, Method};
use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::{DateTime, ModelType};

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
    }

    pub fn read_test_env() -> TestEnv {
        let env_toml = include_str!("../../../.test-env.toml");
        toml::from_str(env_toml).unwrap()
    }
}

pub type JoplinServerResult<T> = Result<T, JoplinServerError>;

#[derive(Error, Debug)]
pub enum JoplinServerError {
    #[error("res error")]
    ResError(#[from] ResError),
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
    pub updated_time: DateTime,
    pub created_time: Option<DateTime>,
}

#[derive(Debug, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub name: String,
    pub updated_time: DateTime,
    pub created_time: DateTime,
}

#[derive(Debug, Deserialize)]
pub struct DeltaItem {
    pub id: String,
    pub item_id: String,
    pub item_name: String,
    pub r#type: ModelType,
    pub update_time: DateTime,
    pub jop_updated_time: DateTime,
}

#[derive(Debug, Deserialize)]
pub struct DeltaResult {
    pub items: Vec<DeltaItem>,
    pub cursor: String,
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

    pub fn login(&self, host: &str, email: &str, password: &str) -> JoplinServerResult<Self> {
        let login_form = LoginForm { email, password };
        let host = host.to_string();
        let client = Client::new();
        let res = client
            .post(format!("{}/{}", host, "api/sessions"))
            .json(&login_form)
            .send()?
            .error_for_status()?;
        let login_result = res.json::<LoginResult>()?;
        Ok(Self {
            host,
            client,
            session_id: login_result.id,
        })
    }

    pub fn put(&self, path: &str, bytes: Vec<u8>) -> JoplinServerResult<PutResult> {
        let res = self
            .request_builder(Method::PUT, &format!("{}/content", self.with_path(path)))
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()?
            .error_for_status()?;
        Ok(res.json()?)
    }

    pub fn delete(&self, path: &str) -> JoplinServerResult<()> {
        self.request_builder(Method::DELETE, &self.with_path(path))
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn get(&self, path: &str) -> JoplinServerResult<Vec<u8>> {
        let res = self
            .request_builder(Method::GET, &format!("{}/content", self.with_path(path)))
            .send()?
            .error_for_status()?;
        Ok(res.bytes()?.to_vec())
    }

    pub fn metadata(&self, path: &str) -> JoplinServerResult<FileMetadata> {
        let res = self
            .request_builder(Method::GET, &self.with_path(path))
            .send()?
            .error_for_status()?;
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
        let res = builder.send()?.error_for_status()?;
        Ok(res.json()?)
    }
}

#[cfg(test)]
mod tests {
    use super::{test_env, JoplinServerAPI, JoplinServerResult};

    #[test]
    fn test_simple() -> JoplinServerResult<()> {
        let test_config = test_env::read_test_env().joplin_server;
        let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        let path = "testing.bin";
        let create_result = api.put(path, b"testing1".to_vec())?;
        let create_metadata = api.metadata(path)?;
        assert_eq!(b"testing1".to_vec(), api.get(path)?);
        let update_result = api.put(path, b"testing2".to_vec())?;
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
        let test_config = test_env::read_test_env().joplin_server;
        let api = JoplinServerAPI::new(&test_config.host, &test_config.session_id);
        let path = "test-delta.md";
        api.put(path, b"testing1".to_vec())?;
        assert_eq!(b"testing1".to_vec(), api.get(path)?);
        let delta_result = api.root_delta(None);
        println!("r: {:?}", delta_result);
        Ok(())
    }
}
