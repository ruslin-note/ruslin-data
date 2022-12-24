mod deserialize;
mod error;
mod file_api;
pub mod remote_api;
mod serializer;

use std::{fmt::Debug, str::FromStr, sync::Arc};

pub use deserialize::{DeserializeForSync, ForSyncDeserializer};
pub use error::{SyncError, SyncResult};
pub use file_api::*;
use serde::{Deserialize, Serialize};
pub use serializer::{ForSyncSerializer, SerializeForSync};
use tokio::{task::JoinSet, time::Instant};

use crate::{
    Database, DateTimeTimestamp, Folder, ModelType, Note, Setting, SyncItem, UpdateSource,
};

#[derive(Serialize, Deserialize, Clone)]
pub enum SyncConfig {
    JoplinServer {
        host: String,
        email: String,
        password: String,
    },
}

impl Debug for SyncConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Self::JoplinServer { host, email, password } => f.debug_struct("JoplinServer").field("host", host).field("email", email).field("password", password).finish(),
            Self::JoplinServer { .. } => f.write_str("SyncConfig.JoplinServer"),
        }
    }
}

const LOG_TARGET: &str = "Synchronizer";

pub struct Synchronizer {
    db: Arc<Database>,
    file_api_driver: Arc<Box<dyn FileApiDriver>>,
}

impl Synchronizer {
    pub fn new(db: Arc<Database>, file_api_driver: Box<dyn FileApiDriver>) -> Self {
        Self {
            db,
            file_api_driver: Arc::new(file_api_driver),
        }
    }

    pub async fn start(&self) -> SyncResult<()> {
        let now = Instant::now();
        self.delete_remote().await?;
        self.upload().await?;
        self.delta().await?;
        let elapsed = now.elapsed();
        let elapsed = if elapsed.as_secs() >= 1 {
            format!("{}s", elapsed.as_secs_f64())
        } else {
            format!("{}ms", elapsed.as_millis())
        };
        log::info!(
            target: LOG_TARGET,
            "finished sync at {} ({})",
            DateTimeTimestamp::now().format_ymd_hms(),
            elapsed
        );
        Ok(())
    }

    async fn delete_remote(&self) -> SyncResult<()> {
        log::info!(
            target: LOG_TARGET,
            "starting the delete remote content task"
        );
        let deleted_items = self.db.load_deleted_items()?;
        let mut task_set = JoinSet::new();
        for item in deleted_items {
            let file_api_driver = self.file_api_driver.clone();
            task_set.spawn(async move {
                log::debug!(
                    target: LOG_TARGET,
                    "the {}({:?}) will be deleted",
                    item.item_id,
                    item.item_type
                );
                file_api_driver
                    .delete(item.filepath().as_str())
                    .await
                    .map(|_| item)
            });
        }
        while let Some(res) = task_set.join_next().await {
            let deleted_item = res??;
            log::debug!(
                target: LOG_TARGET,
                "the {}({:?}) deleted",
                deleted_item.item_id,
                deleted_item.item_type
            );
            self.db.delete_deleted_item(deleted_item)?;
        }
        Ok(())
    }

    async fn upload(&self) -> SyncResult<()> {
        log::info!(target: LOG_TARGET, "starting the upload local content task");
        let need_upload_sync_items = self.db.load_need_upload_sync_items()?;
        for item in need_upload_sync_items {
            let stat = self.file_api_driver.stat(&item.filepath()).await?;
            if stat.is_some() {
                let content = self.file_api_driver.get(&item.filepath()).await?;
                let remote_des = ForSyncDeserializer::from_str(&content)?;
                assert_eq!(item.item_type, remote_des.r#type);
                let remote_updated_time = remote_des.get_updated_time()?;
                if remote_updated_time > item.sync_time {
                    // remote.updated_time > local.sync_time -> conflict. both remote and local have changes
                    log::warn!(
                        target: LOG_TARGET,
                        "both remote and local have changes {:?} {}",
                        remote_des.r#type,
                        remote_des.id
                    );
                    match remote_des.r#type {
                        ModelType::Note => {
                            let local_note = self.db.load_note(&item.item_id)?;
                            let remote_note = Note::dserialize(&remote_des)?;
                            self.create_conflict_note(&local_note, Some(&remote_note))?;
                            self.write_remote_to_local(&remote_des)?;
                        }
                        ModelType::Folder | ModelType::Unsupported => {
                            // take the remote version
                            self.write_remote_to_local(&remote_des)?;
                        }
                    }
                } else {
                    log::debug!(
                        target: LOG_TARGET,
                        "updating {}({:?})",
                        item.item_id,
                        item.item_type
                    );
                    // remote.updated_time < local.sync_time -> updateRemote
                    let upload_content = self.db.load_sync_item_content(&item)?;
                    self.file_api_driver
                        .put(&item.filepath(), upload_content.as_str())
                        .await?;
                }
            } else if item.never_synced() {
                log::debug!(
                    target: LOG_TARGET,
                    "creating {}({:?})",
                    item.item_id,
                    item.item_type
                );
                // remote == None && first sync -> createRemote
                let upload_content = self.db.load_sync_item_content(&item)?;
                self.file_api_driver
                    .put(&item.filepath(), upload_content.as_str())
                    .await?;
            } else {
                // remote == None && not first sync -> conflict. remote has beed deleted, but local has changes
                log::warn!(
                    "remote has beed deleted, but local has changes {:?} {}",
                    item.item_type,
                    item.item_id
                );
                match item.item_type {
                    ModelType::Note => {
                        let local_note = self.db.load_note(&item.item_id)?;
                        self.create_conflict_note(&local_note, None)?;
                        self.delete_local_by_sync(&item)?;
                    }
                    ModelType::Folder | ModelType::Unsupported => {
                        self.delete_local_by_sync(&item)?;
                    }
                }
            }
        }
        Ok(())
    }

    async fn delta(&self) -> SyncResult<()> {
        let mut context = if let Some(delta_context_setting) =
            self.db.get_setting_value(Setting::FILE_API_DELTA_CONTEXT)?
        {
            Some(
                self.file_api_driver
                    .deserializer_delta_context(&delta_context_setting.value)?,
            )
        } else {
            None
        };
        log::info!(
            target: LOG_TARGET,
            "starting the delta remote content task from context: {:?}",
            context
        );
        loop {
            let list_result = self.file_api_driver.delta("", context.as_deref()).await?;

            let mut handles = Vec::with_capacity(list_result.items.len());

            for item in list_result.items.iter() {
                let path = item.path.to_string();
                let file_api_driver = self.file_api_driver.clone();
                handles.push(tokio::spawn(
                    async move { file_api_driver.get(&path).await },
                ));
            }

            let remote_ids: Vec<&str> = list_result.items.iter().map(|i| i.path_id()).collect();
            let local_sync_items = self.db.load_sync_items(&remote_ids)?;

            let tasks = list_result.items.iter().zip(handles.into_iter());
            for (remote_item, handle) in tasks {
                let local_sync_item = local_sync_items
                    .iter()
                    .find(|i| i.item_id == remote_item.path_id());
                if remote_item.is_deleted {
                    if let Some(local_sync_item) = local_sync_item {
                        self.delete_local_by_sync(local_sync_item)?;
                    }
                } else {
                    if let Some(local_sync_item) = local_sync_item {
                        if local_sync_item.sync_time > remote_item.updated_time {
                            log::error!(target: LOG_TARGET, "skip the update because the local sync time({:?}) is later than the remote update time({:?})", local_sync_item.sync_time, remote_item.updated_time);
                            continue;
                        }
                    }
                    let content = handle.await??;
                    let des = ForSyncDeserializer::from_str(&content)?;
                    self.write_remote_to_local(&des)?;
                }
            }
            context = list_result.context;
            match &context {
                Some(ctx) => {
                    log::debug!(target: LOG_TARGET, "saving delta context: {:?}", ctx);
                    self.db
                        .replace_setting(Setting::FILE_API_DELTA_CONTEXT, &ctx.to_string())?;
                }
                None => {
                    self.db.delete_setting(Setting::FILE_API_DELTA_CONTEXT)?;
                }
            }
            if !list_result.has_more {
                break;
            }
        }
        Ok(())
    }

    fn write_remote_to_local(&self, des: &ForSyncDeserializer) -> SyncResult<()> {
        match des.r#type {
            ModelType::Note => {
                let note = Note::dserialize(des)?;
                log::debug!(
                    target: LOG_TARGET,
                    "pulling note {} to local",
                    note.get_title()
                );
                self.db.replace_note(&note, UpdateSource::RemoteSync)?;
            }
            ModelType::Folder => {
                let folder = Folder::dserialize(des)?;
                log::debug!(
                    target: LOG_TARGET,
                    "pulling folder {} to local",
                    folder.get_title()
                );
                self.db.replace_folder(&folder, UpdateSource::RemoteSync)?;
            }
            ModelType::Unsupported => {
                log::warn!("skip unsupported type {:?}", des);
            }
        }
        Ok(())
    }

    fn delete_local_by_sync(&self, sync_item: &SyncItem) -> SyncResult<()> {
        log::debug!(
            target: LOG_TARGET,
            "deleting {}({:?})",
            sync_item.item_id,
            sync_item.item_type
        );
        match sync_item.item_type {
            ModelType::Note => self
                .db
                .delete_note(&sync_item.item_id, UpdateSource::RemoteSync)?,
            ModelType::Folder => self
                .db
                .delete_folder(&sync_item.item_id, UpdateSource::RemoteSync)?,
            ModelType::Unsupported => {
                log::warn!("skip unsupported type {}", sync_item.item_id);
            }
        }
        Ok(())
    }

    fn create_conflict_note(
        &self,
        local_note: &Note,
        remote_note: Option<&Note>,
    ) -> SyncResult<()> {
        if let Some(remote_note) = remote_note {
            if local_note.id != remote_note.id {
                return Err(SyncError::HandleConflictForDiffNote);
            }
            if local_note.title == remote_note.title && local_note.body == remote_note.body {
                return Ok(());
            }
        }
        let conflict_note = local_note.create_conflict_note();
        self.db
            .replace_note(&conflict_note, UpdateSource::RemoteSync)?;
        Ok(())
    }
}
