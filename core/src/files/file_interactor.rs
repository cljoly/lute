use super::{
  file_content_store::FileContentStore,
  file_metadata::{
    file_metadata::FileMetadata, file_metadata_repository::FileMetadataRepository,
    file_name::FileName, file_timestamp::FileTimestamp, page_type::PageType,
  },
};
use crate::{
  events::{
    event::{Event, EventPayloadBuilder, Stream},
    event_publisher::EventPublisher,
  },
  settings::Settings,
  sqlite::SqliteConnection,
};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rustis::{bb8::Pool, client::PooledClientManager};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone)]
pub struct FileInteractor {
  settings: Arc<Settings>,
  file_content_store: FileContentStore,
  file_metadata_repository: FileMetadataRepository,
  event_publisher: EventPublisher,
}

impl FileInteractor {
  pub fn new(
    settings: Arc<Settings>,
    redis_connection_pool: Arc<Pool<PooledClientManager>>,
    sqlite_connection: Arc<SqliteConnection>,
  ) -> Self {
    Self {
      settings: Arc::clone(&settings),
      file_content_store: FileContentStore::new(&settings.file.content_store).unwrap(),
      file_metadata_repository: FileMetadataRepository {
        redis_connection_pool: Arc::clone(&redis_connection_pool),
      },
      event_publisher: EventPublisher::new(Arc::clone(&settings), sqlite_connection),
    }
  }

  pub async fn is_file_stale(&self, file_name: &FileName) -> Result<bool> {
    let file_metadata = self
      .file_metadata_repository
      .find_by_name(file_name)
      .await?;

    let ttl_days = match file_name.page_type() {
      PageType::Artist => self.settings.file.ttl_days.artist,
      PageType::Album => self.settings.file.ttl_days.album,
      PageType::Chart => self.settings.file.ttl_days.chart,
      PageType::AlbumSearchResult => self.settings.file.ttl_days.search,
    };

    Ok(
      file_metadata
        .map(|file_metadata| {
          let now: DateTime<Utc> = FileTimestamp::now().into();
          let last_saved_at: DateTime<Utc> = file_metadata.last_saved_at.into();
          let stale_at = last_saved_at + Duration::days(ttl_days.into());
          now > stale_at
        })
        .unwrap_or(true),
    )
  }

  pub async fn put_file_metadata(
    &self,
    file_name: &FileName,
    correlation_id: Option<String>,
  ) -> Result<FileMetadata> {
    let file_metadata = self.file_metadata_repository.upsert(file_name).await?;
    info!(file_name = file_name.to_string(), "File metadata saved");
    self
      .event_publisher
      .publish(
        Stream::File,
        EventPayloadBuilder::default()
          .event(Event::FileSaved {
            file_id: file_metadata.id,
            file_name: file_metadata.name.clone(),
          })
          .correlation_id(correlation_id)
          .build()?,
      )
      .await?;
    Ok(file_metadata)
  }

  pub async fn put_file(
    &self,
    file_name: &FileName,
    content: String,
    correlation_id: Option<String>,
  ) -> Result<FileMetadata> {
    self.file_content_store.put(file_name, content).await?;
    self.put_file_metadata(file_name, correlation_id).await
  }

  pub async fn list_files(&self) -> Result<Vec<FileName>> {
    self.file_content_store.list_files().await
  }

  pub async fn get_file_metadata(&self, file_name: &FileName) -> Result<FileMetadata> {
    self
      .file_metadata_repository
      .find_by_name(file_name)
      .await?
      .ok_or_else(|| {
        anyhow::anyhow!(
          "File metadata not found for file name: {}",
          file_name.to_string()
        )
      })
  }

  pub async fn delete_file(&self, file_name: &FileName) -> Result<()> {
    let file_metadata = self.get_file_metadata(file_name).await?;
    self.file_metadata_repository.delete(file_name).await?;
    self.file_content_store.delete(file_name).await?;
    self
      .event_publisher
      .publish(
        Stream::File,
        EventPayloadBuilder::default()
          .event(Event::FileDeleted {
            file_id: file_metadata.id,
            file_name: file_metadata.name.clone(),
          })
          .build()?,
      )
      .await?;
    Ok(())
  }

  pub async fn get_file_content(&self, file_name: &FileName) -> Result<String> {
    self.file_content_store.get(file_name).await
  }
}
