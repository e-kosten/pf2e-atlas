use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    CreatureSurfaceActionCostView, CreatureSurfaceContentBlockView, CreatureSurfaceContentView,
    CreatureSurfaceDamageView,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub teaser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub complexity: Option<HazardSurfaceComplexityView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub size: Option<HazardSurfaceSizeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub emits_sound: Option<HazardSurfaceEmitsSoundView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub detection: Option<HazardSurfaceDetectionView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub defenses: Option<HazardSurfaceDefensesView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub lifecycle: Option<HazardSurfaceLifecycleView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub activities: Option<Vec<HazardSurfaceActivityView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub relationships: Option<Vec<HazardSurfaceRelationshipView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub unavailable_fields: Option<Vec<HazardSurfaceUnavailableView>>,
    pub provenance: HazardSurfaceProvenanceView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceComplexityView {
    Simple,
    Complex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceSizeView {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "sound_type", rename_all = "snake_case")]
#[ts(tag = "sound_type", rename_all = "snake_case")]
pub enum HazardSurfaceEmitsSoundView {
    Boolean { value: bool },
    Named { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceDetectionView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub stealth_modifier: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub difficulty_class: Option<i64>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub details: Option<Vec<CreatureSurfaceContentBlockView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceDefensesView {
    pub applicability: HazardSurfaceDefenseApplicabilityView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub armor_class: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub hardness: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub hit_points: Option<HazardSurfaceHitPointsView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub saves: Option<HazardSurfaceSavesView>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub immunities: Option<Vec<HazardSurfaceIwrView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub weaknesses: Option<Vec<HazardSurfaceIwrView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub resistances: Option<Vec<HazardSurfaceIwrView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceDefenseApplicabilityView {
    pub health: HazardSurfaceApplicabilityStateView,
    pub structure: HazardSurfaceApplicabilityStateView,
    pub rule_id: String,
    pub rule_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceApplicabilityStateView {
    Applicable,
    Inapplicable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceHitPointsView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub current: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub temporary: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub broken_threshold: Option<i64>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub details: Option<Vec<CreatureSurfaceContentBlockView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceSavesView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub fortitude: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub reflex: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub will: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceIwrView {
    pub component_id: String,
    pub authored_order: u32,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub amount: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub double_vs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceLifecycleView {
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub description: Option<Vec<CreatureSurfaceContentBlockView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub disable: Option<Vec<CreatureSurfaceContentBlockView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub routine: Option<Vec<CreatureSurfaceContentBlockView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub reset: Option<Vec<CreatureSurfaceContentBlockView>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceActivityView {
    pub occurrence_id: String,
    pub entity_id: String,
    pub authored_order: u32,
    pub source_ordinal: u32,
    pub identity_stability: HazardSurfaceOccurrenceIdentityStabilityView,
    pub label: String,
    pub activity_type: HazardSurfaceActivityTypeView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub child_type: Option<String>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub traits: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attack_mode: Option<HazardSurfaceAttackModeView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_cost: Option<CreatureSurfaceActionCostView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub frequency: Option<HazardSurfaceFrequencyView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub death_note: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub self_effect: Option<HazardSurfaceSelfEffectView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub attack_bonus: Option<i64>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub attack_effects: Option<Vec<HazardSurfaceAttackEffectView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub damage: Option<Vec<CreatureSurfaceDamageView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub rules: Option<Vec<HazardSurfaceRuleView>>,
    #[serde(skip_serializing_if = "optional_vec_is_empty")]
    #[ts(optional)]
    pub content: Option<Vec<CreatureSurfaceContentView>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceActivityTypeView {
    Action,
    Strike,
    Condition,
    Effect,
    UnsupportedChild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceAttackEffectView {
    NoMultipleAttackPenalty,
    IndependentLimbs,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceFrequencyView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "crate::json_integer::optional")]
    #[ts(optional, type = "number")]
    pub maximum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub period: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceSelfEffectView {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "rule_type", rename_all = "snake_case")]
#[ts(tag = "rule_type", rename_all = "snake_case")]
pub enum HazardSurfaceRuleView {
    Immunity {
        authored_order: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        mode: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        immunity_types: Vec<String>,
    },
    ActiveEffectLike {
        authored_order: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        mode: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        path: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        value: Option<bool>,
    },
    Aura {
        authored_order: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::json_integer::optional")]
        #[ts(optional, type = "number")]
        radius: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        slug: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        traits: Vec<String>,
    },
    DamageDice {
        authored_order: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        critical: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::json_integer::optional")]
        #[ts(optional, type = "number")]
        dice_number: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        die_size: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        damage_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        selector: Option<String>,
    },
    FlatModifier {
        authored_order: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        critical: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        damage_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        selector: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(default, with = "crate::json_integer::optional")]
        #[ts(optional, type = "number")]
        value: Option<i64>,
    },
    Note {
        authored_order: u32,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        outcomes: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        selector: Option<String>,
        #[serde(skip_serializing_if = "optional_vec_is_empty")]
        #[ts(optional)]
        text: Option<Vec<CreatureSurfaceContentBlockView>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        visibility: Option<String>,
    },
    Unsupported {
        authored_order: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceOccurrenceIdentityStabilityView {
    StableSourceIdentity,
    UnstableAuthoredOrdinal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceRelationshipView {
    pub relationship_id: String,
    pub authored_order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub source_occurrence_id: Option<String>,
    pub target: HazardSurfaceRelationshipTargetView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "target_type", rename_all = "snake_case")]
#[ts(tag = "target_type", rename_all = "snake_case")]
pub enum HazardSurfaceRelationshipTargetView {
    Entity { entity_id: String },
    Occurrence { occurrence_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceUnavailableView {
    // Opaque identity for a distinct retained fact; never display as a label.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fact_id: Option<String>,
    pub state: HazardSurfaceUnavailableStateView,
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub component_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceUnavailableStateView {
    Missing,
    Null,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceProvenanceView {
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub convenience_rule_id: String,
    pub convenience_rule_version: u32,
    pub image: HazardSurfaceProvenanceTextView,
    pub publication_license: HazardSurfaceProvenanceTextView,
    pub source_metadata: Vec<HazardSurfaceSourceMetadataFactView>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "field", rename_all = "snake_case")]
#[ts(tag = "field", rename_all = "snake_case")]
pub enum HazardSurfaceSourceMetadataFactView {
    TokenName {
        value: HazardSurfaceSourceFactView<String>,
    },
    HasHealth {
        value: HazardSurfaceSourceFactView<bool>,
    },
    TemporaryMaximum {
        #[ts(type = "HazardSurfaceSourceFactView<number>")]
        value: HazardSurfaceSourceFactView<i64>,
    },
    SaveDetail {
        save: HazardSurfaceSaveKindView,
        value: HazardSurfaceSourceFactView<String>,
    },
    ItemRarity {
        entity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        component_label: Option<String>,
        value: HazardSurfaceSourceFactView<String>,
    },
    ItemLineage {
        entity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        component_label: Option<String>,
        value: HazardSurfaceSourceFactView<HazardSurfaceItemLineageView>,
    },
    StrikeAttack {
        entity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        component_label: Option<String>,
        #[ts(type = "HazardSurfaceSourceFactView<number>")]
        value: HazardSurfaceSourceFactView<i64>,
    },
    StrikeWeaponType {
        entity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        component_label: Option<String>,
        value: HazardSurfaceSourceFactView<HazardSurfaceAttackModeView>,
    },
    StrikeAttackEffectsCustom {
        entity_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        component_label: Option<String>,
        value: HazardSurfaceSourceFactView<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum HazardSurfaceSourceFactView<T> {
    Missing {
        source_path: String,
    },
    Null {
        source_path: String,
    },
    Typed {
        source_path: String,
        value: T,
    },
    Unsupported {
        source_path: String,
        exact_json: String,
        expected_shape: HazardSurfaceExpectedShapeView,
        actual_shape: HazardSurfaceSourceShapeView,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct HazardSurfaceItemLineageView {
    pub compendium_source: Box<HazardSurfaceSourceFactView<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceSaveKindView {
    Fortitude,
    Reflex,
    Will,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceAttackModeView {
    Melee,
    Ranged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceExpectedShapeView {
    Any,
    Boolean,
    Integer,
    String,
    StringOrBoolean,
    StringOrArray,
    Array,
    Object,
    ClosedVocabulary,
    RichDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum HazardSurfaceSourceShapeView {
    Missing,
    Null,
    Boolean,
    Number,
    String,
    Array,
    Object,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum HazardSurfaceProvenanceTextView {
    Missing,
    Null,
    Value { value: String },
    Unsupported,
}

fn optional_vec_is_empty<T>(values: &Option<Vec<T>>) -> bool {
    values.as_ref().is_none_or(Vec::is_empty)
}
