//! Bounded authored reading; activity facts remain in the parent presentation.
use crate::{PreparedContentFieldView, RecordNavigationView, RecordRelationshipView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedReadingRequest {
    pub parent: RecordNavigationView,
    pub artifact_generation: String,
    pub selection: EmbeddedReadingSelection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EmbeddedReadingSelection {
    TableResults {
        cursor: Option<EmbeddedReadingCursor>,
    },
    Abilities {
        cursor: Option<EmbeddedReadingCursor>,
        target: Option<RecordNavigationView>,
    },
    Activity {
        navigation: RecordNavigationView,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedReadingCursor {
    pub artifact_generation: String,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedReadingBatchView {
    pub artifact_generation: String,
    pub parent: RecordNavigationView,
    pub availability: atlas_domain::QueryFieldState,
    pub total: usize,
    pub start: usize,
    pub end: usize,
    pub previous: Option<EmbeddedReadingCursor>,
    pub next: Option<EmbeddedReadingCursor>,
    pub entries: Vec<EmbeddedReadingEntryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EmbeddedReadingEntryView {
    pub navigation: RecordNavigationView,
    pub content: Vec<PreparedContentFieldView>,
    pub relationships: Vec<RecordRelationshipView>,
}
