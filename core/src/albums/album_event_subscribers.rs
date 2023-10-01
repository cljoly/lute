use super::{
  album_repository::{
    AlbumReadModel, AlbumReadModelArtist, AlbumReadModelCredit, AlbumReadModelTrack,
    AlbumRepository,
  },
  redis_album_repository::RedisAlbumRepository,
};
use crate::{
  crawler::{
    crawler_interactor::CrawlerInteractor,
    priority_queue::{Priority, QueuePushParameters},
  },
  events::{
    event::{Event, Stream},
    event_subscriber::{EventSubscriber, EventSubscriberBuilder, SubscriberContext},
  },
  files::file_metadata::{file_name::FileName, page_type::PageType},
  parser::parsed_file_data::{
    ParsedAlbum, ParsedArtistReference, ParsedCredit, ParsedFileData, ParsedTrack,
  },
  settings::Settings,
};
use anyhow::Result;
use rustis::{bb8::Pool, client::PooledClientManager};
use std::sync::Arc;

impl From<&ParsedTrack> for AlbumReadModelTrack {
  fn from(parsed_track: &ParsedTrack) -> Self {
    Self {
      name: parsed_track.name.clone(),
      duration_seconds: parsed_track.duration_seconds,
      rating: parsed_track.rating,
      position: parsed_track.position.clone(),
    }
  }
}

impl From<&ParsedArtistReference> for AlbumReadModelArtist {
  fn from(parsed_artist: &ParsedArtistReference) -> Self {
    Self {
      name: parsed_artist.name.clone(),
      file_name: parsed_artist.file_name.clone(),
    }
  }
}

impl From<&ParsedCredit> for AlbumReadModelCredit {
  fn from(parsed_credit: &ParsedCredit) -> Self {
    Self {
      artist: (&parsed_credit.artist).into(),
      roles: parsed_credit.roles.clone(),
    }
  }
}

impl AlbumReadModel {
  pub fn from_parsed_album(file_name: &FileName, parsed_album: ParsedAlbum) -> Self {
    Self {
      name: parsed_album.name.clone(),
      file_name: file_name.clone(),
      rating: parsed_album.rating,
      rating_count: parsed_album.rating_count,
      artists: parsed_album
        .artists
        .iter()
        .map(AlbumReadModelArtist::from)
        .collect::<Vec<AlbumReadModelArtist>>(),
      primary_genres: parsed_album.primary_genres.clone(),
      secondary_genres: parsed_album.secondary_genres.clone(),
      descriptors: parsed_album.descriptors.clone(),
      tracks: parsed_album
        .tracks
        .iter()
        .map(AlbumReadModelTrack::from)
        .collect::<Vec<AlbumReadModelTrack>>(),
      release_date: parsed_album.release_date,
      languages: parsed_album.languages.clone(),
      credits: parsed_album
        .credits
        .iter()
        .map(AlbumReadModelCredit::from)
        .collect::<Vec<AlbumReadModelCredit>>(),
    }
  }
}

async fn update_album_read_models(context: SubscriberContext) -> Result<()> {
  if let Event::FileParsed {
    file_id: _,
    file_name,
    data: ParsedFileData::Album(parsed_album),
  } = context.payload.event
  {
    let album_read_model_repository =
      RedisAlbumRepository::new(Arc::clone(&context.redis_connection_pool));
    let album_read_model = AlbumReadModel::from_parsed_album(&file_name, parsed_album);
    album_read_model_repository.put(album_read_model).await?;
  }
  Ok(())
}

async fn delete_album_read_models(context: SubscriberContext) -> Result<()> {
  if let Event::FileDeleted { file_name, .. } = context.payload.event {
    let album_read_model_repository =
      RedisAlbumRepository::new(Arc::clone(&context.redis_connection_pool));
    album_read_model_repository.delete(&file_name).await?;
  }
  Ok(())
}

fn get_crawl_priority(correlation_id: Option<String>) -> Priority {
  correlation_id
    .map(|cid| {
      if cid.starts_with("crawl_similar_albums:") {
        Priority::Low
      } else {
        Priority::Standard
      }
    })
    .unwrap_or(Priority::Standard)
}

async fn crawl_chart_albums(
  context: SubscriberContext,
  crawler_interactor: Arc<CrawlerInteractor>,
) -> Result<()> {
  if let Event::FileParsed {
    file_id: _,
    file_name,
    data: ParsedFileData::Chart(albums),
  } = context.payload.event
  {
    let priority = get_crawl_priority(context.payload.correlation_id);
    for album in albums {
      crawler_interactor
        .enqueue_if_stale(QueuePushParameters {
          file_name: album.file_name,
          priority: Some(priority),
          correlation_id: Some(format!("crawl_chart_albums:{}", file_name.to_string())),
          ..Default::default()
        })
        .await?;
    }
  }
  Ok(())
}

async fn crawl_artist_albums(
  context: SubscriberContext,
  crawler_interactor: Arc<CrawlerInteractor>,
) -> Result<()> {
  if let Event::FileParsed {
    file_id: _,
    file_name,
    data: ParsedFileData::Artist(parsed_artist),
  } = context.payload.event
  {
    let priority = get_crawl_priority(context.payload.correlation_id);
    for album in parsed_artist.albums {
      crawler_interactor
        .enqueue_if_stale(QueuePushParameters {
          file_name: album.file_name,
          priority: Some(priority),
          correlation_id: Some(format!("crawl_artist_albums:{}", file_name.to_string())),
          ..Default::default()
        })
        .await?;
    }
  }
  Ok(())
}

pub fn build_album_event_subscribers(
  redis_connection_pool: Arc<Pool<PooledClientManager>>,
  sqlite_connection: Arc<tokio_rusqlite::Connection>,
  settings: Arc<Settings>,
  crawler_interactor: Arc<CrawlerInteractor>,
) -> Result<Vec<EventSubscriber>> {
  let album_crawler_interactor = Arc::clone(&crawler_interactor);
  let artist_crawler_interactor = Arc::clone(&crawler_interactor);
  Ok(vec![
    EventSubscriberBuilder::default()
      .id("update_album_read_models".to_string())
      .stream(Stream::Parser)
      .batch_size(250)
      .redis_connection_pool(Arc::clone(&redis_connection_pool))
      .sqlite_connection(Arc::clone(&sqlite_connection))
      .settings(Arc::clone(&settings))
      .generate_ordered_processing_group_id(Some(Arc::new(|(_, payload)| match &payload.event {
        Event::FileParsed {
          data: ParsedFileData::Album(ParsedAlbum { name, .. }),
          ..
        } => Some(name.clone()), // Ensure potential duplicates are processed sequentially
        _ => None,
      })))
      .handle(Arc::new(|context| {
        Box::pin(async move { update_album_read_models(context).await })
      }))
      .build()?,
    EventSubscriberBuilder::default()
      .id("delete_album_read_models".to_string())
      .stream(Stream::File)
      .batch_size(250)
      .redis_connection_pool(Arc::clone(&redis_connection_pool))
      .sqlite_connection(Arc::clone(&sqlite_connection))
      .settings(Arc::clone(&settings))
      .generate_ordered_processing_group_id(Some(Arc::new(|(_, payload)| match &payload.event {
        Event::FileDeleted { file_name, .. } => {
          match file_name.page_type() {
            PageType::Album => Some(file_name.to_string()), // Ensure potential duplicates are processed sequentially
            _ => None,
          }
        }
        _ => None,
      })))
      .handle(Arc::new(|context| {
        Box::pin(async move { delete_album_read_models(context).await })
      }))
      .build()?,
    EventSubscriberBuilder::default()
      .id("crawl_chart_albums".to_string())
      .stream(Stream::Parser)
      .batch_size(250)
      .redis_connection_pool(Arc::clone(&redis_connection_pool))
      .sqlite_connection(Arc::clone(&sqlite_connection))
      .settings(Arc::clone(&settings))
      .handle(Arc::new(move |context| {
        let crawler_interactor = Arc::clone(&artist_crawler_interactor);
        Box::pin(async move { crawl_chart_albums(context, crawler_interactor).await })
      }))
      .build()?,
    EventSubscriberBuilder::default()
      .id("crawl_artist_albums".to_string())
      .stream(Stream::Parser)
      .batch_size(250)
      .redis_connection_pool(Arc::clone(&redis_connection_pool))
      .sqlite_connection(Arc::clone(&sqlite_connection))
      .settings(Arc::clone(&settings))
      .handle(Arc::new(move |context| {
        let crawler_interactor = Arc::clone(&album_crawler_interactor);
        Box::pin(async move { crawl_artist_albums(context, crawler_interactor).await })
      }))
      .build()?,
  ])
}
