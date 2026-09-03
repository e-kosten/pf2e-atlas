use std::collections::BTreeMap;

use atlas_app_model::{
    EncounterRuntimeSpellView, EncounterRuntimeView, EncounterSpellCastAvailabilityView,
    EncounterSpellCastBlockedReasonView, EncounterSpellCastStateView,
    EncounterSpellCastUnavailableReasonView, EncounterSpellSpendTargetView, RuntimeCountView,
    RuntimeFactProvenanceView, RuntimeFactSourceView, RuntimeNumberView,
};
use atlas_local_state::{
    EncounterParticipant, EncounterParticipantSpellState, EncounterSpellResource,
    EncounterSpellResourceTarget,
};
use atlas_record::{
    CreatureCapability, CreatureEmbeddedEntities, CreatureEntityOccurrence,
    CreatureOccurrenceParent, CreaturePreparedSpellSlot, CreatureRecord, CreatureSourceScalar,
    CreatureSpellPreparation, FactValue, RecordBody, ResourceCurrentPolicy, RetrievedRecord,
};

use crate::error::AppServiceResult;
use crate::service::AtlasAppService;

const JS_SAFE_INTEGER_MAX: i64 = 9_007_199_254_740_991;

#[derive(Debug, Clone)]
pub(super) struct ParticipantSpellCastContext {
    catalog: SpellCastCatalog,
    state: EncounterParticipantSpellState,
}

#[derive(Debug, Clone, Default)]
struct SpellCastCatalog {
    spells: BTreeMap<String, CanonicalSpellCast>,
    resources: BTreeMap<EncounterSpellResourceTarget, EncounterSpellResource>,
}

#[derive(Debug, Clone)]
enum CanonicalSpellCast {
    AtWill,
    Tracked(EncounterSpellResourceTarget),
    Unavailable {
        target: Option<EncounterSpellResourceTarget>,
        reason: EncounterSpellCastUnavailableReasonView,
    },
}

#[derive(Debug, Clone)]
struct CanonicalResourceCandidate {
    target: EncounterSpellResourceTarget,
    state: Result<EncounterSpellResource, EncounterSpellCastUnavailableReasonView>,
}

fn unavailable_cast(reason: EncounterSpellCastUnavailableReasonView) -> CanonicalSpellCast {
    CanonicalSpellCast::Unavailable {
        target: None,
        reason,
    }
}

fn unavailable_target(
    target: EncounterSpellResourceTarget,
    reason: EncounterSpellCastUnavailableReasonView,
) -> CanonicalSpellCast {
    CanonicalSpellCast::Unavailable {
        target: Some(target),
        reason,
    }
}

pub(super) fn participant_spell_cast_context(
    service: &AtlasAppService,
    participant: &EncounterParticipant,
    retrieved: &RetrievedRecord,
) -> AppServiceResult<ParticipantSpellCastContext> {
    let catalog = spell_cast_catalog(retrieved);
    let state = service
        .local_state_store()?
        .encounters()
        .spell_state(&participant.participant_key)?;
    Ok(ParticipantSpellCastContext { catalog, state })
}

pub(super) fn initial_spell_resources(retrieved: &RetrievedRecord) -> Vec<EncounterSpellResource> {
    spell_cast_catalog(retrieved)
        .resources
        .into_values()
        .collect()
}

pub(super) fn attach_spell_cast_availability(
    participant: &EncounterParticipant,
    context: &ParticipantSpellCastContext,
    runtime: &mut EncounterRuntimeView,
) {
    for entry in &mut runtime.spellcasting {
        for spell in &mut entry.spells {
            spell.cast = context.availability(participant, spell);
        }
        for slot in &mut entry.slots {
            let spontaneous = EncounterSpellResourceTarget::SpontaneousPool {
                entry_id: entry.entry_id.clone(),
                rank: slot.rank,
            };
            if let Some(resource) = context
                .state
                .resources
                .iter()
                .find(|resource| resource.target == spontaneous)
            {
                slot.current = Some(runtime_count(
                    format!("Rank {} slots remaining", slot.rank),
                    resource,
                ));
                continue;
            }
            let prepared = context
                .state
                .resources
                .iter()
                .filter(|resource| {
                    matches!(
                        &resource.target,
                        EncounterSpellResourceTarget::PreparedSlot { entry_id, rank, .. }
                            if entry_id == &entry.entry_id && rank == &slot.rank
                    )
                })
                .collect::<Vec<_>>();
            if !prepared.is_empty() {
                slot.current = Some(RuntimeCountView {
                    label: format!("Rank {} prepared spells remaining", slot.rank),
                    base_value: prepared
                        .iter()
                        .map(|resource| resource.initial_remaining)
                        .sum(),
                    adjusted_value: prepared.iter().map(|resource| resource.remaining).sum(),
                    segments: Vec::new(),
                    adjustments: Vec::new(),
                    suppressed_adjustments: Vec::new(),
                    provenance: participant_state_provenance(),
                });
            }
        }
    }
    for spell in &mut runtime.standalone_spells {
        spell.cast = context.availability(participant, spell);
    }
    for resource_view in &mut runtime.resources {
        let target = EncounterSpellResourceTarget::FocusPool {
            resource_id: resource_view.resource_id.clone(),
        };
        if let Some(resource) = context
            .state
            .resources
            .iter()
            .find(|resource| resource.target == target)
        {
            resource_view.current = Some(RuntimeNumberView {
                label: format!("Current {}", resource_view.label),
                base_value: resource.initial_remaining,
                adjusted_value: resource.remaining,
                modifiers: Vec::new(),
                suppressed_modifiers: Vec::new(),
                provenance: participant_state_provenance(),
            });
        }
    }
}

fn runtime_count(label: String, resource: &EncounterSpellResource) -> RuntimeCountView {
    RuntimeCountView {
        label,
        base_value: resource.initial_remaining,
        adjusted_value: resource.remaining,
        segments: Vec::new(),
        adjustments: Vec::new(),
        suppressed_adjustments: Vec::new(),
        provenance: participant_state_provenance(),
    }
}

fn participant_state_provenance() -> RuntimeFactProvenanceView {
    RuntimeFactProvenanceView {
        source: RuntimeFactSourceView::ParticipantState,
        canonical_target: None,
    }
}

pub(crate) fn unavailable_spell_cast() -> EncounterSpellCastAvailabilityView {
    unavailable(
        None,
        EncounterSpellCastUnavailableReasonView::UnresolvedParticipant,
    )
}

impl ParticipantSpellCastContext {
    pub(super) fn expected_target(
        &self,
        spell_occurrence_id: &str,
    ) -> Option<EncounterSpellSpendTargetView> {
        match self.catalog.spells.get(spell_occurrence_id)? {
            CanonicalSpellCast::AtWill => Some(EncounterSpellSpendTargetView::AtWill),
            CanonicalSpellCast::Tracked(target) => Some(target_view(target)),
            CanonicalSpellCast::Unavailable { .. } => None,
        }
    }

    pub(super) fn availability_for_id(
        &self,
        participant: &EncounterParticipant,
        spell_occurrence_id: &str,
    ) -> EncounterSpellCastAvailabilityView {
        self.availability_for(participant, spell_occurrence_id)
    }

    fn availability(
        &self,
        participant: &EncounterParticipant,
        spell: &EncounterRuntimeSpellView,
    ) -> EncounterSpellCastAvailabilityView {
        self.availability_for(participant, &spell.occurrence_id)
    }

    fn availability_for(
        &self,
        participant: &EncounterParticipant,
        spell_occurrence_id: &str,
    ) -> EncounterSpellCastAvailabilityView {
        let Some(cast) = self.catalog.spells.get(spell_occurrence_id) else {
            return unavailable(
                None,
                EncounterSpellCastUnavailableReasonView::AmbiguousOwnership,
            );
        };
        let defeated = participant.defeated;
        match cast {
            CanonicalSpellCast::AtWill => EncounterSpellCastAvailabilityView {
                spend_target: Some(EncounterSpellSpendTargetView::AtWill),
                available: !defeated,
                state: EncounterSpellCastStateView::AtWill,
                blocked_reason: defeated
                    .then_some(EncounterSpellCastBlockedReasonView::ParticipantDefeated),
            },
            CanonicalSpellCast::Unavailable { target, reason } => {
                unavailable(target.as_ref().map(target_view), *reason)
            }
            CanonicalSpellCast::Tracked(target) => {
                let Some(resource) = self
                    .state
                    .resources
                    .iter()
                    .find(|resource| &resource.target == target)
                else {
                    return unavailable(
                        Some(target_view(target)),
                        EncounterSpellCastUnavailableReasonView::StateUnavailable,
                    );
                };
                let blocked_reason = if defeated {
                    Some(EncounterSpellCastBlockedReasonView::ParticipantDefeated)
                } else if resource.remaining == 0 {
                    Some(EncounterSpellCastBlockedReasonView::Exhausted)
                } else {
                    None
                };
                EncounterSpellCastAvailabilityView {
                    spend_target: Some(target_view(target)),
                    available: blocked_reason.is_none(),
                    state: EncounterSpellCastStateView::Tracked {
                        maximum: resource.maximum,
                        initial_remaining: resource.initial_remaining,
                        remaining: resource.remaining,
                    },
                    blocked_reason,
                }
            }
        }
    }
}

pub(super) fn local_target(
    target: &EncounterSpellSpendTargetView,
    spell_occurrence_id: &str,
) -> Option<EncounterSpellResourceTarget> {
    match target {
        EncounterSpellSpendTargetView::PreparedSlot {
            entry_id,
            rank,
            slot_id,
        } => Some(EncounterSpellResourceTarget::PreparedSlot {
            entry_id: entry_id.clone(),
            spell_occurrence_id: spell_occurrence_id.to_string(),
            rank: *rank,
            slot_id: slot_id.clone(),
        }),
        EncounterSpellSpendTargetView::SpontaneousPool { entry_id, rank } => {
            Some(EncounterSpellResourceTarget::SpontaneousPool {
                entry_id: entry_id.clone(),
                rank: *rank,
            })
        }
        EncounterSpellSpendTargetView::InnateUse {
            entry_id,
            spell_occurrence_id: target_spell_id,
        } if target_spell_id == spell_occurrence_id => {
            Some(EncounterSpellResourceTarget::InnateUse {
                entry_id: entry_id.clone(),
                spell_occurrence_id: spell_occurrence_id.to_string(),
            })
        }
        EncounterSpellSpendTargetView::FocusPool { resource_id } => {
            Some(EncounterSpellResourceTarget::FocusPool {
                resource_id: resource_id.clone(),
            })
        }
        EncounterSpellSpendTargetView::AtWill | EncounterSpellSpendTargetView::InnateUse { .. } => {
            None
        }
    }
}

fn target_view(target: &EncounterSpellResourceTarget) -> EncounterSpellSpendTargetView {
    match target {
        EncounterSpellResourceTarget::PreparedSlot {
            entry_id,
            rank,
            slot_id,
            ..
        } => EncounterSpellSpendTargetView::PreparedSlot {
            entry_id: entry_id.clone(),
            rank: *rank,
            slot_id: slot_id.clone(),
        },
        EncounterSpellResourceTarget::SpontaneousPool { entry_id, rank } => {
            EncounterSpellSpendTargetView::SpontaneousPool {
                entry_id: entry_id.clone(),
                rank: *rank,
            }
        }
        EncounterSpellResourceTarget::InnateUse {
            entry_id,
            spell_occurrence_id,
        } => EncounterSpellSpendTargetView::InnateUse {
            entry_id: entry_id.clone(),
            spell_occurrence_id: spell_occurrence_id.clone(),
        },
        EncounterSpellResourceTarget::FocusPool { resource_id } => {
            EncounterSpellSpendTargetView::FocusPool {
                resource_id: resource_id.clone(),
            }
        }
    }
}

fn unavailable(
    spend_target: Option<EncounterSpellSpendTargetView>,
    reason: EncounterSpellCastUnavailableReasonView,
) -> EncounterSpellCastAvailabilityView {
    EncounterSpellCastAvailabilityView {
        spend_target,
        available: false,
        state: EncounterSpellCastStateView::Unavailable { reason },
        blocked_reason: Some(EncounterSpellCastBlockedReasonView::StateUnavailable),
    }
}

fn spell_cast_catalog(retrieved: &RetrievedRecord) -> SpellCastCatalog {
    let Some(RecordBody::Creature(creature)) = retrieved.body.as_ref() else {
        return SpellCastCatalog::default();
    };
    let Some(embedded) = creature.embedded_entities.value.as_value() else {
        return SpellCastCatalog::default();
    };
    build_catalog(creature, embedded)
}

fn build_catalog(
    creature: &CreatureRecord,
    embedded: &CreatureEmbeddedEntities,
) -> SpellCastCatalog {
    let mut catalog = SpellCastCatalog::default();
    let occurrence_counts = embedded.occurrences.iter().fold(
        BTreeMap::<&str, usize>::new(),
        |mut counts, occurrence| {
            *counts.entry(occurrence.id.as_str()).or_default() += 1;
            counts
        },
    );
    let entries = embedded
        .occurrences
        .iter()
        .filter(|occurrence| {
            matches!(
                occurrence.capability,
                CreatureCapability::SpellcastingEntry(_)
            ) && occurrence_counts.get(occurrence.id.as_str()) == Some(&1)
        })
        .map(|entry| (entry.id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let focus = focus_resource(creature);

    for spell in embedded
        .occurrences
        .iter()
        .filter(|occurrence| matches!(occurrence.capability, CreatureCapability::Spell(_)))
    {
        let id = spell.id.as_str().to_string();
        if occurrence_counts.get(spell.id.as_str()) != Some(&1) {
            catalog.spells.insert(
                id,
                unavailable_cast(EncounterSpellCastUnavailableReasonView::AmbiguousOwnership),
            );
            continue;
        }
        let cast = match &spell.parent {
            CreatureOccurrenceParent::Creature => standalone_spell_cast(spell, &mut catalog),
            CreatureOccurrenceParent::SpellcastingEntry(parent) => {
                entries.get(parent.as_str()).map_or_else(
                    || {
                        unavailable_cast(
                            EncounterSpellCastUnavailableReasonView::AmbiguousOwnership,
                        )
                    },
                    |entry| entry_spell_cast(entry, spell, focus.as_ref(), &mut catalog),
                )
            }
        };
        catalog.spells.insert(id, cast);
    }
    catalog
}

fn standalone_spell_cast(
    spell: &CreatureEntityOccurrence,
    catalog: &mut SpellCastCatalog,
) -> CanonicalSpellCast {
    uses_cast(None, spell, catalog)
}

fn entry_spell_cast(
    entry: &CreatureEntityOccurrence,
    spell: &CreatureEntityOccurrence,
    focus: Option<&CanonicalResourceCandidate>,
    catalog: &mut SpellCastCatalog,
) -> CanonicalSpellCast {
    let CreatureCapability::SpellcastingEntry(capability) = &entry.capability else {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::AmbiguousOwnership);
    };
    match capability.preparation.as_value() {
        Some(CreatureSpellPreparation::Prepared) => {
            prepared_cast(entry, spell, capability, catalog)
        }
        Some(CreatureSpellPreparation::Spontaneous) => {
            spontaneous_cast(entry, spell, capability, catalog)
        }
        Some(CreatureSpellPreparation::Innate) => {
            uses_cast(Some(entry.id.as_str()), spell, catalog)
        }
        Some(CreatureSpellPreparation::Focus) => match focus {
            Some(CanonicalResourceCandidate {
                state: Ok(resource),
                ..
            }) => {
                let target = resource.target.clone();
                catalog
                    .resources
                    .entry(target.clone())
                    .or_insert_with(|| resource.clone());
                CanonicalSpellCast::Tracked(target)
            }
            Some(CanonicalResourceCandidate {
                target,
                state: Err(reason),
            }) => unavailable_target(target.clone(), *reason),
            None => unavailable_cast(EncounterSpellCastUnavailableReasonView::MissingMaximum),
        },
        Some(CreatureSpellPreparation::Ritual | CreatureSpellPreparation::Unsupported(_))
        | None => unavailable_cast(EncounterSpellCastUnavailableReasonView::UnsupportedPreparation),
    }
}

fn prepared_cast(
    entry: &CreatureEntityOccurrence,
    spell: &CreatureEntityOccurrence,
    capability: &atlas_record::CreatureSpellcastingEntryCapability,
    catalog: &mut SpellCastCatalog,
) -> CanonicalSpellCast {
    let (Some(rank), Some(slot_id), Some(source_id), Some(slots)) = (
        spell.context.rank.as_value().copied(),
        spell.context.slot.as_value(),
        spell.source_identity.nested_source_id.as_value(),
        capability.slots.as_value(),
    ) else {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::MissingIdentity);
    };
    if !(0..=JS_SAFE_INTEGER_MAX).contains(&rank) {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::UnsafeInteger);
    }
    let mut matches = slots
        .iter()
        .filter(|slot| slot.rank == rank)
        .filter_map(|slot| slot.prepared.as_value())
        .flatten()
        .filter_map(|prepared| match prepared {
            CreaturePreparedSpellSlot::Spell {
                id: FactValue::Value(id),
                expended,
                prepared,
                ..
            } if id == source_id => Some((expended, prepared)),
            _ => None,
        });
    let Some((expended, prepared)) = matches.next() else {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::MissingIdentity);
    };
    if matches.next().is_some() {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::AmbiguousOwnership);
    }
    let (Some(expended), Some(prepared)) = (expended.as_value(), prepared.as_value()) else {
        return unavailable_target(
            EncounterSpellResourceTarget::PreparedSlot {
                entry_id: entry.id.as_str().to_string(),
                spell_occurrence_id: spell.id.as_str().to_string(),
                rank,
                slot_id: slot_id.clone(),
            },
            EncounterSpellCastUnavailableReasonView::MissingCurrent,
        );
    };
    let initial = i64::from(*prepared && !*expended);
    let target = EncounterSpellResourceTarget::PreparedSlot {
        entry_id: entry.id.as_str().to_string(),
        spell_occurrence_id: spell.id.as_str().to_string(),
        rank,
        slot_id: slot_id.clone(),
    };
    insert_resource(catalog, target, 1, initial)
}

fn spontaneous_cast(
    entry: &CreatureEntityOccurrence,
    spell: &CreatureEntityOccurrence,
    capability: &atlas_record::CreatureSpellcastingEntryCapability,
    catalog: &mut SpellCastCatalog,
) -> CanonicalSpellCast {
    let (Some(rank), Some(slots)) = (
        spell.context.rank.as_value().copied(),
        capability.slots.as_value(),
    ) else {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::MissingIdentity);
    };
    if !(0..=JS_SAFE_INTEGER_MAX).contains(&rank) {
        return unavailable_cast(EncounterSpellCastUnavailableReasonView::UnsafeInteger);
    }
    let matching = slots
        .iter()
        .filter(|slot| slot.rank == rank)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return unavailable_cast(if matching.is_empty() {
            EncounterSpellCastUnavailableReasonView::MissingMaximum
        } else {
            EncounterSpellCastUnavailableReasonView::AmbiguousOwnership
        });
    }
    let slot = matching[0];
    let maximum = source_count(&slot.maximum);
    let current = source_count(&slot.serialized_value);
    tracked_counts(
        catalog,
        EncounterSpellResourceTarget::SpontaneousPool {
            entry_id: entry.id.as_str().to_string(),
            rank,
        },
        maximum,
        current,
    )
}

fn uses_cast(
    entry_id: Option<&str>,
    spell: &CreatureEntityOccurrence,
    catalog: &mut SpellCastCatalog,
) -> CanonicalSpellCast {
    match &spell.context.uses {
        FactValue::Missing => CanonicalSpellCast::AtWill,
        FactValue::Null => unavailable_target(
            EncounterSpellResourceTarget::InnateUse {
                entry_id: entry_id.map(str::to_string),
                spell_occurrence_id: spell.id.as_str().to_string(),
            },
            EncounterSpellCastUnavailableReasonView::MissingCurrent,
        ),
        FactValue::Value(uses) => tracked_counts(
            catalog,
            EncounterSpellResourceTarget::InnateUse {
                entry_id: entry_id.map(str::to_string),
                spell_occurrence_id: spell.id.as_str().to_string(),
            },
            uses.maximum.as_value().copied(),
            uses.serialized_value.as_value().copied(),
        ),
    }
}

fn focus_resource(creature: &CreatureRecord) -> Option<CanonicalResourceCandidate> {
    let resources = creature.resources.value.as_value()?;
    let focus = resources
        .iter()
        .filter(|resource| resource.kind.as_str() == "focus")
        .collect::<Vec<_>>();
    if focus.len() != 1 {
        return (focus.len() > 1).then_some(CanonicalResourceCandidate {
            target: EncounterSpellResourceTarget::FocusPool {
                resource_id: String::new(),
            },
            state: Err(EncounterSpellCastUnavailableReasonView::AmbiguousOwnership),
        });
    }
    let resource = focus[0];
    let maximum = resource_count(&resource.maximum);
    let current = match resource.current_policy {
        ResourceCurrentPolicy::SerializedValueIsProvenanceOnly => None,
    };
    let target = EncounterSpellResourceTarget::FocusPool {
        resource_id: resource.id.as_str().to_string(),
    };
    Some(CanonicalResourceCandidate {
        target: target.clone(),
        state: counts(target, maximum, current),
    })
}

fn tracked_counts(
    catalog: &mut SpellCastCatalog,
    target: EncounterSpellResourceTarget,
    maximum: Option<i64>,
    current: Option<i64>,
) -> CanonicalSpellCast {
    match counts(target.clone(), maximum, current) {
        Ok(resource) => {
            if let Some(existing) = catalog.resources.get(&target)
                && existing != &resource
            {
                return unavailable_target(
                    target,
                    EncounterSpellCastUnavailableReasonView::AmbiguousOwnership,
                );
            }
            catalog.resources.entry(target.clone()).or_insert(resource);
            CanonicalSpellCast::Tracked(target)
        }
        Err(reason) => unavailable_target(target, reason),
    }
}

fn insert_resource(
    catalog: &mut SpellCastCatalog,
    target: EncounterSpellResourceTarget,
    maximum: i64,
    current: i64,
) -> CanonicalSpellCast {
    tracked_counts(catalog, target, Some(maximum), Some(current))
}

fn counts(
    target: EncounterSpellResourceTarget,
    maximum: Option<i64>,
    current: Option<i64>,
) -> Result<EncounterSpellResource, EncounterSpellCastUnavailableReasonView> {
    let Some(maximum) = maximum else {
        return Err(EncounterSpellCastUnavailableReasonView::MissingMaximum);
    };
    let Some(current) = current else {
        return Err(EncounterSpellCastUnavailableReasonView::MissingCurrent);
    };
    if maximum > JS_SAFE_INTEGER_MAX || current > JS_SAFE_INTEGER_MAX {
        return Err(EncounterSpellCastUnavailableReasonView::UnsafeInteger);
    }
    if maximum < 0 || current < 0 || current > maximum {
        return Err(EncounterSpellCastUnavailableReasonView::MissingCurrent);
    }
    Ok(EncounterSpellResource {
        target,
        maximum,
        initial_remaining: current,
        remaining: current,
    })
}

fn source_count(value: &FactValue<CreatureSourceScalar<i64>>) -> Option<i64> {
    match value.as_value()? {
        CreatureSourceScalar::Value(value) => Some(*value),
        CreatureSourceScalar::Unsupported(_) => None,
    }
}

fn resource_count(value: &FactValue<atlas_record::CreatureResourceAmount>) -> Option<i64> {
    match value.as_value()? {
        atlas_record::CreatureResourceAmount::Integer(value) => Some(*value),
        atlas_record::CreatureResourceAmount::Unsupported(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordKey;
    use atlas_local_state::{ParticipantKind, ParticipantSide, ParticipantVariant};
    use atlas_record::{
        CreatureActionCost, CreatureComponentId, CreatureEntityFamily, CreatureEntityId,
        CreatureEntitySourceIdentity, CreatureFact, CreatureFamily, CreatureIdentity,
        CreatureOccurrenceContext, CreatureOccurrenceId, CreatureProvenance, CreatureResource,
        CreatureResourceAmount, CreatureResourceKind, CreatureSourceField, CreatureSourceId,
        OccurrenceIdentityStability,
    };

    use super::*;

    #[test]
    fn canonical_catalog_preserves_each_spell_spend_owner() {
        let mut creature = creature();
        let owner = creature.identity.record_key.clone();
        let prepared_entry = entry(
            &owner,
            "prepared-entry",
            CreatureSpellPreparation::Prepared,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 4,
                maximum: scalar(1),
                serialized_value: scalar(1),
                prepared: FactValue::Value(vec![CreaturePreparedSpellSlot::Spell {
                    id: FactValue::Value(source_id("prepared-spell")),
                    name: FactValue::Value("Fireball".to_string()),
                    expended: FactValue::Value(false),
                    prepared: FactValue::Value(true),
                    authored_order: 0,
                }]),
            }]),
        );
        let spontaneous_entry = entry(
            &owner,
            "spontaneous-entry",
            CreatureSpellPreparation::Spontaneous,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 3,
                maximum: scalar(3),
                serialized_value: scalar(2),
                prepared: FactValue::Missing,
            }]),
        );
        let innate_entry = entry(
            &owner,
            "innate-entry",
            CreatureSpellPreparation::Innate,
            FactValue::Value(Vec::new()),
        );
        let focus_entry = entry(
            &owner,
            "focus-entry",
            CreatureSpellPreparation::Focus,
            FactValue::Value(Vec::new()),
        );
        let mut prepared_spell = spell(&owner, "prepared-spell", "prepared-entry", 4);
        prepared_spell.context.slot = FactValue::Value("slot4:0".to_string());
        let spontaneous_spell = spell(&owner, "spontaneous-spell", "spontaneous-entry", 3);
        let mut innate_spell = spell(&owner, "innate-spell", "innate-entry", 5);
        innate_spell.context.uses = FactValue::Value(atlas_record::CreatureUseLimit {
            maximum: FactValue::Value(2),
            serialized_value: FactValue::Value(1),
        });
        let innate_at_will = spell(&owner, "innate-at-will", "innate-entry", 2);
        let focus_spell = spell(&owner, "focus-spell", "focus-entry", 1);
        creature.resources = CreatureFact::source(
            FactValue::Value(vec![CreatureResource {
                id: CreatureComponentId::new("resource:focus").expect("focus id"),
                authored_order: 0,
                kind: CreatureResourceKind::new("focus").expect("focus kind"),
                label: "Focus".to_string(),
                maximum: FactValue::Value(CreatureResourceAmount::Integer(3)),
                serialized_value: FactValue::Value(CreatureResourceAmount::Integer(1)),
                source_drift: FactValue::Missing,
                current_policy: ResourceCurrentPolicy::SerializedValueIsProvenanceOnly,
            }]),
            CreatureSourceField::Resources,
        );
        creature.embedded_entities = CreatureFact::source(
            FactValue::Value(CreatureEmbeddedEntities {
                entities: Vec::new(),
                occurrences: vec![
                    prepared_entry,
                    spontaneous_entry,
                    innate_entry,
                    focus_entry,
                    prepared_spell,
                    spontaneous_spell,
                    innate_spell,
                    innate_at_will,
                    focus_spell,
                ],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            }),
            CreatureSourceField::EmbeddedEntities,
        );

        let embedded = creature
            .embedded_entities
            .value
            .as_value()
            .expect("embedded fixture");
        let catalog = build_catalog(&creature, embedded);
        assert!(matches!(
            catalog.spells.get("prepared-spell"),
            Some(CanonicalSpellCast::Tracked(EncounterSpellResourceTarget::PreparedSlot {
                entry_id,
                rank: 4,
                slot_id,
                ..
            })) if entry_id == "prepared-entry" && slot_id == "slot4:0"
        ));
        assert!(matches!(
            catalog.spells.get("spontaneous-spell"),
            Some(CanonicalSpellCast::Tracked(EncounterSpellResourceTarget::SpontaneousPool {
                entry_id,
                rank: 3,
            })) if entry_id == "spontaneous-entry"
        ));
        assert!(matches!(
            catalog.spells.get("innate-spell"),
            Some(CanonicalSpellCast::Tracked(EncounterSpellResourceTarget::InnateUse {
                entry_id: Some(entry_id),
                spell_occurrence_id,
            })) if entry_id == "innate-entry" && spell_occurrence_id == "innate-spell"
        ));
        assert!(matches!(
            catalog.spells.get("innate-at-will"),
            Some(CanonicalSpellCast::AtWill)
        ));
        assert!(matches!(
            catalog.spells.get("focus-spell"),
            Some(CanonicalSpellCast::Unavailable {
                target: Some(EncounterSpellResourceTarget::FocusPool { resource_id }),
                reason: EncounterSpellCastUnavailableReasonView::MissingCurrent,
            }) if resource_id == "resource:focus"
        ));
        assert!(catalog.resources.values().any(|resource| {
            matches!(
                resource.target,
                EncounterSpellResourceTarget::SpontaneousPool { .. }
            ) && resource.maximum == 3
                && resource.initial_remaining == 2
        }));
        assert!(catalog.resources.values().any(|resource| {
            matches!(
                resource.target,
                EncounterSpellResourceTarget::InnateUse { .. }
            ) && resource.maximum == 2
                && resource.initial_remaining == 1
        }));
        assert!(!catalog.resources.values().any(|resource| matches!(
            resource.target,
            EncounterSpellResourceTarget::FocusPool { .. }
        )));
    }

    #[test]
    fn missing_current_and_prepared_identity_fail_only_the_affected_spell_closed() {
        let mut creature = creature();
        let owner = creature.identity.record_key.clone();
        let spontaneous_entry = entry(
            &owner,
            "spontaneous-entry",
            CreatureSpellPreparation::Spontaneous,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 3,
                maximum: scalar(3),
                serialized_value: FactValue::Missing,
                prepared: FactValue::Missing,
            }]),
        );
        let prepared_entry = entry(
            &owner,
            "prepared-entry",
            CreatureSpellPreparation::Prepared,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 4,
                maximum: scalar(1),
                serialized_value: scalar(1),
                prepared: FactValue::Value(Vec::new()),
            }]),
        );
        let spontaneous_spell = spell(&owner, "spontaneous-spell", "spontaneous-entry", 3);
        let mut prepared_spell = spell(&owner, "prepared-spell", "prepared-entry", 4);
        prepared_spell.context.slot = FactValue::Value("slot4:0".to_string());
        creature.embedded_entities = CreatureFact::source(
            FactValue::Value(CreatureEmbeddedEntities {
                entities: Vec::new(),
                occurrences: vec![
                    spontaneous_entry,
                    prepared_entry,
                    spontaneous_spell,
                    prepared_spell,
                ],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            }),
            CreatureSourceField::EmbeddedEntities,
        );
        let catalog = build_catalog(
            &creature,
            creature
                .embedded_entities
                .value
                .as_value()
                .expect("embedded"),
        );
        assert!(matches!(
            catalog.spells.get("spontaneous-spell"),
            Some(CanonicalSpellCast::Unavailable {
                target: Some(EncounterSpellResourceTarget::SpontaneousPool {
                    entry_id,
                    rank: 3,
                }),
                reason: EncounterSpellCastUnavailableReasonView::MissingCurrent,
            }) if entry_id == "spontaneous-entry"
        ));
        assert!(matches!(
            catalog.spells.get("prepared-spell"),
            Some(CanonicalSpellCast::Unavailable {
                target: None,
                reason: EncounterSpellCastUnavailableReasonView::MissingIdentity,
            })
        ));

        let context = ParticipantSpellCastContext {
            catalog,
            state: EncounterParticipantSpellState {
                initialized: true,
                resources: Vec::new(),
            },
        };
        let unavailable = context.availability_for_id(&participant(false), "spontaneous-spell");
        assert!(matches!(
            unavailable.spend_target,
            Some(EncounterSpellSpendTargetView::SpontaneousPool {
                ref entry_id,
                rank: 3,
            }) if entry_id == "spontaneous-entry"
        ));
        assert!(matches!(
            unavailable.state,
            EncounterSpellCastStateView::Unavailable {
                reason: EncounterSpellCastUnavailableReasonView::MissingCurrent,
            }
        ));
    }

    #[test]
    fn runtime_availability_preserves_typed_target_and_blocks_exhausted_or_defeated_casts() {
        let target = EncounterSpellResourceTarget::SpontaneousPool {
            entry_id: "entry-spontaneous".to_string(),
            rank: 3,
        };
        let context = ParticipantSpellCastContext {
            catalog: SpellCastCatalog {
                spells: BTreeMap::from([
                    (
                        "spell-pool".to_string(),
                        CanonicalSpellCast::Tracked(target.clone()),
                    ),
                    ("spell-at-will".to_string(), CanonicalSpellCast::AtWill),
                ]),
                resources: BTreeMap::new(),
            },
            state: EncounterParticipantSpellState {
                initialized: true,
                resources: vec![EncounterSpellResource {
                    target,
                    maximum: 3,
                    initial_remaining: 2,
                    remaining: 0,
                }],
            },
        };

        let active = participant(false);
        let exhausted = context.availability_for_id(&active, "spell-pool");
        assert_eq!(
            exhausted.blocked_reason,
            Some(EncounterSpellCastBlockedReasonView::Exhausted)
        );
        assert!(matches!(
            exhausted.spend_target,
            Some(EncounterSpellSpendTargetView::SpontaneousPool {
                ref entry_id,
                rank: 3
            }) if entry_id == "entry-spontaneous"
        ));
        assert!(matches!(
            exhausted.state,
            EncounterSpellCastStateView::Tracked {
                maximum: 3,
                initial_remaining: 2,
                remaining: 0,
            }
        ));

        let defeated = context.availability_for_id(&participant(true), "spell-at-will");
        assert_eq!(
            defeated.blocked_reason,
            Some(EncounterSpellCastBlockedReasonView::ParticipantDefeated)
        );
        assert!(matches!(
            defeated.state,
            EncounterSpellCastStateView::AtWill
        ));
    }

    #[test]
    fn unsafe_canonical_spell_counts_and_ranks_fail_closed_before_public_projection() {
        let mut creature = creature();
        let owner = creature.identity.record_key.clone();
        let entry = entry(
            &owner,
            "spontaneous-entry",
            CreatureSpellPreparation::Spontaneous,
            FactValue::Value(vec![atlas_record::CreatureSpellSlot {
                rank: 3,
                maximum: scalar(JS_SAFE_INTEGER_MAX + 1),
                serialized_value: scalar(JS_SAFE_INTEGER_MAX + 1),
                prepared: FactValue::Missing,
            }]),
        );
        let unsafe_count = spell(&owner, "unsafe-count", "spontaneous-entry", 3);
        let unsafe_rank = spell(
            &owner,
            "unsafe-rank",
            "spontaneous-entry",
            JS_SAFE_INTEGER_MAX + 1,
        );
        creature.embedded_entities = CreatureFact::source(
            FactValue::Value(CreatureEmbeddedEntities {
                entities: Vec::new(),
                occurrences: vec![entry, unsafe_count, unsafe_rank],
                relationships: Vec::new(),
                actor_spellcasting: FactValue::Missing,
            }),
            CreatureSourceField::EmbeddedEntities,
        );

        let catalog = build_catalog(
            &creature,
            creature
                .embedded_entities
                .value
                .as_value()
                .expect("embedded"),
        );
        for spell_id in ["unsafe-count", "unsafe-rank"] {
            assert!(matches!(
                catalog.spells.get(spell_id),
                Some(CanonicalSpellCast::Unavailable {
                    reason: EncounterSpellCastUnavailableReasonView::UnsafeInteger,
                    ..
                })
            ));
        }
    }

    fn participant(defeated: bool) -> EncounterParticipant {
        EncounterParticipant {
            participant_key: "participant-spellcaster".to_string(),
            record_key: Some("actors:test-spellcaster".to_string()),
            participant_kind: ParticipantKind::Creature,
            participant_variant: ParticipantVariant::Normal,
            position: 1,
            display_name: "Spellcaster".to_string(),
            record_title_snapshot: Some("Spellcaster".to_string()),
            record_kind_snapshot: Some("creature".to_string()),
            side: ParticipantSide::Enemy,
            initiative: Some(20),
            initiative_order: 1,
            max_hp: Some(30),
            current_hp: Some(30),
            temporary_hp: 0,
            defeated,
            hidden: false,
            note: None,
            created_at: "2026-09-03T00:00:00Z".to_string(),
            updated_at: "2026-09-03T00:00:00Z".to_string(),
            conditions: Vec::new(),
        }
    }

    fn entry(
        owner: &RecordKey,
        id: &str,
        preparation: CreatureSpellPreparation,
        slots: FactValue<Vec<atlas_record::CreatureSpellSlot>>,
    ) -> CreatureEntityOccurrence {
        occurrence(
            owner,
            id,
            CreatureEntityFamily::SpellcastingEntry,
            CreatureOccurrenceParent::Creature,
            CreatureCapability::SpellcastingEntry(
                atlas_record::CreatureSpellcastingEntryCapability {
                    preparation: FactValue::Value(preparation),
                    tradition: FactValue::Value("arcane".to_string()),
                    attack: FactValue::Value(12),
                    dc: FactValue::Value(22),
                    slots,
                    unsupported_notes: Vec::new(),
                },
            ),
        )
    }

    fn spell(owner: &RecordKey, id: &str, entry_id: &str, rank: i64) -> CreatureEntityOccurrence {
        let mut spell = occurrence(
            owner,
            id,
            CreatureEntityFamily::Spell,
            CreatureOccurrenceParent::SpellcastingEntry(
                CreatureOccurrenceId::new(entry_id).expect("entry id"),
            ),
            CreatureCapability::Spell(atlas_record::CreatureSpellCapability {
                traits: FactValue::Value(vec!["spell".to_string()]),
                base_rank: FactValue::Value(rank),
                signature: FactValue::Missing,
                traditions: FactValue::Missing,
                requirements: FactValue::Missing,
                cost: FactValue::Missing,
                counteraction: FactValue::Missing,
                ritual: FactValue::Missing,
                target: FactValue::Missing,
                area: FactValue::Missing,
                range: FactValue::Missing,
                time: FactValue::Missing,
                duration: FactValue::Missing,
                defense: FactValue::Missing,
                damage: FactValue::Value(Vec::new()),
                action_cost: CreatureActionCost::Actions(2),
                unsupported_notes: Vec::new(),
            }),
        );
        spell.context.rank = FactValue::Value(rank);
        spell
    }

    fn occurrence(
        owner: &RecordKey,
        id: &str,
        family: CreatureEntityFamily,
        parent: CreatureOccurrenceParent,
        capability: CreatureCapability,
    ) -> CreatureEntityOccurrence {
        CreatureEntityOccurrence {
            id: CreatureOccurrenceId::new(id).expect("occurrence id"),
            identity_stability: OccurrenceIdentityStability::StableNestedSourceId,
            owner: owner.clone(),
            target: atlas_record::CreatureEntityTarget::ActorOwned(
                CreatureEntityId::new(format!("entity-{id}")).expect("entity id"),
            ),
            family,
            authored_order: 0,
            source_sort: FactValue::Missing,
            source_folder: FactValue::Missing,
            source_identity: CreatureEntitySourceIdentity {
                nested_source_id: FactValue::Value(source_id(id)),
                stable_source_locator: FactValue::Missing,
                source_locators: Vec::new(),
            },
            parent,
            context: CreatureOccurrenceContext::default(),
            capability,
            deltas: Vec::new(),
        }
    }

    fn creature() -> CreatureRecord {
        macro_rules! missing {
            ($field:expr) => {
                CreatureFact::source(FactValue::Missing, $field)
            };
        }
        let key = RecordKey::parse("actors:spellcaster").expect("record key");
        CreatureRecord {
            identity: CreatureIdentity {
                record_key: key,
                source_id: source_id("spellcaster"),
                name: "Spellcaster".to_string(),
                family: CreatureFamily::Npc,
            },
            level: missing!(CreatureSourceField::Level),
            rarity: missing!(CreatureSourceField::Rarity),
            traits: missing!(CreatureSourceField::Traits),
            size: missing!(CreatureSourceField::Size),
            publication: missing!(CreatureSourceField::Publication),
            adjustment: missing!(CreatureSourceField::Adjustment),
            source_alliance: missing!(CreatureSourceField::SourceAlliance),
            perception: missing!(CreatureSourceField::Perception),
            initiative: missing!(CreatureSourceField::Initiative),
            languages: missing!(CreatureSourceField::Languages),
            skills: missing!(CreatureSourceField::Skills),
            legacy_abilities: missing!(CreatureSourceField::LegacyAbilities),
            defenses: missing!(CreatureSourceField::Defenses),
            movement: missing!(CreatureSourceField::Movement),
            resources: missing!(CreatureSourceField::Resources),
            embedded_entities: missing!(CreatureSourceField::EmbeddedEntities),
            content: atlas_record::OwnedRichContent::default(),
            provenance: CreatureProvenance {
                source_path: "fixture".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "fixture".to_string(),
                source_upstream_commit: "fixture".to_string(),
            },
        }
    }

    fn scalar(value: i64) -> FactValue<CreatureSourceScalar<i64>> {
        FactValue::Value(CreatureSourceScalar::Value(value))
    }

    fn source_id(value: &str) -> CreatureSourceId {
        CreatureSourceId::new(value).expect("source id")
    }
}
