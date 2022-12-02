use async_trait::async_trait;

use crate::sync::{
    remote_api::{DeltaItem, JoplinServerAPI},
    SyncResult,
};

use super::{
    file_api_driver::{DeltaList, RemoteItem, SyncContext},
    FileApiDriver,
};

#[derive(Debug)]
pub struct JoplinServerSyncContext {
    cursor: String,
}

impl SyncContext for JoplinServerSyncContext {
    fn to_joplin_server_sync_context(&self) -> &JoplinServerSyncContext {
        self
    }
}

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
        todo!()
    }

    fn request_repeat_count(&self) -> u32 {
        todo!()
    }

    async fn stat(&self, _path: &str) -> SyncResult<super::Stat> {
        todo!()
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
            items: delta_items.items.into_iter().map(|i| i.into()).collect(),
            has_more: delta_items.has_more,
            context: delta_items
                .cursor
                .map(|cursor| Box::new(JoplinServerSyncContext { cursor }) as Box<dyn SyncContext>),
        })
    }

    async fn list(&self, _path: &str) -> SyncResult<super::StatList> {
        todo!()
    }

    async fn get(&self, _path: &str) -> SyncResult<String> {
        todo!()
    }

    async fn mkdir(&self, _path: &str) -> SyncResult<()> {
        todo!()
    }

    async fn put(&self, _path: &str, _content: &str) -> SyncResult<()> {
        todo!()
    }

    async fn multi_put(&self, _items: &[super::file_api_driver::MultiPutItem]) -> SyncResult<()> {
        todo!()
    }

    async fn delete(&self, _path: &str) -> SyncResult<()> {
        todo!()
    }

    async fn r#move(&self, _old_path: &str, _new_path: &str) -> SyncResult<()> {
        todo!()
    }

    async fn clear_root(&self, _base_dir: &str) -> SyncResult<()> {
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
