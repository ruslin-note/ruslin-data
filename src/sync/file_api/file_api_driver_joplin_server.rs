use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::sync::{
    lock_handler::{Lock, LockClientType, LockList, LockType},
    remote_api::{DeltaItem, JoplinServerAPI},
    SyncError, SyncResult,
};

use super::{
    file_api_driver::{DeltaList, RemoteItem, SyncContext},
    FileApiDriver, Stat,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct JoplinServerSyncContext {
    cursor: String,
}

impl SyncContext for JoplinServerSyncContext {
    fn to_joplin_server_sync_context(&self) -> &JoplinServerSyncContext {
        self
    }

    fn to_string(&self) -> String {
        serde_json::to_string(self)
            .unwrap_or_else(|_| panic!("unwrap error in {}:{}", file!(), line!()))
    }
}

#[derive(Debug)]
pub struct FileApiDriverJoplinServer {
    api: JoplinServerAPI,
}

impl FileApiDriverJoplinServer {
    pub fn new(api: JoplinServerAPI) -> Self {
        Self { api }
    }
}

#[async_trait]
impl FileApiDriver for FileApiDriverJoplinServer {
    fn supports_multi_put(&self) -> bool {
        todo!()
    }

    fn supports_accurate_timestamp(&self) -> bool {
        todo!()
    }

    fn supports_locks(&self) -> bool {
        true
    }

    fn request_repeat_count(&self) -> u32 {
        todo!()
    }

    async fn stat(&self, path: &str) -> SyncResult<Option<Stat>> {
        Ok(self.api.metadata(path).await.map(|m| {
            m.map(|m| Stat {
                path: m.name,
                updated_time: m.updated_time,
                is_dir: false,
            })
        })?)
    }

    async fn delta(
        &self,
        path: &str,
        ctx: Option<&dyn SyncContext>,
    ) -> SyncResult<super::file_api_driver::DeltaList> {
        let cursor = if let Some(ctx) = ctx {
            let ctx = ctx.to_joplin_server_sync_context();
            Some(ctx.cursor.clone())
        } else {
            None
        };
        let delta_items = self.api.delta(path, cursor.as_deref()).await?;
        Ok(DeltaList {
            items: delta_items
                .items
                .into_iter()
                .filter_map(|i| {
                    if i.item_name.starts_with(".resource/")
                        || i.item_name.starts_with("locks/")
                        || i.item_name.starts_with("temp/")
                    {
                        None
                    } else {
                        Some(i.into())
                    }
                })
                .collect(),
            has_more: delta_items.has_more,
            context: delta_items
                .cursor
                .map(|cursor| Box::new(JoplinServerSyncContext { cursor }) as Box<dyn SyncContext>),
        })
    }

    fn deserializer_delta_context(&self, s: &str) -> SyncResult<Box<dyn SyncContext>> {
        let sync_context: JoplinServerSyncContext = serde_json::from_str(s)?;
        Ok(Box::new(sync_context))
    }

    async fn list(&self, _path: &str) -> SyncResult<super::StatList> {
        todo!()
    }

    async fn get_text(&self, path: &str) -> SyncResult<String> {
        Ok(self.api.get_text(path).await?)
    }

    async fn get_file(&self, path: &str, destination: &Path) -> SyncResult<()> {
        Ok(self.api.get_file(path, destination).await?)
    }

    async fn mkdir(&self, _path: &str) -> SyncResult<()> {
        // ignore
        Ok(())
    }

    async fn put_text(&self, path: &str, content: &str) -> SyncResult<()> {
        self.api.put_text(path, content).await?;
        Ok(())
    }

    async fn put_file(&self, path: &str, local_file_path: &Path) -> SyncResult<()> {
        self.api.put_file(path, local_file_path).await?;
        Ok(())
    }

    async fn multi_put(&self, _items: &[super::file_api_driver::MultiPutItem]) -> SyncResult<()> {
        todo!()
    }

    async fn delete(&self, path: &str) -> SyncResult<()> {
        self.api.delete(path).await?;
        Ok(())
    }

    async fn r#move(&self, _old_path: &str, _new_path: &str) -> SyncResult<()> {
        todo!()
    }

    async fn clear_root(&self, _base_dir: &str) -> SyncResult<()> {
        Ok(self.api.clear_root().await?)
    }

    async fn check_config(&self) -> SyncResult<()> {
        let path = "testing.txt";
        let content = "testing";
        self.api.put_text(path, content.to_string()).await?;
        if content != self.api.get_text(path).await? {
            return Err(SyncError::Misconfiguration);
        }
        self.api.delete(path).await?;
        Ok(())
    }

    async fn acquire_lock(
        &self,
        _type: LockType,
        _client_type: LockClientType,
        _client_id: &str,
    ) -> SyncResult<Lock> {
        todo!()
    }

    async fn release_lock(
        &self,
        _type: LockType,
        _client_type: LockClientType,
        _client_id: &str,
    ) -> SyncResult<()> {
        todo!()
    }

    async fn list_locks(&self) -> SyncResult<LockList> {
        todo!()
    }
}

impl From<DeltaItem> for RemoteItem {
    fn from(delta_item: DeltaItem) -> Self {
        Self {
            path: delta_item.item_name,
            is_deleted: delta_item.r#type.is_deleted(),
            updated_time: delta_item.updated_time,
            jop_updated_time: delta_item.jop_updated_time,
        }
    }
}
