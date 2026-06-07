use std::collections::BTreeSet;

use atlas_domain::RecordKind;
use atlas_record::FoundryRecordType;
use serde::{Deserialize, Serialize};

use crate::TagId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagDefinition {
    pub id: TagId,
    pub label: String,
    pub description: String,
    pub display: TagPresentation,
    pub applicability: TagApplicability,
    pub guidance: TagGuidance,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_tags: Vec<TagId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TagDisplayGroup {
    SettingAndPlace,
    EncounterAndRole,
    ProblemSolving,
    ExplorationAndUtility,
    CombatAndRulesEffect,
    ThemeAndMotif,
    HazardAndObstacle,
    BuildAndEquipmentSupport,
    ConditionAndAffliction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum TagDisplaySubgroup {
    Habitat,
    Planar,
    Regional,
    Site,
    CombatRole,
    SceneRole,
    SocialRole,
    ThreatProfile,
    Countermeasure,
    AccessAndBarriers,
    Investigation,
    Communication,
    TravelAndMovement,
    Reconnaissance,
    BattlefieldControl,
    Impact,
    Delivery,
    OffensiveProfile,
    DefenseProfile,
    ForcedPosition,
    PerceptionControl,
    CreatureFamily,
    Genre,
    Story,
    Visual,
    ObjectMotif,
    ChallengeStructure,
    CreatureMechanism,
    EnvironmentalDanger,
    Haunt,
    ItemMechanical,
    PartyRole,
    PlayPattern,
    AfflictionProgression,
    AfflictionResponse,
    PhysiologyOverride,
}

impl TagDisplaySubgroup {
    pub const fn group(self) -> TagDisplayGroup {
        match self {
            Self::Habitat | Self::Planar | Self::Regional | Self::Site => {
                TagDisplayGroup::SettingAndPlace
            }
            Self::CombatRole | Self::SceneRole | Self::SocialRole | Self::ThreatProfile => {
                TagDisplayGroup::EncounterAndRole
            }
            Self::Countermeasure
            | Self::AccessAndBarriers
            | Self::Investigation
            | Self::Communication => TagDisplayGroup::ProblemSolving,
            Self::TravelAndMovement | Self::Reconnaissance => {
                TagDisplayGroup::ExplorationAndUtility
            }
            Self::BattlefieldControl
            | Self::Impact
            | Self::Delivery
            | Self::OffensiveProfile
            | Self::DefenseProfile
            | Self::ForcedPosition
            | Self::PerceptionControl => TagDisplayGroup::CombatAndRulesEffect,
            Self::CreatureFamily | Self::Genre | Self::Story | Self::Visual | Self::ObjectMotif => {
                TagDisplayGroup::ThemeAndMotif
            }
            Self::ChallengeStructure
            | Self::CreatureMechanism
            | Self::EnvironmentalDanger
            | Self::Haunt => TagDisplayGroup::HazardAndObstacle,
            Self::ItemMechanical | Self::PartyRole | Self::PlayPattern => {
                TagDisplayGroup::BuildAndEquipmentSupport
            }
            Self::AfflictionProgression | Self::AfflictionResponse | Self::PhysiologyOverride => {
                TagDisplayGroup::ConditionAndAffliction
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagPresentation {
    pub group: TagDisplayGroup,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subgroup: Option<TagDisplaySubgroup>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagApplicability {
    pub any_of: Vec<TagApplicabilityClause>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagApplicabilityClause {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub record_kinds: BTreeSet<RecordKind>,
    #[serde(
        default,
        with = "foundry_record_type_vec",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub foundry_record_types: Vec<FoundryRecordType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_facts: Vec<TagFactPredicate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_facts: Vec<TagFactPredicate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum TagFactPredicate {
    HasTrait { value: String },
    HasPublicationFamily { value: String },
    HasMetric { key: String },
    HasMetadataSetValue { field: String, value: String },
    HasMetadataEnumValue { field: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagGuidance {
    pub applies_when: Vec<String>,
    pub does_not_apply_when: Vec<String>,
}

pub(crate) mod foundry_record_type_vec {
    use atlas_record::FoundryRecordType;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(values: &[FoundryRecordType], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        values
            .iter()
            .map(FoundryRecordType::as_str)
            .collect::<Vec<_>>()
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<FoundryRecordType>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values = Vec::<String>::deserialize(deserializer)?;
        Ok(values
            .into_iter()
            .map(|value| FoundryRecordType::from_foundry(&value))
            .collect())
    }
}
