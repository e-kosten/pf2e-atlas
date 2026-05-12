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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Subcategory {
    Consumable,
    Gear,
    Weapon,
    Armor,
    Shield,
    Ammo,
    Backpack,
    Treasure,
    Kit,
    Vehicle,
    Class,
    Ancestry,
    Skill,
    General,
    Archetype,
    BoonCurse,
    Character,
    Familiar,
    Haunt,
    Trap,
    Curse,
    Disease,
    Poison,
    Action,
    Condition,
    Effect,
    CampaignFeature,
    Heritage,
    Background,
    Deity,
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
            Self::BoonCurse => "boon_curse",
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
            Self::CampaignFeature => "campaign_feature",
            Self::Heritage => "heritage",
            Self::Background => "background",
            Self::Deity => "deity",
            Self::Journal => "journal",
        }
    }

    pub fn from_canonical(value: &str) -> Option<Self> {
        match value {
            "action" => Some(Self::Action),
            "ammo" => Some(Self::Ammo),
            "ancestry" => Some(Self::Ancestry),
            "archetype" => Some(Self::Archetype),
            "armor" => Some(Self::Armor),
            "backpack" => Some(Self::Backpack),
            "background" => Some(Self::Background),
            "boon_curse" => Some(Self::BoonCurse),
            "campaign_feature" => Some(Self::CampaignFeature),
            "character" => Some(Self::Character),
            "class" => Some(Self::Class),
            "condition" => Some(Self::Condition),
            "consumable" => Some(Self::Consumable),
            "curse" => Some(Self::Curse),
            "deity" => Some(Self::Deity),
            "disease" => Some(Self::Disease),
            "effect" => Some(Self::Effect),
            "familiar" => Some(Self::Familiar),
            "gear" => Some(Self::Gear),
            "general" => Some(Self::General),
            "haunt" => Some(Self::Haunt),
            "heritage" => Some(Self::Heritage),
            "journal" => Some(Self::Journal),
            "kit" => Some(Self::Kit),
            "poison" => Some(Self::Poison),
            "shield" => Some(Self::Shield),
            "skill" => Some(Self::Skill),
            "trap" => Some(Self::Trap),
            "treasure" => Some(Self::Treasure),
            "vehicle" => Some(Self::Vehicle),
            "weapon" => Some(Self::Weapon),
            _ => None,
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
        Self::from_canonical(value).ok_or_else(|| SubcategoryParseError {
            value: value.to_string(),
        })
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
    fn subcategory_serializes_to_rust_canonical_values() {
        assert_eq!(
            serde_json::to_string(&Subcategory::BoonCurse).expect("subcategory serializes"),
            "\"boon_curse\""
        );
        assert_eq!(
            serde_json::to_string(&Subcategory::CampaignFeature).expect("subcategory serializes"),
            "\"campaign_feature\""
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

        let subcategory: Subcategory =
            serde_json::from_str("\"campaign_feature\"").expect("canonical value deserializes");
        assert_eq!(subcategory, Subcategory::CampaignFeature);
        assert_eq!(
            serde_json::to_string(&subcategory).expect("subcategory serializes"),
            "\"campaign_feature\""
        );
        assert!(serde_json::from_str::<Subcategory>("\"campaignFeature\"").is_err());
        assert!(serde_json::from_str::<Subcategory>("\"campaign\"").is_err());
    }
}
