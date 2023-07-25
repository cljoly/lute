use crate::{
  albums::album_read_model_repository::AlbumReadModel,
  profile::{profile::Profile, profile_summary::ProfileSummary},
};
use anyhow::Result;
use async_trait::async_trait;
use std::{cmp::Ordering, collections::HashMap};

pub struct AlbumRecommendationSettings {
  pub count: u32,
}

#[derive(Clone, Debug)]
pub struct AlbumAssessment {
  pub score: f32,
  pub metadata: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug)]
pub struct AlbumRecommendation {
  pub album: AlbumReadModel,
  pub assessment: AlbumAssessment,
}

impl PartialEq for AlbumRecommendation {
  fn eq(&self, other: &Self) -> bool {
    self.assessment.score == other.assessment.score
  }
}

impl Eq for AlbumRecommendation {}

impl PartialOrd for AlbumRecommendation {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    self.assessment.score.partial_cmp(&other.assessment.score)
  }
}

impl Ord for AlbumRecommendation {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .assessment
      .score
      .partial_cmp(&other.assessment.score)
      .unwrap_or(Ordering::Equal)
  }
}

#[async_trait]
pub trait RecommendationMethodInteractor<
  TAssessableAlbum: TryFrom<AlbumReadModel>,
  TAlbumAssessmentSettings,
>
{
  async fn assess_album(
    &self,
    profile_summary: &ProfileSummary,
    album: &TAssessableAlbum,
    settings: TAlbumAssessmentSettings,
  ) -> Result<AlbumAssessment>;

  async fn recommend_albums(
    &self,
    profile: &Profile,
    profile_summary: ProfileSummary,
    assessment_settings: TAlbumAssessmentSettings,
    recommendation_settings: AlbumRecommendationSettings,
  ) -> Result<Vec<AlbumRecommendation>>;
}
