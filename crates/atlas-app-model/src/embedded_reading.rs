//! Bounded authored reading; activity facts remain in the parent presentation.
use crate::{PreparedContentFieldView, RecordNavigationView, RecordRelationshipView};
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedReadingRequest {
    pub parent: RecordNavigationView,
    pub artifact_generation: String,
    pub selection: EmbeddedReadingSelection,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EmbeddedReadingSelection {
    /// Root authored actor text; effective runtime facts remain in the encounter.
    ActorContent,
    TableResults {
        cursor: Option<EmbeddedReadingCursor>,
    },
    Abilities {
        group: Option<crate::ActorAbilityGroupView>,
        cursor: Option<EmbeddedReadingCursor>,
        target: Option<RecordNavigationView>,
    },
    Activity {
        navigation: RecordNavigationView,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedReadingCursor {
    pub artifact_generation: String,
    pub offset: usize,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedReadingBatchView {
    pub artifact_generation: String,
    pub parent: RecordNavigationView,
    pub availability: atlas_domain::QueryFieldState,
    pub total: usize,
    pub start: usize,
    pub end: usize,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub previous: Option<EmbeddedReadingCursor>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub next: Option<EmbeddedReadingCursor>,
    pub entries: Vec<EmbeddedReadingEntryView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmbeddedReadingEntryView {
    pub navigation: RecordNavigationView,
    pub content: Vec<PreparedContentFieldView>,
    pub relationships: Vec<RecordRelationshipView>,
}
