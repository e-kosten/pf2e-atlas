use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AppErrorCode, CreateEncounterRequest,
    DeleteEncounterView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterParticipantPreservedDomainView, EncounterParticipantResetConfirmationView,
    EncounterParticipantResetDomainView, EncounterParticipantResetResultView,
    EncounterParticipantView, EncounterSpellCastOperationView, EncounterSpellCastRequest,
    EncounterSpellCastResultView, EncounterSpellSpendTargetView, EncounterUpdateView,
    ReorderEncounterParticipantRequest, ResetEncounterParticipantRequest, SetEncounterTurnRequest,
    UpdateEncounterParticipantConditionRequest, UpdateEncounterParticipantRequest,
    UpdateEncounterRequest,
};
use atlas_domain::RecordKind;
use atlas_local_state::{
    AddEncounterParticipant, AddEncounterParticipantCondition, EncounterParticipantCondition,
    EncounterParticipantResetDomain, EncounterSpellResourceOperation, EncounterStatus,
    NewEncounter, ParticipantKind, ParticipantSide, ReorderEncounterParticipant,
    UpdateEncounter as LocalUpdateEncounter, UpdateEncounterParticipant,
    UpdateEncounterParticipantCondition, derive_slug,
};
use atlas_record::FoundryRecordType;

use crate::error::{AppServiceError, AppServiceResult};
use crate::service::AtlasAppService;

use super::conditions::{condition_catalog, modeled_condition_by_ref};
use super::hydration::{
    default_hp, hydrate_participant_records, resolve_record_ref, resolve_retrieved_record_ref,
};
use super::mechanics::{canonical_creature_level, variant_hp_adjustment_delta};
use super::projection::{
    encounter_detail_view, encounter_not_found, encounter_status_local, encounter_summary,
    participant_side, participant_variant, participant_view, reorder_placement,
};
use super::spells::{initial_spell_resources, local_target, participant_spell_cast_context};
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
    ) -> AppServiceResult<EncounterUpdateView> {
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
        let participant_count = self
            .local_state_store()?
            .encounters()
            .get_with_participants(&encounter.encounter_key)?
            .map(|detail| detail.participants.len() as u32)
            .unwrap_or_default();
        Ok(EncounterUpdateView {
            encounter: encounter_summary(encounter, participant_count),
        })
    }

    pub fn add_encounter_record_participant(
        &self,
        request: AddEncounterRecordParticipantRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        if request.quantity == 0 || request.quantity > MAX_ADD_QUANTITY {
            return Err(AppServiceError::invalid_request(format!(
                "quantity must be between 1 and {MAX_ADD_QUANTITY}"
            )));
        }
        let retrieved = resolve_retrieved_record_ref(self, &request.record_ref)?;
        let record = &retrieved.record;
        let participant_kind = match record.classification.kind {
            RecordKind::Creature => ParticipantKind::Creature,
            RecordKind::Hazard => ParticipantKind::Hazard,
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
        let (max_hp, current_hp) = default_hp(&retrieved);
        let spell_resources = initial_spell_resources(&retrieved);
        let store = self.local_state_store()?;
        for index in 0..request.quantity {
            let display_name = if request.quantity == 1 {
                record.identity.name.clone()
            } else {
                format!("{} {}", record.identity.name, index + 1)
            };
            store.encounters().add_participant_with_spell_resources(
                &request.encounter_ref,
                AddEncounterParticipant {
                    record_key: Some(record.identity.key.clone()),
                    participant_kind,
                    display_name,
                    record_title_snapshot: Some(record.identity.name.clone()),
                    record_kind_snapshot: Some(record.classification.kind.as_str().to_string()),
                    side,
                    initiative: request.initiative,
                    max_hp,
                    current_hp,
                    temporary_hp: 0,
                    note: None,
                },
                &spell_resources,
            )?;
        }
        self.encounter(&request.encounter_ref)
    }

    pub fn add_encounter_manual_participant(
        &self,
        request: AddEncounterManualParticipantRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        self.local_state_store()?.encounters().add_participant(
            &request.encounter_ref,
            AddEncounterParticipant {
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
        self.encounter(&request.encounter_ref)
    }

    pub fn update_encounter_participant(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterParticipantView> {
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
        let new_variant = participant_variant(request.participant_variant);
        let mut current_hp = request.current_hp.map(|value| value.max(0));
        if detail.encounter.status == EncounterStatus::Draft
            && existing.participant_variant != new_variant
        {
            let records_by_key =
                hydrate_participant_records(self, std::slice::from_ref(&existing))?;
            let level = existing
                .record_key
                .as_ref()
                .and_then(|key| records_by_key.records_by_key.get(key))
                .and_then(canonical_creature_level);
            let hp_delta =
                variant_hp_adjustment_delta(existing.participant_variant, new_variant, level);
            current_hp = current_hp.map(|value| (value + hp_delta).max(0));
        }
        let participant = self
            .local_state_store()?
            .encounters()
            .update_participant(UpdateEncounterParticipant {
                participant_key: request.participant_key.clone(),
                display_name: request.display_name,
                side: participant_side(request.side),
                participant_variant: new_variant,
                initiative: request.initiative,
                max_hp: request.max_hp,
                current_hp,
                temporary_hp: request.temporary_hp.max(0),
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
        let records_by_key = hydrate_participant_records(self, std::slice::from_ref(&participant))?;
        participant_view(self, participant, &records_by_key)
    }

    pub fn reorder_encounter_participant(
        &self,
        encounter_ref: &str,
        request: ReorderEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        ensure_participant_in_encounter(self, encounter_ref, &request.participant_key)?;
        ensure_participant_in_encounter(self, encounter_ref, &request.target_participant_key)?;
        self.local_state_store()?
            .encounters()
            .reorder_participant(ReorderEncounterParticipant {
                participant_key: request.participant_key,
                target_participant_key: request.target_participant_key,
                placement: reorder_placement(request.placement),
            })?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        self.encounter(encounter_ref)
    }

    pub fn remove_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> AppServiceResult<EncounterDetailView> {
        ensure_participant_in_encounter(self, encounter_ref, participant_key)?;
        let store = self.local_state_store()?;
        let detail = store
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        let was_current =
            detail.encounter.current_turn_participant_key.as_deref() == Some(participant_key);
        let replacement_turn = if was_current {
            next_turn_after_removed(
                &detail.participants,
                participant_key,
                detail.encounter.round_number,
            )
        } else {
            None
        };
        store.encounters().remove_participant(participant_key)?;
        if let Some(turn) = replacement_turn {
            store.encounters().set_turn_state(
                encounter_ref,
                turn.participant_key.as_deref(),
                turn.round_number,
                true,
            )?;
        }
        self.encounter(encounter_ref)
    }

    pub fn add_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: AddEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        ensure_participant_in_encounter(self, encounter_ref, &request.participant_key)?;
        if let Some(source) = &request.source_participant_key {
            ensure_participant_in_encounter(self, encounter_ref, source)?;
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
        self.encounter(encounter_ref)
    }

    pub fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        let existing =
            condition_in_participant(self, encounter_ref, participant_key, request.condition_id)?;
        let condition = resolve_condition_update_input(
            self,
            request.condition_ref,
            request.name,
            existing.condition_key,
        )?;
        if let Some(source) = &request.source_participant_key {
            ensure_participant_in_encounter(self, encounter_ref, source)?;
        }
        let updated = self.local_state_store()?.encounters().update_condition(
            UpdateEncounterParticipantCondition {
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
        self.encounter(encounter_ref)
    }

    pub fn remove_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        condition_id: i64,
    ) -> AppServiceResult<EncounterDetailView> {
        condition_in_participant(self, encounter_ref, participant_key, condition_id)?;
        if !self
            .local_state_store()?
            .encounters()
            .remove_condition(condition_id)?
        {
            return Err(AppServiceError::new(
                AppErrorCode::EncounterParticipantNotFound,
                format!("encounter participant condition `{condition_id}` was not found"),
            ));
        }
        self.encounter(encounter_ref)
    }

    pub fn set_encounter_turn(
        &self,
        request: SetEncounterTurnRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        let store = self.local_state_store()?;
        match request.participant_key {
            Some(participant_key) => {
                store
                    .encounters()
                    .set_current_turn(&request.encounter_ref, Some(&participant_key))?;
            }
            None => {
                let turn = next_turn(&store.encounters(), &request.encounter_ref)?;
                store.encounters().set_turn_state(
                    &request.encounter_ref,
                    turn.participant_key.as_deref(),
                    turn.round_number,
                    true,
                )?;
            }
        }
        self.encounter(&request.encounter_ref)
    }

    pub fn mutate_encounter_spell_cast(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: EncounterSpellCastRequest,
    ) -> AppServiceResult<EncounterSpellCastResultView> {
        let detail = self
            .local_state_store()?
            .encounters()
            .get_with_participants(encounter_ref)?
            .ok_or_else(|| encounter_not_found(encounter_ref))?;
        let participant = detail
            .participants
            .into_iter()
            .find(|participant| participant.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!(
                        "encounter participant `{participant_key}` was not found in encounter `{encounter_ref}`"
                    ),
                )
            })?;
        let hydrated = hydrate_participant_records(self, std::slice::from_ref(&participant))?;
        let retrieved = participant
            .record_key
            .as_ref()
            .and_then(|key| hydrated.records_by_key.get(key))
            .ok_or_else(|| {
                AppServiceError::invalid_request(
                    "spell casting requires a resolved canonical creature participant",
                )
            })?;
        let context = participant_spell_cast_context(self, &participant, retrieved)?;
        let expected = context
            .expected_target(&request.spell_occurrence_id)
            .ok_or_else(|| {
                AppServiceError::invalid_request(format!(
                    "spell occurrence `{}` has no source-backed cast state",
                    request.spell_occurrence_id
                ))
            })?;
        if request.spend_target != expected {
            return Err(AppServiceError::invalid_request(
                "spell spend target does not match the canonical occurrence ownership",
            ));
        }
        if participant.defeated && request.operation == EncounterSpellCastOperationView::CastOne {
            return Err(AppServiceError::invalid_request(
                "defeated encounter participants cannot cast spells",
            ));
        }
        let before = context.availability_for_id(&participant, &request.spell_occurrence_id);
        match &request.spend_target {
            EncounterSpellSpendTargetView::AtWill => {
                if request.operation == EncounterSpellCastOperationView::RestoreOne {
                    return Err(AppServiceError::invalid_request(
                        "at-will spells do not have tracked availability to restore",
                    ));
                }
            }
            target => {
                let target =
                    local_target(target, &request.spell_occurrence_id).ok_or_else(|| {
                        AppServiceError::invalid_request(
                            "spell spend target identity is incomplete or inconsistent",
                        )
                    })?;
                self.local_state_store()?
                    .encounters()
                    .mutate_spell_resource(
                        participant_key,
                        &target,
                        match request.operation {
                            EncounterSpellCastOperationView::CastOne => {
                                EncounterSpellResourceOperation::CastOne
                            }
                            EncounterSpellCastOperationView::RestoreOne => {
                                EncounterSpellResourceOperation::RestoreOne
                            }
                        },
                    )?;
            }
        }
        let after_context = participant_spell_cast_context(self, &participant, retrieved)?;
        let after = after_context.availability_for_id(&participant, &request.spell_occurrence_id);
        let participant_view = self
            .encounter(encounter_ref)?
            .participants
            .into_iter()
            .find(|candidate| candidate.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!("encounter participant `{participant_key}` was not found"),
                )
            })?;
        Ok(EncounterSpellCastResultView {
            operation: request.operation,
            participant_key: participant_key.to_string(),
            spell_occurrence_id: request.spell_occurrence_id,
            before,
            after,
            participant: participant_view,
        })
    }

    pub fn reset_encounter_participant(
        &self,
        encounter_ref: &str,
        participant_key: &str,
        request: ResetEncounterParticipantRequest,
    ) -> AppServiceResult<EncounterParticipantResetResultView> {
        let EncounterParticipantResetConfirmationView::ResetParticipant = request.confirmation;
        ensure_participant_in_encounter(self, encounter_ref, participant_key)?;
        let reset = self
            .local_state_store()?
            .encounters()
            .reset_participant(participant_key)?;
        let participant = self
            .encounter(encounter_ref)?
            .participants
            .into_iter()
            .find(|candidate| candidate.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!("encounter participant `{participant_key}` was not found"),
                )
            })?;
        Ok(EncounterParticipantResetResultView {
            participant_key: reset.participant.participant_key,
            reset_domains: reset
                .reset_domains
                .into_iter()
                .map(reset_domain_view)
                .collect(),
            preserved_domains: vec![
                EncounterParticipantPreservedDomainView::DisplayName,
                EncounterParticipantPreservedDomainView::Notes,
                EncounterParticipantPreservedDomainView::Visibility,
                EncounterParticipantPreservedDomainView::Side,
            ],
            cleared_current_turn: reset.cleared_current_turn,
            participant,
        })
    }
}

fn reset_domain_view(
    domain: EncounterParticipantResetDomain,
) -> EncounterParticipantResetDomainView {
    match domain {
        EncounterParticipantResetDomain::HitPoints => {
            EncounterParticipantResetDomainView::HitPoints
        }
        EncounterParticipantResetDomain::Defeated => EncounterParticipantResetDomainView::Defeated,
        EncounterParticipantResetDomain::Conditions => {
            EncounterParticipantResetDomainView::Conditions
        }
        EncounterParticipantResetDomain::InitiativeTurnState => {
            EncounterParticipantResetDomainView::InitiativeTurnState
        }
        EncounterParticipantResetDomain::VariantAdjustments => {
            EncounterParticipantResetDomainView::VariantAdjustments
        }
        EncounterParticipantResetDomain::ActionBudget => {
            EncounterParticipantResetDomainView::ActionBudget
        }
        EncounterParticipantResetDomain::SpellResources => {
            EncounterParticipantResetDomainView::SpellResources
        }
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
        if record.foundry.record_type != FoundryRecordType::Condition {
            return Err(AppServiceError::invalid_request(
                "encounter conditions must resolve to condition records",
            ));
        }
        return Ok(ConditionInput {
            key: Some(record.identity.key.to_string()),
            name: record.identity.name,
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
