use serde::Serialize;

use crate::{
    ConsumableDamage, ConsumableDefinition, ConsumableEntityTarget, ConsumableFact,
    ConsumableMaterial, ConsumableMismatchValue, ConsumableOccurrenceIdentityStability,
    ConsumableOccurrenceSet, ConsumablePrice, ConsumablePublication, ConsumableRecord,
    ConsumableSourceState, ConsumableSourceValue, ConsumableSpellMismatchReason,
    ConsumableSpellReuse, ConsumableTargetResolution, FactValue, SpellChildId,
    UnsupportedSourceReason, UnsupportedSourceShape,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableJson {
    pub definition: ConsumableDefinitionJson,
    pub source_state: ConsumableSourceStateJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ConsumableContentJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_content: Vec<ConsumableUnsupportedJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ConsumableProvenanceJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableDefinitionJson {
    pub slug: ConsumableFactJson<String>,
    pub level: ConsumableFactJson<i64>,
    pub category: ConsumableFactJson<String>,
    pub rarity: ConsumableFactJson<String>,
    pub traits: ConsumableFactJson<Vec<String>>,
    pub other_tags: ConsumableFactJson<Vec<String>>,
    pub base_item: ConsumableFactJson<String>,
    pub bulk: ConsumableFactJson<String>,
    pub size: ConsumableFactJson<String>,
    pub stack_group: ConsumableFactJson<String>,
    pub material: ConsumableFactJson<ConsumableMaterialJson>,
    pub price: ConsumableFactJson<ConsumablePriceJson>,
    pub usage: ConsumableFactJson<String>,
    pub maximum_uses: ConsumableFactJson<i64>,
    pub auto_destroy: ConsumableFactJson<bool>,
    pub maximum_hp: ConsumableFactJson<i64>,
    pub hardness: ConsumableFactJson<i64>,
    pub damage: ConsumableFactJson<ConsumableDamageJson>,
    pub publication: ConsumableFactJson<ConsumablePublicationJson>,
    pub rules: ConsumableFactJson<Vec<ConsumableUnsupportedJson>>,
    pub spell_child_id: ConsumableSpellChildFactJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableSourceStateJson {
    pub quantity: ConsumableFactJson<i64>,
    pub current_uses: ConsumableFactJson<i64>,
    pub current_hp: ConsumableFactJson<i64>,
    pub container_id: ConsumableFactJson<String>,
    pub equipped: ConsumableFactJson<ConsumableEquippedStateJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableEquippedStateJson {
    pub carry_type: ConsumableFactJson<String>,
    pub hands_held: ConsumableFactJson<i64>,
    pub in_slot: ConsumableFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumablePriceJson {
    pub denominations: ConsumableFactJson<Vec<ConsumablePriceDenominationJson>>,
    pub per: ConsumableFactJson<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumablePriceDenominationJson {
    pub denomination: String,
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableDamageJson {
    pub formula: ConsumableFactJson<String>,
    pub category: ConsumableFactJson<String>,
    pub damage_type: ConsumableFactJson<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumablePublicationJson {
    pub title: ConsumableFactJson<String>,
    pub license: ConsumableFactJson<String>,
    pub remaster: ConsumableFactJson<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableMaterialJson {
    pub grade: ConsumableFactJson<String>,
    pub material_type: ConsumableFactJson<String>,
    pub effects: ConsumableFactJson<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ConsumableSpellChildFactJson {
    Missing,
    Null,
    Known(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableProvenanceJson {
    pub image: ConsumableFactJson<String>,
    pub folder: ConsumableFactJson<String>,
    pub source_sort: ConsumableFactJson<i64>,
    pub target_locator: ConsumableLocatorStateJson,
    pub source_path: String,
    pub source_contract_version: String,
    pub source_system_version: String,
    pub source_upstream_commit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableContentJson {
    pub content_key: String,
    pub authored_order: u32,
    pub source_kind: &'static str,
    pub visibility: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub content_hash: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_from: Option<ConsumableCopiedContentJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableCopiedContentJson {
    pub record_key: String,
    pub content_key: String,
    pub content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableOccurrenceJson {
    pub occurrence_id: String,
    pub source_id: ConsumableFactJson<String>,
    pub authored_order: u32,
    pub name: String,
    pub source_path: String,
    pub identity_stability: &'static str,
    pub source_image: ConsumableFactJson<String>,
    pub source_folder: ConsumableFactJson<String>,
    pub source_sort: ConsumableFactJson<i64>,
    pub locator: ConsumableLocatorStateJson,
    pub target: ConsumableOccurrenceTargetJson,
    pub source_state: ConsumableSourceStateJson,
    pub spell_child: ConsumableSpellReuseJson,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub content: Vec<ConsumableContentJson>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsupported_content: Vec<ConsumableUnsupportedJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConsumableOccurrenceTargetJson {
    Resolved {
        record_key: String,
        immutable_mismatches: Vec<ConsumableMismatchJson>,
    },
    ParentOwned {
        definition: Box<ConsumableDefinitionJson>,
        resolution: ConsumableTargetResolutionJson,
        content_identity: ConsumableContentIdentityFactJson,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableMismatchJson {
    pub field_path: String,
    pub local_value: ConsumableMismatchValueJson,
    pub target_value: ConsumableMismatchValueJson,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "value_type", content = "fact", rename_all = "snake_case")]
pub enum ConsumableMismatchValueJson {
    String(ConsumableFactJson<String>),
    Integer(ConsumableFactJson<i64>),
    Boolean(ConsumableFactJson<bool>),
    Rarity(ConsumableFactJson<String>),
    Strings(ConsumableFactJson<Vec<String>>),
    ExactDecimal(ConsumableFactJson<String>),
    Material(ConsumableFactJson<ConsumableMaterialJson>),
    Price(ConsumableFactJson<ConsumablePriceJson>),
    Damage(ConsumableFactJson<ConsumableDamageJson>),
    Publication(ConsumableFactJson<ConsumablePublicationJson>),
    Rules(ConsumableFactJson<Vec<ConsumableUnsupportedJson>>),
    SpellChildId(ConsumableSpellChildFactJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConsumableTargetResolutionJson {
    NoLocator,
    MalformedOrDuplicateLocator { evidence: ConsumableUnsupportedJson },
    TargetMissing { locator: String },
    WrongDocumentOrFamily { locator: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConsumableLocatorStateJson {
    Missing,
    Null,
    Known { locator: String },
    Unsupported { evidence: ConsumableUnsupportedJson },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ConsumableContentIdentityFactJson {
    Missing,
    Null,
    Known {
        content_key: String,
        content_hash: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConsumableSpellReuseJson {
    NotPresent,
    Reused {
        target_child_id: String,
    },
    Mismatch {
        reason: &'static str,
        local_evidence: ConsumableLocalSpellEvidenceJson,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConsumableLocalSpellEvidenceJson {
    Missing,
    Null,
    Malformed { evidence: ConsumableUnsupportedJson },
    Child { child_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum ConsumableFactJson<T> {
    Missing,
    Null,
    Known(T),
    Unsupported(ConsumableUnsupportedJson),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableUnsupportedJson {
    pub shape: &'static str,
    pub value: String,
    pub reason: &'static str,
}

pub(super) fn consumable_json(
    record: &ConsumableRecord,
    include_provenance: bool,
) -> ConsumableJson {
    ConsumableJson {
        definition: definition_json(&record.definition),
        source_state: state_json(&record.source_state),
        content: record.content.documents.iter().map(content_json).collect(),
        unsupported_content: record
            .unsupported_content
            .iter()
            .map(unsupported_json)
            .collect(),
        provenance: include_provenance.then(|| ConsumableProvenanceJson {
            image: fact(&record.provenance.image, Clone::clone),
            folder: fact(&record.provenance.folder, Clone::clone),
            source_sort: fact(&record.provenance.source_sort, Clone::clone),
            target_locator: locator_json(&record.provenance.target_locator),
            source_path: record.provenance.source_path.clone(),
            source_contract_version: record.provenance.source_contract_version.clone(),
            source_system_version: record.provenance.source_system_version.clone(),
            source_upstream_commit: record.provenance.source_upstream_commit.clone(),
        }),
    }
}

pub(super) fn occurrence_json(
    set: &ConsumableOccurrenceSet,
) -> Result<Vec<ConsumableOccurrenceJson>, super::RecordJsonError> {
    let entities = set
        .validated_entities()
        .map_err(super::RecordJsonError::InvalidConsumableOccurrences)?;
    set.occurrences
        .iter()
        .map(|occurrence| {
            let entity = entities[&occurrence.entity_id];
            let target = match &entity.target {
                ConsumableEntityTarget::Resolved {
                    record_key,
                    immutable_mismatches,
                } => ConsumableOccurrenceTargetJson::Resolved {
                    record_key: record_key.to_string(),
                    immutable_mismatches: immutable_mismatches
                        .iter()
                        .map(|mismatch| ConsumableMismatchJson {
                            field_path: mismatch.field_path.clone(),
                            local_value: mismatch_value_json(&mismatch.local_value),
                            target_value: mismatch_value_json(&mismatch.target_value),
                        })
                        .collect(),
                },
                ConsumableEntityTarget::ParentOwned {
                    definition,
                    resolution,
                    content_identity,
                } => ConsumableOccurrenceTargetJson::ParentOwned {
                    definition: Box::new(definition_json(definition)),
                    resolution: target_resolution_json(resolution),
                    content_identity: match content_identity {
                        FactValue::Missing => ConsumableContentIdentityFactJson::Missing,
                        FactValue::Null => ConsumableContentIdentityFactJson::Null,
                        FactValue::Value(value) => ConsumableContentIdentityFactJson::Known {
                            content_key: value.content_key.clone(),
                            content_hash: value.content_hash.clone(),
                        },
                    },
                },
            };
            Ok(ConsumableOccurrenceJson {
                occurrence_id: occurrence.id.as_str().to_string(),
                source_id: fact(&occurrence.source_id, |value| value.as_str().to_string()),
                authored_order: occurrence.authored_order,
                name: occurrence.contextual_name.clone(),
                source_path: occurrence.source_path.clone(),
                identity_stability: match occurrence.identity_stability {
                    ConsumableOccurrenceIdentityStability::StableSourceIdentity => {
                        "stable_source_identity"
                    }
                    ConsumableOccurrenceIdentityStability::UnstableOwnerOrdinal => {
                        "unstable_owner_ordinal"
                    }
                },
                source_image: fact(&occurrence.source_image, Clone::clone),
                source_folder: fact(&occurrence.source_folder, Clone::clone),
                source_sort: fact(&occurrence.source_sort, Clone::clone),
                locator: locator_json(&occurrence.locator),
                target,
                source_state: state_json(&occurrence.state),
                spell_child: spell_reuse_json(&occurrence.spell_reuse),
                content: occurrence
                    .authored_content
                    .documents
                    .iter()
                    .map(content_json)
                    .collect(),
                unsupported_content: occurrence
                    .unsupported_content
                    .iter()
                    .map(unsupported_json)
                    .collect(),
            })
        })
        .collect()
}

fn locator_json(locator: &crate::ConsumableLocatorState) -> ConsumableLocatorStateJson {
    match locator {
        crate::ConsumableLocatorState::Missing => ConsumableLocatorStateJson::Missing,
        crate::ConsumableLocatorState::Null => ConsumableLocatorStateJson::Null,
        crate::ConsumableLocatorState::Known(locator) => ConsumableLocatorStateJson::Known {
            locator: locator.as_str().to_string(),
        },
        crate::ConsumableLocatorState::Unsupported(value) => {
            ConsumableLocatorStateJson::Unsupported {
                evidence: unsupported_json(value),
            }
        }
    }
}

fn content_json(document: &crate::OwnedRichContentDocument) -> ConsumableContentJson {
    ConsumableContentJson {
        content_key: document.id.content_key.as_str().to_string(),
        authored_order: document.authored_order,
        source_kind: document.source_kind.as_str(),
        visibility: document.visibility.as_str(),
        label: document.label.clone(),
        content_hash: document.content_hash.as_str().to_string(),
        text: crate::render_plain_text(&document.document),
        copied_from: match &document.duplicate_status {
            crate::DuplicateContentStatus::CopiedFromConsumableTarget {
                target_record_key,
                target_content_key,
                target_content_hash,
            } => Some(ConsumableCopiedContentJson {
                record_key: target_record_key.to_string(),
                content_key: target_content_key.as_str().to_string(),
                content_hash: target_content_hash.clone(),
            }),
            crate::DuplicateContentStatus::Unique
            | crate::DuplicateContentStatus::CopiedFromCanonicalTarget { .. } => None,
        },
    }
}

fn definition_json(value: &ConsumableDefinition) -> ConsumableDefinitionJson {
    ConsumableDefinitionJson {
        slug: fact(&value.slug, Clone::clone),
        level: fact(&value.level, |value| *value),
        category: fact(&value.category, Clone::clone),
        rarity: fact(&value.rarity, |value| value.as_str().to_string()),
        traits: fact(&value.traits, Clone::clone),
        other_tags: fact(&value.other_tags, Clone::clone),
        base_item: fact(&value.base_item, Clone::clone),
        bulk: fact(&value.bulk, |value| value.as_str().to_string()),
        size: fact(&value.size, Clone::clone),
        stack_group: fact(&value.stack_group, Clone::clone),
        material: fact(&value.material, material_json),
        price: fact(&value.price, price_json),
        usage: fact(&value.usage, Clone::clone),
        maximum_uses: fact(&value.maximum_uses, |value| *value),
        auto_destroy: fact(&value.auto_destroy, |value| *value),
        maximum_hp: fact(&value.maximum_hp, |value| *value),
        hardness: fact(&value.hardness, |value| *value),
        damage: fact(&value.damage, damage_json),
        publication: fact(&value.publication, publication_json),
        rules: fact(&value.rules, |values| {
            values.iter().map(unsupported_json).collect()
        }),
        spell_child_id: spell_child_fact(&value.spell_child_id),
    }
}

fn state_json(value: &ConsumableSourceState) -> ConsumableSourceStateJson {
    ConsumableSourceStateJson {
        quantity: fact(&value.quantity, |value| *value),
        current_uses: fact(&value.current_uses, |value| *value),
        current_hp: fact(&value.current_hp, |value| *value),
        container_id: fact(&value.container_id, Clone::clone),
        equipped: fact(&value.equipped, |value| ConsumableEquippedStateJson {
            carry_type: fact(&value.carry_type, Clone::clone),
            hands_held: fact(&value.hands_held, |value| *value),
            in_slot: fact(&value.in_slot, |value| *value),
        }),
    }
}

fn price_json(value: &ConsumablePrice) -> ConsumablePriceJson {
    ConsumablePriceJson {
        denominations: fact(&value.denominations, |values| {
            values
                .iter()
                .map(|value| ConsumablePriceDenominationJson {
                    denomination: value.denomination.clone(),
                    amount: value.amount,
                })
                .collect()
        }),
        per: fact(&value.per, |value| *value),
    }
}

fn damage_json(value: &ConsumableDamage) -> ConsumableDamageJson {
    ConsumableDamageJson {
        formula: fact(&value.formula, Clone::clone),
        category: fact(&value.category, Clone::clone),
        damage_type: fact(&value.damage_type, Clone::clone),
    }
}

fn publication_json(value: &ConsumablePublication) -> ConsumablePublicationJson {
    ConsumablePublicationJson {
        title: fact(&value.title, Clone::clone),
        license: fact(&value.license, |value| value.as_str().to_string()),
        remaster: fact(&value.remaster, |value| *value),
    }
}

fn material_json(value: &ConsumableMaterial) -> ConsumableMaterialJson {
    ConsumableMaterialJson {
        grade: fact(&value.grade, Clone::clone),
        material_type: fact(&value.material_type, Clone::clone),
        effects: fact(&value.effects, Clone::clone),
    }
}

fn spell_child_fact(value: &FactValue<SpellChildId>) -> ConsumableSpellChildFactJson {
    match value {
        FactValue::Missing => ConsumableSpellChildFactJson::Missing,
        FactValue::Null => ConsumableSpellChildFactJson::Null,
        FactValue::Value(value) => ConsumableSpellChildFactJson::Known(value.as_str().to_string()),
    }
}

fn target_resolution_json(value: &ConsumableTargetResolution) -> ConsumableTargetResolutionJson {
    match value {
        ConsumableTargetResolution::NoLocator => ConsumableTargetResolutionJson::NoLocator,
        ConsumableTargetResolution::MalformedOrDuplicateLocator(value) => {
            ConsumableTargetResolutionJson::MalformedOrDuplicateLocator {
                evidence: unsupported_json(value),
            }
        }
        ConsumableTargetResolution::TargetMissing(locator) => {
            ConsumableTargetResolutionJson::TargetMissing {
                locator: locator.as_str().to_string(),
            }
        }
        ConsumableTargetResolution::WrongDocumentOrFamily(locator) => {
            ConsumableTargetResolutionJson::WrongDocumentOrFamily {
                locator: locator.as_str().to_string(),
            }
        }
    }
}

fn spell_reuse_json(value: &ConsumableSpellReuse) -> ConsumableSpellReuseJson {
    match value {
        ConsumableSpellReuse::NotPresent => ConsumableSpellReuseJson::NotPresent,
        ConsumableSpellReuse::Reused { target_child_id } => ConsumableSpellReuseJson::Reused {
            target_child_id: target_child_id.as_str().to_string(),
        },
        ConsumableSpellReuse::Mismatch {
            reason,
            local_evidence,
        } => ConsumableSpellReuseJson::Mismatch {
            reason: match reason {
                ConsumableSpellMismatchReason::UnresolvedParent => "unresolved_parent",
                ConsumableSpellMismatchReason::TargetWithoutChild => "target_without_child",
                ConsumableSpellMismatchReason::LocalChildMissing => "local_child_missing",
                ConsumableSpellMismatchReason::LocalChildMalformed => "local_child_malformed",
                ConsumableSpellMismatchReason::ChildIdentity => "child_identity",
                ConsumableSpellMismatchReason::SourceContext => "source_context",
                ConsumableSpellMismatchReason::Definition => "definition",
                ConsumableSpellMismatchReason::ContentOrReferences => "content_or_references",
                ConsumableSpellMismatchReason::OverlayOrFormOrder => "overlay_or_form_order",
            },
            local_evidence: match local_evidence {
                crate::ConsumableLocalSpellEvidence::Missing => {
                    ConsumableLocalSpellEvidenceJson::Missing
                }
                crate::ConsumableLocalSpellEvidence::Null => ConsumableLocalSpellEvidenceJson::Null,
                crate::ConsumableLocalSpellEvidence::Malformed(value) => {
                    ConsumableLocalSpellEvidenceJson::Malformed {
                        evidence: unsupported_json(value),
                    }
                }
                crate::ConsumableLocalSpellEvidence::Child(child) => {
                    ConsumableLocalSpellEvidenceJson::Child {
                        child_id: child.child_id.as_str().to_string(),
                    }
                }
            },
        },
    }
}

fn mismatch_value_json(value: &ConsumableMismatchValue) -> ConsumableMismatchValueJson {
    match value {
        ConsumableMismatchValue::String(value) => {
            ConsumableMismatchValueJson::String(fact(value, Clone::clone))
        }
        ConsumableMismatchValue::Integer(value) => {
            ConsumableMismatchValueJson::Integer(fact(value, |value| *value))
        }
        ConsumableMismatchValue::Boolean(value) => {
            ConsumableMismatchValueJson::Boolean(fact(value, |value| *value))
        }
        ConsumableMismatchValue::Rarity(value) => {
            ConsumableMismatchValueJson::Rarity(fact(value, |value| value.as_str().to_string()))
        }
        ConsumableMismatchValue::Strings(value) => {
            ConsumableMismatchValueJson::Strings(fact(value, Clone::clone))
        }
        ConsumableMismatchValue::ExactDecimal(value) => {
            ConsumableMismatchValueJson::ExactDecimal(fact(value, |value| {
                value.as_str().to_string()
            }))
        }
        ConsumableMismatchValue::Material(value) => {
            ConsumableMismatchValueJson::Material(fact(value, material_json))
        }
        ConsumableMismatchValue::Price(value) => {
            ConsumableMismatchValueJson::Price(fact(value, price_json))
        }
        ConsumableMismatchValue::Damage(value) => {
            ConsumableMismatchValueJson::Damage(fact(value, damage_json))
        }
        ConsumableMismatchValue::Publication(value) => {
            ConsumableMismatchValueJson::Publication(fact(value, publication_json))
        }
        ConsumableMismatchValue::Rules(value) => {
            ConsumableMismatchValueJson::Rules(fact(value, |values| {
                values.iter().map(unsupported_json).collect()
            }))
        }
        ConsumableMismatchValue::SpellChildId(value) => {
            ConsumableMismatchValueJson::SpellChildId(spell_child_fact(value))
        }
    }
}

fn fact<T, U>(value: &ConsumableFact<T>, project: impl Fn(&T) -> U) -> ConsumableFactJson<U> {
    match value {
        FactValue::Missing => ConsumableFactJson::Missing,
        FactValue::Null => ConsumableFactJson::Null,
        FactValue::Value(ConsumableSourceValue::Known(value)) => {
            ConsumableFactJson::Known(project(value))
        }
        FactValue::Value(ConsumableSourceValue::Unsupported(value)) => {
            ConsumableFactJson::Unsupported(unsupported_json(value))
        }
    }
}

fn unsupported_json(value: &crate::UnsupportedSourceValue) -> ConsumableUnsupportedJson {
    ConsumableUnsupportedJson {
        shape: shape(value.shape),
        value: value.value.clone(),
        reason: reason(value.reason),
    }
}

const fn shape(value: UnsupportedSourceShape) -> &'static str {
    match value {
        UnsupportedSourceShape::Missing => "missing",
        UnsupportedSourceShape::Null => "null",
        UnsupportedSourceShape::String => "string",
        UnsupportedSourceShape::Number => "number",
        UnsupportedSourceShape::Boolean => "boolean",
        UnsupportedSourceShape::Array => "array",
        UnsupportedSourceShape::Object => "object",
    }
}

const fn reason(value: UnsupportedSourceReason) -> &'static str {
    match value {
        UnsupportedSourceReason::OpenVocabulary => "open_vocabulary",
        UnsupportedSourceReason::AmbiguousLegacyShape => "ambiguous_legacy_shape",
        UnsupportedSourceReason::InvalidPredicate => "invalid_predicate",
        UnsupportedSourceReason::NonCanonicalRuntimeValue => "noncanonical_runtime_value",
        UnsupportedSourceReason::SourceFieldDrift => "source_field_drift",
    }
}
