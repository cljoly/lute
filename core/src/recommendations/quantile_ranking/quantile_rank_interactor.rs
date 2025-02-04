use super::{
  bounded_min_heap::BoundedMinHeap, quantile_rank_assessment::QuantileRankAlbumAssessmentContext,
};
use crate::{
  albums::{album_read_model::AlbumReadModel, album_search_index::AlbumSearchIndex},
  profile::profile::Profile,
  recommendations::types::{
    AlbumAssessment, AlbumRecommendation, AlbumRecommendationSettings,
    RecommendationMethodInteractor,
  },
};
use anyhow::Result;
use async_trait::async_trait;
use derive_builder::Builder;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

#[derive(Builder, Clone, Debug)]
#[builder(setter(into), default)]
pub struct QuantileRankAlbumAssessmentSettings {
  pub primary_genre_weight: u32,
  pub secondary_genre_weight: u32,
  pub descriptor_weight: u32,
  pub rating_weight: u32,
  pub rating_count_weight: u32,
  pub novelty_score: f64,
  pub descriptor_count_weight: u32,
  pub credit_tag_weight: u32,
}

impl Default for QuantileRankAlbumAssessmentSettings {
  fn default() -> Self {
    Self {
      primary_genre_weight: 4,
      secondary_genre_weight: 2,
      descriptor_weight: 7,
      rating_weight: 2,
      rating_count_weight: 1,
      novelty_score: 0.2,
      descriptor_count_weight: 2,
      credit_tag_weight: 1,
    }
  }
}

pub struct QuantileRankInteractor {
  album_search_index: Arc<dyn AlbumSearchIndex + Send + Sync + 'static>,
}

impl QuantileRankInteractor {
  pub fn new(album_search_index: Arc<dyn AlbumSearchIndex + Send + Sync + 'static>) -> Self {
    Self {
      album_search_index: album_search_index,
    }
  }
}

#[derive(Clone, Debug)]
pub struct QuantileRankAssessableAlbum(AlbumReadModel);

impl TryFrom<AlbumReadModel> for QuantileRankAssessableAlbum {
  type Error = anyhow::Error;

  fn try_from(album_read_model: AlbumReadModel) -> Result<Self, Self::Error> {
    if album_read_model.descriptors.len() < 5 {
      return Err(anyhow::anyhow!("Not enough descriptors"));
    }

    Ok(Self(album_read_model))
  }
}

#[async_trait]
impl
  RecommendationMethodInteractor<QuantileRankAssessableAlbum, QuantileRankAlbumAssessmentSettings>
  for QuantileRankInteractor
{
  #[instrument(name = "QuantileRankInteractor::assess_album", skip(self))]
  async fn assess_album(
    &self,
    profile: &Profile,
    profile_albums: &[AlbumReadModel],
    album_read_model: &QuantileRankAssessableAlbum,
    settings: QuantileRankAlbumAssessmentSettings,
  ) -> Result<AlbumAssessment> {
    QuantileRankAlbumAssessmentContext::new(profile, profile_albums, settings)
      .assess(&album_read_model.0)
  }

  #[instrument(name = "QuantileRankInteractor::recommend_albums", skip(self))]
  async fn recommend_albums(
    &self,
    profile: &Profile,
    profile_albums: &[AlbumReadModel],
    assessment_settings: QuantileRankAlbumAssessmentSettings,
    recommendation_settings: AlbumRecommendationSettings,
  ) -> Result<Vec<AlbumRecommendation>> {
    let search_query = recommendation_settings.to_search_query(profile, profile_albums)?;
    let search_results = self.album_search_index.search(&search_query, None).await?;
    let context =
      QuantileRankAlbumAssessmentContext::new(profile, profile_albums, assessment_settings);
    let mut result_heap = BoundedMinHeap::new(recommendation_settings.count as usize);
    let (recommendation_sender, mut recommendation_receiver) = mpsc::unbounded_channel();
    rayon::spawn(move || {
      search_results
        .albums
        .par_iter()
        .for_each(|album| match context.assess(album) {
          Ok(assessment) => {
            let recommendation = AlbumRecommendation {
              album: album.clone(),
              assessment,
            };
            recommendation_sender.send(recommendation).unwrap();
          }
          Err(error) => {
            warn!("Error assessing album: {}", error);
          }
        });
    });
    while let Some(recommendation) = recommendation_receiver.recv().await {
      result_heap.push(recommendation);
    }
    let recommendations = result_heap.drain_sorted_desc();
    Ok(recommendations)
  }
}
