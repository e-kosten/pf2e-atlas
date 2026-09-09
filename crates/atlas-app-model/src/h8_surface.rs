use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CreatureSurfaceContentView;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
#[ts(tag = "state", content = "value", rename_all = "snake_case")]
pub enum H8FactView<T> {
    Missing,
    Null,
    Known(T),
    Unsupported(H8UnsupportedValueView),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8UnsupportedValueView {
    pub shape: String,
    pub exact_value: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8UnsupportedFieldView {
    pub relative_path: String,
    pub authored_order: u32,
    pub value: H8UnsupportedValueView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8SourceMetadataView {
    pub folder: H8FactView<String>,
    #[ts(type = "H8FactView<number>")]
    pub sort: H8FactView<i64>,
    pub ownership: H8FactView<String>,
    pub flags: H8FactView<String>,
    pub stats: H8FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8ProvenanceView {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct JournalSurfaceView {
    pub source_id: String,
    pub pages: H8FactView<Vec<JournalPageEntryView>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<CreatureSurfaceContentView>,
    pub source_metadata: H8SourceMetadataView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<H8UnsupportedFieldView>,
    pub provenance: H8ProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "entry_type", rename_all = "snake_case")]
#[ts(tag = "entry_type", rename_all = "snake_case")]
pub enum JournalPageEntryView {
    Page { page: Box<JournalPageView> },
    Unsupported { unsupported: H8UnsupportedChildView },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct JournalPageView {
    pub locator: String,
    pub identity_stability: H8IdentityStabilityView,
    pub source_id: H8FactView<String>,
    pub source_ordinal: u32,
    pub name: H8FactView<String>,
    pub page_kind: H8FactView<String>,
    #[ts(type = "H8FactView<number>")]
    pub sort: H8FactView<i64>,
    pub title: H8FactView<JournalPageTitleView>,
    pub text: H8FactView<JournalPageTextView>,
    pub source: H8FactView<String>,
    pub image_source: H8FactView<String>,
    pub image_caption: H8FactView<String>,
    pub video: H8FactView<JournalPageVideoView>,
    pub source_system: H8FactView<String>,
    pub source_metadata: H8PageSourceMetadataView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<H8UnsupportedFieldView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8PageSourceMetadataView {
    pub ownership: H8FactView<String>,
    pub flags: H8FactView<String>,
    pub stats: H8FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct JournalPageTitleView {
    pub show: H8FactView<bool>,
    #[ts(type = "H8FactView<number>")]
    pub level: H8FactView<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct JournalPageTextView {
    pub content: H8FactView<Vec<crate::CreatureSurfaceContentBlockView>>,
    #[ts(type = "H8FactView<number>")]
    pub format: H8FactView<i64>,
    pub markdown: H8FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct JournalPageVideoView {
    pub controls: H8FactView<bool>,
    pub loop_playback: H8FactView<bool>,
    pub autoplay: H8FactView<bool>,
    pub volume: H8FactView<String>,
    pub timestamp: H8FactView<String>,
    pub width: H8FactView<String>,
    pub height: H8FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct RollTableSurfaceView {
    pub source_id: String,
    pub description: H8FactView<Vec<crate::CreatureSurfaceContentBlockView>>,
    pub results: H8FactView<Vec<TableResultEntryView>>,
    pub formula: H8FactView<String>,
    pub replacement: H8FactView<bool>,
    pub display_roll: H8FactView<bool>,
    pub image: H8FactView<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<CreatureSurfaceContentView>,
    pub source_metadata: H8SourceMetadataView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<H8UnsupportedFieldView>,
    pub provenance: H8ProvenanceView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "entry_type", rename_all = "snake_case")]
#[ts(tag = "entry_type", rename_all = "snake_case")]
pub enum TableResultEntryView {
    Result { result: Box<TableResultView> },
    Unsupported { unsupported: H8UnsupportedChildView },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct TableResultView {
    pub locator: String,
    pub identity_stability: H8IdentityStabilityView,
    pub source_id: H8FactView<String>,
    pub source_ordinal: u32,
    pub result_kind: H8FactView<String>,
    pub text: H8FactView<Vec<crate::CreatureSurfaceContentBlockView>>,
    pub collection: H8FactView<String>,
    pub document_id: H8FactView<String>,
    pub weight: H8FactView<String>,
    pub range: H8FactView<TableResultRangeView>,
    pub drawn: H8FactView<bool>,
    pub image: H8FactView<String>,
    pub source_metadata: TableResultSourceMetadataView,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsupported_fields: Vec<H8UnsupportedFieldView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct TableResultSourceMetadataView {
    pub flags: H8FactView<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct TableResultRangeView {
    #[serde(with = "crate::json_integer")]
    #[ts(type = "number")]
    pub first: i64,
    #[serde(with = "crate::json_integer")]
    #[ts(type = "number")]
    pub last: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct H8UnsupportedChildView {
    pub locator: String,
    pub identity_stability: H8IdentityStabilityView,
    pub source_ordinal: u32,
    pub exact_source: String,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum H8IdentityStabilityView {
    StableSourceId,
    UnstableAuthoredOrdinal,
}
