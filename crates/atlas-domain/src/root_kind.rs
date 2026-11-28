//! Public selectors for standalone records. Embedded collection predicates
//! have their own family-specific vocabulary.
use crate::PackName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootKindChoice {
    pub id: &'static str,
    pub parent: Option<&'static str>,
    /// Foundry family of roots classified directly under this selector.
    pub source_family: Option<&'static str>,
}

macro_rules! choices {
    ($( $parent:literal ($family:expr) => [$( $child:literal : $child_family:literal ),*] ),* $(,)?) => {
        &[$(
            RootKindChoice { id: $parent, parent: None, source_family: $family },
            $(RootKindChoice { id: $child, parent: Some($parent), source_family: Some($child_family) },)*
        )*]
    };
}

pub const ROOT_KINDS: &[RootKindChoice] = choices! {
    "action" (Some("action")) => [],
    "affliction" (Some("affliction")) => [],
    "character" (Some("character")) => [],
    "character_advancement" (None) => [
        "ancestry_feat": "feat", "ancestry_feature": "feat", "bonus_feat": "feat",
        "mythic_calling": "feat", "class_feat": "feat", "class_feature": "feat",
        "general_feat": "feat", "skill_feat": "feat", "society_boon": "feat"],
    "character_creation" (None) => [
        "ancestry": "ancestry", "background": "background", "class": "class", "heritage": "heritage"],
    "condition" (Some("condition")) => [],
    "creature" (Some("npc")) => [],
    "all_deities" (None) => ["deity": "deity", "deity_boon": "feat", "deity_curse": "feat"],
    "effect" (Some("effect")) => [],
    "all_equipment" (None) => [
        "ammunition": "ammo", "armor": "armor", "consumable": "consumable", "container": "backpack",
        "general_equipment": "equipment", "kit": "kit", "shield": "shield", "treasure": "treasure",
        "weapon": "weapon"],
    "familiar" (Some("familiar")) => [],
    "hazard" (Some("hazard")) => [],
    "journal" (None) => [],
    "kingdom_and_warfare" (Some("campaignFeature")) => [
        "army": "army", "army_tactic": "campaignFeature", "army_war_action": "campaignFeature",
        "kingdom_activity": "campaignFeature", "kingdom_feat": "campaignFeature",
        "kingdom_feature": "campaignFeature"],
    "roll_table" (None) => [],
    "all_spells" (None) => ["cantrip": "spell", "ritual": "spell", "spell": "spell"],
    "vehicle" (Some("vehicle")) => [],
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

/// An unclassified campaign feature proves the group but cannot prove or
/// disprove membership of one of its specific feature categories.
pub fn is_campaign_feature_selector(id: &str) -> bool {
    id != "army" && parent(id) == Some("kingdom_and_warfare")
}
