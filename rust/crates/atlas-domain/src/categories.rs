use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordFamily {
    Creature,
    Character,
    Companion,
    Army,
    Hazard,
    Vehicle,
    Equipment,
    Feat,
    Spell,
    Rule,
    CharacterOption,
    Lore,
    Tooling,
    CampaignFeature,
}

impl RecordFamily {
    pub const ALL: [Self; 14] = [
        Self::Creature,
        Self::Character,
        Self::Companion,
        Self::Army,
        Self::Hazard,
        Self::Vehicle,
        Self::Equipment,
        Self::Feat,
        Self::Spell,
        Self::Rule,
        Self::CharacterOption,
        Self::Lore,
        Self::Tooling,
        Self::CampaignFeature,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Creature => "creature",
            Self::Character => "character",
            Self::Companion => "companion",
            Self::Army => "army",
            Self::Hazard => "hazard",
            Self::Vehicle => "vehicle",
            Self::Equipment => "equipment",
            Self::Feat => "feat",
            Self::Spell => "spell",
            Self::Rule => "rule",
            Self::CharacterOption => "character_option",
            Self::Lore => "lore",
            Self::Tooling => "tooling",
            Self::CampaignFeature => "campaign_feature",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "creature" => Some(Self::Creature),
            "character" => Some(Self::Character),
            "companion" => Some(Self::Companion),
            "army" => Some(Self::Army),
            "hazard" => Some(Self::Hazard),
            "vehicle" => Some(Self::Vehicle),
            "equipment" => Some(Self::Equipment),
            "feat" => Some(Self::Feat),
            "spell" => Some(Self::Spell),
            "rule" => Some(Self::Rule),
            "character_option" => Some(Self::CharacterOption),
            "lore" => Some(Self::Lore),
            "tooling" => Some(Self::Tooling),
            "campaign_feature" => Some(Self::CampaignFeature),
            _ => None,
        }
    }

    pub fn from_input(value: &str) -> Option<Self> {
        match normalize_input(value).as_str() {
            "creature" | "creatures" => Some(Self::Creature),
            "character" | "characters" => Some(Self::Character),
            "companion" | "companions" | "familiar" | "familiars" => Some(Self::Companion),
            "army" | "armies" => Some(Self::Army),
            "hazard" | "hazards" => Some(Self::Hazard),
            "vehicle" | "vehicles" => Some(Self::Vehicle),
            "equipment" => Some(Self::Equipment),
            "feat" | "feats" => Some(Self::Feat),
            "spell" | "spells" => Some(Self::Spell),
            "rule" | "rules" => Some(Self::Rule),
            "characteroption" | "character option" | "character options" | "charactercreation"
            | "character creation" => Some(Self::CharacterOption),
            "lore" => Some(Self::Lore),
            "tooling" | "tool" | "tools" => Some(Self::Tooling),
            "campaignfeature" | "campaign feature" | "campaign features" => {
                Some(Self::CampaignFeature)
            }
            _ => None,
        }
    }
}

impl fmt::Display for RecordFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RecordFamily {
    type Err = RecordFamilyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_canonical(value).ok_or_else(|| RecordFamilyParseError {
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFamilyParseError {
    value: String,
}

impl fmt::Display for RecordFamilyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown record family `{}`", self.value)
    }
}

impl std::error::Error for RecordFamilyParseError {}

fn normalize_input(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_family_serializes_to_rust_canonical_values() {
        assert_eq!(
            serde_json::to_string(&RecordFamily::CharacterOption)
                .expect("record family serializes"),
            "\"character_option\""
        );
    }

    #[test]
    fn record_family_accepts_current_ts_aliases() {
        assert_eq!(
            RecordFamily::from_input("character creation"),
            Some(RecordFamily::CharacterOption)
        );
        assert_eq!(RecordFamily::from_input("feats"), Some(RecordFamily::Feat));
        assert_eq!(RecordFamily::from_input("rules"), Some(RecordFamily::Rule));
        assert_eq!(
            RecordFamily::from_input("familiars"),
            Some(RecordFamily::Companion)
        );
    }

    #[test]
    fn serde_accepts_only_rust_canonical_values() {
        let record_family: RecordFamily =
            serde_json::from_str("\"character_option\"").expect("canonical value deserializes");
        assert_eq!(record_family, RecordFamily::CharacterOption);
        assert_eq!(
            serde_json::to_string(&record_family).expect("record family serializes"),
            "\"character_option\""
        );
        assert!(serde_json::from_str::<RecordFamily>("\"characterCreation\"").is_err());
        assert!(serde_json::from_str::<RecordFamily>("\"feats\"").is_err());
    }
}
