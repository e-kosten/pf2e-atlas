use crate::RecordPresentationView;
use atlas_domain::OwnedContentLocator;
use atlas_domain::SourcePassageAddress;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordSummaryView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub creature_display: Option<CreatureDisplayView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub unit_price_label: Option<String>,
    /// The selected document's explicit publication era; absent when unavailable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub edition: Option<RecordEditionView>,
    pub record_key: String,
    pub title: String,
    pub kind: String,
    pub kind_label: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_type: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub level_label: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub level_basis: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub rarity: Option<crate::ReferenceValueView>,
    pub traits: Vec<RecordBadgeView>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub publication: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub pack: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureDisplayView {
    pub title: String,
    pub qualifiers: Vec<String>,
    pub uses: Vec<CreatureScenarioUseView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub scenario_context: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureScenarioUseView {
    pub id: String,
    pub label: String,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureFamilyView {
    pub id: String,
    pub name: String,
    pub versions: Vec<RecordSummaryView>,
    pub named_creatures: Vec<RecordSummaryView>,
    pub named_bases: std::collections::BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub groupings: Vec<CreatureFamilyGroupingView>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub all_variants_by_default: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub version_context: Option<crate::VariantFamilyView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureFamilyGroupingView {
    pub id: String,
    pub label: String,
    pub groups: Vec<CreatureFamilyGroupView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureFamilyGroupView {
    pub heading: String,
    pub members: Vec<String>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub qualifiers: std::collections::BTreeMap<String, Vec<String>>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatureOverrideView {
    pub use_id: String,
    pub adjustment: crate::EncounterParticipantVariantView,
    pub record_key: String,
}

/// Publication era does not imply that a record has a replacement or is unusable.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordEditionView {
    Legacy,
    Remaster,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordBadgeView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub help: Option<crate::ReferenceHelpView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub trait_key: Option<String>,
    pub kind: String,
    pub label: String,
    pub value: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordNavigationView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Selected casting rank for a spell or its explicitly linked effect preview.
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub spell_rank: Option<u8>,
    pub record_key: String,
    #[serde(rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub field: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub passage: Option<SourcePassageAddress>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub source_fingerprint: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordAddress {
    pub record_key: String,
    #[serde(default, rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    pub source_fingerprint: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordSelection {
    pub address: RecordAddress,
    pub spell_rank: Option<u8>,
    #[serde(default)]
    pub fields: Vec<String>,
    pub passage: Option<SourcePassageAddress>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordDetailRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_artifact_generation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Selected casting rank for a spell or its explicitly linked effect preview.
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
impl RecordDetailRequest {
    pub fn into_selection(self) -> RecordSelection {
        RecordSelection {
            address: RecordAddress {
                record_key: self.record_key,
                owners: self.owners,
                source_fingerprint: self.source_fingerprint,
            },
            spell_rank: self.spell_rank,
            fields: self.fields,
            passage: self.passage,
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDetailView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub creature_family: Option<CreatureFamilyView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "openapi", schema(required = false))]
    pub creature_overrides: Vec<CreatureOverrideView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub equipment_family: Option<crate::EquipmentFamilyReferenceView>,
    /// Change statements associated with the exact selected document.
    pub remaster_relationships: crate::RelatedCategoryView<crate::RemasterRelationshipsView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub source_remaster_context:
        Option<crate::RelatedCategoryView<crate::RemasterSourceContextView>>,
    /// Default adjustment for adding this root creature to an encounter.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub encounter_adjustment: Option<crate::EncounterParticipantVariantView>,
    pub artifact_generation: String,
    pub record: RecordSummaryView,
    pub presentation: RecordPresentationView,
    pub selected: RecordNavigationView,
    pub relationships: Vec<RecordRelationshipView>,
    pub relationships_truncated: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecordReferenceDestinationView {
    ResolvedRecord { target: RecordNavigationView },
    ResolvedEmbedded { target: RecordNavigationView },
    ResolvedUrl { url: String },
    UnverifiedUrl { url: String },
    Unresolved,
    Blocked,
}

impl RecordReferenceDestinationView {
    pub fn record_target(&self) -> Option<&RecordNavigationView> {
        match self {
            Self::ResolvedRecord { target } | Self::ResolvedEmbedded { target } => Some(target),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordRelationshipView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub source_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub target_title: Option<String>,
    pub source: ContentLocationView,
    pub source_navigation: RecordNavigationView,
    pub ordinal: usize,
    pub origin: String,
    pub kind: String,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub authored_target: Option<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub occurrence_path: Option<String>,
    pub audiences: Vec<String>,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub availability: Option<atlas_domain::QueryFieldState>,
    pub destination: RecordReferenceDestinationView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub effect_label: Option<EffectReferenceLabelView>,
    // Verified local reading context; the authored target remains unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub contextual_target: Option<RecordNavigationView>,
    /// Derived action command targets its containing root action; readers retain
    /// the label without offering redundant navigation. Target evidence remains.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub self_action_reference: Option<bool>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectReferenceLabelView {
    pub text: String,
    pub title: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordResolutionAmbiguousView {
    pub record_ref: String,
    pub matches: Vec<RecordResolutionCandidateView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordResolutionCandidateView {
    pub record: RecordSummaryView,
    pub query: String,
    pub normalized_query: String,
    pub match_kind: String,
    pub matched_text: String,
    pub evidence: Option<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RecordRefResolutionView {
    Key(atlas_domain::RecordKey),
    Miss,
    Ambiguous(Vec<RecordResolutionCandidateView>),
}

/// Product content address; persisted source addresses retain their storage vocabulary.
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentLocationView {
    pub record: atlas_domain::RecordKey,
    #[serde(rename = "embedded_path")]
    pub owners: Vec<OwnedContentLocator>,
    pub field: String,
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
