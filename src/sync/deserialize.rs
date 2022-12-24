use std::{collections::HashMap, str::FromStr};

use crate::{DateTimeRFC333, DateTimeTimestamp, ModelType};

use super::{SyncError, SyncResult};

pub struct ForSyncDeserializer {
    pub title: String,
    pub body: Option<String>,
    pub kvs: HashMap<String, String>,
    pub r#type: ModelType,
}

impl FromStr for ForSyncDeserializer {
    type Err = SyncError;

    fn from_str(s: &str) -> SyncResult<Self> {
        let mut iter = s.split('\n');
        let mut kvs = HashMap::<String, String>::new();
        while let Some(s) = iter.next_back() {
            if let Some((mut k, mut v)) = s.split_once(':') {
                k = k.trim();
                v = v.trim();
                if v.is_empty() {
                    continue;
                }
                kvs.insert(k.to_string(), v.to_string());
            } else {
                break;
            }
        }
        let title = iter.next().map(|i| i.to_string()).unwrap_or_default(); // String::from(iter.next().expect(&format!("ForSyncDeserializer iter next error\n {s}")));
        iter.next();
        let body: Option<String> = if let Some(s) = iter.next() {
            let mut string = String::from(s);
            for s in iter {
                string.push('\n');
                string.push_str(s);
            }
            Some(string)
        } else {
            None
        };
        let r#type: i32 = kvs
            .get("type_")
            .expect("get type_ error")
            .parse()
            .expect("type_ parse error");
        Ok(Self {
            title,
            body,
            kvs,
            r#type: ModelType::from(r#type),
        })
    }
}

impl ForSyncDeserializer {
    pub fn get_updated_time(&self) -> SyncResult<DateTimeTimestamp> {
        self.get_date_time_timestamp("updated_time")
    }
}

impl ForSyncDeserializer {
    pub fn get_string(&self, k: &str) -> SyncResult<String> {
        self.kvs
            .get(k)
            .ok_or(SyncError::DeserializeError {
                key: k.to_string(),
                val: self.get_opt_string(k).unwrap_or_default(),
            })
            .map(|s| s.to_string())
    }

    pub fn get_opt_string(&self, k: &str) -> Option<String> {
        self.kvs.get(k).map(|s| s.to_string())
    }

    pub fn get_str(&self, k: &str) -> SyncResult<&str> {
        self.kvs
            .get(k)
            .ok_or(SyncError::DeserializeError {
                key: k.to_string(),
                val: self.get_opt_string(k).unwrap_or_default(),
            })
            .map(|s| s.as_str())
    }

    pub fn get_opt_str(&self, k: &str) -> Option<&str> {
        self.kvs.get(k).map(|s| s.as_str())
    }

    pub fn get_date_time_timestamp(&self, k: &str) -> SyncResult<DateTimeTimestamp> {
        self.get_str(k)
            .map(DateTimeRFC333::from_raw_str)
            .map(|t| t.into())
    }

    pub fn get_bool(&self, k: &str) -> SyncResult<bool> {
        self.get_str(k).map(|s| s == "1")
    }

    pub fn get_f64(&self, k: &str) -> SyncResult<f64> {
        self.get_str(k).and_then(|s| {
            s.parse::<f64>().map_err(|_| SyncError::DeserializeError {
                key: k.to_string(),
                val: self.get_opt_string(k).unwrap_or_default(),
            })
        })
    }

    pub fn get_i64(&self, k: &str) -> SyncResult<i64> {
        self.get_str(k).and_then(|s| {
            s.parse::<i64>().map_err(|_| SyncError::DeserializeError {
                key: k.to_string(),
                val: self.get_opt_string(k).unwrap_or_default(),
            })
        })
    }

    pub fn get_opt_i64(&self, k: &str) -> Option<i64> {
        self.get_i64(k).ok()
    }
}

pub trait DeserializeForSync: Sized {
    fn dserialize(des: &ForSyncDeserializer) -> SyncResult<Self>;
}

#[cfg(test)]
mod tests {
    use super::ForSyncDeserializer;
    use std::str::FromStr;
    #[test]
    fn test_dserialize_without_body() {
        let s = r#"Folder1

id: fd7d741357e2451283166354c512df3b
created_time: 2022-11-20T05:27:50.593Z
updated_time: 2022-11-20T05:27:50.593Z
user_created_time: 2022-11-20T05:27:50.593Z
user_updated_time: 2022-11-20T05:27:50.593Z
encryption_cipher_text: 
encryption_applied: 0
parent_id: 
is_shared: 0
share_id: 
master_key_id: 
icon: 
type_: 2"#;
        let des = ForSyncDeserializer::from_str(s).expect(&format!(
            "unwrap error in {}:{}",
            file!(),
            line!()
        ));
        assert_eq!("Folder1", des.title);
        assert!(des.body.is_none());
        assert_eq!(
            "fd7d741357e2451283166354c512df3b",
            des.kvs
                .get("id")
                .expect(&format!("unwrap error in {}:{}", file!(), line!()))
        );
        assert!(des.kvs.get("encryption_cipher_text").is_none());
        assert_eq!(
            "2",
            des.kvs
                .get("type_")
                .expect(&format!("unwrap error in {}:{}", file!(), line!()))
        );
    }

    #[test]
    fn test_dserialize_with_body() {
        let s = r#"Note1

Content1

id: a8e8dae6e666492c90d293c914452b94

id: a8e8dae6e666492c90d293c914452b94
parent_id: fd7d741357e2451283166354c512df3b
created_time: 2022-11-20T05:28:03.344Z
updated_time: 2022-11-20T05:29:29.514Z
is_conflict: 0
latitude: 0.00000000
longitude: 0.00000000
altitude: 0.0000
author: 
source_url: 
is_todo: 0
todo_due: 0
todo_completed: 0
source: joplin
source_application: net.cozic.joplin-cli
application_data: 
order: 1668922083344
user_created_time: 2022-11-20T05:28:03.344Z
user_updated_time: 2022-11-20T05:29:29.514Z
encryption_cipher_text: 
encryption_applied: 0
markup_language: 1
is_shared: 0
share_id: 
conflict_original_id: 
master_key_id: 
type_: 1"#;
        let des = ForSyncDeserializer::from_str(s).expect(&format!(
            "unwrap error in {}:{}",
            file!(),
            line!()
        ));
        assert_eq!("Note1", des.title);
        assert!(des.body.is_some());
        assert_eq!(
            "Content1\n\nid: a8e8dae6e666492c90d293c914452b94",
            des.body
                .expect(&format!("unwrap error in {}:{}", file!(), line!()))
        );
        assert_eq!(
            "1",
            des.kvs
                .get("type_")
                .expect(&format!("unwrap error in {}:{}", file!(), line!()))
        );
    }
}
