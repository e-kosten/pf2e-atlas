use atlas_domain::DetailLevel;
use serde::Serialize;

use crate::{
    ContentRole, ContentVisibility, FactValue, PresentationContent, ResolvedSpellForm,
    SpellAreaValue, SpellCasting, SpellClassification, SpellDamage, SpellDefenseValue,
    SpellDuration, SpellFact, SpellFixedHeighteningLayer, SpellFormContext, SpellFormField,
    SpellFormId, SpellFormPatchSource, SpellFormSelectionError, SpellFormUnavailableReason,
    SpellHeightening, SpellIntervalHeightening, SpellOrderedMember, SpellOverlay, SpellRecord,
    SpellResolvedField, SpellRitual, SpellRule, SpellRuleElement, SpellSave, SpellSourceValue,
    SpellTargeting, UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    project_presentation_content,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellJson {
    pub classification: SpellFactJson<SpellClassificationJson>,
    pub casting: SpellFactJson<SpellCastingJson>,
    pub targeting: SpellFactJson<SpellTargetingJson>,
    pub defense: SpellFactJson<SpellDefenseJson>,
    pub damage: SpellFactJson<Vec<SpellDamageJson>>,
    pub duration: SpellFactJson<SpellDurationJson>,
    pub heightening: SpellFactJson<SpellHeighteningJson>,
    pub ritual: SpellFactJson<SpellRitualJson>,
    pub rules: SpellFactJson<Vec<SpellRuleJson>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub forms: Vec<SpellFormJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form_catalog_unavailable: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<SpellContentJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<SpellProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum SpellFactJson<T> {
    Missing,
    Null,
    Known(T),
    Unsupported(SpellUnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellUnsupportedValueJson {
    pub shape: &'static str,
    pub value: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellClassificationJson {
    pub rank: SpellFactJson<u8>,
    pub traits: SpellFactJson<Vec<String>>,
    pub traditions: SpellFactJson<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellCastingJson {
    pub time: SpellFactJson<String>,
    pub cost: SpellFactJson<String>,
    pub requirements: SpellFactJson<String>,
    pub counteraction: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellTargetingJson {
    pub target: SpellFactJson<String>,
    pub range: SpellFactJson<SpellRangeJson>,
    pub area: SpellFactJson<SpellAreaJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellRangeJson {
    /// Exact authored source text. Numeric query derivatives are intentionally absent.
    pub authored_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellAreaJson {
    pub value: SpellFactJson<u32>,
    pub area_type: SpellFactJson<String>,
    pub details: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDefenseJson {
    pub passive: SpellFactJson<String>,
    pub save: SpellFactJson<SpellSaveJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellSaveJson {
    pub statistic: SpellFactJson<String>,
    pub basic: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamageJson {
    pub key: String,
    pub order: u32,
    pub formula: SpellFactJson<String>,
    pub damage_type: SpellFactJson<String>,
    pub category: SpellFactJson<String>,
    pub kinds: SpellFactJson<Vec<String>>,
    pub materials: SpellFactJson<Vec<String>>,
    pub apply_modifier: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDurationJson {
    pub value: SpellFactJson<String>,
    pub sustained: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SpellHeighteningJson {
    Interval {
        interval: SpellFactJson<u8>,
        area: SpellFactJson<u32>,
        damage: SpellFactJson<Vec<SpellHeighteningDamageJson>>,
    },
    Fixed {
        layers: Vec<SpellFixedHeighteningJson>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellHeighteningDamageJson {
    pub key: String,
    pub order: u32,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellFixedHeighteningJson {
    pub key: String,
    pub order: u32,
    pub rank: SpellFactJson<u8>,
    pub changed_fields: Vec<&'static str>,
    pub patch: SpellPatchJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellPatchJson {
    pub classification: SpellFactJson<SpellClassificationJson>,
    pub casting: SpellFactJson<SpellCastingJson>,
    pub targeting: SpellFactJson<SpellTargetingJson>,
    pub defense: SpellFactJson<SpellDefenseJson>,
    pub damage: SpellFactJson<SpellDamagePatchSetJson>,
    pub duration: SpellFactJson<SpellDurationJson>,
    pub heightening: SpellFactJson<SpellHeighteningPatchJson>,
    pub rules: SpellFactJson<Vec<SpellRuleJson>>,
    pub unsupported: Vec<SpellUnsupportedFactJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamagePatchSetJson {
    pub members: Vec<SpellDamagePatchMemberJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamagePatchMemberJson {
    pub key: String,
    pub order: u32,
    #[serde(flatten)]
    pub operation: SpellDamagePatchOperationJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", content = "value", rename_all = "snake_case")]
pub enum SpellDamagePatchOperationJson {
    Merge(Box<SpellDamagePatchJson>),
    Delete,
    Unsupported(SpellUnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamagePatchJson {
    pub formula: SpellFactJson<String>,
    pub damage_type: SpellFactJson<String>,
    pub category: SpellFactJson<String>,
    pub kinds: SpellFactJson<Vec<String>>,
    pub materials: SpellFactJson<Vec<String>>,
    pub apply_modifier: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellHeighteningPatchJson {
    pub kind: SpellFactJson<String>,
    pub interval: SpellFactJson<u8>,
    pub area: SpellFactJson<u32>,
    pub damage: SpellFactJson<SpellTextPatchSetJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellTextPatchSetJson {
    pub members: Vec<SpellTextPatchMemberJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellTextPatchMemberJson {
    pub key: String,
    pub order: u32,
    #[serde(flatten)]
    pub operation: SpellTextPatchOperationJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", content = "value", rename_all = "snake_case")]
pub enum SpellTextPatchOperationJson {
    Merge(SpellFactJson<String>),
    Delete,
    Unsupported(SpellUnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellRitualJson {
    pub primary_check: SpellFactJson<String>,
    pub secondary_casters: SpellFactJson<u32>,
    pub secondary_checks: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellRuleJson {
    pub authored_key: String,
    pub order: u32,
    #[serde(flatten)]
    pub rule: SpellRuleDetailJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpellRuleDetailJson {
    DamageDice(Box<SpellDamageDiceRuleJson>),
    EphemeralEffect(Box<SpellEphemeralEffectRuleJson>),
    DamageAlteration(Box<SpellDamageAlterationRuleJson>),
    RollOption(Box<SpellRollOptionRuleJson>),
    ItemAlteration(Box<SpellItemAlterationRuleJson>),
    Unsupported(SpellUnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamageDiceRuleJson {
    pub selector: SpellFactJson<String>,
    pub predicate: SpellFactJson<Vec<SpellRulePredicateJson>>,
    pub dice_number: SpellFactJson<String>,
    pub die_size: SpellFactJson<String>,
    pub damage_type: SpellFactJson<String>,
    pub hide_if_disabled: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellEphemeralEffectRuleJson {
    pub predicate: SpellFactJson<Vec<SpellRulePredicateJson>>,
    pub selectors: SpellFactJson<Vec<String>>,
    pub uuid: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellDamageAlterationRuleJson {
    pub mode: SpellFactJson<String>,
    pub predicate: SpellFactJson<Vec<SpellRulePredicateJson>>,
    pub property: SpellFactJson<String>,
    pub selectors: SpellFactJson<Vec<String>>,
    pub slug: SpellFactJson<String>,
    pub value: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellRollOptionRuleJson {
    pub domain: SpellFactJson<String>,
    pub label: SpellFactJson<String>,
    pub option: SpellFactJson<String>,
    pub placement: SpellFactJson<String>,
    pub predicate: SpellFactJson<Vec<SpellRulePredicateJson>>,
    pub suboptions: SpellFactJson<Vec<SpellRuleSuboptionJson>>,
    pub toggleable: SpellFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellItemAlterationRuleJson {
    pub item_id: SpellFactJson<String>,
    pub mode: SpellFactJson<String>,
    pub predicate: SpellFactJson<Vec<SpellRulePredicateJson>>,
    pub property: SpellFactJson<String>,
    pub value: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SpellRulePredicateJson {
    Term(String),
    Or(Vec<String>),
    Unsupported(SpellUnsupportedValueJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellRuleSuboptionJson {
    pub label: SpellFactJson<String>,
    pub value: SpellFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellFormJson {
    pub id: String,
    pub label: String,
    pub order: u32,
    pub cast_rank: u8,
    pub kind: &'static str,
    pub result: SpellFormResultJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SpellFormResultJson {
    Available {
        definition: Box<SpellResolvedDefinitionJson>,
    },
    Unavailable {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellResolvedDefinitionJson {
    pub applied_fixed_ranks: Vec<u8>,
    pub classification: SpellResolvedFieldJson<SpellClassificationJson>,
    pub casting: SpellResolvedFieldJson<SpellCastingJson>,
    pub targeting: SpellResolvedFieldJson<SpellTargetingJson>,
    pub defense: SpellResolvedFieldJson<SpellDefenseJson>,
    pub damage: SpellResolvedFieldJson<Vec<SpellDamageJson>>,
    pub duration: SpellResolvedFieldJson<SpellDurationJson>,
    pub heightening: SpellResolvedFieldJson<SpellHeighteningJson>,
    pub rules: SpellResolvedFieldJson<Vec<SpellRuleJson>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SpellResolvedFieldJson<T> {
    Available {
        value: SpellFactJson<T>,
    },
    Unavailable {
        field: &'static str,
        source: &'static str,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellContentJson {
    pub content_key: String,
    pub role: &'static str,
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
    pub document: PresentationContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellProvenanceJson {
    pub source_id: String,
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
    pub standalone_location: SpellFactJson<String>,
    pub image: SpellFactJson<String>,
    pub publication_license: SpellFactJson<String>,
    pub members: Vec<SpellMemberProvenanceJson>,
    pub unsupported: Vec<SpellUnsupportedFactJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellMemberProvenanceJson {
    pub field: &'static str,
    pub source_path: String,
    pub authored_key: String,
    pub order: u32,
    pub state: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpellUnsupportedFactJson {
    pub field: &'static str,
    pub source_path: String,
    pub authored_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    pub value: SpellUnsupportedValueJson,
}

pub(super) fn spell_presentation(
    spell: &SpellRecord,
    detail: DetailLevel,
    include_provenance: bool,
) -> SpellJson {
    let definition = &spell.definition;
    let (forms, form_catalog_unavailable) = if detail == DetailLevel::Summary {
        (Vec::new(), None)
    } else {
        spell_forms(spell)
    };
    let content = if matches!(detail, DetailLevel::Description | DetailLevel::Full) {
        spell_content(spell)
    } else {
        Vec::new()
    };
    SpellJson {
        classification: map_fact(&definition.classification, classification_json),
        casting: map_fact(&definition.casting, casting_json),
        targeting: map_fact(&definition.targeting, targeting_json),
        defense: map_fact(&definition.defense, defense_json),
        damage: map_fact(&definition.damage, |values| damage_json(values)),
        duration: map_fact(&definition.duration, duration_json),
        heightening: map_fact(&definition.heightening, heightening_json),
        ritual: map_fact(&definition.ritual, ritual_json),
        rules: map_fact(&definition.rules, |values| rules_json(values)),
        forms,
        form_catalog_unavailable,
        content,
        provenance: include_provenance.then(|| spell_provenance(spell)),
    }
}

fn map_fact<T, U>(fact: &SpellFact<T>, map: impl FnOnce(&T) -> U) -> SpellFactJson<U> {
    match fact {
        FactValue::Missing => SpellFactJson::Missing,
        FactValue::Null => SpellFactJson::Null,
        FactValue::Value(SpellSourceValue::Known(value)) => SpellFactJson::Known(map(value)),
        FactValue::Value(SpellSourceValue::Unsupported(value)) => {
            SpellFactJson::Unsupported(unsupported_json(value))
        }
    }
}

fn map_plain_fact<T, U>(fact: &FactValue<T>, map: impl FnOnce(&T) -> U) -> SpellFactJson<U> {
    match fact {
        FactValue::Missing => SpellFactJson::Missing,
        FactValue::Null => SpellFactJson::Null,
        FactValue::Value(value) => SpellFactJson::Known(map(value)),
    }
}

fn classification_json(value: &SpellClassification) -> SpellClassificationJson {
    SpellClassificationJson {
        rank: map_fact(&value.rank, |value| *value),
        traits: map_fact(&value.traits, |values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect()
        }),
        traditions: map_fact(&value.traditions, |values| {
            values
                .iter()
                .map(|value| value.as_str().to_string())
                .collect()
        }),
    }
}

fn casting_json(value: &SpellCasting) -> SpellCastingJson {
    SpellCastingJson {
        time: map_fact(&value.time, Clone::clone),
        cost: map_fact(&value.cost, Clone::clone),
        requirements: map_fact(&value.requirements, Clone::clone),
        counteraction: map_fact(&value.counteraction, |value| *value),
    }
}

fn targeting_json(value: &SpellTargeting) -> SpellTargetingJson {
    SpellTargetingJson {
        target: map_fact(&value.target, Clone::clone),
        range: map_fact(&value.range, |value| SpellRangeJson {
            authored_text: value.authored_text.clone(),
        }),
        area: map_fact(&value.area, area_json),
    }
}

fn area_json(value: &SpellAreaValue) -> SpellAreaJson {
    SpellAreaJson {
        value: map_fact(&value.value, |value| *value),
        area_type: map_fact(&value.area_type, |value| value.as_str().to_string()),
        details: map_fact(&value.details, Clone::clone),
    }
}

fn defense_json(value: &SpellDefenseValue) -> SpellDefenseJson {
    SpellDefenseJson {
        passive: map_fact(&value.passive, |value| value.as_str().to_string()),
        save: map_fact(&value.save, save_json),
    }
}

fn save_json(value: &SpellSave) -> SpellSaveJson {
    SpellSaveJson {
        statistic: map_fact(&value.statistic, |value| value.as_str().to_string()),
        basic: map_fact(&value.basic, |value| *value),
    }
}

fn damage_json(values: &[SpellOrderedMember<SpellDamage>]) -> Vec<SpellDamageJson> {
    values
        .iter()
        .map(|member| SpellDamageJson {
            key: member.key.clone(),
            order: member.authored_order,
            formula: map_fact(&member.value.formula, Clone::clone),
            damage_type: map_fact(&member.value.damage_type, Clone::clone),
            category: map_fact(&member.value.category, Clone::clone),
            kinds: map_fact(&member.value.kinds, Clone::clone),
            materials: map_fact(&member.value.materials, Clone::clone),
            apply_modifier: map_fact(&member.value.apply_mod, |value| *value),
        })
        .collect()
}

fn duration_json(value: &SpellDuration) -> SpellDurationJson {
    SpellDurationJson {
        value: map_fact(&value.value, Clone::clone),
        sustained: map_fact(&value.sustained, |value| *value),
    }
}

fn heightening_json(value: &SpellHeightening) -> SpellHeighteningJson {
    match value {
        SpellHeightening::Interval(value) => interval_heightening_json(value),
        SpellHeightening::Fixed(layers) => SpellHeighteningJson::Fixed {
            layers: layers.iter().map(fixed_heightening_json).collect(),
        },
    }
}

fn interval_heightening_json(value: &SpellIntervalHeightening) -> SpellHeighteningJson {
    SpellHeighteningJson::Interval {
        interval: map_fact(&value.interval, |value| *value),
        area: map_fact(&value.area, |value| *value),
        damage: map_fact(&value.damage, |values| {
            values
                .iter()
                .map(|value| SpellHeighteningDamageJson {
                    key: value.key.clone(),
                    order: value.authored_order,
                    value: value.value.clone(),
                })
                .collect()
        }),
    }
}

fn fixed_heightening_json(value: &SpellFixedHeighteningLayer) -> SpellFixedHeighteningJson {
    SpellFixedHeighteningJson {
        key: value.key.clone(),
        order: value.authored_order,
        rank: match &value.rank {
            SpellSourceValue::Known(rank) => SpellFactJson::Known(*rank),
            SpellSourceValue::Unsupported(value) => {
                SpellFactJson::Unsupported(unsupported_json(value))
            }
        },
        changed_fields: patch_fields(&value.patch),
        patch: patch_json(&value.patch),
    }
}

fn ritual_json(value: &SpellRitual) -> SpellRitualJson {
    SpellRitualJson {
        primary_check: map_fact(&value.primary_check, Clone::clone),
        secondary_casters: map_fact(&value.secondary_casters, |value| *value),
        secondary_checks: map_fact(&value.secondary_checks, Clone::clone),
    }
}

fn rules_json(values: &[SpellRuleElement]) -> Vec<SpellRuleJson> {
    values
        .iter()
        .map(|value| SpellRuleJson {
            authored_key: value.authored_key.clone(),
            order: value.authored_order,
            rule: match &value.rule {
                SpellRule::DamageDice(rule) => {
                    SpellRuleDetailJson::DamageDice(Box::new(SpellDamageDiceRuleJson {
                        selector: map_fact(&rule.selector, Clone::clone),
                        predicate: rule_predicates(&rule.predicate),
                        dice_number: map_fact(&rule.dice_number, Clone::clone),
                        die_size: map_fact(&rule.die_size, Clone::clone),
                        damage_type: map_fact(&rule.damage_type, Clone::clone),
                        hide_if_disabled: map_fact(&rule.hide_if_disabled, |value| *value),
                    }))
                }
                SpellRule::EphemeralEffect(rule) => {
                    SpellRuleDetailJson::EphemeralEffect(Box::new(SpellEphemeralEffectRuleJson {
                        predicate: rule_predicates(&rule.predicate),
                        selectors: map_fact(&rule.selectors, Clone::clone),
                        uuid: map_fact(&rule.uuid, Clone::clone),
                    }))
                }
                SpellRule::DamageAlteration(rule) => {
                    SpellRuleDetailJson::DamageAlteration(Box::new(SpellDamageAlterationRuleJson {
                        mode: map_fact(&rule.mode, Clone::clone),
                        predicate: rule_predicates(&rule.predicate),
                        property: map_fact(&rule.property, Clone::clone),
                        selectors: map_fact(&rule.selectors, Clone::clone),
                        slug: map_fact(&rule.slug, Clone::clone),
                        value: map_fact(&rule.value, Clone::clone),
                    }))
                }
                SpellRule::RollOption(rule) => {
                    SpellRuleDetailJson::RollOption(Box::new(SpellRollOptionRuleJson {
                        domain: map_fact(&rule.domain, Clone::clone),
                        label: map_fact(&rule.label, Clone::clone),
                        option: map_fact(&rule.option, Clone::clone),
                        placement: map_fact(&rule.placement, Clone::clone),
                        predicate: rule_predicates(&rule.predicate),
                        suboptions: map_fact(&rule.suboptions, |values| {
                            values
                                .iter()
                                .map(|value| SpellRuleSuboptionJson {
                                    label: map_fact(&value.label, Clone::clone),
                                    value: map_fact(&value.value, Clone::clone),
                                })
                                .collect()
                        }),
                        toggleable: map_fact(&rule.toggleable, |value| *value),
                    }))
                }
                SpellRule::ItemAlteration(rule) => {
                    SpellRuleDetailJson::ItemAlteration(Box::new(SpellItemAlterationRuleJson {
                        item_id: map_fact(&rule.item_id, Clone::clone),
                        mode: map_fact(&rule.mode, Clone::clone),
                        predicate: rule_predicates(&rule.predicate),
                        property: map_fact(&rule.property, Clone::clone),
                        value: map_fact(&rule.value, Clone::clone),
                    }))
                }
                SpellRule::Unsupported(rule) => {
                    SpellRuleDetailJson::Unsupported(unsupported_json(&rule.value))
                }
            },
        })
        .collect()
}

fn rule_predicates(
    value: &SpellFact<Vec<crate::SpellRulePredicate>>,
) -> SpellFactJson<Vec<SpellRulePredicateJson>> {
    map_fact(value, |values| {
        values
            .iter()
            .map(|value| match value {
                crate::SpellRulePredicate::Term(value) => {
                    SpellRulePredicateJson::Term(value.clone())
                }
                crate::SpellRulePredicate::Or(values) => SpellRulePredicateJson::Or(values.clone()),
                crate::SpellRulePredicate::Unsupported(value) => {
                    SpellRulePredicateJson::Unsupported(unsupported_json(&value.value))
                }
            })
            .collect()
    })
}

fn patch_json(patch: &crate::SpellPatch) -> SpellPatchJson {
    SpellPatchJson {
        classification: map_fact(&patch.classification, |value| SpellClassificationJson {
            rank: map_fact(&value.rank, |value| *value),
            traits: map_fact(&value.traits, |values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            }),
            traditions: map_fact(&value.traditions, |values| {
                values
                    .iter()
                    .map(|value| value.as_str().to_string())
                    .collect()
            }),
        }),
        casting: map_fact(&patch.casting, |value| SpellCastingJson {
            time: map_fact(&value.time, Clone::clone),
            cost: map_fact(&value.cost, Clone::clone),
            requirements: map_fact(&value.requirements, Clone::clone),
            counteraction: map_fact(&value.counteraction, |value| *value),
        }),
        targeting: map_fact(&patch.targeting, |value| SpellTargetingJson {
            target: map_fact(&value.target, Clone::clone),
            range: map_fact(&value.range, |value| SpellRangeJson {
                authored_text: value.authored_text.clone(),
            }),
            area: map_fact(&value.area, |value| SpellAreaJson {
                value: map_fact(&value.value, |value| *value),
                area_type: map_fact(&value.area_type, |value| value.as_str().to_string()),
                details: map_fact(&value.details, Clone::clone),
            }),
        }),
        defense: map_fact(&patch.defense, |value| SpellDefenseJson {
            passive: map_fact(&value.passive, |value| value.as_str().to_string()),
            save: map_fact(&value.save, |value| SpellSaveJson {
                statistic: map_fact(&value.statistic, |value| value.as_str().to_string()),
                basic: map_fact(&value.basic, |value| *value),
            }),
        }),
        damage: map_fact(&patch.damage, |value| SpellDamagePatchSetJson {
            members: value
                .members
                .iter()
                .map(|member| SpellDamagePatchMemberJson {
                    key: member.key.clone(),
                    order: member.authored_order,
                    operation: match &member.operation {
                        crate::SpellKeyedPatchOperation::Merge(value) => {
                            SpellDamagePatchOperationJson::Merge(Box::new(SpellDamagePatchJson {
                                formula: map_fact(&value.formula, Clone::clone),
                                damage_type: map_fact(&value.damage_type, Clone::clone),
                                category: map_fact(&value.category, Clone::clone),
                                kinds: map_fact(&value.kinds, Clone::clone),
                                materials: map_fact(&value.materials, Clone::clone),
                                apply_modifier: map_fact(&value.apply_mod, |value| *value),
                            }))
                        }
                        crate::SpellKeyedPatchOperation::Delete => {
                            SpellDamagePatchOperationJson::Delete
                        }
                        crate::SpellKeyedPatchOperation::Unsupported(value) => {
                            SpellDamagePatchOperationJson::Unsupported(unsupported_json(value))
                        }
                    },
                })
                .collect(),
        }),
        duration: map_fact(&patch.duration, |value| SpellDurationJson {
            value: map_fact(&value.value, Clone::clone),
            sustained: map_fact(&value.sustained, |value| *value),
        }),
        heightening: map_fact(&patch.heightening, |value| SpellHeighteningPatchJson {
            kind: map_fact(&value.kind, |_| "interval".to_string()),
            interval: map_fact(&value.interval, |value| *value),
            area: map_fact(&value.area, |value| *value),
            damage: map_fact(&value.interval_damage, |value| SpellTextPatchSetJson {
                members: value
                    .members
                    .iter()
                    .map(|member| SpellTextPatchMemberJson {
                        key: member.key.clone(),
                        order: member.authored_order,
                        operation: match &member.operation {
                            crate::SpellKeyedPatchOperation::Merge(value) => {
                                SpellTextPatchOperationJson::Merge(map_fact(
                                    &value.value,
                                    Clone::clone,
                                ))
                            }
                            crate::SpellKeyedPatchOperation::Delete => {
                                SpellTextPatchOperationJson::Delete
                            }
                            crate::SpellKeyedPatchOperation::Unsupported(value) => {
                                SpellTextPatchOperationJson::Unsupported(unsupported_json(value))
                            }
                        },
                    })
                    .collect(),
            }),
        }),
        rules: map_fact(&patch.rules, |values| rules_json(values)),
        unsupported: patch
            .unsupported
            .iter()
            .map(|value| SpellUnsupportedFactJson {
                field: form_field(value.field),
                source_path: value.source_path.clone(),
                authored_key: value.authored_key.clone(),
                order: value.authored_order,
                value: unsupported_json(&value.value),
            })
            .collect(),
    }
}

fn patch_fields(patch: &crate::SpellPatch) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if !matches!(patch.classification, FactValue::Missing) {
        fields.push("classification");
    }
    if !matches!(patch.casting, FactValue::Missing) {
        fields.push("casting");
    }
    if !matches!(patch.targeting, FactValue::Missing) {
        fields.push("targeting");
    }
    if !matches!(patch.defense, FactValue::Missing) {
        fields.push("defense");
    }
    if !matches!(patch.damage, FactValue::Missing) {
        fields.push("damage");
    }
    if !matches!(patch.duration, FactValue::Missing) {
        fields.push("duration");
    }
    if !matches!(patch.heightening, FactValue::Missing) {
        fields.push("heightening");
    }
    if !matches!(patch.rules, FactValue::Missing) {
        fields.push("rules");
    }
    fields.extend(
        patch
            .unsupported
            .iter()
            .map(|field| form_field(field.field)),
    );
    fields.sort_unstable();
    fields.dedup();
    fields
}

fn spell_forms(spell: &SpellRecord) -> (Vec<SpellFormJson>, Option<String>) {
    let cast_rank = base_rank(spell).unwrap_or(0);
    let base_id = SpellFormId::base(&spell.identity.record_key);
    let mut forms = vec![form_json(
        spell,
        base_id,
        "Base".to_string(),
        0,
        cast_rank,
        None,
        "base",
    )];
    match spell.ordered_overlays() {
        Ok(overlays) => {
            forms.extend(overlays.into_iter().enumerate().map(|(index, overlay)| {
                let label = overlay_label(overlay, index);
                form_json(
                    spell,
                    overlay.form_id(&spell.identity.record_key),
                    label,
                    u32::try_from(index + 1).unwrap_or(u32::MAX),
                    cast_rank,
                    Some(overlay),
                    "overlay",
                )
            }));
            (forms, None)
        }
        Err(error) => (
            forms,
            Some(format!("overlay catalog unavailable: {error:?}")),
        ),
    }
}

fn form_json(
    spell: &SpellRecord,
    id: SpellFormId,
    label: String,
    order: u32,
    cast_rank: u8,
    overlay: Option<&SpellOverlay>,
    kind: &'static str,
) -> SpellFormJson {
    let context = SpellFormContext {
        cast_rank,
        overlay_id: overlay.map(|value| value.overlay_id.clone()),
    };
    let result = match spell.resolve_form(id.clone(), context) {
        Ok(value) => SpellFormResultJson::Available {
            definition: Box::new(resolved_definition_json(value)),
        },
        Err(error) => SpellFormResultJson::Unavailable {
            message: selection_error(error),
        },
    };
    SpellFormJson {
        id: id.as_str().to_string(),
        label,
        order,
        cast_rank,
        kind,
        result,
    }
}

fn resolved_definition_json(value: ResolvedSpellForm) -> SpellResolvedDefinitionJson {
    SpellResolvedDefinitionJson {
        applied_fixed_ranks: value.applied_fixed_ranks,
        classification: resolved_field(value.classification, classification_json),
        casting: resolved_field(value.casting, casting_json),
        targeting: resolved_field(value.targeting, targeting_json),
        defense: resolved_field(value.defense, defense_json),
        damage: resolved_field(value.damage, |values| damage_json(values)),
        duration: resolved_field(value.duration, duration_json),
        heightening: resolved_field(value.heightening, heightening_json),
        rules: resolved_field(value.rules, |values| rules_json(values)),
    }
}

fn resolved_field<T, U>(
    value: SpellResolvedField<SpellFact<T>>,
    map: impl FnOnce(&T) -> U,
) -> SpellResolvedFieldJson<U> {
    match value {
        SpellResolvedField::Available(value) => SpellResolvedFieldJson::Available {
            value: map_fact(&value, map),
        },
        SpellResolvedField::Unavailable(value) => SpellResolvedFieldJson::Unavailable {
            field: form_field(value.field),
            source: patch_source(value.source),
            reason: unavailable_reason(value.reason),
        },
    }
}

fn spell_content(spell: &SpellRecord) -> Vec<SpellContentJson> {
    let mut documents = spell
        .definition
        .content
        .documents
        .iter()
        .filter(|document| document.visibility == ContentVisibility::Public)
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| (document.authored_order, document.id.content_key.as_str()));
    documents
        .into_iter()
        .map(|document| SpellContentJson {
            content_key: document.id.content_key.as_str().to_string(),
            role: content_role(document.role),
            order: document.authored_order,
            label: document.label.clone(),
            content_hash: document.content_hash.as_str().to_string(),
            document: project_presentation_content(&document.document),
        })
        .collect()
}

fn spell_provenance(spell: &SpellRecord) -> SpellProvenanceJson {
    let definition = &spell.definition;
    let mut members = Vec::new();
    let mut unsupported = definition
        .unsupported_notes
        .iter()
        .map(unsupported_source_fact_json)
        .collect::<Vec<_>>();
    if let Some(damage) = known_fact(&definition.damage) {
        for member in damage {
            members.push(member_provenance(
                "damage",
                format!("system.damage.{}", member.key),
                &member.key,
                member.authored_order,
                "known",
            ));
            collect_damage_provenance(
                &member.value,
                &format!("system.damage.{}", member.key),
                Some(member.authored_order),
                &mut unsupported,
            );
        }
    }
    if let Some(targeting) = known_fact(&definition.targeting)
        && let Some(area) = known_fact(&targeting.area)
    {
        collect_area_provenance(area, "system.area", &mut unsupported);
    }
    if let Some(heightening) = known_fact(&definition.heightening) {
        match heightening {
            SpellHeightening::Interval(value) => {
                if let Some(damage) = known_fact(&value.damage) {
                    for member in damage {
                        members.push(member_provenance(
                            "heightening_damage",
                            format!("system.heightening.damage.{}", member.key),
                            &member.key,
                            member.authored_order,
                            "known",
                        ));
                    }
                }
            }
            SpellHeightening::Fixed(layers) => {
                for layer in layers {
                    let path = format!("system.heightening.levels.{}", layer.key);
                    members.push(member_provenance(
                        "fixed_heightening",
                        path.clone(),
                        &layer.key,
                        layer.authored_order,
                        match layer.rank {
                            SpellSourceValue::Known(_) => "known",
                            SpellSourceValue::Unsupported(_) => "unsupported",
                        },
                    ));
                    if let SpellSourceValue::Unsupported(value) = &layer.rank {
                        unsupported.push(unsupported_at(
                            "fixed_heightening",
                            path.clone(),
                            &layer.key,
                            Some(layer.authored_order),
                            value,
                        ));
                    }
                    collect_patch_provenance(&layer.patch, &path, &mut members, &mut unsupported);
                }
            }
        }
    }
    if let Some(rules) = known_fact(&definition.rules) {
        collect_rule_provenance(rules, &mut members, &mut unsupported);
    }
    if let Some(overlays) = known_fact(&definition.overlays) {
        for overlay in overlays {
            let path = format!("system.overlays.{}", overlay.key);
            members.push(member_provenance(
                "overlay",
                path.clone(),
                &overlay.key,
                overlay.authored_order,
                "known",
            ));
            if let FactValue::Value(SpellSourceValue::Unsupported(value)) = &overlay.overlay_type {
                unsupported.push(unsupported_at(
                    "overlay_member",
                    format!("{path}.overlayType"),
                    "overlayType",
                    None,
                    value,
                ));
            }
            collect_patch_provenance(
                &overlay.patch,
                &format!("{path}.system"),
                &mut members,
                &mut unsupported,
            );
        }
    }
    SpellProvenanceJson {
        source_id: spell.identity.source_id.as_str().to_string(),
        source_path: definition.provenance.source_path.clone(),
        source_contract_version: definition.provenance.source_contract_version.clone(),
        source_system_version: definition.provenance.source_system_version.clone(),
        source_upstream_commit: definition.provenance.source_upstream_commit.clone(),
        standalone_location: map_plain_fact(&definition.provenance.standalone_location, |value| {
            value.value.clone()
        }),
        image: map_fact(&definition.source_context.image, Clone::clone),
        publication_license: map_fact(&definition.source_context.publication_license, |value| {
            value.as_str().to_string()
        }),
        members,
        unsupported,
    }
}

fn known_fact<T>(fact: &SpellFact<T>) -> Option<&T> {
    fact.as_value().and_then(SpellSourceValue::as_known)
}

fn member_provenance(
    field: &'static str,
    source_path: String,
    authored_key: &str,
    order: u32,
    state: &'static str,
) -> SpellMemberProvenanceJson {
    SpellMemberProvenanceJson {
        field,
        source_path,
        authored_key: authored_key.to_string(),
        order,
        state,
    }
}

fn unsupported_source_fact_json(
    value: &crate::SpellUnsupportedSourceFact,
) -> SpellUnsupportedFactJson {
    unsupported_at(
        unsupported_field(value.field),
        value.source_path.clone(),
        &value.authored_key,
        value.authored_order,
        &value.value,
    )
}

fn unsupported_at(
    field: &'static str,
    source_path: String,
    authored_key: &str,
    order: Option<u32>,
    value: &UnsupportedSourceValue,
) -> SpellUnsupportedFactJson {
    SpellUnsupportedFactJson {
        field,
        source_path,
        authored_key: authored_key.to_string(),
        order,
        value: unsupported_json(value),
    }
}

fn collect_rule_provenance(
    rules: &[SpellRuleElement],
    members: &mut Vec<SpellMemberProvenanceJson>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    for rule in rules {
        let state = if matches!(rule.rule, SpellRule::Unsupported(_)) {
            "unsupported"
        } else {
            "known"
        };
        members.push(member_provenance(
            "rule",
            rule.source_path.clone(),
            &rule.authored_key,
            rule.authored_order,
            state,
        ));
        if let SpellRule::Unsupported(value) = &rule.rule {
            unsupported.push(unsupported_at(
                "rule",
                value.source_path.clone(),
                &value.authored_key,
                Some(rule.authored_order),
                &value.value,
            ));
            continue;
        }
        collect_rule_field_provenance(rule, unsupported);
    }
}

fn collect_rule_field_provenance(
    element: &SpellRuleElement,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    let path = &element.source_path;
    let order = Some(element.authored_order);
    macro_rules! field {
        ($fact:expr, $key:literal) => {
            collect_unsupported_fact(
                $fact,
                "rule",
                format!("{path}.{}", $key),
                $key,
                order,
                unsupported,
            )
        };
    }
    let predicate = match &element.rule {
        SpellRule::DamageDice(rule) => {
            field!(&rule.selector, "selector");
            field!(&rule.dice_number, "diceNumber");
            field!(&rule.die_size, "dieSize");
            field!(&rule.damage_type, "damageType");
            collect_unsupported_fact(
                &rule.hide_if_disabled,
                "rule",
                format!("{path}.hideIfDisabled"),
                "hideIfDisabled",
                order,
                unsupported,
            );
            &rule.predicate
        }
        SpellRule::EphemeralEffect(rule) => {
            collect_unsupported_fact(
                &rule.selectors,
                "rule",
                format!("{path}.selectors"),
                "selectors",
                order,
                unsupported,
            );
            field!(&rule.uuid, "uuid");
            &rule.predicate
        }
        SpellRule::DamageAlteration(rule) => {
            field!(&rule.mode, "mode");
            field!(&rule.property, "property");
            collect_unsupported_fact(
                &rule.selectors,
                "rule",
                format!("{path}.selectors"),
                "selectors",
                order,
                unsupported,
            );
            field!(&rule.slug, "slug");
            field!(&rule.value, "value");
            &rule.predicate
        }
        SpellRule::RollOption(rule) => {
            field!(&rule.domain, "domain");
            field!(&rule.label, "label");
            field!(&rule.option, "option");
            field!(&rule.placement, "placement");
            collect_unsupported_fact(
                &rule.toggleable,
                "rule",
                format!("{path}.toggleable"),
                "toggleable",
                order,
                unsupported,
            );
            collect_unsupported_fact(
                &rule.suboptions,
                "rule",
                format!("{path}.suboptions"),
                "suboptions",
                order,
                unsupported,
            );
            if let Some(suboptions) = known_fact(&rule.suboptions) {
                for (index, suboption) in suboptions.iter().enumerate() {
                    collect_unsupported_fact(
                        &suboption.label,
                        "rule_suboption",
                        format!("{path}.suboptions.{index}.label"),
                        "label",
                        Some(index as u32),
                        unsupported,
                    );
                    collect_unsupported_fact(
                        &suboption.value,
                        "rule_suboption",
                        format!("{path}.suboptions.{index}.value"),
                        "value",
                        Some(index as u32),
                        unsupported,
                    );
                }
            }
            &rule.predicate
        }
        SpellRule::ItemAlteration(rule) => {
            field!(&rule.item_id, "itemId");
            field!(&rule.mode, "mode");
            field!(&rule.property, "property");
            field!(&rule.value, "value");
            &rule.predicate
        }
        SpellRule::Unsupported(_) => return,
    };
    collect_unsupported_fact(
        predicate,
        "rule",
        format!("{path}.predicate"),
        "predicate",
        order,
        unsupported,
    );
    if let Some(predicates) = known_fact(predicate) {
        for predicate in predicates {
            if let crate::SpellRulePredicate::Unsupported(value) = predicate {
                unsupported.push(unsupported_at(
                    "rule_predicate",
                    value.source_path.clone(),
                    value.authored_key.as_deref().unwrap_or("predicate"),
                    Some(value.authored_order),
                    &value.value,
                ));
            }
        }
    }
}

fn collect_unsupported_fact<T>(
    fact: &SpellFact<T>,
    field: &'static str,
    source_path: String,
    authored_key: &str,
    order: Option<u32>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    if let FactValue::Value(SpellSourceValue::Unsupported(value)) = fact {
        unsupported.push(unsupported_at(
            field,
            source_path,
            authored_key,
            order,
            value,
        ));
    }
}

fn collect_patch_provenance(
    patch: &crate::SpellPatch,
    path: &str,
    members: &mut Vec<SpellMemberProvenanceJson>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    unsupported.extend(patch.unsupported.iter().map(|value| {
        unsupported_at(
            form_field(value.field),
            value.source_path.clone(),
            &value.authored_key,
            value.authored_order,
            &value.value,
        )
    }));
    collect_unsupported_fact(
        &patch.defense,
        "defense_member",
        format!("{path}.defense"),
        "defense",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &patch.damage,
        "damage_member",
        format!("{path}.damage"),
        "damage",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &patch.heightening,
        "heightening_member",
        format!("{path}.heightening"),
        "heightening",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &patch.rules,
        "rule",
        format!("{path}.rules"),
        "rules",
        None,
        unsupported,
    );
    if let Some(classification) = known_fact(&patch.classification) {
        collect_unsupported_fact(
            &classification.rank,
            "classification_member",
            format!("{path}.level.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &classification.traits,
            "classification_member",
            format!("{path}.traits.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &classification.traditions,
            "classification_member",
            format!("{path}.traits.traditions"),
            "traditions",
            None,
            unsupported,
        );
    }
    if let Some(casting) = known_fact(&patch.casting) {
        collect_unsupported_fact(
            &casting.time,
            "casting_member",
            format!("{path}.time.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &casting.cost,
            "casting_member",
            format!("{path}.cost.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &casting.requirements,
            "casting_member",
            format!("{path}.requirements"),
            "requirements",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &casting.counteraction,
            "casting_member",
            format!("{path}.counteraction"),
            "counteraction",
            None,
            unsupported,
        );
    }
    if let Some(targeting) = known_fact(&patch.targeting) {
        collect_unsupported_fact(
            &targeting.target,
            "targeting_member",
            format!("{path}.target.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &targeting.range,
            "targeting_member",
            format!("{path}.range.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &targeting.area,
            "targeting_member",
            format!("{path}.area"),
            "area",
            None,
            unsupported,
        );
        if let Some(area) = known_fact(&targeting.area) {
            collect_area_patch_provenance(area, &format!("{path}.area"), unsupported);
        }
    }
    if let Some(defense) = known_fact(&patch.defense) {
        collect_unsupported_fact(
            &defense.passive,
            "defense_member",
            format!("{path}.defense.passive.statistic"),
            "statistic",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &defense.save,
            "defense_member",
            format!("{path}.defense.save"),
            "save",
            None,
            unsupported,
        );
        if let Some(save) = known_fact(&defense.save) {
            collect_unsupported_fact(
                &save.statistic,
                "defense_member",
                format!("{path}.defense.save.statistic"),
                "statistic",
                None,
                unsupported,
            );
            collect_unsupported_fact(
                &save.basic,
                "defense_member",
                format!("{path}.defense.save.basic"),
                "basic",
                None,
                unsupported,
            );
        }
    }
    if let Some(damage) = known_fact(&patch.damage) {
        for member in &damage.members {
            let state = match member.operation {
                crate::SpellKeyedPatchOperation::Merge(_) => "known",
                crate::SpellKeyedPatchOperation::Delete => "delete",
                crate::SpellKeyedPatchOperation::Unsupported(_) => "unsupported",
            };
            members.push(member_provenance(
                "patch_damage",
                format!("{path}.damage.{}", member.key),
                &member.key,
                member.authored_order,
                state,
            ));
            match &member.operation {
                crate::SpellKeyedPatchOperation::Merge(value) => collect_damage_patch_provenance(
                    value,
                    &format!("{path}.damage.{}", member.key),
                    Some(member.authored_order),
                    unsupported,
                ),
                crate::SpellKeyedPatchOperation::Unsupported(value) => {
                    unsupported.push(unsupported_at(
                        "damage",
                        format!("{path}.damage.{}", member.key),
                        &member.key,
                        Some(member.authored_order),
                        value,
                    ))
                }
                crate::SpellKeyedPatchOperation::Delete => {}
            }
        }
    }
    if let Some(duration) = known_fact(&patch.duration) {
        collect_unsupported_fact(
            &duration.value,
            "duration_member",
            format!("{path}.duration.value"),
            "value",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &duration.sustained,
            "duration_member",
            format!("{path}.duration.sustained"),
            "sustained",
            None,
            unsupported,
        );
    }
    if let Some(heightening) = known_fact(&patch.heightening) {
        collect_unsupported_fact(
            &heightening.kind,
            "heightening_member",
            format!("{path}.heightening.type"),
            "type",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &heightening.interval,
            "heightening_member",
            format!("{path}.heightening.interval"),
            "interval",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &heightening.area,
            "heightening_member",
            format!("{path}.heightening.area"),
            "area",
            None,
            unsupported,
        );
        collect_unsupported_fact(
            &heightening.interval_damage,
            "heightening_member",
            format!("{path}.heightening.damage"),
            "damage",
            None,
            unsupported,
        );
        if let Some(damage) = known_fact(&heightening.interval_damage) {
            for member in &damage.members {
                let state = match member.operation {
                    crate::SpellKeyedPatchOperation::Merge(_) => "known",
                    crate::SpellKeyedPatchOperation::Delete => "delete",
                    crate::SpellKeyedPatchOperation::Unsupported(_) => "unsupported",
                };
                members.push(member_provenance(
                    "patch_heightening_damage",
                    format!("{path}.heightening.damage.{}", member.key),
                    &member.key,
                    member.authored_order,
                    state,
                ));
                match &member.operation {
                    crate::SpellKeyedPatchOperation::Merge(value) => collect_unsupported_fact(
                        &value.value,
                        "heightening_member",
                        format!("{path}.heightening.damage.{}", member.key),
                        &member.key,
                        Some(member.authored_order),
                        unsupported,
                    ),
                    crate::SpellKeyedPatchOperation::Unsupported(value) => {
                        unsupported.push(unsupported_at(
                            "heightening",
                            format!("{path}.heightening.damage.{}", member.key),
                            &member.key,
                            Some(member.authored_order),
                            value,
                        ))
                    }
                    crate::SpellKeyedPatchOperation::Delete => {}
                }
            }
        }
    }
    if let Some(rules) = known_fact(&patch.rules) {
        collect_rule_provenance(rules, members, unsupported);
    }
}

fn collect_damage_provenance(
    damage: &crate::SpellDamage,
    path: &str,
    order: Option<u32>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    collect_damage_fields_provenance(
        &damage.apply_mod,
        &damage.category,
        &damage.formula,
        &damage.kinds,
        &damage.materials,
        &damage.damage_type,
        path,
        order,
        unsupported,
    );
}

fn collect_damage_patch_provenance(
    damage: &crate::SpellDamagePatch,
    path: &str,
    order: Option<u32>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    collect_damage_fields_provenance(
        &damage.apply_mod,
        &damage.category,
        &damage.formula,
        &damage.kinds,
        &damage.materials,
        &damage.damage_type,
        path,
        order,
        unsupported,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_damage_fields_provenance(
    apply_mod: &SpellFact<bool>,
    category: &SpellFact<String>,
    formula: &SpellFact<String>,
    kinds: &SpellFact<Vec<String>>,
    materials: &SpellFact<Vec<String>>,
    damage_type: &SpellFact<String>,
    path: &str,
    order: Option<u32>,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    collect_unsupported_fact(
        apply_mod,
        "damage_member",
        format!("{path}.applyMod"),
        "applyMod",
        order,
        unsupported,
    );
    collect_unsupported_fact(
        category,
        "damage_member",
        format!("{path}.category"),
        "category",
        order,
        unsupported,
    );
    collect_unsupported_fact(
        formula,
        "damage_member",
        format!("{path}.formula"),
        "formula",
        order,
        unsupported,
    );
    collect_unsupported_fact(
        kinds,
        "damage_member",
        format!("{path}.kinds"),
        "kinds",
        order,
        unsupported,
    );
    collect_unsupported_fact(
        materials,
        "damage_member",
        format!("{path}.materials"),
        "materials",
        order,
        unsupported,
    );
    collect_unsupported_fact(
        damage_type,
        "damage_member",
        format!("{path}.type"),
        "type",
        order,
        unsupported,
    );
}

fn collect_area_provenance(
    area: &SpellAreaValue,
    path: &str,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    collect_unsupported_fact(
        &area.value,
        "area_member",
        format!("{path}.value"),
        "value",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.area_type,
        "area_member",
        format!("{path}.type"),
        "type",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.legacy_area_type,
        "area_member",
        format!("{path}.areaType"),
        "areaType",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.details,
        "area_member",
        format!("{path}.details"),
        "details",
        None,
        unsupported,
    );
}

fn collect_area_patch_provenance(
    area: &crate::SpellAreaPatch,
    path: &str,
    unsupported: &mut Vec<SpellUnsupportedFactJson>,
) {
    collect_unsupported_fact(
        &area.value,
        "area_member",
        format!("{path}.value"),
        "value",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.area_type,
        "area_member",
        format!("{path}.type"),
        "type",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.legacy_area_type,
        "area_member",
        format!("{path}.areaType"),
        "areaType",
        None,
        unsupported,
    );
    collect_unsupported_fact(
        &area.details,
        "area_member",
        format!("{path}.details"),
        "details",
        None,
        unsupported,
    );
}

fn base_rank(spell: &SpellRecord) -> Option<u8> {
    match &spell.definition.classification {
        FactValue::Value(SpellSourceValue::Known(value)) => match &value.rank {
            FactValue::Value(SpellSourceValue::Known(rank)) => Some(*rank),
            _ => None,
        },
        _ => None,
    }
}

fn overlay_label(overlay: &SpellOverlay, index: usize) -> String {
    match &overlay.name {
        FactValue::Value(SpellSourceValue::Known(name)) if !name.trim().is_empty() => name.clone(),
        _ => format!("Overlay {}", index + 1),
    }
}

fn unsupported_json(value: &UnsupportedSourceValue) -> SpellUnsupportedValueJson {
    SpellUnsupportedValueJson {
        shape: match value.shape {
            UnsupportedSourceShape::Missing => "missing",
            UnsupportedSourceShape::Null => "null",
            UnsupportedSourceShape::String => "string",
            UnsupportedSourceShape::Number => "number",
            UnsupportedSourceShape::Boolean => "boolean",
            UnsupportedSourceShape::Array => "array",
            UnsupportedSourceShape::Object => "object",
        },
        value: value.value.clone(),
        reason: match value.reason {
            UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
            UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
            UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
            UnsupportedSourceReason::NonCanonicalRuntimeValue => "noncanonical_runtime_value",
            UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
        },
    }
}

fn selection_error(value: SpellFormSelectionError) -> String {
    format!("{value:?}")
}

fn form_field(value: SpellFormField) -> &'static str {
    match value {
        SpellFormField::Classification => "classification",
        SpellFormField::Casting => "casting",
        SpellFormField::Targeting => "targeting",
        SpellFormField::Defense => "defense",
        SpellFormField::Damage => "damage",
        SpellFormField::Duration => "duration",
        SpellFormField::Heightening => "heightening",
        SpellFormField::Rules => "rules",
    }
}

fn patch_source(value: SpellFormPatchSource) -> &'static str {
    match value {
        SpellFormPatchSource::Base => "base",
        SpellFormPatchSource::Overlay => "overlay",
        SpellFormPatchSource::FixedHeightening => "fixed_heightening",
    }
}

fn unavailable_reason(value: SpellFormUnavailableReason) -> String {
    format!("{value:?}")
}

fn content_role(value: ContentRole) -> &'static str {
    match value {
        ContentRole::PrimaryDescription => "primary_description",
        ContentRole::Summary => "summary",
        ContentRole::SupplementalRules => "supplemental_rules",
        ContentRole::EmbeddedCapability => "embedded_capability",
        ContentRole::JournalPage => "journal_page",
        ContentRole::TableResult => "table_result",
        ContentRole::GeneratedNarrative => "generated_narrative",
        ContentRole::Provenance => "provenance",
    }
}

fn unsupported_field(value: crate::SpellUnsupportedSourceField) -> &'static str {
    use crate::SpellUnsupportedSourceField as Field;
    match value {
        Field::SystemMember => "system_member",
        Field::ProvenanceMember => "provenance_member",
        Field::ClassificationMember => "classification_member",
        Field::CastingMember => "casting_member",
        Field::TargetingMember => "targeting_member",
        Field::DefenseMember => "defense_member",
        Field::DurationMember => "duration_member",
        Field::LocationMember => "location_member",
        Field::LegacyTraitSelection => "legacy_trait_selection",
        Field::AreaMember => "area_member",
        Field::DamageMember => "damage_member",
        Field::HeighteningMember => "heightening_member",
        Field::OverlayMember => "overlay_member",
        Field::RitualMember => "ritual_member",
    }
}
