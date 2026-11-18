use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathbuilderPreviewRequest {
    pub json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_key: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathbuilderPreviewView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub character_revision: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub previous_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub previous_level: Option<i64>,
    pub retained_manual_count: usize,
    pub name: String,
    pub level: i64,
    pub artifact_generation: String,
    pub abilities: Vec<PcAbilityView>,
    pub sections: Vec<PcAbilitySectionView>,
    pub resolved_count: usize,
    pub unresolved_count: usize,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePathbuilderPcRequest {
    pub request_id: String,
    pub json: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<crate::CharacterPartyTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_character_revision: Option<i64>,
    pub expected_artifact_generation: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_key: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcAbilityView {
    pub ability_key: String,
    pub label: String,
    pub kind: String,
    pub ambiguous: bool,
    pub manual: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record_key: Option<RecordKey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub record_title: Option<String>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcReferenceView {
    pub character_key: String,
    pub character_revision: i64,
    pub parties: Vec<crate::CharacterPartyView>,
    pub name: String,
    pub imported: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub level: Option<i64>,
    pub abilities: Vec<PcAbilityView>,
    pub sections: Vec<PcAbilitySectionView>,
    pub resolved_count: usize,
    pub unresolved_count: usize,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePcAbilityLinkRequest {
    pub expected_character_revision: i64,
    pub selection: PcAbilitySelection,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PcAbilitySelection {
    Automatic,
    Manual {
        record_key: RecordKey,
        expected_artifact_generation: String,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterPcReferenceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub reference: Option<PcReferenceView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEncounterPcReferenceRequest {
    pub expected_encounter_revision: i64,
    pub association: EncounterPcAssociation,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterPcAssociation {
    Attach { character_key: String },
    Detach,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcAbilitySectionView {
    pub kind: String,
    pub label: String,
    pub abilities: Vec<PcAbilityView>,
}
