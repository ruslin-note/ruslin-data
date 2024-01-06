use crate::schema::settings;
use diesel::prelude::*;

#[derive(Clone, Identifiable, Queryable, Insertable, Debug)]
#[diesel(primary_key(key))]
#[diesel(table_name = settings)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

impl Setting {
    pub const FILE_API_SYNC_CONFIG: &'static str = "file_api.sync_config";
    pub const FILE_API_DELTA_CONTEXT: &'static str = "file_api.delta_context";
    pub const CLIENT_ID: &'static str = "client_id";
}

#[derive(Debug, Insertable)]
#[diesel(table_name = settings)]
pub struct NewSetting<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

impl<'a> NewSetting<'a> {
    pub fn new(key: &'a str, value: &'a str) -> Self {
        Self { key, value }
    }
}
