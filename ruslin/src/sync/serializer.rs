use crate::{DateTimeRFC333, DateTimeTimestamp, ModelType};

use super::SyncResult;

pub trait SerializeForSync {
    fn serialize(&self) -> SyncResult<ForSyncSerializer>;
}

pub struct ForSyncSerializer(String);

impl ForSyncSerializer {
    pub fn new(title: &str, body: Option<&str>) -> Self {
        let mut s = title.trim_start().trim_end().to_string();
        s.push('\n');
        if let Some(body) = body {
            s.push('\n');
            s.push_str(body);
            s.push('\n');
        }
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl ForSyncSerializer {
    pub fn serialize_str(&mut self, k: &str, v: &str) {
        self.0.push('\n');
        self.0.push_str(&format!("{k}: {v}"));
    }

    pub fn serialize_datetime_rfc333(&mut self, k: &str, v: &DateTimeRFC333) {
        self.0.push('\n');
        self.0.push_str(&format!("{k}: {}", v.as_string()));
    }

    pub fn serialize_datetime(&mut self, k: &str, v: DateTimeTimestamp) {
        let dt: DateTimeRFC333 = v.into();
        self.serialize_datetime_rfc333(k, &dt);
    }

    pub fn serialize_bool(&mut self, k: &str, v: bool) {
        self.0.push('\n');
        let v = match v {
            true => 1,
            false => 0,
        };
        self.0.push_str(&format!("{k}: {v}"));
    }

    pub fn serialize_opt_str(&mut self, k: &str, v: Option<&str>) {
        self.0.push('\n');
        match v {
            Some(v) => {
                self.0.push_str(&format!("{k}: {v}"));
            }
            None => {
                self.0.push_str(&format!("{k}: "));
            }
        }
    }

    pub fn serialize_type(&mut self, k: &str, v: ModelType) {
        self.0.push('\n');
        self.0.push_str(&format!("{k}: {}", v as i32));
    }
}
