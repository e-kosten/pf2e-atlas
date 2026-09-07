use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::CreatureSurfaceContentView;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellSurfaceView {
    pub family: SpellFamilyView,
    pub forms: Vec<SpellFormView>,
    pub effective_form: SpellEffectiveFormView,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub form_catalog_unavailable: Option<SpellFormCatalogUnavailableReasonView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<CreatureSurfaceContentView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFamilyView {
    Spell,
    Ritual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
#[ts(tag = "state", content = "value", rename_all = "snake_case")]
pub enum SpellFactView<T> {
    Missing,
    Null,
    Known(T),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
#[ts(tag = "state", content = "value", rename_all = "snake_case")]
pub enum SpellSourceValueView<T> {
    Known(T),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellClassificationView {
    pub rank: SpellFactView<u8>,
    pub traits: SpellFactView<Vec<String>>,
    pub traditions: SpellFactView<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellCastingView {
    pub time: SpellFactView<String>,
    pub cost: SpellFactView<String>,
    pub requirements: SpellFactView<String>,
    pub counteraction: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellTargetingView {
    pub target: SpellFactView<String>,
    pub range: SpellFactView<SpellRangeView>,
    pub area: SpellFactView<SpellAreaView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellRangeView {
    pub authored_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellAreaView {
    pub value: SpellFactView<u32>,
    pub area_type: SpellFactView<String>,
    pub legacy_area_type: SpellFactView<String>,
    pub details: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellDefenseView {
    pub passive: SpellFactView<String>,
    pub save: SpellFactView<SpellSaveView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellSaveView {
    pub statistic: SpellFactView<String>,
    pub basic: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellDamageView {
    pub label: String,
    pub formula: SpellFactView<String>,
    pub damage_type: SpellFactView<String>,
    pub category: SpellFactView<String>,
    pub kinds: SpellFactView<Vec<String>>,
    pub materials: SpellFactView<Vec<String>>,
    pub apply_modifier: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellDurationView {
    pub value: SpellFactView<String>,
    pub sustained: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(tag = "kind", rename_all = "snake_case")]
pub enum SpellHeighteningView {
    Interval {
        interval: SpellFactView<u8>,
        area: SpellFactView<u32>,
        damage: SpellFactView<Vec<SpellHeighteningDamageView>>,
    },
    Fixed {
        layers: Vec<SpellFixedHeighteningView>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellHeighteningDamageView {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellFixedHeighteningView {
    pub rank: SpellSourceValueView<u8>,
    pub changes: Vec<SpellFixedHeighteningChangeView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "field", rename_all = "snake_case")]
#[ts(tag = "field", rename_all = "snake_case")]
pub enum SpellFixedHeighteningChangeView {
    Classification,
    Casting,
    Targeting,
    Defense,
    Effect {
        operation: SpellEffectChangeOperationView,
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[ts(optional)]
        value: Option<SpellEffectChangeView>,
    },
    Duration,
    Heightening,
    Rules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellEffectChangeOperationView {
    Merge,
    Delete,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellEffectChangeView {
    pub formula: SpellFactView<String>,
    pub damage_type: SpellFactView<String>,
    pub category: SpellFactView<String>,
    pub kinds: SpellFactView<Vec<String>>,
    pub materials: SpellFactView<Vec<String>>,
    pub apply_modifier: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellRitualView {
    pub primary_check: SpellFactView<String>,
    pub secondary_casters: SpellFactView<u32>,
    pub secondary_checks: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellRuleView {
    pub order: u32,
    pub rule: SpellRuleDetailView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpellRuleDetailView {
    DamageDice(SpellDamageDiceRuleView),
    EphemeralEffect(SpellEphemeralEffectRuleView),
    DamageAlteration(SpellDamageAlterationRuleView),
    RollOption(SpellRollOptionRuleView),
    ItemAlteration(SpellItemAlterationRuleView),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellDamageDiceRuleView {
    pub selector: SpellFactView<String>,
    pub predicate: SpellFactView<Vec<SpellRulePredicateView>>,
    pub dice_number: SpellFactView<String>,
    pub die_size: SpellFactView<String>,
    pub damage_type: SpellFactView<String>,
    pub hide_if_disabled: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellEphemeralEffectRuleView {
    pub predicate: SpellFactView<Vec<SpellRulePredicateView>>,
    pub selectors: SpellFactView<Vec<String>>,
    pub uuid: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellDamageAlterationRuleView {
    pub mode: SpellFactView<String>,
    pub predicate: SpellFactView<Vec<SpellRulePredicateView>>,
    pub property: SpellFactView<String>,
    pub selectors: SpellFactView<Vec<String>>,
    pub slug: SpellFactView<String>,
    pub value: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellRollOptionRuleView {
    pub domain: SpellFactView<String>,
    pub label: SpellFactView<String>,
    pub option: SpellFactView<String>,
    pub placement: SpellFactView<String>,
    pub predicate: SpellFactView<Vec<SpellRulePredicateView>>,
    pub suboptions: SpellFactView<Vec<SpellRuleSuboptionView>>,
    pub toggleable: SpellFactView<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellItemAlterationRuleView {
    pub item_id: SpellFactView<String>,
    pub mode: SpellFactView<String>,
    pub predicate: SpellFactView<Vec<SpellRulePredicateView>>,
    pub property: SpellFactView<String>,
    pub value: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[ts(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpellRulePredicateView {
    Term(String),
    Or(Vec<String>),
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellRuleSuboptionView {
    pub label: SpellFactView<String>,
    pub value: SpellFactView<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellFormView {
    pub id: String,
    pub label: String,
    pub order: u32,
    pub minimum_cast_rank: u8,
    pub kind: SpellFormKindView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellEffectiveFormView {
    pub id: String,
    pub cast_rank: u8,
    pub result: SpellFormResultView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFormKindView {
    Base,
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum SpellFormResultView {
    Available {
        definition: Box<SpellResolvedDefinitionView>,
    },
    Unavailable {
        reason: SpellFormSelectionUnavailableReasonView,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellResolvedDefinitionView {
    pub applied_fixed_ranks: Vec<u8>,
    pub classification: SpellResolvedFieldView<SpellClassificationView>,
    pub casting: SpellResolvedFieldView<SpellCastingView>,
    pub targeting: SpellResolvedFieldView<SpellTargetingView>,
    pub defense: SpellResolvedFieldView<SpellDefenseView>,
    pub damage: SpellResolvedFieldView<Vec<SpellDamageView>>,
    pub duration: SpellResolvedFieldView<SpellDurationView>,
    pub heightening: SpellResolvedFieldView<SpellHeighteningView>,
    pub ritual: SpellFactView<SpellRitualView>,
    pub rules: SpellResolvedFieldView<Vec<SpellRuleView>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(tag = "state", rename_all = "snake_case")]
pub enum SpellResolvedFieldView<T> {
    Available {
        value: SpellFactView<T>,
    },
    Unavailable {
        unavailable: SpellFormFieldUnavailableView,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub struct SpellFormFieldUnavailableView {
    pub field: SpellFormFieldView,
    pub source: SpellFormPatchSourceView,
    pub reason: SpellFormFieldUnavailableReasonView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFormFieldView {
    Classification,
    Casting,
    Targeting,
    Defense,
    Damage,
    Duration,
    Heightening,
    Rules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFormPatchSourceView {
    Base,
    Overlay,
    FixedHeightening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFormFieldUnavailableReasonView {
    UnsupportedPatch,
    DuplicateKey,
    IncompatibleHeightening,
    UnknownFixedRank,
    FixedRankKeyMismatch,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "reason", rename_all = "snake_case")]
#[ts(tag = "reason", rename_all = "snake_case")]
pub enum SpellFormSelectionUnavailableReasonView {
    BaseRankUnavailable,
    CastRankBelowBase { base_rank: u8, cast_rank: u8 },
    UnknownOverlay,
    OverlayRootUnavailable,
    DuplicateOverlay,
    OverlayIdentityUnavailable,
    OverlayIdentityMismatch,
    UnsupportedOverlayType,
    FormIdMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case")]
pub enum SpellFormCatalogUnavailableReasonView {
    UnsupportedOverlayRoot,
    UnavailableOverlaySort,
}
