use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Creature,
    Character,
    Companion,
    Army,
    Hazard,
    Vehicle,
    Equipment,
    Feat,
    Spell,
    Affliction,
    Rule,
    CharacterOption,
    Lore,
    RollTable,
    Tooling,
    CampaignFeature,
}

impl RecordKind {
    pub const ALL: [Self; 16] = [
        Self::Creature,
        Self::Character,
        Self::Companion,
        Self::Army,
        Self::Hazard,
        Self::Vehicle,
        Self::Equipment,
        Self::Feat,
        Self::Spell,
        Self::Affliction,
        Self::Rule,
        Self::CharacterOption,
        Self::Lore,
        Self::RollTable,
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
            Self::Affliction => "affliction",
            Self::Rule => "rule",
            Self::CharacterOption => "character_option",
            Self::Lore => "lore",
            Self::RollTable => "roll_table",
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
            "affliction" => Some(Self::Affliction),
            "rule" => Some(Self::Rule),
            "character_option" => Some(Self::CharacterOption),
            "lore" => Some(Self::Lore),
            "roll_table" => Some(Self::RollTable),
            "tooling" => Some(Self::Tooling),
            "campaign_feature" => Some(Self::CampaignFeature),
            _ => None,
        }
    }

    pub fn from_input(value: &str) -> Option<Self> {
        let normalized = normalize_input(value);
        if let Some(kind) = Self::from_canonical(&normalized) {
            return Some(kind);
        }
        match normalized.as_str() {
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
            "rolltable" | "roll table" | "roll tables" => Some(Self::RollTable),
            "tooling" | "tool" | "tools" => Some(Self::Tooling),
            "campaignfeature" | "campaign feature" | "campaign features" => {
                Some(Self::CampaignFeature)
            }
            _ => None,
        }
    }
}

impl fmt::Display for RecordKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for RecordKind {
    type Err = RecordKindParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_canonical(value).ok_or_else(|| RecordKindParseError {
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordKindParseError {
    value: String,
}

impl fmt::Display for RecordKindParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown record kind `{}`", self.value)
    }
}

impl std::error::Error for RecordKindParseError {}

fn normalize_input(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_record_kind_vocabulary_is_stable() {
        let cases = [
            (RecordKind::Creature, "creature"),
            (RecordKind::Character, "character"),
            (RecordKind::Companion, "companion"),
            (RecordKind::Army, "army"),
            (RecordKind::Hazard, "hazard"),
            (RecordKind::Vehicle, "vehicle"),
            (RecordKind::Equipment, "equipment"),
            (RecordKind::Feat, "feat"),
            (RecordKind::Spell, "spell"),
            (RecordKind::Affliction, "affliction"),
            (RecordKind::Rule, "rule"),
            (RecordKind::CharacterOption, "character_option"),
            (RecordKind::Lore, "lore"),
            (RecordKind::RollTable, "roll_table"),
            (RecordKind::Tooling, "tooling"),
            (RecordKind::CampaignFeature, "campaign_feature"),
        ];
        assert_eq!(
            RecordKind::ALL
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>(),
            cases.map(|(kind, _)| kind).into(),
        );
        for (kind, canonical) in cases {
            assert_eq!(kind.as_str(), canonical);
            assert_eq!(kind.to_string(), canonical);
            assert_eq!(RecordKind::from_canonical(canonical), Some(kind));
            assert_eq!(RecordKind::from_input(canonical), Some(kind));
            assert_eq!(serde_json::to_value(kind).unwrap(), canonical);
            assert_eq!(
                serde_json::from_value::<RecordKind>(serde_json::json!(canonical)).unwrap(),
                kind
            );
        }
    }

    #[test]
    fn input_aliases_do_not_expand_the_canonical_wire_vocabulary() {
        for (alias, kind) in [
            ("character creation", RecordKind::CharacterOption),
            ("characterCreation", RecordKind::CharacterOption),
            ("feats", RecordKind::Feat),
            ("rules", RecordKind::Rule),
            ("familiars", RecordKind::Companion),
        ] {
            assert_eq!(RecordKind::from_input(alias), Some(kind), "{alias}");
            assert_eq!(RecordKind::from_canonical(alias), None, "{alias}");
            assert!(
                serde_json::from_value::<RecordKind>(serde_json::json!(alias)).is_err(),
                "{alias}"
            );
        }
    }
}
