use std::collections::BTreeMap;

use atlas_app_model::{
    AppErrorCode, EncounterDetailView, EncounterParticipantConditionView,
    EncounterParticipantKindView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantVariantView, EncounterParticipantView, EncounterStatusView,
    EncounterSummaryView, ReorderEncounterParticipantPlacementView,
};
use atlas_local_state::{
    Encounter, EncounterParticipant, EncounterParticipantCondition, EncounterStatus,
    ParticipantKind, ParticipantSide, ParticipantVariant, ReorderPlacement,
};

use crate::error::{AppServiceError, AppServiceResult};

use super::hydration::hydrate_participant_records;
use super::rules::participant_runtime_block;
use super::rules::project_participant;

pub(super) fn encounter_detail_view(
    retrieval: &mut atlas_search::AtlasRetrievalService,
    encounter: Encounter,
    participants: Vec<EncounterParticipant>,
) -> AppServiceResult<EncounterDetailView> {
    let (artifact_generation, records_by_key) =
        hydrate_participant_records(retrieval, &participants)?;
    Ok(encounter_detail_from_records(
        encounter,
        participants,
        artifact_generation,
        records_by_key,
    ))
}

pub(super) fn encounter_detail_from_records(
    encounter: Encounter,
    participants: Vec<EncounterParticipant>,
    artifact_generation: Option<String>,
    records_by_key: BTreeMap<String, super::hydration::HydratedParticipantRecord>,
) -> EncounterDetailView {
    let participant_count = participants.len() as u32;
    let current_turn_participant_key = encounter.current_turn_participant_key.clone();
    let note = encounter.note.clone();
    let difficulty = super::difficulty::project(
        encounter.difficulty_reference,
        &participants,
        &records_by_key,
    );
    EncounterDetailView {
        difficulty,
        revision: encounter.revision,
        artifact_generation,
        encounter: encounter_summary(encounter, participant_count),
        note,
        current_turn_participant_key: current_turn_participant_key.clone(),
        participants: participants
            .iter()
            .map(|participant| {
                let mut view =
                    participant_view(participant.clone(), &records_by_key, &participants);
                view.turn_resources = super::turn_resources::view(
                    participant,
                    current_turn_participant_key.as_deref(),
                    &super::hydration::participant_source_actions(participant, &records_by_key),
                );
                view
            })
            .collect(),
    }
}

pub(super) fn encounter_summary(
    encounter: Encounter,
    participant_count: u32,
) -> EncounterSummaryView {
    EncounterSummaryView {
        encounter_key: encounter.encounter_key,
        slug: encounter.slug,
        name: encounter.name,
        description: encounter.description,
        status: encounter_status(encounter.status),
        round_number: encounter.round_number,
        participant_count,
        created_at: encounter.created_at,
        updated_at: encounter.updated_at,
    }
}

pub(super) fn participant_view(
    participant: EncounterParticipant,
    records_by_key: &BTreeMap<String, super::hydration::HydratedParticipantRecord>,
    participants: &[EncounterParticipant],
) -> EncounterParticipantView {
    let relationship_causes = super::rules::relationships::observer_defense_causes(
        &participant.participant_key,
        participants,
    );
    let record_detail = participant
        .record_key
        .as_ref()
        .and_then(|key| records_by_key.get(key));
    let record = record_detail.map(|r| r.view.clone());
    let stat_block = record_detail
        .and_then(|record| {
            record.arithmetic_baseline.as_ref().map(|baseline| {
                project_participant(
                    &participant,
                    baseline,
                    record.adjustment_applicable,
                    &relationship_causes,
                )
            })
        })
        .or_else(|| {
            (participant.participant_kind == ParticipantKind::Pc)
                .then(|| participant_runtime_block(&participant))
        })
        .map(|mut block| {
            super::rules::resolve_condition_subject_hints(&mut block, &participant, participants);
            block
        });
    let status = if participant.participant_kind == ParticipantKind::Pc {
        EncounterParticipantStatusView::Manual
    } else if record.is_some() {
        EncounterParticipantStatusView::Active
    } else {
        EncounterParticipantStatusView::Unresolved
    };
    let note_hint = participant
        .note
        .as_ref()
        .map(|note| note.chars().take(40).collect::<String>().trim().to_string());
    let presentation = stat_block
        .as_ref()
        .and_then(|block| {
            record_detail
                .map(|r| crate::presentation::compose_participant(r.presentation.clone(), block))
        })
        .or_else(|| {
            stat_block
                .as_ref()
                .map(|block| crate::presentation::actor::manual_presentation(&participant, block))
        });
    let effective_max_hp = super::rules::hp_maximum_stat(
        &participant,
        stat_block
            .as_ref()
            .and_then(|block| block.adjusted_level.as_ref())
            .and_then(serde_json::Number::as_i64),
    )
    .map(|stat| crate::presentation::actor::number_with_modifiers(&stat))
    .unwrap_or_else(|| atlas_app_model::NumberFactView {
        state: atlas_domain::QueryFieldState::Missing,
        value: None,
        adjustment: participant
            .max_hp
            .map(|maximum| atlas_app_model::NumberAdjustmentView {
                authored: maximum.into(),
                applied: vec![],
                suppressed: vec![],
                notes: vec![atlas_app_model::RuntimeEffectNoteView {
                    source: "maximum-hp".into(),
                    label: "Maximum HP".into(),
                    reason: "Level or condition value needed.".into(),
                }],
            }),
    });
    let knockout_facts = record_detail
        .map(|record| {
            super::rules::source_applicability(
                &record.detail.source,
                participant.participant_kind == ParticipantKind::Hazard,
            )
        })
        .unwrap_or_default();
    let knockout_unavailable_reason =
        super::conditions::modeled_condition_by_rule(super::conditions::ConditionRule::Dying)
            .and_then(|dying| {
                match super::rules::condition_applicability(dying, &participant, &knockout_facts) {
                    Some(atlas_app_model::EncounterConditionEligibilityView::Immune { reason }) => {
                        Some(reason)
                    }
                    None => Some("Dying does not apply to this participant kind.".into()),
                    _ => None,
                }
            });
    let knockout_options = if knockout_unavailable_reason.is_none() {
        super::rules::hp_events::knockout_options(&participant, None)
    } else {
        vec![]
    };
    EncounterParticipantView {
        turn_resources: None,
        knockout_options,
        knockout_unavailable_reason,
        effective_max_hp,
        track_dying: participant.track_dying,
        level_override: participant.level_override,
        base_death_threshold: participant.base_death_threshold,
        mortality: match participant.mortality {
            atlas_local_state::MortalityState::Alive => atlas_app_model::MortalityStateView::Alive,
            atlas_local_state::MortalityState::Dead => atlas_app_model::MortalityStateView::Dead,
        },
        persistent_damage_groups: super::persistent_damage::group_views(
            &participant,
            record_detail.map(|record| &record.detail.source),
        ),
        cast_unavailable_reason: super::rules::capabilities::cannot_act_reason(&participant)
            .map(str::to_owned),
        spellcasting: record_detail.map(|r| {
            super::spell_catalog::catalog(
                &r.detail.source,
                &participant.spell_resources,
                &participant.expended_spells,
                Some(&r.source_fingerprint),
            )
        }),
        revision: participant.revision,
        hp_origin: match participant.hp_origin {
            atlas_local_state::ParticipantHpOrigin::DerivedPristine => "derived_pristine",
            atlas_local_state::ParticipantHpOrigin::DerivedEdited => "derived_edited",
            atlas_local_state::ParticipantHpOrigin::Explicit => "explicit",
            atlas_local_state::ParticipantHpOrigin::Unknown => "unknown",
        }
        .into(),
        variant_origin: match participant.variant_origin {
            atlas_local_state::ParticipantVariantOrigin::DefaultUnadjusted => "default_unadjusted",
            atlas_local_state::ParticipantVariantOrigin::InheritedKnown => "inherited_known",
            atlas_local_state::ParticipantVariantOrigin::InheritedUnknown => "inherited_unknown",
            atlas_local_state::ParticipantVariantOrigin::Explicit => "explicit",
        }
        .into(),
        participant_key: participant.participant_key,
        record_key: participant.record_key,
        participant_kind: participant_kind(participant.participant_kind),
        participant_variant: participant_variant_view(participant.participant_variant),
        status,
        position: participant.position,
        display_name: participant.display_name,
        side: participant_side_view(participant.side),
        initiative: participant.initiative,
        initiative_order: participant.initiative_order,
        max_hp: participant.max_hp,
        current_hp: participant.current_hp,
        temporary_hp: participant.temporary_hp,
        defeated: participant.defeated,
        hazard_disabled: participant.hazard_disabled,
        hidden: participant.hidden,
        note: participant.note,
        note_hint: note_hint.filter(|value| !value.is_empty()),
        conditions: participant
            .conditions
            .into_iter()
            .map(condition_view)
            .collect(),
        presentation,
        record,
    }
}

pub(super) fn encounter_not_found(encounter_ref: &str) -> AppServiceError {
    AppServiceError::new(
        AppErrorCode::EncounterNotFound,
        format!("encounter `{encounter_ref}` was not found"),
    )
}

pub(super) fn encounter_status_local(status: EncounterStatusView) -> EncounterStatus {
    match status {
        EncounterStatusView::Draft => EncounterStatus::Draft,
        EncounterStatusView::Running => EncounterStatus::Running,
        EncounterStatusView::Complete => EncounterStatus::Complete,
        EncounterStatusView::Archived => EncounterStatus::Archived,
    }
}

pub(super) fn participant_side(side: EncounterParticipantSideView) -> ParticipantSide {
    match side {
        EncounterParticipantSideView::Pc => ParticipantSide::Pc,
        EncounterParticipantSideView::Ally => ParticipantSide::Ally,
        EncounterParticipantSideView::Enemy => ParticipantSide::Enemy,
        EncounterParticipantSideView::Neutral => ParticipantSide::Neutral,
        EncounterParticipantSideView::Hazard => ParticipantSide::Hazard,
    }
}

pub(super) fn participant_side_view(side: ParticipantSide) -> EncounterParticipantSideView {
    match side {
        ParticipantSide::Pc => EncounterParticipantSideView::Pc,
        ParticipantSide::Ally => EncounterParticipantSideView::Ally,
        ParticipantSide::Enemy => EncounterParticipantSideView::Enemy,
        ParticipantSide::Neutral => EncounterParticipantSideView::Neutral,
        ParticipantSide::Hazard => EncounterParticipantSideView::Hazard,
    }
}

pub(super) fn participant_variant(variant: EncounterParticipantVariantView) -> ParticipantVariant {
    match variant {
        EncounterParticipantVariantView::Normal => ParticipantVariant::Normal,
        EncounterParticipantVariantView::Elite => ParticipantVariant::Elite,
        EncounterParticipantVariantView::Weak => ParticipantVariant::Weak,
    }
}

pub(super) fn participant_variant_view(
    variant: ParticipantVariant,
) -> EncounterParticipantVariantView {
    match variant {
        ParticipantVariant::Normal => EncounterParticipantVariantView::Normal,
        ParticipantVariant::Elite => EncounterParticipantVariantView::Elite,
        ParticipantVariant::Weak => EncounterParticipantVariantView::Weak,
    }
}

pub(super) fn reorder_placement(
    placement: ReorderEncounterParticipantPlacementView,
) -> ReorderPlacement {
    match placement {
        ReorderEncounterParticipantPlacementView::Before => ReorderPlacement::Before,
        ReorderEncounterParticipantPlacementView::After => ReorderPlacement::After,
    }
}

fn condition_view(condition: EncounterParticipantCondition) -> EncounterParticipantConditionView {
    EncounterParticipantConditionView {
        condition_id: condition.condition_id,
        condition_key: condition.condition_key,
        name: condition.name,
        value: condition.value,
        source_participant_key: condition.source_participant_key,
        scope: condition_scope_view(condition.scope),
        extra_action: condition_extra_action_view(condition.extra_action),
        immunity_override: condition.immunity_override,
        parameters: condition_parameters_view(condition.parameters),
        origin: condition.origin.map(condition_origin_view),
        duration: condition.duration.map(super::timing::duration_view),
        note: condition.note,
        created_at: condition.created_at,
        updated_at: condition.updated_at,
    }
}

fn encounter_status(status: EncounterStatus) -> EncounterStatusView {
    match status {
        EncounterStatus::Draft => EncounterStatusView::Draft,
        EncounterStatus::Running => EncounterStatusView::Running,
        EncounterStatus::Complete => EncounterStatusView::Complete,
        EncounterStatus::Archived => EncounterStatusView::Archived,
    }
}

fn participant_kind(kind: ParticipantKind) -> EncounterParticipantKindView {
    match kind {
        ParticipantKind::Creature => EncounterParticipantKindView::Creature,
        ParticipantKind::Hazard => EncounterParticipantKindView::Hazard,
        ParticipantKind::Pc => EncounterParticipantKindView::Pc,
    }
}

pub(super) fn condition_scope_view(
    value: atlas_local_state::ConditionScope,
) -> atlas_app_model::ConditionScopeView {
    match value {
        atlas_local_state::ConditionScope::Everyone => {
            atlas_app_model::ConditionScopeView::Everyone
        }
        atlas_local_state::ConditionScope::SelectedParticipants { participant_keys } => {
            atlas_app_model::ConditionScopeView::SelectedParticipants { participant_keys }
        }
    }
}

pub(super) fn condition_extra_action_view(
    value: atlas_local_state::ConditionExtraAction,
) -> atlas_app_model::ConditionExtraActionView {
    use atlas_app_model::{ConditionActionView as V, ConditionExtraActionView as View};
    use atlas_local_state::{ConditionAction as A, ConditionExtraAction as Local};
    match value {
        Local::Unspecified => View::Unspecified,
        Local::Unrestricted => View::Unrestricted,
        Local::Restricted { actions, guidance } => View::Restricted {
            actions: actions
                .into_iter()
                .map(|a| match a {
                    A::Stride => V::Stride,
                    A::Strike => V::Strike,
                    A::Step => V::Step,
                    A::Leap => V::Leap,
                    A::Stand => V::Stand,
                    A::Interact => V::Interact,
                    A::CastSpell => V::CastSpell,
                    A::Sustain => V::Sustain,
                    A::Escape => V::Escape,
                    A::Crawl => V::Crawl,
                    A::Fly => V::Fly,
                    A::Swim => V::Swim,
                    A::Climb => V::Climb,
                    A::Burrow => V::Burrow,
                })
                .collect(),
            guidance,
        },
    }
}

pub(super) fn condition_parameters_view(
    value: atlas_local_state::ConditionParameters,
) -> atlas_app_model::ConditionParametersView {
    use atlas_app_model::{
        ConditionParametersView as V, DamageMultiplierView as MV, PersistentRecoveryView as RV,
        WakingRuleView as WV,
    };
    use atlas_local_state::{
        ConditionParameters as L, DamageMultiplier as M, PersistentRecovery as R, WakingRule as W,
    };
    match value {
        L::Fleeing { subject } => V::Fleeing {
            subject: subject.map(|subject| match subject {
                atlas_local_state::ConditionSubject::Participant { participant_key } => {
                    atlas_app_model::ConditionSubjectView::Participant { participant_key }
                }
                atlas_local_state::ConditionSubject::Other { description } => {
                    atlas_app_model::ConditionSubjectView::Other { description }
                }
            }),
        },
        L::Fascinated { subject } => V::Fascinated {
            subject: subject.map(|subject| match subject {
                atlas_local_state::ConditionSubject::Participant { participant_key } => {
                    atlas_app_model::ConditionSubjectView::Participant { participant_key }
                }
                atlas_local_state::ConditionSubject::Other { description } => {
                    atlas_app_model::ConditionSubjectView::Other { description }
                }
            }),
        },
        L::Attitude => V::Attitude,
        L::None => V::None,
        L::Broken { threshold } => V::Broken { threshold },
        L::Frightened { recovery } => V::Frightened {
            recovery: match recovery {
                atlas_local_state::FrightenedRecovery::EndTurn { minimum } => {
                    atlas_app_model::FrightenedRecoveryView::EndTurn { minimum }
                }
                atlas_local_state::FrightenedRecovery::Held => {
                    atlas_app_model::FrightenedRecoveryView::Held
                }
                atlas_local_state::FrightenedRecovery::SourceDependent => {
                    atlas_app_model::FrightenedRecoveryView::SourceDependent
                }
            },
        },
        L::Sickened { recovery_dc } => V::Sickened { recovery_dc },
        L::Unconscious { waking } => V::Unconscious {
            waking: match waking {
                W::Normal => WV::Normal,
                W::SourceControlled => WV::SourceControlled,
            },
        },
        L::PersistentDamageReminder {
            formula,
            damage_type,
            multiplier,
            recovery,
        } => V::PersistentDamageReminder {
            formula,
            damage_type,
            multiplier: match multiplier {
                M::Normal => MV::Normal,
                M::Double => MV::Double,
                M::Half => MV::Half,
            },
            recovery: match recovery {
                R::Standard => RV::Standard,
                R::Specified { dc } => RV::Specified { dc },
                R::SourceDependent => RV::SourceDependent,
            },
        },
        L::PersistentDamage {
            formula,
            damage_type,
            multiplier,
            recovery,
            episode_id,
        } => V::PersistentDamage {
            formula,
            damage_type,
            multiplier: match multiplier {
                M::Normal => MV::Normal,
                M::Double => MV::Double,
                M::Half => MV::Half,
            },
            recovery: match recovery {
                R::Standard => RV::Standard,
                R::Specified { dc } => RV::Specified { dc },
                R::SourceDependent => RV::SourceDependent,
            },
            episode_id: Some(episode_id),
        },
    }
}
pub(super) fn condition_origin_view(
    value: atlas_local_state::ConditionOrigin,
) -> atlas_app_model::ConditionOriginView {
    atlas_app_model::ConditionOriginView {
        application_id: value.application_id,
        grant_slot: value.grant_slot,
        participant_key: value.participant_key,
        record_key: value.record_key,
        activity_id: value.activity_id,
        artifact_generation: value.artifact_generation,
    }
}
