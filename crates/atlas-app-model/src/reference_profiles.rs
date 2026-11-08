//! Supplied reference facts; no character building or actor-runtime calculations.
use crate::{FactView, IwrEntryView, NumberFactView, RecordNavigationView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ReferenceGrantView {
    pub name: FactView<String>,
    pub target: FactView<String>,
    pub level: NumberFactView,
    #[cfg_attr(feature = "openapi", schema(required))]
    pub navigation: Option<RecordNavigationView>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AttributeChoiceView {
    pub free_choice: bool,
    pub choices: FactView<Vec<crate::ReferenceValueView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct HeritageReferenceView {
    pub ancestry: FactView<String>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct AncestryReferenceView {
    pub hit_points: NumberFactView,
    pub speed: NumberFactView,
    pub size: FactView<crate::ReferenceValueView>,
    pub vision: FactView<crate::ReferenceValueView>,
    pub hands: NumberFactView,
    pub reach: NumberFactView,
    pub boosts: FactView<Vec<AttributeChoiceView>>,
    pub flaws: FactView<Vec<AttributeChoiceView>>,
    pub languages: FactView<Vec<crate::ReferenceValueView>>,
    pub language_details: FactView<String>,
    pub additional_languages: FactView<Vec<crate::ReferenceValueView>>,
    pub additional_language_count: NumberFactView,
    pub additional_language_details: FactView<String>,
    pub grants: FactView<Vec<ReferenceGrantView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ClassReferenceView {
    pub hit_points: NumberFactView,
    pub key_attributes: FactView<Vec<crate::ReferenceValueView>>,
    pub perception: FactView<String>,
    pub fortitude: FactView<String>,
    pub reflex: FactView<String>,
    pub will: FactView<String>,
    pub simple_weapons: FactView<String>,
    pub martial_weapons: FactView<String>,
    pub advanced_weapons: FactView<String>,
    pub unarmed: FactView<String>,
    pub other_weapons: FactView<String>,
    pub other_weapon_rank: FactView<String>,
    pub unarmored: FactView<String>,
    pub light_armor: FactView<String>,
    pub medium_armor: FactView<String>,
    pub heavy_armor: FactView<String>,
    pub spellcasting: FactView<String>,
    pub trained_skills: FactView<Vec<crate::ReferenceValueView>>,
    pub additional_skills: NumberFactView,
    pub grants: FactView<Vec<ReferenceGrantView>>,
    pub ancestry_feat_levels: FactView<Vec<u8>>,
    pub class_feat_levels: FactView<Vec<u8>>,
    pub general_feat_levels: FactView<Vec<u8>>,
    pub skill_feat_levels: FactView<Vec<u8>>,
    pub skill_increase_levels: FactView<Vec<u8>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DeitySanctificationView {
    pub requirement: FactView<String>,
    pub choices: FactView<Vec<String>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct DeityReferenceView {
    pub attributes: FactView<Vec<crate::ReferenceValueView>>,
    pub fonts: FactView<Vec<String>>,
    pub sanctification: FactView<DeitySanctificationView>,
    pub skills: FactView<Vec<crate::ReferenceValueView>>,
    pub primary_domains: FactView<Vec<crate::ReferenceValueView>>,
    pub alternate_domains: FactView<Vec<crate::ReferenceValueView>>,
    pub favored_weapons: FactView<Vec<String>>,
    pub spells: FactView<Vec<ReferenceGrantView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ObjectDefensesReferenceView {
    pub armor_class: NumberFactView,
    pub hit_points: NumberFactView,
    pub hardness: NumberFactView,
    pub immunities: FactView<Vec<IwrEntryView>>,
    pub weaknesses: FactView<Vec<IwrEntryView>>,
    pub resistances: FactView<Vec<IwrEntryView>>,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct VehicleReferenceView {
    pub crew: FactView<String>,
    pub passengers: FactView<String>,
    pub speed: FactView<String>,
    pub piloting: FactView<String>,
    pub price_gp: NumberFactView,
    pub length: NumberFactView,
    pub width: NumberFactView,
    pub height: NumberFactView,
    pub size: FactView<crate::ReferenceValueView>,
    pub collision_damage: FactView<String>,
    pub collision_dc: NumberFactView,
    pub fortitude: NumberFactView,
    pub defenses: ObjectDefensesReferenceView,
}
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ArmyReferenceView {
    pub army_type: FactView<String>,
    pub armor_class: NumberFactView,
    pub hit_points: NumberFactView,
    pub rout_threshold: NumberFactView,
    pub maneuver: NumberFactView,
    pub morale: NumberFactView,
    pub scouting: NumberFactView,
    pub consumption: NumberFactView,
    pub recruitment_dc: NumberFactView,
    pub ammunition: NumberFactView,
    pub potions: NumberFactView,
    pub melee_weapon: FactView<String>,
    pub ranged_weapon: FactView<String>,
    pub immunities: FactView<Vec<IwrEntryView>>,
    pub weaknesses: FactView<Vec<IwrEntryView>>,
    pub resistances: FactView<Vec<IwrEntryView>>,
}
