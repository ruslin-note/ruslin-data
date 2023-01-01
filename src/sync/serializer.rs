use crate::{DateTimeRFC333, DateTimeTimestamp, ModelType};

pub trait SerializeForSync {
    fn serialize(&self) -> ForSyncSerializer;
}

pub struct ForSyncSerializer(String);

impl ForSyncSerializer {
    pub fn new(title: Option<&str>, body: Option<&str>) -> Self {
        let mut s = String::new();
        if let Some(title) = title {
            s.push_str(title.trim_start().trim_end());
            s.push('\n');
        }
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

    pub fn insert_next_line(&mut self) {
        if !self.0.is_empty() {
            self.0.push('\n');
        }
    }
}

impl ForSyncSerializer {
    pub fn serialize_str(&mut self, k: &str, v: &str) {
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {v}"));
    }

    pub fn serialize_datetime_rfc333(&mut self, k: &str, v: &DateTimeRFC333) {
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {}", v.as_string()));
    }

    pub fn serialize_datetime(&mut self, k: &str, v: DateTimeTimestamp) {
        let dt: DateTimeRFC333 = v.into();
        self.serialize_datetime_rfc333(k, &dt);
    }

    pub fn serialize_f64(&mut self, k: &str, v: f64) {
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {}", v));
    }

    pub fn serialize_i64(&mut self, k: &str, v: i64) {
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {}", v));
    }

    pub fn serialize_i32(&mut self, k: &str, v: i32) {
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {}", v));
    }

    pub fn serialize_bool(&mut self, k: &str, v: bool) {
        self.insert_next_line();
        let v = v as i32;
        self.0.push_str(&format!("{k}: {v}"));
    }

    pub fn serialize_opt_str(&mut self, k: &str, v: Option<&str>) {
        self.insert_next_line();
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
        self.insert_next_line();
        self.0.push_str(&format!("{k}: {}", v as i32));
    }
}
