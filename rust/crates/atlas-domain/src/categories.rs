use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Category {
    #[serde(rename = "equipment")]
    Equipment,
    #[serde(rename = "feat")]
    Feat,
    #[serde(rename = "creature")]
    Creature,
    #[serde(rename = "hazard")]
    Hazard,
    #[serde(rename = "affliction")]
    Affliction,
    #[serde(rename = "rule")]
    Rule,
    #[serde(rename = "spell")]
    Spell,
    #[serde(rename = "characterCreation")]
    CharacterCreation,
    #[serde(rename = "lore")]
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
            Self::CharacterCreation => "characterCreation",
            Self::Lore => "lore",
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

    pub fn subcategories(self) -> &'static [Subcategory] {
        match self {
            Self::Equipment => &[
                Subcategory::Consumable,
                Subcategory::Gear,
                Subcategory::Weapon,
                Subcategory::Armor,
                Subcategory::Shield,
                Subcategory::Ammo,
                Subcategory::Backpack,
                Subcategory::Treasure,
                Subcategory::Kit,
                Subcategory::Vehicle,
            ],
            Self::Feat => &[
                Subcategory::Class,
                Subcategory::Ancestry,
                Subcategory::Skill,
                Subcategory::General,
                Subcategory::Archetype,
                Subcategory::BoonCurse,
            ],
            Self::Creature => &[Subcategory::Character, Subcategory::Familiar],
            Self::Hazard => &[Subcategory::Haunt, Subcategory::Trap],
            Self::Affliction => &[
                Subcategory::Curse,
                Subcategory::Disease,
                Subcategory::Poison,
            ],
            Self::Rule => &[
                Subcategory::Action,
                Subcategory::Condition,
                Subcategory::Effect,
                Subcategory::CampaignFeature,
            ],
            Self::Spell => &[],
            Self::CharacterCreation => &[
                Subcategory::Ancestry,
                Subcategory::Heritage,
                Subcategory::Background,
                Subcategory::Class,
            ],
            Self::Lore => &[Subcategory::Deity, Subcategory::Journal],
        }
    }

    pub fn supports_subcategory(self, subcategory: Subcategory) -> bool {
        self.subcategories().contains(&subcategory)
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
        Self::from_input(value).ok_or_else(|| CategoryParseError {
            value: value.to_string(),
        })
    }
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_input(&value)
            .ok_or_else(|| serde::de::Error::custom(CategoryParseError { value }))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Subcategory {
    #[serde(rename = "consumable")]
    Consumable,
    #[serde(rename = "gear")]
    Gear,
    #[serde(rename = "weapon")]
    Weapon,
    #[serde(rename = "armor")]
    Armor,
    #[serde(rename = "shield")]
    Shield,
    #[serde(rename = "ammo")]
    Ammo,
    #[serde(rename = "backpack")]
    Backpack,
    #[serde(rename = "treasure")]
    Treasure,
    #[serde(rename = "kit")]
    Kit,
    #[serde(rename = "vehicle")]
    Vehicle,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "ancestry")]
    Ancestry,
    #[serde(rename = "skill")]
    Skill,
    #[serde(rename = "general")]
    General,
    #[serde(rename = "archetype")]
    Archetype,
    #[serde(rename = "boonCurse")]
    BoonCurse,
    #[serde(rename = "character")]
    Character,
    #[serde(rename = "familiar")]
    Familiar,
    #[serde(rename = "haunt")]
    Haunt,
    #[serde(rename = "trap")]
    Trap,
    #[serde(rename = "curse")]
    Curse,
    #[serde(rename = "disease")]
    Disease,
    #[serde(rename = "poison")]
    Poison,
    #[serde(rename = "action")]
    Action,
    #[serde(rename = "condition")]
    Condition,
    #[serde(rename = "effect")]
    Effect,
    #[serde(rename = "campaignFeature")]
    CampaignFeature,
    #[serde(rename = "heritage")]
    Heritage,
    #[serde(rename = "background")]
    Background,
    #[serde(rename = "deity")]
    Deity,
    #[serde(rename = "journal")]
    Journal,
}

impl Subcategory {
    pub const ALL: [Self; 31] = [
        Self::Action,
        Self::Ammo,
        Self::Ancestry,
        Self::Archetype,
        Self::Armor,
        Self::Backpack,
        Self::Background,
        Self::BoonCurse,
        Self::CampaignFeature,
        Self::Character,
        Self::Class,
        Self::Condition,
        Self::Consumable,
        Self::Curse,
        Self::Deity,
        Self::Disease,
        Self::Effect,
        Self::Familiar,
        Self::Gear,
        Self::General,
        Self::Haunt,
        Self::Heritage,
        Self::Journal,
        Self::Kit,
        Self::Poison,
        Self::Shield,
        Self::Skill,
        Self::Trap,
        Self::Treasure,
        Self::Vehicle,
        Self::Weapon,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Consumable => "consumable",
            Self::Gear => "gear",
            Self::Weapon => "weapon",
            Self::Armor => "armor",
            Self::Shield => "shield",
            Self::Ammo => "ammo",
            Self::Backpack => "backpack",
            Self::Treasure => "treasure",
            Self::Kit => "kit",
            Self::Vehicle => "vehicle",
            Self::Class => "class",
            Self::Ancestry => "ancestry",
            Self::Skill => "skill",
            Self::General => "general",
            Self::Archetype => "archetype",
            Self::BoonCurse => "boonCurse",
            Self::Character => "character",
            Self::Familiar => "familiar",
            Self::Haunt => "haunt",
            Self::Trap => "trap",
            Self::Curse => "curse",
            Self::Disease => "disease",
            Self::Poison => "poison",
            Self::Action => "action",
            Self::Condition => "condition",
            Self::Effect => "effect",
            Self::CampaignFeature => "campaignFeature",
            Self::Heritage => "heritage",
            Self::Background => "background",
            Self::Deity => "deity",
            Self::Journal => "journal",
        }
    }

    pub fn from_input(value: &str) -> Option<Self> {
        match normalize_input(value).as_str() {
            "action" | "actions" => Some(Self::Action),
            "ammo" => Some(Self::Ammo),
            "ancestry" | "ancestries" => Some(Self::Ancestry),
            "archetype" | "archetypes" => Some(Self::Archetype),
            "armor" => Some(Self::Armor),
            "backpack" | "backpacks" => Some(Self::Backpack),
            "background" | "backgrounds" => Some(Self::Background),
            "booncurse" => Some(Self::BoonCurse),
            "campaign" | "campaigns" | "campaign feature" | "campaign features"
            | "campaignfeature" => Some(Self::CampaignFeature),
            "character" => Some(Self::Character),
            "class" | "classes" => Some(Self::Class),
            "condition" | "conditions" => Some(Self::Condition),
            "consumable" | "consumables" => Some(Self::Consumable),
            "curse" | "curses" => Some(Self::Curse),
            "deity" | "deities" => Some(Self::Deity),
            "disease" | "diseases" => Some(Self::Disease),
            "effect" | "effects" => Some(Self::Effect),
            "familiar" => Some(Self::Familiar),
            "gear" => Some(Self::Gear),
            "general" => Some(Self::General),
            "haunt" | "haunts" => Some(Self::Haunt),
            "heritage" | "heritages" => Some(Self::Heritage),
            "journal" | "journals" => Some(Self::Journal),
            "kit" | "kits" => Some(Self::Kit),
            "poison" | "poisons" => Some(Self::Poison),
            "shield" | "shields" => Some(Self::Shield),
            "skill" => Some(Self::Skill),
            "trap" | "traps" => Some(Self::Trap),
            "treasure" | "treasures" => Some(Self::Treasure),
            "vehicle" | "vehicles" => Some(Self::Vehicle),
            "weapon" | "weapons" => Some(Self::Weapon),
            _ => None,
        }
    }
}

impl fmt::Display for Subcategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Subcategory {
    type Err = SubcategoryParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::from_input(value).ok_or_else(|| SubcategoryParseError {
            value: value.to_string(),
        })
    }
}

impl<'de> Deserialize<'de> for Subcategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_input(&value)
            .ok_or_else(|| serde::de::Error::custom(SubcategoryParseError { value }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubcategoryParseError {
    value: String,
}

impl fmt::Display for SubcategoryParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unknown subcategory `{}`", self.value)
    }
}

impl std::error::Error for SubcategoryParseError {}

fn normalize_input(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_serializes_to_ts_wire_values() {
        assert_eq!(
            serde_json::to_string(&Category::CharacterCreation).expect("category serializes"),
            "\"characterCreation\""
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
    fn subcategory_serializes_to_ts_wire_values() {
        assert_eq!(
            serde_json::to_string(&Subcategory::BoonCurse).expect("subcategory serializes"),
            "\"boonCurse\""
        );
        assert_eq!(
            serde_json::to_string(&Subcategory::CampaignFeature).expect("subcategory serializes"),
            "\"campaignFeature\""
        );
    }

    #[test]
    fn subcategory_accepts_current_ts_aliases() {
        assert_eq!(
            Subcategory::from_input("actions"),
            Some(Subcategory::Action)
        );
        assert_eq!(
            Subcategory::from_input("campaign features"),
            Some(Subcategory::CampaignFeature)
        );
        assert_eq!(
            Subcategory::from_input("ancestries"),
            Some(Subcategory::Ancestry)
        );
    }

    #[test]
    fn category_reports_supported_subcategories() {
        assert!(Category::Equipment.supports_subcategory(Subcategory::Weapon));
        assert!(Category::CharacterCreation.supports_subcategory(Subcategory::Class));
        assert!(!Category::Spell.supports_subcategory(Subcategory::Action));
    }

    #[test]
    fn deserializes_aliases_but_serializes_canonical_values() {
        let category: Category = serde_json::from_str("\"feats\"").expect("alias deserializes");
        assert_eq!(category, Category::Feat);
        assert_eq!(
            serde_json::to_string(&category).expect("category serializes"),
            "\"feat\""
        );

        let subcategory: Subcategory =
            serde_json::from_str("\"campaign\"").expect("alias deserializes");
        assert_eq!(subcategory, Subcategory::CampaignFeature);
        assert_eq!(
            serde_json::to_string(&subcategory).expect("subcategory serializes"),
            "\"campaignFeature\""
        );
    }
}
