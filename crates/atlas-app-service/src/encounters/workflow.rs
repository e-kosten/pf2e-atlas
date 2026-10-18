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
    AddEncounterParticipant, AddEncounterParticipantCondition, EncounterParticipantCondition,
    EncounterStatus, NewEncounter, ParticipantKind, ParticipantSide, ReorderEncounterParticipant,
    UpdateEncounter as LocalUpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition, derive_slug,
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
use super::timing::{duration_input, turn_review, turn_update};
use super::turns::next_turn_after_removed;

const MAX_ADD_QUANTITY: u32 = 50;

impl AtlasAppService {
    pub fn encounter_condition_definitions(
        &self,
    ) -> AppServiceResult<atlas_app_model::EncounterConditionCatalogView> {
        Ok(condition_catalog())
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
            session
                .local_state()
                .encounters()
                .get_with_participants(encounter_ref)?
                .ok_or_else(|| encounter_not_found(encounter_ref))
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
        let (condition, reference_revision) =
            prepare_condition_input(self, request.condition_ref, request.name)?;
        self.with_write(move |session| {
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }

            let receipt = encounter_receipt(&session.local_state().encounters(), encounter_ref)?;
            ensure_participant_in_encounter(
                &session.local_state().encounters(),
                &receipt.encounter_key,
                &request.participant_key,
            )?;
            if let Some(source) = &request.source_participant_key {
                ensure_participant_in_encounter(
                    &session.local_state().encounters(),
                    &receipt.encounter_key,
                    source,
                )?;
            }
            let snapshot = session
                .local_state()
                .encounters()
                .get_with_participants(&receipt.encounter_key)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            let duration = duration_input(request.duration, None, &snapshot)?;

            session
                .local_state()
                .encounters()
                .add_condition(AddEncounterParticipantCondition {
                    participant_key: request.participant_key,
                    condition_key: condition.key,
                    name: condition.name,
                    value: request.value,
                    source_participant_key: request.source_participant_key,
                    duration,
                    note: request.note,
                })?;
            Ok(receipt)
        })
    }

    pub fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let (prepared_condition, reference_revision) = if let Some(condition_ref) =
            request.condition_ref.clone()
        {
            let (condition, revision) = prepare_condition_input(self, Some(condition_ref), None)?;
            (Some(condition), revision)
        } else {
            (None, None)
        };
        self.with_write(move |session| {
            if let Some(revision) = reference_revision {
                session.require_reference_revision(revision)?;
            }

            if request.expected_encounter_revision < 0 {
                return Err(AppServiceError::invalid_request(
                    "expected encounter revision must be nonnegative",
                ));
            }
            let receipt = encounter_receipt(&session.local_state().encounters(), encounter_ref)?;
            let snapshot = session
                .local_state()
                .encounters()
                .get_with_participants(&receipt.encounter_key)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            let existing = condition_in_snapshot(&snapshot, participant_key, request.condition_id)?;
            if request.expected_encounter_revision != snapshot.encounter.revision {
                return Err(atlas_local_state::LocalStateError::StaleEncounterRevision {
                    encounter_key: snapshot.encounter.encounter_key,
                    expected_revision: request.expected_encounter_revision,
                    actual_revision: snapshot.encounter.revision,
                }
                .into());
            }
            let duration = duration_input(request.duration, existing.duration.as_ref(), &snapshot)?;
            let condition = prepared_condition.unwrap_or(ConditionInput {
                key: existing.condition_key,
                name: request.name.trim().to_string(),
            });
            if condition.name.is_empty() {
                return Err(AppServiceError::invalid_request(
                    "condition name must not be empty",
                ));
            }
            if let Some(source) = &request.source_participant_key {
                ensure_participant_in_encounter(
                    &session.local_state().encounters(),
                    &receipt.encounter_key,
                    source,
                )?;
            }
            let updated = session.local_state().encounters().update_condition(
                UpdateEncounterParticipantCondition {
                    expected_encounter_revision: request.expected_encounter_revision,
                    participant_key: participant_key.to_string(),
                    condition_id: request.condition_id,
                    condition_key: condition.key,
                    name: condition.name,
                    value: request.value,
                    source_participant_key: request.source_participant_key,
                    duration,
                    note: request.note,
                },
            )?;
            if updated.is_none() {
                return Err(AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!(
                        "encounter participant condition `{}` was not found",
                        request.condition_id
                    ),
                ));
            }
            Ok(receipt)
        })
    }

    pub fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> AppServiceResult<EncounterMutationView> {
        self.with_write(move |session| {
            let receipt = encounter_receipt(&session.local_state().encounters(), encounter_ref)?;
            condition_in_participant(
                &session.local_state().encounters(),
                &receipt.encounter_key,
                participant_key,
                condition_id,
            )?;
            if !session
                .local_state()
                .encounters()
                .remove_condition(participant_key, condition_id)?
            {
                return Err(AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!("encounter participant condition `{condition_id}` was not found"),
                ));
            }
            Ok(receipt)
        })
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
            Ok(turn_review(&snapshot))
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
            let encounter = session
                .local_state()
                .encounters()
                .advance_reviewed_turn(encounter_ref, request.expected_revision, turn_update)?
                .ok_or_else(|| encounter_not_found(encounter_ref))?;
            Ok(EncounterMutationView {
                encounter_key: encounter.encounter_key,
            })
        })
    }
}

fn condition_in_participant(
    encounters: &atlas_local_state::EncountersRead<'_>,
    encounter_ref: &str,
    participant_key: &str,
    condition_id: i64,
) -> AppServiceResult<EncounterParticipantCondition> {
    let detail = encounters
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    condition_in_snapshot(&detail, participant_key, condition_id)
}

fn condition_in_snapshot(
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

struct ConditionInput {
    key: Option<String>,
    name: String,
}

fn prepare_condition_input(
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
