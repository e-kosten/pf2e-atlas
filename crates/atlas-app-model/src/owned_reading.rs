//! Bounded authored reading; activity facts remain in the parent presentation.
use crate::{PreparedContentFieldView, RecordNavigationView, RecordRelationshipView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OwnedReadingRequest {
    pub parent: RecordNavigationView,
    pub artifact_generation: String,
    pub selection: OwnedReadingSelection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OwnedReadingSelection {
    TableResults {
        cursor: Option<OwnedReadingCursor>,
    },
    Abilities {
        cursor: Option<OwnedReadingCursor>,
        target: Option<RecordNavigationView>,
    },
    Activity {
        navigation: RecordNavigationView,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OwnedReadingCursor {
    pub artifact_generation: String,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OwnedReadingBatchView {
    pub artifact_generation: String,
    pub parent: RecordNavigationView,
    pub availability: atlas_domain::QueryFieldState,
    pub total: usize,
    pub start: usize,
    pub end: usize,
    pub previous: Option<OwnedReadingCursor>,
    pub next: Option<OwnedReadingCursor>,
    pub entries: Vec<OwnedReadingEntryView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct OwnedReadingEntryView {
    pub navigation: RecordNavigationView,
    pub content: Vec<PreparedContentFieldView>,
    pub relationships: Vec<RecordRelationshipView>,
}
