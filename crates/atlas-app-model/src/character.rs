use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterPartyView {
    pub party_key: String,
    pub name: String,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterSummaryView {
    pub character_key: String,
    pub name: String,
    pub revision: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "openapi", schema(nullable = false))]
    pub level: Option<i64>,
    pub imported: bool,
    pub parties: Vec<CharacterPartyView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterIndexView {
    pub characters: Vec<CharacterSummaryView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterPartyTarget {
    pub party_key: String,
    pub expected_revision: i64,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCharacterRequest {
    pub request_id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<CharacterPartyTarget>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCharacterRequest {
    pub expected_revision: i64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteCharacterRequest {
    pub expected_revision: i64,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterMutationView {
    pub character_key: String,
    pub available: bool,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterRevision {
    pub character_key: String,
    pub expected_revision: i64,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterLevelUpdate {
    pub character_key: String,
    pub expected_character_revision: i64,
    pub level: Option<i64>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddEncounterCharacterRequest {
    pub request_id: String,
    pub expected_encounter_revision: i64,
    pub source: EncounterCharacterSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<CharacterPartyTarget>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EncounterCharacterSource {
    Existing {
        character_key: String,
        expected_character_revision: i64,
    },
    Manual {
        name: String,
        #[serde(default)]
        level: Option<i64>,
        #[serde(default)]
        max_hp: Option<i64>,
        #[serde(default)]
        initiative: Option<i64>,
    },
    Pathbuilder {
        json: String,
        expected_artifact_generation: String,
    },
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncounterCharacterMutationView {
    pub encounter_key: String,
    pub character_key: String,
    pub participant_key: String,
    pub available: bool,
}
