use crate::RecordPresentationView;
use atlas_domain::SourcePassageAddress;
use atlas_record::source_content::{OwnedContentLocator, SourceContentLocator};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordSummaryView {
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    pub source_type: Option<String>,
    pub level_label: Option<String>,
    pub level_basis: Option<String>,
    pub rarity: Option<String>,
    pub traits: Vec<RecordBadgeView>,
    pub publication: Option<String>,
    pub pack: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordBadgeView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub trait_key: Option<String>,
    pub kind: String,
    pub label: String,
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RecordNavigationView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_rank: Option<u8>,
    pub record_key: String,
    #[serde(rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    pub field: Option<String>,
    pub passage: Option<SourcePassageAddress>,
    pub source_fingerprint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct RecordDetailRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub spell_rank: Option<u8>,
    pub record_key: String,
    #[serde(default)]
    #[serde(rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    #[serde(default)]
    pub fields: Vec<String>,
    pub passage: Option<SourcePassageAddress>,
    pub source_fingerprint: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordDetailView {
    pub artifact_generation: String,
    pub record: RecordSummaryView,
    pub presentation: RecordPresentationView,
    pub selected: RecordNavigationView,
    pub relationships: Vec<RecordRelationshipView>,
    pub relationships_truncated: bool,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordRelationshipView {
    pub source: ContentLocationView,
    pub source_navigation: RecordNavigationView,
    pub ordinal: usize,
    pub origin: String,
    pub kind: String,
    pub authored_target: Option<String>,
    pub occurrence_path: Option<String>,
    pub audiences: Vec<String>,
    pub availability: Option<atlas_domain::QueryFieldState>,
    pub status: String,
    pub target: Option<RecordNavigationView>,
    pub url: Option<String>,
    pub effect_label: Option<EffectReferenceLabelView>,
    // Verified local reading context; the authored target remains unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub contextual_target: Option<RecordNavigationView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct EffectReferenceLabelView {
    pub text: String,
    pub title: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordResolutionAmbiguousView {
    pub record_ref: String,
    pub matches: Vec<RecordResolutionCandidateView>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct RecordResolutionCandidateView {
    pub record: RecordSummaryView,
    pub query: String,
    pub normalized_query: String,
    pub match_kind: String,
    pub matched_text: String,
    pub evidence: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RecordRefResolutionView {
    Key(atlas_domain::RecordKey),
    Miss,
    Ambiguous(Vec<RecordResolutionCandidateView>),
}

/// Product content address; persisted source addresses retain their storage vocabulary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ContentLocationView {
    pub record: atlas_domain::RecordKey,
    #[serde(rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
}
impl From<SourceContentLocator> for ContentLocationView {
    fn from(value: SourceContentLocator) -> Self {
        Self {
            record: value.record,
            owners: value.owners,
            field: value.field,
        }
    }
}

#[cfg(test)]
mod embedded_contract_tests {
    #[test]
    fn old_owner_selection_is_rejected_instead_of_silently_opening_root() {
        let old = serde_json::json!({"record_key":"equipment:wand", "owners":[], "passage":null,"source_fingerprint":null});
        assert!(serde_json::from_value::<super::RecordDetailRequest>(old).is_err());
        let new = serde_json::json!({"record_key":"equipment:wand", "embedded_path":[], "passage":null,"source_fingerprint":null});
        assert!(serde_json::from_value::<super::RecordDetailRequest>(new).is_ok());
    }
}
