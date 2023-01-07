use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncTargetInfo {
    pub version: i32,
    pub e2ee: SyncTargetInfoValue<bool>,
    pub active_master_key_id: SyncTargetInfoValue<String>,
    pub master_keys: Vec<MasterKey>,
    pub ppk: Option<SyncTargetInfoValue<Option<PublicPrivateKeyPair>>>,
}

impl SyncTargetInfo {
    pub fn new_support_info() -> Self {
        Self {
            version: 3,
            e2ee: SyncTargetInfoValue {
                value: false,
                updated_time: 0,
            },
            active_master_key_id: SyncTargetInfoValue {
                value: String::new(),
                updated_time: 0,
            },
            master_keys: Vec::new(),
            ppk: Some(SyncTargetInfoValue {
                value: None,
                updated_time: 0,
            }),
        }
    }

    pub fn is_supported(&self) -> bool {
        if self.version != 3 {
            return false;
        }
        if self.e2ee.value {
            return false;
        }
        if !self.active_master_key_id.value.is_empty() {
            return false;
        }
        if let Some(ppk) = &self.ppk {
            if ppk.value.is_some() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncTargetInfoValue<T> {
    pub value: T,
    pub updated_time: i64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MasterKey {
    pub id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PublicPrivateKeyPair {
    pub id: String,
}

#[cfg(test)]
mod tests {
    use crate::sync::SyncResult;

    use super::SyncTargetInfo;

    #[test]
    fn test_sync_target_info_deserialize() -> SyncResult<()> {
        let json_str = r#"{"version":3,"e2ee":{"value":false,"updatedTime":0},"activeMasterKeyId":{"value":"","updatedTime":0},"masterKeys":[],"ppk":{"value":null,"updatedTime":0}}"#;
        let sync_target_info: SyncTargetInfo = serde_json::from_str(json_str)?;
        let serialize_string = serde_json::to_string(&sync_target_info)?;
        assert_eq!(json_str, &serialize_string);
        let support_sync_target_info = SyncTargetInfo::new_support_info();
        assert_eq!(sync_target_info, support_sync_target_info);
        assert_eq!(json_str, &serde_json::to_string(&support_sync_target_info)?);
        assert!(sync_target_info.is_supported());
        Ok(())
    }
}
