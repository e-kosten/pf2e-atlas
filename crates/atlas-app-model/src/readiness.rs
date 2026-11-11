use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AppReadinessView {
    pub default_search_profile: atlas_domain::SearchProfileId,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub artifact_generation: Option<String>,
    pub search_profiles: Vec<SearchProfileReadinessView>,
    pub status: AppReadinessStatus,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub saved_writes_paused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub workspace_error: Option<crate::AppError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference_error: Option<crate::AppError>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchProfileReadinessView {
    pub id: atlas_domain::SearchProfileId,
    pub label: String,
    pub stored_vectors: bool,
    pub assets_ready: bool,
    pub usable: bool,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppReadinessStatus {
    Ready,
    Blocked,
}
