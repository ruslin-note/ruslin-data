use reqwest::blocking::Client;
use reqwest::Error as ResError;
use serde::{Deserialize, Serialize};

use thiserror::Error;

use crate::DateTime;

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
            .client
            .put(&format!("{}/content", self.with_path(path)))
            .header("X-API-AUTH", &self.session_id)
            .header("X-API-MIN-VERSION", "2.6.0")
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()?
            .error_for_status()?;
        Ok(res.json()?)
    }

    pub fn delete(&self, path: &str) -> JoplinServerResult<()> {
        self.client
            .delete(self.with_path(path))
            .header("X-API-AUTH", &self.session_id)
            .header("X-API-MIN-VERSION", "2.6.0")
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn get(&self, path: &str) -> JoplinServerResult<Vec<u8>> {
        let res = self.client
        .get(&format!("{}/content", self.with_path(path)))
        .header("X-API-AUTH", &self.session_id)
            .header("X-API-MIN-VERSION", "2.6.0")
            .send()?
            .error_for_status()?;
        Ok(res.bytes()?.to_vec())
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
        assert_eq!(b"testing1".to_vec(), api.get(path)?);
        let update_result = api.put(path, b"testing2".to_vec())?;
        assert_eq!(b"testing2".to_vec(), api.get(path)?);
        assert!(update_result.created_time.is_none());
        assert_eq!(create_result.id, update_result.id);
        assert_eq!(create_result.name, update_result.name);
        api.delete(path)?;
        Ok(())
    }
}
