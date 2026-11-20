use atlas_domain::RecordKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub name: String,
    pub level: i64,
    pub artifact_generation: String,
    pub sections: Vec<PcAbilitySectionView>,
}

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePathbuilderPcRequest {
    pub request_id: String,
    pub json: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub rule_links: BTreeMap<String, RecordKey>,
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
    pub automatic_match_available: bool,
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
    pub sections: Vec<PcAbilitySectionView>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EncounterCharacterSource;
    use serde_json::{from_value, json, to_value};

    #[test]
    fn pathbuilder_rule_links_are_optional_and_use_record_keys() {
        let save =
            json!({"request_id": "import", "json": "{}", "expected_artifact_generation": "ready"});
        let encounter =
            json!({"kind": "pathbuilder", "json": "{}", "expected_artifact_generation": "ready"});
        let save_request: SavePathbuilderPcRequest = from_value(save.clone()).unwrap();
        let encounter_source: EncounterCharacterSource = from_value(encounter.clone()).unwrap();
        assert!(save_request.rule_links.is_empty());
        assert_eq!(to_value(save_request).unwrap(), save);
        assert_eq!(to_value(encounter_source).unwrap(), encounter);

        let links = json!({"ability-key": "feats:linked0000000001"});
        let mut save = save;
        save["rule_links"] = links.clone();
        let request: SavePathbuilderPcRequest = from_value(save.clone()).unwrap();
        assert_eq!(
            request.rule_links["ability-key"].to_string(),
            "feats:linked0000000001"
        );
        assert_eq!(to_value(request).unwrap(), save);
        let mut encounter = encounter;
        encounter["rule_links"] = links;
        let source: EncounterCharacterSource = from_value(encounter.clone()).unwrap();
        assert_eq!(to_value(source).unwrap(), encounter);
        encounter["rule_links"] = serde_json::Value::Null;
        assert!(from_value::<EncounterCharacterSource>(encounter).is_err());
    }
}
