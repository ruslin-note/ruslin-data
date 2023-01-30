use std::{
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use crate::{
    new_id,
    schema::resources,
    sync::{DeserializeForSync, ForSyncSerializer, SerializeForSync, SyncResult},
    DateTimeTimestamp, ModelType,
};
use diesel::prelude::*;

#[derive(Clone, Identifiable, Insertable, Queryable, Eq, Debug)]
#[diesel(primary_key(id))]
#[diesel(table_name = resources)]
pub struct Resource {
    pub id: String,
    pub title: String,
    pub mime: String,
    pub filename: String,
    pub created_time: DateTimeTimestamp,
    pub updated_time: DateTimeTimestamp,
    pub user_created_time: DateTimeTimestamp,
    pub user_updated_time: DateTimeTimestamp,
    pub file_extension: String,
    pub encryption_cipher_text: String,
    pub encryption_applied: bool,
    pub encryption_blob_encrypted: bool,
    pub size: i32,
    pub is_shared: bool,
    pub share_id: String,
    pub master_key_id: String,
}

impl Resource {
    pub fn updated(&self) -> Self {
        let mut it = self.clone();
        let dt = DateTimeTimestamp::now();
        it.updated_time = dt;
        it.user_updated_time = dt;
        it
    }

    pub fn new(
        title: impl Into<String>,
        mime: impl Into<String>,
        file_extension: impl Into<String>,
        size: i32,
    ) -> Self {
        let dt = DateTimeTimestamp::now();
        Self {
            id: new_id(),
            title: title.into(),
            mime: mime.into(),
            filename: String::new(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            file_extension: file_extension.into(),
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            encryption_blob_encrypted: false,
            size,
            is_shared: false,
            share_id: String::new(),
            master_key_id: String::new(),
        }
    }

    pub fn resource_file_path(&self, resource_dir: &Path) -> PathBuf {
        resource_dir
            .join(&self.id)
            .with_extension(&self.file_extension)
    }

    pub fn remote_path(&self) -> String {
        format!(".resource/{}", self.id)
    }
}

impl Hash for Resource {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id.hash(hasher);
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Resource) -> bool {
        self.id == other.id
    }
}

impl SerializeForSync for Resource {
    fn serialize(&self) -> ForSyncSerializer {
        let mut ser = ForSyncSerializer::new(Some(&self.title), None);
        ser.serialize_str("id", &self.id);
        ser.serialize_str("mime", &self.mime);
        ser.serialize_str("filename", &self.filename);
        ser.serialize_datetime("created_time", self.created_time);
        ser.serialize_datetime("updated_time", self.updated_time);
        ser.serialize_datetime("user_created_time", self.user_created_time);
        ser.serialize_datetime("user_updated_time", self.user_updated_time);
        ser.serialize_str("file_extension", &self.file_extension);
        ser.serialize_str("encryption_cipher_text", &self.encryption_cipher_text);
        ser.serialize_bool("encryption_applied", self.encryption_applied);
        ser.serialize_bool("encryption_blob_encrypted", self.encryption_blob_encrypted);
        ser.serialize_i32("size", self.size);
        ser.serialize_bool("is_shared", self.is_shared);
        ser.serialize_str("share_id", &self.share_id);
        ser.serialize_str("master_key_id", &self.master_key_id);
        ser.serialize_type("type_", ModelType::Resource);
        ser
    }
}

impl DeserializeForSync for Resource {
    fn dserialize(des: &crate::sync::ForSyncDeserializer) -> SyncResult<Self> {
        assert!(des.r#type == ModelType::Resource);
        Ok(Self {
            id: des.get_string("id")?,
            title: des.title.to_string(),
            mime: des.get_string("mime")?,
            filename: des.get_opt_string("filename").unwrap_or_default(),
            created_time: des.get_date_time_timestamp("created_time")?,
            updated_time: des.get_date_time_timestamp("updated_time")?,
            user_created_time: des.get_date_time_timestamp("user_created_time")?,
            user_updated_time: des.get_date_time_timestamp("user_updated_time")?,
            file_extension: des.get_string("file_extension")?,
            encryption_cipher_text: des
                .get_opt_string("encryption_cipher_text")
                .unwrap_or_default(),
            encryption_applied: des.get_bool("encryption_applied")?,
            encryption_blob_encrypted: des.get_bool("encryption_blob_encrypted")?,
            size: des.get_i32("size")?,
            is_shared: des.get_bool("is_shared")?,
            share_id: des.get_opt_string("share_id").unwrap_or_default(),
            master_key_id: des.get_opt_string("master_key_id").unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        models::resource::Resource,
        sync::{DeserializeForSync, ForSyncDeserializer, SerializeForSync},
        DateTimeRFC333, DateTimeTimestamp,
    };

    #[test]
    fn test_serialize_and_dserialize_resource() {
        let dt = DateTimeRFC333::from_raw_str("2023-01-01T02:33:57.541Z");
        let dt: DateTimeTimestamp = dt.into();
        let resource = Resource {
            id: "a8693477a4a343f781f6e562b3148290".to_string(),
            title: "example.png".to_string(),
            mime: "image/png".to_string(),
            filename: "".to_string(),
            created_time: dt,
            updated_time: dt,
            user_created_time: dt,
            user_updated_time: dt,
            file_extension: "png".to_string(),
            encryption_cipher_text: String::new(),
            encryption_applied: false,
            encryption_blob_encrypted: false,
            size: 29343,
            is_shared: false,
            share_id: "".to_string(),
            master_key_id: "".to_string(),
        };
        let binding = resource.serialize();
        let serialize_result = binding.as_str();
        let expected_str = "example.png

id: a8693477a4a343f781f6e562b3148290
mime: image/png
filename: 
created_time: 2023-01-01T02:33:57.541Z
updated_time: 2023-01-01T02:33:57.541Z
user_created_time: 2023-01-01T02:33:57.541Z
user_updated_time: 2023-01-01T02:33:57.541Z
file_extension: png
encryption_cipher_text: 
encryption_applied: 0
encryption_blob_encrypted: 0
size: 29343
is_shared: 0
share_id: 
master_key_id: 
type_: 4";
        assert_eq!(expected_str, serialize_result);
        let des = ForSyncDeserializer::from_str(expected_str)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()));
        let des_resource = Resource::dserialize(&des).expect("dserialize failed");
        assert_eq!(resource, des_resource);
    }
}
