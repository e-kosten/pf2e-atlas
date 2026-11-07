use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct AppReadinessView {
    pub default_search_profile: atlas_domain::SearchProfileId,
    pub artifact_generation: Option<String>,
    pub search_profiles: Vec<SearchProfileReadinessView>,
    pub status: AppReadinessStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub workspace_error: Option<crate::AppError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub reference_error: Option<crate::AppError>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct SearchProfileReadinessView {
    pub id: atlas_domain::SearchProfileId,
    pub label: String,
    pub stored_vectors: bool,
    pub assets_ready: bool,
    pub usable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum AppReadinessStatus {
    Ready,
    Blocked,
}
