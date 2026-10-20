use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AdvanceEncounterTurnRequest, AppErrorCode,
    CreateEncounterRequest, DeleteEncounterView, EncounterCreateView, EncounterDetailView,
    EncounterIndexView, EncounterMutationView, EncounterTurnReviewView,
    ReorderEncounterParticipantRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
use atlas_domain::RecordKind;
use atlas_local_state::{
    AddEncounterParticipant, EncounterParticipantCondition, EncounterStatus, NewEncounter,
    ParticipantKind, ParticipantSide, ReorderEncounterParticipant,
    UpdateEncounter as LocalUpdateEncounter, UpdateEncounterParticipant, derive_slug,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::service::AtlasAppService;

use super::conditions::{condition_catalog, modeled_condition_by_ref};
use super::hydration::{hydrate_participant_records, resolve_record_ref};
use super::mechanics::{effective_max, initial_variant};
use super::projection::{
    encounter_detail_view, encounter_not_found, encounter_status_local, encounter_summary,
    participant_side, participant_variant, reorder_placement,
};
use super::timing::{turn_review, turn_update};
use super::turns::next_turn_after_removed;

const MAX_ADD_QUANTITY: u32 = 50;

impl AtlasAppService {
    pub fn encounter_condition_definitions(
        &self,
    ) -> AppServiceResult<atlas_app_model::EncounterConditionCatalogView> {
        Ok(condition_catalog())
    }

    pub fn participant_encounter_condition_definitions(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> AppServiceResult<atlas_app_model::EncounterConditionCatalogView> {
        let context = condition_subject_context(self, encounter_ref, participant_key)?;
        let mut catalog = condition_catalog();
        catalog.conditions.retain_mut(|definition| {
            let Some(modeled) = modeled_condition_by_ref(&definition.condition_ref) else {
                return false;
            };
            match super::rules::condition_applicability(
                modeled,
                &context.participant,
                &context.facts,
            ) {
                None => false,
                Some(eligibility) => {
                    definition.eligibility = eligibility;
                    true
                }
            }
        });
        Ok(catalog)
    }

    pub fn encounters(&self) -> AppServiceResult<EncounterIndexView> {
        self.with_state(move |session| {
            let store = session.local_state();
            let details = store
                .encounters()
                .list()?
                .into_iter()
                .map(|encounter| {
                    let participant_count = store
                        .encounters()
                        .get_with_participants(&encounter.encounter_key)?
                        .map(|detail| detail.participants.len() as u32)
                        .unwrap_or_default();
                    Ok(encounter_summary(encounter, participant_count))
                })
                .collect::<AppServiceResult<Vec<_>>>()?;
            Ok(EncounterIndexView {
                encounters: details,
            })
        })
    }

    pub fn encounter(&self, encounter_ref: &str) -> AppServiceResult<EncounterDetailView> {
        let initial = self.with_state(|session| {
            let snapshot = session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            Ok(snapshot)
        })?;
        if !initial.participants.iter().any(|p| {
            p.record_key
                .as_deref()
                .is_some_and(|key| atlas_domain::RecordKey::parse(key).is_ok())
        }) {
            return Ok(super::projection::encounter_detail_from_records(
                initial.encounter,
                initial.participants,
                None,
                Default::default(),
            ));
        }
        let encounter_ref = encounter_ref.to_owned();
        self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let detail = session
                    .local_state()
                    .encounters()
                    .get_with_participants(&encounter_ref)?
                    .ok_or_else(|| encounter_not_found(&encounter_ref))?;
                encounter_detail_view(
                    session.retrieval_mut(),
                    detail.encounter,
                    detail.participants,
                )
            },
        )
    }

    pub fn create_encounter(
        &self,
        request: CreateEncounterRequest,
    ) -> AppServiceResult<EncounterCreateView> {
        self.with_write(move |session| {
            let slug = derive_slug(&request.name);
            let encounter = session.local_state().encounters().create(NewEncounter {
                slug,
                name: request.name,
                description: request.description,
                note: request.note,
            })?;
            Ok(EncounterCreateView {
                encounter: encounter_summary(encounter, 0),
            })
        })
    }

    pub fn delete_encounter(&self, encounter_ref: &str) -> AppServiceResult<DeleteEncounterView> {
        self.with_write(move |session| {
            let store = session.local_state();
            let encounter = store
                .encounters()
                .get(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            let deleted = store.encounters().delete(&encounter.encounter_key)?;
            Ok(DeleteEncounterView {
                encounter_key: encounter.encounter_key,
                slug: encounter.slug,
                deleted,
            })
        })
    }

    pub fn update_encounter(
        &self,
        request: UpdateEncounterRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let encounter = session
                .local_state()
                .encounters()
                .update(LocalUpdateEncounter {
                    encounter_key: request.encounter_key.clone(),
                    slug: request.slug,
                    name: request.name,
                    description: request.description,
                    note: request.note,
                    status: encounter_status_local(request.status),
                })?
                .ok_or_else(|| encounter_not_found(&request.encounter_key))?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }

    pub fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let encounter_ref = request.encounter_ref.clone();
        let record_ref = request.record_ref.clone();
        let (receipt, record, revision) = self.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let receipt =
                    encounter_receipt(&session.local_state().encounters(), &encounter_ref)?;
                let record = resolve_record_ref(session.retrieval_mut(), &record_ref)?;
                Ok((receipt, record, session.reference_revision()?))
            },
        )?;
        if request.quantity == 0 || request.quantity > MAX_ADD_QUANTITY {
            return Err(AppServiceError::invalid_request(format!(
                "quantity must be between 1 and {MAX_ADD_QUANTITY}"
            )));
        }
        let participant_kind = match record.summary.record_kind.as_value().copied() {
            Some(RecordKind::Creature) => ParticipantKind::Creature,
            Some(RecordKind::Hazard) => ParticipantKind::Hazard,
            _ => {
                return Err(AppServiceError::invalid_request(
                    "encounter participants must be creatures, hazards, or manually named PCs",
                ));
            }
        };
        let side = if participant_kind == ParticipantKind::Hazard {
            ParticipantSide::Hazard
        } else {
            ParticipantSide::Enemy
        };
        let (inherited, variant_origin) = initial_variant(&record.source);
        let variant = inherited.unwrap_or(atlas_local_state::ParticipantVariant::Normal);
        let max_hp = effective_max(&record.source, inherited);
        let current_hp = max_hp;
        let mut participants = Vec::with_capacity(request.quantity as usize);
        for index in 0..request.quantity {
            let display_name = if request.quantity == 1 {
                record
                    .summary
                    .name
                    .as_value()
                    .cloned()
                    .unwrap_or_else(|| record.summary.key.to_string())
            } else {
                format!(
                    "{} {}",
                    record
                        .summary
                        .name
                        .as_value()
                        .cloned()
                        .unwrap_or_else(|| record.summary.key.to_string()),
                    index + 1
                )
            };
            participants.push(AddEncounterParticipant {
                spell_resources: super::spell_catalog::initial_resources(&record.source),
                participant_variant: variant,
                variant_origin,
                hp_origin: if max_hp.is_some() {
                    atlas_local_state::ParticipantHpOrigin::DerivedPristine
                } else {
                    atlas_local_state::ParticipantHpOrigin::Unknown
                },
                record_key: Some(record.summary.key.clone()),
                participant_kind,
                display_name,
                record_title_snapshot: Some(
                    record
                        .summary
                        .name
                        .as_value()
                        .cloned()
                        .unwrap_or_else(|| record.summary.key.to_string()),
                ),
                record_kind_snapshot: Some(
                    record
                        .summary
                        .record_kind
                        .as_value()
                        .map(|k| k.as_str().to_owned())
                        .unwrap_or_default(),
                ),
                side,
                initiative: request.initiative,
                max_hp,
                current_hp,
                temporary_hp: 0,
                note: None,
            });
        }
        self.with_write(move |session| {
            session.require_reference_revision(revision)?;
            session
                .local_state()
                .encounters()
                .add_participants(&receipt.encounter_key, participants)?;
            Ok(receipt)
        })
    }

    pub fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let receipt =
                encounter_receipt(&session.local_state().encounters(), &request.encounter_ref)?;
            session.local_state().encounters().add_participant(
                &receipt.encounter_key,
                AddEncounterParticipant {
                    spell_resources: Vec::new(),
                    participant_variant: atlas_local_state::ParticipantVariant::Normal,
                    variant_origin: atlas_local_state::ParticipantVariantOrigin::Explicit,
                    hp_origin: atlas_local_state::ParticipantHpOrigin::Explicit,
                    record_key: None,
                    participant_kind: ParticipantKind::Pc,
                    display_name: request.display_name,
                    record_title_snapshot: None,
                    record_kind_snapshot: None,
                    side: ParticipantSide::Pc,
                    initiative: request.initiative,
                    max_hp: request.max_hp,
                    current_hp: request.current_hp.or(request.max_hp),
                    temporary_hp: 0,
                    note: None,
                },
            )?;
            Ok(receipt)
        })
    }

    pub fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let detail = self.with_state(|session| {
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))
        })?;
        let mut reference_revision = None;
        let existing = detail
            .participants
            .iter()
            .find(|participant| participant.participant_key == request.participant_key)
            .cloned()
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!(
                        "encounter participant `{}` was not found in encounter `{encounter_ref}`",
                        request.participant_key
                    ),
                )
            })?;
        if request.expected_revision != existing.revision {
            return Err(
                atlas_local_state::LocalStateError::StaleParticipantRevision {
                    participant_key: request.participant_key,
                    expected_revision: request.expected_revision,
                    actual_revision: existing.revision,
                }
                .into(),
            );
        }
        let new_variant = participant_variant(request.participant_variant);
        let current_edited = request.hp_edit || request.current_hp != existing.current_hp;
        let max_edited = request.max_hp_edit || request.max_hp != existing.max_hp;
        let variant_changed = request.variant_edit || existing.participant_variant != new_variant;
        let mut max_hp = request.max_hp;
        if request.hazard_disabled
            && existing.participant_kind != atlas_local_state::ParticipantKind::Hazard
        {
            return Err(AppServiceError::invalid_request(
                "Only hazards can be disabled.",
            ));
        }
        if request.defeated
            && existing.participant_kind == atlas_local_state::ParticipantKind::Hazard
        {
            return Err(AppServiceError::invalid_request(
                "Hazards use Disabled instead of Defeated.",
            ));
        }
        let mut current_hp = request.current_hp;
        let mut hp_origin = existing.hp_origin;
        if max_edited && !request.use_derived_max {
            hp_origin = atlas_local_state::ParticipantHpOrigin::Explicit;
        } else if request.use_derived_max
            || (variant_changed && hp_origin != atlas_local_state::ParticipantHpOrigin::Explicit)
        {
            let saved = existing.clone();
            let encounter_key = detail.encounter.encounter_key.clone();
            let expected_encounter_revision = detail.encounter.revision;
            let (records, revision) = self.with_workspace(
                &crate::ReadExecutionContext::uncancellable(),
                move |session| {
                    let coherent = session
                        .local_state()
                        .encounters()
                        .get(&encounter_key)?
                        .ok_or_else(|| encounter_not_found(&encounter_key))?;
                    if coherent.revision != expected_encounter_revision {
                        return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                            encounter_key: coherent.encounter_key,
                            expected_revision: expected_encounter_revision,
                            actual_revision: coherent.revision,
                        }
                        .into());
                    }
                    let (_, records) = hydrate_participant_records(
                        session.retrieval_mut(),
                        std::slice::from_ref(&saved),
                    )?;
                    Ok((records, session.reference_revision()?))
                },
            )?;
            reference_revision = Some(revision);
            let derived = existing
                .record_key
                .as_ref()
                .and_then(|k| records.get(k))
                .and_then(|r| effective_max(&r.detail.source, Some(new_variant)));
            max_hp = derived;
            current_hp = if detail.encounter.status == EncounterStatus::Draft
                && hp_origin == atlas_local_state::ParticipantHpOrigin::DerivedPristine
                && !current_edited
            {
                derived
            } else if !current_edited {
                match (existing.max_hp, existing.current_hp, derived) {
                    (Some(old), Some(current), Some(new))
                        if old >= 0 && current >= 0 && current <= old =>
                    {
                        old.checked_sub(current)
                            .and_then(|damage| new.checked_sub(damage))
                            .map(|n| n.max(0))
                    }
                    _ => existing.current_hp,
                }
            } else {
                request.current_hp
            };
            hp_origin = if derived.is_none() {
                atlas_local_state::ParticipantHpOrigin::Unknown
            } else if hp_origin == atlas_local_state::ParticipantHpOrigin::DerivedPristine
                && !current_edited
                && detail.encounter.status == EncounterStatus::Draft
            {
                atlas_local_state::ParticipantHpOrigin::DerivedPristine
            } else {
                atlas_local_state::ParticipantHpOrigin::DerivedEdited
            };
        }
        if current_edited && hp_origin == atlas_local_state::ParticipantHpOrigin::DerivedPristine {
            hp_origin = atlas_local_state::ParticipantHpOrigin::DerivedEdited;
        }
        self.with_write(move |session| {
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }
            let current = session
                .local_state()
                .encounters()
                .get_with_participants(&detail.encounter.encounter_key)?
                .ok_or_else(|| encounter_not_found(&detail.encounter.encounter_key))?;
            if let Some(participant) = current
                .participants
                .iter()
                .find(|participant| participant.participant_key == request.participant_key)
                && participant.revision != request.expected_revision
            {
                return Err(
                    atlas_local_state::LocalStateError::StaleParticipantRevision {
                        participant_key: participant.participant_key.clone(),
                        expected_revision: request.expected_revision,
                        actual_revision: participant.revision,
                    }
                    .into(),
                );
            }
            let current = current.encounter;
            if current.revision != detail.encounter.revision {
                return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                    encounter_key: current.encounter_key,
                    expected_revision: detail.encounter.revision,
                    actual_revision: current.revision,
                }
                .into());
            }
            session
                .local_state()
                .encounters()
                .update_participant(UpdateEncounterParticipant {
                    expected_revision: request.expected_revision,
                    participant_key: request.participant_key.clone(),
                    display_name: request.display_name,
                    side: participant_side(request.side),
                    participant_variant: new_variant,
                    hp_origin,
                    variant_origin: if variant_changed {
                        atlas_local_state::ParticipantVariantOrigin::Explicit
                    } else {
                        existing.variant_origin
                    },
                    initiative: request.initiative,
                    max_hp,
                    current_hp,
                    temporary_hp: request.temporary_hp,
                    defeated: request.defeated,
                    hazard_disabled: request.hazard_disabled,
                    hidden: request.hidden,
                    note: request.note,
                })?
                .ok_or_else(|| {
                    AppServiceError::new(
                        AppErrorCode::EncounterParticipantNotFound,
                        format!(
                            "encounter participant `{}` was not found",
                            request.participant_key
                        ),
                    )
                })?;
            Ok(EncounterMutationView {
                encounter_key: detail.encounter.encounter_key,
            })
        })
    }

    pub fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let receipt = encounter_receipt(&session.local_state().encounters(), encounter_ref)?;
            ensure_participant_in_encounter(
                &session.local_state().encounters(),
                &receipt.encounter_key,
                &request.participant_key,
            )?;
            ensure_participant_in_encounter(
                &session.local_state().encounters(),
                &receipt.encounter_key,
                &request.target_participant_key,
            )?;
            session
                .local_state()
                .encounters()
                .reorder_participant(ReorderEncounterParticipant {
                    participant_key: request.participant_key,
                    target_participant_key: request.target_participant_key,
                    placement: reorder_placement(request.placement),
                })?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            Ok(receipt)
        })
    }

    pub fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
        let store = session.local_state();
        let encounter = store.encounters().remove_participant_and_select_turn(
            encounter_ref, participant_key, |snapshot| {
                if snapshot.encounter.current_turn_participant_key.as_deref() == Some(participant_key) {
                    next_turn_after_removed(&snapshot.participants, participant_key, snapshot.encounter.round_number)
                } else {
                    None
                }
            },
        )?.ok_or_else(|| AppServiceError::new(
            AppErrorCode::EncounterParticipantNotFound,
            format!("encounter participant `{participant_key}` was not found in encounter `{encounter_ref}`"),
        ))?;
        Ok(EncounterMutationView {
            encounter_key: encounter.encounter_key,
        })

        })
    }

    pub fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.commit_condition_add(encounter_ref, request)
    }
    pub fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.commit_condition_update(encounter_ref, participant_key, request)
    }
    pub fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
        request: atlas_app_model::RemoveEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.commit_condition_remove(encounter_ref, participant_key, condition_id, request)
    }

    pub fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let store = session.local_state();
            let encounter = store
                .encounters()
                .advance_turn(&request.encounter_ref, |snapshot| {
                    atlas_local_state::EncounterTurnState {
                        participant_key: Some(request.participant_key),
                        round_number: snapshot.encounter.round_number,
                        mark_running: false,
                    }
                })?
                .ok_or_else(|| encounter_not_found(&request.encounter_ref))?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
    pub fn encounter_turn_review(
        &self,
        encounter_ref: &str,
    ) -> AppServiceResult<EncounterTurnReviewView> {
        self.with_state(move |session| {
            let snapshot = session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            let mut view = turn_review(&snapshot);
            view.outgoing = super::outgoing_turns::latest_review(
                &session.local_state().encounters(),
                &snapshot.encounter.encounter_key,
            )?;
            Ok(view)
        })
    }

    pub fn advance_encounter_turn(
        &self,
        encounter_ref: &str,
        request: AdvanceEncounterTurnRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            if request.expected_revision < 0 {
                return Err(AppServiceError::invalid_request(
                    "expected revision must be nonnegative",
                ));
            }
            let intent = serde_json::to_value(&request)
                .map_err(|error| AppServiceError::invalid_request(error.to_string()))?;
            let encounter = session
                .local_state()
                .encounters()
                .advance_turn_once(
                    encounter_ref,
                    &request.request_id,
                    &intent,
                    request.expected_revision,
                    |snapshot| {
                        (
                            turn_update(snapshot),
                            !turn_review(snapshot).reminders.is_empty(),
                        )
                    },
                )?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}

pub(super) fn condition_in_snapshot(
    detail: &atlas_local_state::EncounterWithParticipants,
    participant_key: &str,
    condition_id: i64,
) -> AppServiceResult<EncounterParticipantCondition> {
    detail
        .participants
        .iter()
        .find(|participant| participant.participant_key == participant_key)
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!("encounter participant `{participant_key}` was not found"),
            )
        })?
        .conditions
        .iter()
        .find(|condition| condition.condition_id == condition_id)
        .cloned()
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!(
                    "encounter participant condition `{condition_id}` was not found for participant `{participant_key}`"
                ),
            )
        })
}

pub(super) struct ConditionInput {
    pub(super) key: Option<String>,
    pub(super) name: String,
}

pub(super) fn prepare_condition_input(
    service: &AtlasAppService,
    condition_ref: Option<String>,
    name: Option<String>,
) -> AppServiceResult<(ConditionInput, Option<u64>)> {
    if let Some(condition_ref) = condition_ref {
        if let Some(condition) = modeled_condition_by_ref(&condition_ref) {
            return Ok((
                ConditionInput {
                    key: Some(condition.condition_ref.to_string()),
                    name: condition.name.to_string(),
                },
                None,
            ));
        }
        let (record, revision) = service.with_workspace(
            &crate::ReadExecutionContext::uncancellable(),
            move |session| {
                let record = resolve_record_ref(session.retrieval_mut(), &condition_ref)?;
                Ok((record, session.reference_revision()?))
            },
        )?;
        if record.summary.source_type.as_value().map(String::as_str) != Some("condition") {
            return Err(AppServiceError::invalid_request(
                "encounter conditions must resolve to condition records",
            ));
        }
        return Ok((
            ConditionInput {
                key: Some(record.summary.key.to_string()),
                name: record
                    .summary
                    .name
                    .as_value()
                    .cloned()
                    .unwrap_or_else(|| record.summary.key.to_string()),
            },
            Some(revision),
        ));
    }

    let name = name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err(AppServiceError::invalid_request(
            "condition name must not be empty",
        ));
    }
    Ok((ConditionInput { key: None, name }, None))
}

fn ensure_participant_in_encounter(
    encounters: &atlas_local_state::EncountersRead<'_>,
    encounter_ref: &str,
    participant_key: &str,
) -> AppServiceResult<()> {
    let detail = encounters
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    if detail
        .participants
        .iter()
        .any(|participant| participant.participant_key == participant_key)
    {
        Ok(())
    } else {
        Err(AppServiceError::new(
            AppErrorCode::EncounterParticipantNotFound,
            format!(
                "encounter participant `{participant_key}` was not found in encounter `{encounter_ref}`"
            ),
        ))
    }
}

fn encounter_receipt(
    encounters: &atlas_local_state::EncountersRead<'_>,
    encounter_ref: &str,
) -> AppServiceResult<EncounterMutationView> {
    let encounter = encounters
        .get(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    Ok(EncounterMutationView {
        encounter_key: encounter.encounter_key,
    })
}

pub(super) fn condition_scope_input(
    value: atlas_app_model::ConditionScopeView,
    snapshot: &atlas_local_state::EncounterWithParticipants,
) -> AppServiceResult<atlas_local_state::ConditionScope> {
    match value {
        atlas_app_model::ConditionScopeView::Everyone => {
            Ok(atlas_local_state::ConditionScope::Everyone)
        }
        atlas_app_model::ConditionScopeView::SelectedParticipants {
            mut participant_keys,
        } => {
            participant_keys.sort();
            participant_keys.dedup();
            if participant_keys.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "select at least one participant",
                ));
            }
            for key in &participant_keys {
                if !snapshot
                    .participants
                    .iter()
                    .any(|p| &p.participant_key == key)
                {
                    return Err(AppServiceError::invalid_request(
                        "selected participant must belong to this encounter",
                    ));
                }
            }
            Ok(atlas_local_state::ConditionScope::SelectedParticipants { participant_keys })
        }
    }
}

pub(super) fn condition_extra_action_input(
    value: atlas_app_model::ConditionExtraActionView,
) -> AppServiceResult<atlas_local_state::ConditionExtraAction> {
    use atlas_app_model::{ConditionActionView as A, ConditionExtraActionView as View};
    use atlas_local_state::{ConditionAction as L, ConditionExtraAction as Local};
    Ok(match value {
        View::Unspecified => Local::Unspecified,
        View::Unrestricted => Local::Unrestricted,
        View::Restricted { actions, guidance } => {
            let guidance = guidance
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty());
            if actions.is_empty() && guidance.is_none() {
                return Err(AppServiceError::invalid_request(
                    "restricted extra action requires an action or guidance",
                ));
            }
            Local::Restricted {
                actions: actions
                    .into_iter()
                    .map(|a| match a {
                        A::Stride => L::Stride,
                        A::Strike => L::Strike,
                        A::Step => L::Step,
                        A::Leap => L::Leap,
                        A::Stand => L::Stand,
                        A::Interact => L::Interact,
                        A::CastSpell => L::CastSpell,
                        A::Sustain => L::Sustain,
                        A::Escape => L::Escape,
                        A::Crawl => L::Crawl,
                        A::Fly => L::Fly,
                        A::Swim => L::Swim,
                        A::Climb => L::Climb,
                        A::Burrow => L::Burrow,
                    })
                    .collect(),
                guidance,
            }
        }
    })
}

pub(super) struct ConditionSubjectContext {
    pub(super) participant: atlas_local_state::EncounterParticipant,
    pub(super) facts: super::rules::ConditionApplicabilityFacts,
    pub(super) reference_revision: Option<u64>,
    pub(super) level: Option<i64>,
    pub(super) fortitude: Option<i64>,
    pub(super) perception: Option<i64>,
}

pub(super) fn condition_subject_context(
    service: &AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
) -> AppServiceResult<ConditionSubjectContext> {
    let participant = service.with_state(|session| {
        let snapshot = session
            .local_state()
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        snapshot
            .participants
            .into_iter()
            .find(|p| p.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    "encounter participant was not found",
                )
            })
    })?;
    if participant.record_key.is_none() {
        let level = participant.level_override;
        return Ok(ConditionSubjectContext {
            participant,
            facts: super::rules::ConditionApplicabilityFacts::default(),
            level,
            fortitude: None,
            perception: None,
            reference_revision: None,
        });
    }
    service.with_workspace(
        &crate::ReadExecutionContext::uncancellable(),
        move |session| {
            let (_, records) = hydrate_participant_records(
                session.retrieval_mut(),
                std::slice::from_ref(&participant),
            )?;
            let record = participant
                .record_key
                .as_ref()
                .and_then(|key| records.get(key));
            let actor = record.and_then(|record| match &record.presentation.body {
                atlas_app_model::RecordBodyView::Creature(actor) => Some(actor.as_ref()),
                atlas_app_model::RecordBodyView::Hazard(hazard) => Some(&hazard.actor),
                _ => None,
            });
            let immunities = record.and_then(|record| {
                use atlas_record::source_record::{ActorIwrEntry, ActorIwrKind};
                let pack = record.detail.summary.key.pack();
                let query = atlas_record::source_record::SourceQueryView::new(
                    record.detail.source.source(),
                    pack.as_str(),
                    "",
                );
                query
                    .actor()
                    .iwr(ActorIwrKind::Immunity)
                    .value()
                    .map(|entries| {
                        entries
                            .iter()
                            .enumerate()
                            .filter_map(|(index, entry)| {
                                let projected = actor
                                    .and_then(|actor| actor.immunities.value.as_ref())
                                    .and_then(|entries| entries.get(index));
                                let (source_type, exceptions) = match entry {
                                    ActorIwrEntry::Immunity(source) => (
                                        source.r#type.as_value(),
                                        authored_immunity_exceptions(
                                            &source.exceptions,
                                            projected
                                                .and_then(|entry| entry.exceptions.value.clone()),
                                            true,
                                        ),
                                    ),
                                    ActorIwrEntry::SchemaImmunity(source) => (
                                        source.r#type.as_value(),
                                        authored_immunity_exceptions(
                                            &source.exceptions,
                                            projected
                                                .and_then(|entry| entry.exceptions.value.clone()),
                                            false,
                                        ),
                                    ),
                                    _ => return None,
                                };
                                // Generated enum serialization owns the canonical source identifier.
                                let token = serde_json::to_value(source_type?).ok()?;
                                let slug = token
                                    .as_str()
                                    .or_else(|| {
                                        token.get("$value").and_then(serde_json::Value::as_str)
                                    })?
                                    .to_owned();
                                Some(super::rules::ConditionImmunity { slug, exceptions })
                            })
                            .collect()
                    })
            });
            // Pinned hazard/document.ts prepareBaseData derives BT=floor(max/2)
            // and hasHealth=max>0; this is derived source policy, not an authored BT field.
            let hazard_maximum = record
                .filter(|_| {
                    participant.participant_kind == atlas_local_state::ParticipantKind::Hazard
                })
                .and_then(|record| {
                    let pack = record.detail.summary.key.pack();
                    atlas_record::source_record::SourceQueryView::new(
                        record.detail.source.source(),
                        pack.as_str(),
                        "",
                    )
                    .actor()
                    .hp_maximum()
                    .value()
                    .cloned()
                });
            let broken_threshold_known_absent = hazard_maximum
                .as_ref()
                .and_then(serde_json::Number::as_f64)
                .is_some_and(|max| max <= 0.0);
            let broken_threshold = hazard_maximum
                .as_ref()
                .and_then(serde_json::Number::as_f64)
                .filter(|max| *max > 0.0)
                .filter(|max| *max <= 9_007_199_254_740_991.0)
                .map(|max| serde_json::Number::from((max / 2.0).floor() as i64));
            let facts = super::rules::ConditionApplicabilityFacts {
                immunities,
                broken_threshold,
                broken_threshold_known_absent,
            };
            let block = record.and_then(|record| {
                record.arithmetic_baseline.as_ref().map(|baseline| {
                    super::rules::project_participant(
                        &participant,
                        baseline,
                        record.adjustment_applicable,
                        &[],
                    )
                })
            });
            let level = participant.level_override.or_else(|| {
                block
                    .as_ref()
                    .and_then(|block| block.adjusted_level.as_ref().or(block.level.as_ref()))
                    .and_then(|number| number.as_i64())
            });
            use super::stat_block::{StatisticSave, StatisticTarget};
            let fortitude = block
                .as_ref()
                .and_then(|block| {
                    block.effective_stat(&StatisticTarget::Save(StatisticSave::Fortitude))
                })
                .and_then(|number| number.as_i64());
            let perception = block
                .as_ref()
                .and_then(|block| block.effective_stat(&StatisticTarget::Perception))
                .and_then(|number| number.as_i64());
            let reference_revision = Some(session.reference_revision()?);
            Ok(ConditionSubjectContext {
                participant,
                facts,
                level,
                fortitude,
                perception,
                reference_revision,
            })
        },
    )
}

pub(super) fn validate_condition_subject(
    context: &ConditionSubjectContext,
    snapshot: &atlas_local_state::EncounterWithParticipants,
    condition_key: Option<&str>,
    immunity_override: bool,
) -> AppServiceResult<()> {
    let current = snapshot
        .participants
        .iter()
        .find(|p| p.participant_key == context.participant.participant_key)
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                "encounter participant was not found",
            )
        })?;
    if current.revision != context.participant.revision {
        return Err(
            atlas_local_state::LocalStateError::StaleParticipantRevision {
                participant_key: current.participant_key.clone(),
                expected_revision: context.participant.revision,
                actual_revision: current.revision,
            }
            .into(),
        );
    }
    let Some(modeled) = condition_key.and_then(modeled_condition_by_ref) else {
        return Ok(());
    };
    match super::rules::condition_applicability(modeled, current, &context.facts) {
        None => Err(AppServiceError::invalid_request(
            "condition does not apply to this participant kind",
        )),
        Some(atlas_app_model::EncounterConditionEligibilityView::Immune { reason })
            if !immunity_override =>
        {
            Err(AppServiceError::invalid_request(format!(
                "{reason} Choose Add anyway to override this immunity."
            )))
        }
        _ => Ok(()),
    }
}

pub(super) fn validate_condition_parameters(
    condition_key: Option<&str>,
    scope: &atlas_app_model::ConditionScopeView,
    extra_action: &atlas_app_model::ConditionExtraActionView,
) -> AppServiceResult<()> {
    use super::conditions::ConditionRule;
    let rule = condition_key
        .and_then(modeled_condition_by_ref)
        .and_then(|condition| condition.rule);
    if !matches!(scope, atlas_app_model::ConditionScopeView::Everyone)
        && condition_key
            .and_then(modeled_condition_by_ref)
            .and_then(|condition| condition.scope_label())
            .is_none()
    {
        return Err(AppServiceError::invalid_request(
            "this condition does not support a participant scope",
        ));
    }
    if !matches!(
        extra_action,
        atlas_app_model::ConditionExtraActionView::Unspecified
    ) && rule != Some(ConditionRule::Quickened)
    {
        return Err(AppServiceError::invalid_request(
            "extra-action restrictions are supported for Quickened",
        ));
    }
    Ok(())
}

fn authored_immunity_exceptions<T>(
    source: &atlas_foundry_model::SourcePresence<T>,
    projected: Option<Vec<String>>,
    optional_exceptions: bool,
) -> Option<Vec<String>> {
    use atlas_foundry_model::SourcePresence;
    match source {
        // Pinned actor/data/iwr.ts defaults omitted optional exceptions with `?? []`.
        SourcePresence::Missing if optional_exceptions => Some(Vec::new()),
        SourcePresence::Value(_) => projected,
        SourcePresence::Missing | SourcePresence::Null | SourcePresence::Invalid(_) => None,
    }
}

#[cfg(test)]
mod immunity_source_defaults_tests {
    use super::authored_immunity_exceptions;
    use atlas_foundry_model::SourcePresence;

    #[test]
    fn only_omitted_optional_exceptions_gain_the_source_default() {
        let missing: SourcePresence<Vec<String>> = SourcePresence::Missing;
        let null: SourcePresence<Vec<String>> = SourcePresence::Null;
        assert_eq!(
            authored_immunity_exceptions(&missing, None, true),
            Some(vec![])
        );
        assert_eq!(authored_immunity_exceptions(&missing, None, false), None);
        assert_eq!(authored_immunity_exceptions(&null, None, true), None);
        let conditional = SourcePresence::Value(vec!["critical-hits".to_owned()]);
        assert_eq!(
            authored_immunity_exceptions(&conditional, Some(vec!["critical-hits".into()]), true),
            Some(vec!["critical-hits".into()])
        );
        assert_eq!(authored_immunity_exceptions(&conditional, None, true), None);
    }
}

#[cfg(test)]
mod condition_scope_tests {
    use super::*;
    #[test]
    fn relative_conditions_accept_named_participants_but_global_conditions_do_not() {
        use atlas_app_model::{ConditionExtraActionView, ConditionScopeView};
        let scope = ConditionScopeView::SelectedParticipants {
            participant_keys: vec!["observer".into()],
        };
        for condition in super::super::conditions::condition_catalog().conditions {
            let accepts = validate_condition_parameters(
                Some(&condition.condition_ref),
                &scope,
                &ConditionExtraActionView::Unspecified,
            )
            .is_ok();
            assert_eq!(
                accepts,
                matches!(
                    condition.name.as_str(),
                    "Off-Guard" | "Concealed" | "Hidden" | "Undetected" | "Unnoticed"
                ),
                "{}",
                condition.name
            );
        }
    }
}

#[cfg(test)]
mod immunity_override_tests {
    use super::*;

    #[test]
    fn known_immunity_requires_explicit_override_but_no_note() {
        let worker = crate::test_support::encounter_fixture_worker().worker;
        let encounter = worker
            .create_encounter(CreateEncounterRequest {
                name: "Immunity override".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter;
        worker
            .add_encounter_manual_participant(AddEncounterManualParticipantRequest {
                encounter_ref: encounter.encounter_key.clone(),
                display_name: "Immune participant".into(),
                max_hp: Some(10),
                current_hp: Some(10),
                initiative: None,
            })
            .unwrap();
        let snapshot = worker
            .with_state(|session| {
                Ok(session
                    .local_state()
                    .encounters()
                    .get_with_participants(&encounter.encounter_key)?
                    .unwrap())
            })
            .unwrap();
        let context = ConditionSubjectContext {
            participant: snapshot.participants[0].clone(),
            reference_revision: None,
            level: None,
            fortitude: None,
            perception: None,
            facts: super::super::rules::ConditionApplicabilityFacts {
                immunities: Some(vec![super::super::rules::ConditionImmunity {
                    slug: "unconscious".into(),
                    exceptions: Some(vec![]),
                }]),
                broken_threshold: None,
                broken_threshold_known_absent: false,
            },
        };
        let catalog = condition_catalog();
        let condition = catalog
            .conditions
            .iter()
            .find(|condition| condition.name == "Unconscious")
            .unwrap();
        assert!(
            validate_condition_subject(&context, &snapshot, Some(&condition.condition_ref), false)
                .is_err()
        );
        assert!(
            validate_condition_subject(&context, &snapshot, Some(&condition.condition_ref), true)
                .is_ok()
        );
        assert_eq!(context.participant.note, None);
    }
}
