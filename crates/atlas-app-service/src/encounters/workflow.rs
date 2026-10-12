use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AppErrorCode, CreateEncounterRequest,
    DeleteEncounterView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterMutationView, ReorderEncounterParticipantRequest, SetEncounterTurnRequest,
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
use super::turns::{next_turn, next_turn_after_removed};

const MAX_ADD_QUANTITY: u32 = 50;

impl AtlasAppService {
    pub fn encounter_condition_definitions(
        &self,
    ) -> AppServiceResult<atlas_app_model::EncounterConditionCatalogView> {
        Ok(condition_catalog())
    }

    pub fn encounters(&self) -> AppServiceResult<EncounterIndexView> {
        let store = self.local_state_store()?;
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
    }

    pub fn encounter(&self, encounter_ref: &str) -> AppServiceResult<EncounterDetailView> {
        let detail = self
            .local_state_store()?
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        encounter_detail_view(self, detail.encounter, detail.participants)
    }

    pub fn create_encounter(
        &self,
        request: CreateEncounterRequest,
    ) -> AppServiceResult<EncounterCreateView> {
        let slug = derive_slug(&request.name);
        let encounter = self
            .local_state_store()?
            .encounters()
            .create(NewEncounter {
                slug,
                name: request.name,
                description: request.description,
                note: request.note,
            })?;
        Ok(EncounterCreateView {
            encounter: encounter_summary(encounter, 0),
        })
    }

    pub fn delete_encounter(&self, encounter_ref: &str) -> AppServiceResult<DeleteEncounterView> {
        let store = self.local_state_store()?;
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
    }

    pub fn update_encounter(
        &self,
        request: UpdateEncounterRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let encounter = self
            .local_state_store()?
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
    }

    pub fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, &request.encounter_ref)?;
        if request.quantity == 0 || request.quantity > MAX_ADD_QUANTITY {
            return Err(AppServiceError::invalid_request(format!(
                "quantity must be between 1 and {MAX_ADD_QUANTITY}"
            )));
        }
        let record = resolve_record_ref(self, &request.record_ref)?;
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
        let store = self.local_state_store()?;
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
        store
            .encounters()
            .add_participants(&receipt.encounter_key, participants)?;
        Ok(receipt)
    }

    pub fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, &request.encounter_ref)?;
        self.local_state_store()?.encounters().add_participant(
            &receipt.encounter_key,
            AddEncounterParticipant {
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
    }

    pub fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let detail = self
            .local_state_store()?
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
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
            let (_, records) = hydrate_participant_records(self, std::slice::from_ref(&existing))?;
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
        self.local_state_store()?
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
    }

    pub fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, encounter_ref)?;
        ensure_participant_in_encounter(self, &receipt.encounter_key, &request.participant_key)?;
        ensure_participant_in_encounter(
            self,
            &receipt.encounter_key,
            &request.target_participant_key,
        )?;
        self.local_state_store()?
            .encounters()
            .reorder_participant(ReorderEncounterParticipant {
                participant_key: request.participant_key,
                target_participant_key: request.target_participant_key,
                placement: reorder_placement(request.placement),
            })?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        Ok(receipt)
    }

    pub fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> AppServiceResult<EncounterMutationView> {
        let store = self.local_state_store()?;
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
    }

    pub fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, encounter_ref)?;
        ensure_participant_in_encounter(self, &receipt.encounter_key, &request.participant_key)?;
        if let Some(source) = &request.source_participant_key {
            ensure_participant_in_encounter(self, &receipt.encounter_key, source)?;
        }
        let condition = resolve_condition_input(self, request.condition_ref, request.name)?;
        self.local_state_store()?
            .encounters()
            .add_condition(AddEncounterParticipantCondition {
                participant_key: request.participant_key,
                condition_key: condition.key,
                name: condition.name,
                value: request.value,
                source_participant_key: request.source_participant_key,
                duration_rounds: request.duration_rounds.map(|value| value.max(0)),
                note: request.note,
            })?;
        Ok(receipt)
    }

    pub fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, encounter_ref)?;
        let existing = condition_in_participant(
            self,
            &receipt.encounter_key,
            participant_key,
            request.condition_id,
        )?;
        let condition = resolve_condition_update_input(
            self,
            request.condition_ref,
            request.name,
            existing.condition_key,
        )?;
        if let Some(source) = &request.source_participant_key {
            ensure_participant_in_encounter(self, &receipt.encounter_key, source)?;
        }
        let updated = self.local_state_store()?.encounters().update_condition(
            UpdateEncounterParticipantCondition {
                participant_key: participant_key.to_string(),
                condition_id: request.condition_id,
                condition_key: condition.key,
                name: condition.name,
                value: request.value,
                source_participant_key: request.source_participant_key,
                duration_rounds: request.duration_rounds.map(|value| value.max(0)),
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
    }

    pub fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> AppServiceResult<EncounterMutationView> {
        let receipt = encounter_receipt(self, encounter_ref)?;
        condition_in_participant(self, &receipt.encounter_key, participant_key, condition_id)?;
        if !self
            .local_state_store()?
            .encounters()
            .remove_condition(participant_key, condition_id)?
        {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!("encounter participant condition `{condition_id}` was not found"),
            ));
        }
        Ok(receipt)
    }

    pub fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> AppServiceResult<EncounterMutationView> {
        let store = self.local_state_store()?;
        let encounter = store
            .encounters()
            .advance_turn(&request.encounter_ref, |snapshot| {
                match request.participant_key {
                    Some(participant_key) => atlas_local_state::EncounterTurnState {
                        participant_key: Some(participant_key),
                        round_number: snapshot.encounter.round_number,
                        mark_running: false,
                    },
                    None => next_turn(snapshot),
                }
            })?
            .ok_or_else(|| encounter_not_found(&request.encounter_ref))?;
        Ok(EncounterMutationView {
            encounter_key: encounter.encounter_key,
        })
    }
}

fn condition_in_participant(
    service: &AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
    condition_id: i64,
) -> AppServiceResult<EncounterParticipantCondition> {
    let detail = service
        .local_state_store()?
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    detail
        .participants
        .into_iter()
        .find(|participant| participant.participant_key == participant_key)
        .ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!("encounter participant `{participant_key}` was not found"),
            )
        })?
        .conditions
        .into_iter()
        .find(|condition| condition.condition_id == condition_id)
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

fn resolve_condition_input(
    service: &AtlasAppService,
    condition_ref: Option<String>,
    name: Option<String>,
) -> AppServiceResult<ConditionInput> {
    if let Some(condition_ref) = condition_ref {
        if let Some(condition) = modeled_condition_by_ref(&condition_ref) {
            return Ok(ConditionInput {
                key: Some(condition.condition_ref.to_string()),
                name: condition.name.to_string(),
            });
        }
        let record = resolve_record_ref(service, &condition_ref)?;
        if record.summary.source_type.as_value().map(String::as_str) != Some("condition") {
            return Err(AppServiceError::invalid_request(
                "encounter conditions must resolve to condition records",
            ));
        }
        return Ok(ConditionInput {
            key: Some(record.summary.key.to_string()),
            name: record
                .summary
                .name
                .as_value()
                .cloned()
                .unwrap_or_else(|| record.summary.key.to_string()),
        });
    }

    let name = name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        return Err(AppServiceError::invalid_request(
            "condition name must not be empty",
        ));
    }
    Ok(ConditionInput { key: None, name })
}

fn resolve_condition_update_input(
    service: &AtlasAppService,
    condition_ref: Option<String>,
    name: String,
    existing_key: Option<String>,
) -> AppServiceResult<ConditionInput> {
    if let Some(condition_ref) = condition_ref {
        return resolve_condition_input(service, Some(condition_ref), None);
    }
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppServiceError::invalid_request(
            "condition name must not be empty",
        ));
    }
    Ok(ConditionInput {
        key: existing_key,
        name,
    })
}

fn ensure_participant_in_encounter(
    service: &AtlasAppService,
    encounter_ref: &str,
    participant_key: &str,
) -> AppServiceResult<()> {
    let detail = service
        .local_state_store()?
        .encounters()
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
    service: &AtlasAppService,
    encounter_ref: &str,
) -> AppServiceResult<EncounterMutationView> {
    let encounter = service
        .local_state_store()?
        .encounters()
        .get(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    Ok(EncounterMutationView {
        encounter_key: encounter.encounter_key,
    })
}
