//! Full-night recovery over known saved/source facts; independent from baseline reset.
use super::super::{
    conditions::{ConditionRule, condition_rule_for_key},
    hydration::HydratedParticipantRecord,
    stat_block::{Ability, StatisticTarget},
};
use crate::{AppServiceError, AppServiceResult};
use atlas_app_model::{EncounterRecoveryParticipantView, OvernightHealingMode};
use atlas_foundry_model::generated::AbilitySystemSourceFrequencyPer as Per;
use atlas_local_state::{
    AbilityStateValue, ConditionParameters, EncounterParticipant, MortalityState,
    ParticipantAbilityState, ParticipantGameplay, ParticipantKind, WakingRule,
};

pub(in crate::encounters) fn prepare(
    p: &EncounterParticipant,
    healing: Option<i64>,
    healing_mode: OvernightHealingMode,
    record: Option<&HydratedParticipantRecord>,
    row: &mut EncounterRecoveryParticipantView,
) -> AppServiceResult<Option<ParticipantGameplay>> {
    let baseline = record.and_then(|r| r.arithmetic_baseline.as_ref());
    let sources = baseline.map(|b| &b.source_conditions);
    let conditions = super::causes::condition_causes(
        p,
        None,
        sources.unwrap_or(super::source_conditions::SourceConditions::empty()),
    )
    .active;
    if p.participant_kind == ParticipantKind::Hazard
        || p.mortality == MortalityState::Dead
        || conditions
            .iter()
            .any(|c| matches!(c.rule, ConditionRule::Dying | ConditionRule::Petrified))
    {
        row.unavailable_reason =
            Some("Choose a living creature that isn't dying or petrified.".into());
        return Ok(None);
    }
    let block = record.and_then(|r| {
        r.arithmetic_baseline
            .as_ref()
            .map(|b| super::project_participant(p, b, r.adjustment_applicable, &[]))
    });
    let level = p.level_override.or_else(|| {
        block
            .as_ref()
            .and_then(|b| b.adjusted_level.as_ref().or(b.level.as_ref()))
            .and_then(|n| n.as_i64())
    });
    let con = baseline
        .and_then(|b| {
            b.values
                .iter()
                .find(|value| value.target == StatisticTarget::Ability(Ability::Constitution))
        })
        .and_then(|value| value.base_value.as_i64());
    row.healing = healing.or_else(|| {
        level
            .filter(|l| *l > 0)
            .zip(con)
            .and_then(|(l, c)| l.checked_mul(c.max(1)))
    });
    let mut state = ParticipantGameplay::from(p);
    state.temporary_hp = 0;
    if p.temporary_hp > 0 {
        row.changes.push("Clear temporary HP".into());
    }
    // Reduce each saved source once. Unknown counters remain visible for adjudication.
    state.conditions.retain_mut(
        |c| match condition_rule_for_key(c.condition_key.as_deref()) {
            Some(ConditionRule::Fatigued) => {
                row.changes.push("Remove Fatigued".into());
                false
            }
            Some(ConditionRule::Drained | ConditionRule::Doomed) => {
                if let Some(value) = c.value {
                    row.changes.push(format!(
                        "{} {} → {}",
                        c.name,
                        value,
                        value.saturating_sub(1).max(0)
                    ));
                    c.value = Some(value.saturating_sub(1));
                    value > 1
                } else {
                    row.changes.push(format!(
                        "{} has no value; set its value before reducing it",
                        c.name
                    ));
                    true
                }
            }
            _ => true,
        },
    );
    state.conditions.retain_mut(|condition| {
        let Some(duration) = &mut condition.duration else {
            return true;
        };
        use atlas_local_state::ConditionDurationUnit as Unit;
        let elapsed = match duration.unit {
            Unit::Rounds => 4800,
            Unit::Minutes => 480,
            Unit::Hours => 8,
            Unit::Days => {
                duration.amount = duration.amount.saturating_mul(24);
                duration.unit = Unit::Hours;
                8
            }
        };
        duration.amount = duration.amount.saturating_sub(elapsed);
        if duration.amount <= 0 {
            row.changes.push(format!("Expire {}", condition.name));
            false
        } else {
            true
        }
    });
    let mut after = p.clone();
    after.conditions = state.conditions.clone();
    let maximum = super::effective_hp_maximum(
        &after,
        sources.unwrap_or(super::source_conditions::SourceConditions::empty()),
        level,
    );
    row.resulting_max_hp = maximum;
    if row.healing.is_none() && p.current_hp.zip(maximum).is_some_and(|(hp, max)| hp >= max) {
        row.healing = Some(0);
    }
    if healing_mode == OvernightHealingMode::Full {
        if let Some(maximum) = maximum {
            state.current_hp = Some(maximum.max(p.current_hp.unwrap_or(0)));
            row.healing = p
                .current_hp
                .map(|current| maximum.saturating_sub(current).max(0));
            if maximum > 0 {
                state.defeated = false;
            }
        } else {
            row.unavailable_reason = Some("Set maximum HP before restoring full HP.".into());
        }
    } else if row.healing.is_none() {
        row.unavailable_reason = Some("Enter the missing HP recovery amounts.".into());
    } else if let Some(amount) = row.healing.filter(|n| *n > 0) {
        if let (Some(current), Some(maximum)) = (p.current_hp, maximum) {
            state.current_hp = Some(current.saturating_add(amount).min(maximum).max(current));
            if state.current_hp.is_some_and(|hp| hp > 0) {
                state.defeated = false;
            }
        } else {
            row.unavailable_reason = Some("Set current and maximum HP before healing.".into());
        }
    }
    if state.current_hp != p.current_hp
        && state.hp_origin == atlas_local_state::ParticipantHpOrigin::DerivedPristine
    {
        state.hp_origin = atlas_local_state::ParticipantHpOrigin::DerivedEdited;
    }
    row.resulting_hp = state.current_hp;
    if state
        .current_hp
        .zip(maximum)
        .is_some_and(|(hp, max)| hp >= max && max > 0)
    {
        state.conditions.retain(|c| {
            if condition_rule_for_key(c.condition_key.as_deref()) == Some(ConditionRule::Wounded) {
                row.changes.push("Remove Wounded at full HP".into());
                false
            } else {
                true
            }
        });
    }
    // A full night also resolves normal sleep/knockout recovery; source-controlled
    // unconsciousness remains attached to its source.
    if state.current_hp.is_some_and(|hp| hp > 0) {
        state.conditions.retain(|c| {
            if matches!(
                c.parameters,
                ConditionParameters::Unconscious {
                    waking: WakingRule::Normal
                }
            ) {
                row.changes.push("Wake from Unconscious".into());
                false
            } else {
                true
            }
        });
    }
    if let Some(record) = record {
        let catalog = super::super::spell_catalog::catalog(
            &record.detail.source,
            &p.spell_resources,
            &p.expended_spells,
            Some(&record.source_fingerprint),
        );
        state.spell_resources = catalog
            .resources
            .iter()
            .map(|r| atlas_local_state::SpellResource {
                key: r.key.clone(),
                capacity: r.capacity,
                remaining: r.capacity,
                undo_remaining: None,
            })
            .collect();
        if catalog.resources.iter().any(|r| r.remaining != r.capacity)
            || !p.expended_spells.is_empty()
        {
            row.changes
                .push("Restore spell slots, focus points and innate spells".into());
        }
    } else {
        for r in &mut state.spell_resources {
            r.remaining = r.capacity;
            r.undo_remaining = None;
        }
        if p.spell_resources.iter().any(|r| r.remaining != r.capacity) {
            row.changes.push("Restore spell resources".into());
        }
    }
    state.expended_spells.clear();
    if let Some(sources) = sources {
        for ability in &sources.uses {
            if matches!(
                ability.per,
                Per::Day | Per::PT24H | Per::PT1H | Per::PT10M | Per::PT1M | Per::Round | Per::Turn
            ) {
                state
                    .ability_states
                    .retain(|a| a.owner != ability.owner || a.rule_ordinal.is_some());
                if let Some(fingerprint) = &sources.source_fingerprint {
                    state.ability_states.push(ParticipantAbilityState {
                        owner: ability.owner.clone(),
                        rule_ordinal: None,
                        declaration_identity: Some(ability.declaration_identity.clone()),
                        source_fingerprint: fingerprint.clone(),
                        value: AbilityStateValue::Uses {
                            remaining: ability.maximum,
                        },
                    });
                    if ability.remaining(p) != ability.maximum {
                        row.changes.push(format!(
                            "Restore {} to {} uses",
                            ability.label, ability.maximum
                        ));
                    }
                }
            }
        }
        for source in sources.active_entries(p).into_iter().filter(|s| {
            matches!(
                s.rule,
                ConditionRule::Fatigued
                    | ConditionRule::Drained
                    | ConditionRule::Doomed
                    | ConditionRule::Wounded
            )
        }) {
            row.changes.push(format!(
                "{} still grants {}; change that ability if it ends",
                source.source_label,
                super::super::conditions::modeled_condition_by_rule(source.rule)
                    .map_or("a condition", |m| m.name)
            ));
        }
    }
    if record.is_none() && p.record_key.is_some() {
        return Err(AppServiceError::invalid_request(
            "This participant's source is unavailable; restore its reference data before resting",
        ));
    }
    Ok(Some(state))
}

/// GM Screen → Resting, from the pinned PF2e journal (GM Core pp.42–43).
pub(in crate::encounters) fn rules_reference() -> atlas_app_model::RecordNavigationView {
    atlas_app_model::RecordNavigationView {
        spell_rank: None,
        record_key: "journals:S55aqwWIzpQRFhcq".into(),
        owners: vec![atlas_domain::OwnedContentLocator {
            collection: "/pages".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("O357iNXwHotbecK1".into()),
        }],
        field: None,
        passage: None,
        source_fingerprint: None,
    }
}
