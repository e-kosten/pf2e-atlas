use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Equipment,
    Feat,
    Creature,
    Hazard,
    Affliction,
    Rule,
    Spell,
    CharacterCreation,
    Lore,
}

impl Category {
    pub const ALL: [Self; 9] = [
        Self::Equipment,
        Self::Feat,
        Self::Creature,
        Self::Hazard,
        Self::Affliction,
        Self::Rule,
        Self::Spell,
        Self::CharacterCreation,
        Self::Lore,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Equipment => "equipment",
            Self::Feat => "feat",
            Self::Creature => "creature",
            Self::Hazard => "hazard",
            Self::Affliction => "affliction",
            Self::Rule => "rule",
            Self::Spell => "spell",
            Self::CharacterCreation => "character_creation",
            Self::Lore => "lore",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "equipment" => Some(Self::Equipment),
            "feat" => Some(Self::Feat),
            "creature" => Some(Self::Creature),
            "hazard" => Some(Self::Hazard),
            "affliction" => Some(Self::Affliction),
            "rule" => Some(Self::Rule),
            "spell" => Some(Self::Spell),
            "character_creation" => Some(Self::CharacterCreation),
            "lore" => Some(Self::Lore),
            _ => None,
        }
    }

    pub fn from_input(value: &str) -> Option<Self> {
        match normalize_input(value).as_str() {
            "equipment" => Some(Self::Equipment),
            "feat" | "feats" => Some(Self::Feat),
            "creature" | "creatures" => Some(Self::Creature),
            "hazard" | "hazards" => Some(Self::Hazard),
            "affliction" | "afflictions" => Some(Self::Affliction),
            "rule" | "rules" => Some(Self::Rule),
            "spell" | "spells" => Some(Self::Spell),
            "charactercreation" | "character creation" => Some(Self::CharacterCreation),
            "lore" => Some(Self::Lore),
            _ => None,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = CategoryParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_canonical(value).ok_or_else(|| CategoryParseError {
            value: value.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryParseError {
    value: String,
}

impl fmt::Display for CategoryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown category `{}`", self.value)
    }
}

impl std::error::Error for CategoryParseError {}

fn normalize_input(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_serializes_to_rust_canonical_values() {
        assert_eq!(
            serde_json::to_string(&Category::CharacterCreation).expect("category serializes"),
            "\"character_creation\""
        );
    }

    #[test]
    fn category_accepts_current_ts_aliases() {
        assert_eq!(
            Category::from_input("character creation"),
            Some(Category::CharacterCreation)
        );
        assert_eq!(Category::from_input("feats"), Some(Category::Feat));
        assert_eq!(Category::from_input("rules"), Some(Category::Rule));
    }

    #[test]
    fn serde_accepts_only_rust_canonical_values() {
        let category: Category =
            serde_json::from_str("\"character_creation\"").expect("canonical value deserializes");
        assert_eq!(category, Category::CharacterCreation);
        assert_eq!(
            serde_json::to_string(&category).expect("category serializes"),
            "\"character_creation\""
        );
        assert!(serde_json::from_str::<Category>("\"characterCreation\"").is_err());
        assert!(serde_json::from_str::<Category>("\"feats\"").is_err());
    }
}
