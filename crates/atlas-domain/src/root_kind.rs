//! Public selectors for standalone records. Embedded collection predicates
//! have their own family-specific vocabulary.
use crate::PackName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootKindChoice {
    pub id: &'static str,
    pub parent: Option<&'static str>,
}

macro_rules! choices {
    ($( $parent:literal => [$( $child:literal ),*] ),* $(,)?) => {
        &[$(
            RootKindChoice { id: $parent, parent: None },
            $(RootKindChoice { id: $child, parent: Some($parent) },)*
        )*]
    };
}

pub const ROOT_KINDS: &[RootKindChoice] = choices! {
    "action" => [],
    "affliction" => [],
    "character" => [],
    "character_advancement" => ["ancestry_feat", "ancestry_feature", "bonus_feat",
        "mythic_calling", "class_feat", "class_feature", "general_feat", "skill_feat", "society_boon"],
    "character_creation" => ["ancestry", "background", "class", "heritage"],
    "condition" => [],
    "creature" => [],
    "all_deities" => ["deity", "deity_boon", "deity_curse"],
    "effect" => [],
    "all_equipment" => ["ammunition", "armor", "consumable", "container", "general_equipment",
        "kit", "shield", "treasure", "weapon"],
    "familiar" => [],
    "hazard" => [],
    "journal" => [],
    "kingdom_and_warfare" => ["army", "army_tactic", "army_war_action",
        "kingdom_activity", "kingdom_feat", "kingdom_feature"],
    "roll_table" => [],
    "all_spells" => ["cantrip", "ritual", "spell"],
    "vehicle" => [],
};

pub const JOURNAL_PACK_PREFIX: &str = "journal_pack/";

pub fn journal_pack(id: &str) -> Option<&str> {
    let pack = id.strip_prefix(JOURNAL_PACK_PREFIX)?;
    PackName::new(pack).ok().map(|_| pack)
}

pub fn parent(id: &str) -> Option<&'static str> {
    if journal_pack(id).is_some() {
        Some("journal")
    } else {
        ROOT_KINDS.iter().find(|choice| choice.id == id)?.parent
    }
}
