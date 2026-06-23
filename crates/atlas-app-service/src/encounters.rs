use std::collections::BTreeMap;

use atlas_app_model::{
    AddEncounterManualParticipantRequest, AddEncounterParticipantConditionRequest,
    AddEncounterRecordParticipantRequest, AppErrorCode, CreateEncounterRequest,
    DeleteEncounterView, EncounterCreateView, EncounterDetailView, EncounterIndexView,
    EncounterParticipantConditionView, EncounterParticipantDetailView,
    EncounterParticipantKindView, EncounterParticipantSideView, EncounterParticipantStatusView,
    EncounterParticipantView, EncounterStatusView, EncounterSummaryView, EncounterUpdateView,
    ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
    SetEncounterTurnRequest, UpdateEncounterParticipantConditionRequest,
    UpdateEncounterParticipantRequest, UpdateEncounterRequest,
};
use atlas_domain::{RecordKey, RecordKind};
use atlas_local_state::{
    AddEncounterParticipant, AddEncounterParticipantCondition, Encounter, EncounterParticipant,
    EncounterParticipantCondition, EncounterStatus, NewEncounter, ParticipantKind, ParticipantSide,
    ReorderEncounterParticipant, ReorderPlacement, UpdateEncounter as LocalUpdateEncounter,
    UpdateEncounterParticipant, UpdateEncounterParticipantCondition, derive_slug,
};
use atlas_record::MetricValue;
use atlas_search::{
    GetRecordsRequest, RecordRefResolutionResult, RecordRetrieval, ResolveRecordRefRequest,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::projection::record_summary;
use crate::service::AtlasAppService;

const MAX_ADD_QUANTITY: u32 = 50;

impl AtlasAppService {
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

    pub fn encounter_participant_detail(
        &self,
        encounter_ref: &str,
        participant_key: &str,
    ) -> AppServiceResult<EncounterParticipantDetailView> {
        let detail = self.encounter(encounter_ref)?;
        let participant = detail
            .participants
            .into_iter()
            .find(|participant| participant.participant_key == participant_key)
            .ok_or_else(|| {
                AppServiceError::new(
                    AppErrorCode::EncounterParticipantNotFound,
                    format!("encounter participant `{participant_key}` was not found"),
                )
            })?;
        let record_detail = match &participant.record_key {
            Some(record_key) if participant.status == EncounterParticipantStatusView::Active => {
                Some(self.record_detail(record_key)?)
            }
            _ => None,
        };
        Ok(EncounterParticipantDetailView {
            participant,
            record_detail,
        })
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
        let record = resolve_record_ref(self, &request.record_ref)?;
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
        let (max_hp, current_hp) = default_hp(&record);
        let store = self.local_state_store()?;
        for index in 0..request.quantity {
            let display_name = if request.quantity == 1 {
                record.identity.name.clone()
            } else {
                format!("{} {}", record.identity.name, index + 1)
            };
            store.encounters().add_participant(
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
        ensure_participant_in_encounter(self, encounter_ref, &request.participant_key)?;
        let current_hp = request.current_hp.map(|value| value.max(0));
        let participant = self
            .local_state_store()?
            .encounters()
            .update_participant(UpdateEncounterParticipant {
                participant_key: request.participant_key.clone(),
                display_name: request.display_name,
                side: participant_side(request.side),
                initiative: request.initiative,
                max_hp: request.max_hp,
                current_hp,
                temporary_hp: request.temporary_hp.max(0),
                defeated: request.defeated || current_hp == Some(0),
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
        Ok(participant_view(participant, &records_by_key))
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
        if request.name.trim().is_empty() {
            return Err(AppServiceError::invalid_request(
                "condition name must not be empty",
            ));
        }
        self.local_state_store()?
            .encounters()
            .add_condition(AddEncounterParticipantCondition {
                participant_key: request.participant_key,
                condition_key: request.condition_key,
                name: request.name.trim().to_string(),
                value: request.value,
                source_participant_key: request.source_participant_key,
                duration_rounds: request.duration_rounds.map(|value| value.max(0)),
                note: request.note,
                source_note: request.source_note,
            })?;
        self.encounter(encounter_ref)
    }

    pub fn update_encounter_participant_condition(
        &self,
        encounter_ref: &str,
        request: UpdateEncounterParticipantConditionRequest,
    ) -> AppServiceResult<EncounterDetailView> {
        ensure_condition_in_encounter(self, encounter_ref, request.condition_id)?;
        if request.name.trim().is_empty() {
            return Err(AppServiceError::invalid_request(
                "condition name must not be empty",
            ));
        }
        if let Some(source) = &request.source_participant_key {
            ensure_participant_in_encounter(self, encounter_ref, source)?;
        }
        let updated = self.local_state_store()?.encounters().update_condition(
            UpdateEncounterParticipantCondition {
                condition_id: request.condition_id,
                condition_key: request.condition_key,
                name: request.name.trim().to_string(),
                value: request.value,
                source_participant_key: request.source_participant_key,
                duration_rounds: request.duration_rounds.map(|value| value.max(0)),
                note: request.note,
                source_note: request.source_note,
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
        condition_id: i64,
    ) -> AppServiceResult<EncounterDetailView> {
        ensure_condition_in_encounter(self, encounter_ref, condition_id)?;
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
                )?;
            }
        }
        self.encounter(&request.encounter_ref)
    }
}

fn ensure_condition_in_encounter(
    service: &AtlasAppService,
    encounter_ref: &str,
    condition_id: i64,
) -> AppServiceResult<()> {
    let detail = service
        .local_state_store()?
        .encounters()
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    if detail.participants.iter().any(|participant| {
        participant
            .conditions
            .iter()
            .any(|condition| condition.condition_id == condition_id)
    }) {
        Ok(())
    } else {
        Err(AppServiceError::new(
            AppErrorCode::EncounterParticipantNotFound,
            format!(
                "encounter participant condition `{condition_id}` was not found in encounter `{encounter_ref}`"
            ),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NextTurn {
    participant_key: Option<String>,
    round_number: i64,
}

fn encounter_detail_view(
    service: &AtlasAppService,
    encounter: Encounter,
    participants: Vec<EncounterParticipant>,
) -> AppServiceResult<EncounterDetailView> {
    let records_by_key = hydrate_participant_records(service, &participants)?;
    let participant_count = participants.len() as u32;
    let current_turn_participant_key = encounter.current_turn_participant_key.clone();
    let note = encounter.note.clone();
    Ok(EncounterDetailView {
        encounter: encounter_summary(encounter, participant_count),
        note,
        current_turn_participant_key,
        participants: participants
            .into_iter()
            .map(|participant| participant_view(participant, &records_by_key))
            .collect(),
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

fn hydrate_participant_records(
    service: &AtlasAppService,
    participants: &[EncounterParticipant],
) -> AppServiceResult<BTreeMap<String, atlas_record::AtlasRecord>> {
    let record_keys = participants
        .iter()
        .filter_map(|participant| {
            participant
                .record_key
                .as_deref()
                .and_then(|value| RecordKey::parse(value).ok())
        })
        .collect::<Vec<_>>();
    if record_keys.is_empty() {
        return Ok(BTreeMap::new());
    }
    service.submit_retrieval(move |retrieval| {
        Ok(retrieval
            .get_records(GetRecordsRequest {
                record_keys: &record_keys,
            })?
            .into_iter()
            .map(|record| (record.identity.key.to_string(), record))
            .collect())
    })
}

fn resolve_record_ref(
    service: &AtlasAppService,
    record_ref: &str,
) -> AppServiceResult<atlas_record::AtlasRecord> {
    let record_ref = record_ref.to_string();
    service.submit_retrieval(move |retrieval| {
        let resolution = retrieval.resolve_record_ref(ResolveRecordRefRequest {
            record_ref: &record_ref,
            filter: None,
        })?;
        let key = match resolution {
            RecordRefResolutionResult::Key(key) => key,
            RecordRefResolutionResult::Miss => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionMiss,
                    format!("record `{record_ref}` was not found"),
                ));
            }
            RecordRefResolutionResult::Ambiguous(_) => {
                return Err(AppServiceError::new(
                    AppErrorCode::RecordResolutionAmbiguous,
                    format!("record `{record_ref}` is ambiguous"),
                ));
            }
        };
        let records = retrieval.get_records(GetRecordsRequest {
            record_keys: std::slice::from_ref(&key),
        })?;
        records.into_iter().next().ok_or_else(|| {
            AppServiceError::new(
                AppErrorCode::RecordNotFound,
                format!("record not found: {key}"),
            )
        })
    })
}

fn next_turn(
    encounters: &atlas_local_state::Encounters<'_>,
    encounter_ref: &str,
) -> AppServiceResult<NextTurn> {
    let detail = encounters
        .get_with_participants(encounter_ref)?
        .ok_or_else(|| encounter_not_found(encounter_ref))?;
    let current_key = detail.encounter.current_turn_participant_key.as_deref();
    let eligible = eligible_turn_participants(&detail.participants);
    if eligible.is_empty() {
        return Ok(NextTurn {
            participant_key: None,
            round_number: detail.encounter.round_number,
        });
    }
    let Some(current_key) = current_key else {
        return Ok(NextTurn {
            participant_key: Some(eligible[0].participant_key.clone()),
            round_number: detail.encounter.round_number,
        });
    };
    let Some(current_index) = eligible
        .iter()
        .position(|participant| participant.participant_key == current_key)
    else {
        return Ok(NextTurn {
            participant_key: Some(eligible[0].participant_key.clone()),
            round_number: detail.encounter.round_number,
        });
    };
    let next_index = (current_index + 1) % eligible.len();
    Ok(NextTurn {
        participant_key: Some(eligible[next_index].participant_key.clone()),
        round_number: if next_index == 0 {
            detail.encounter.round_number + 1
        } else {
            detail.encounter.round_number
        },
    })
}

fn next_turn_after_removed(
    participants: &[EncounterParticipant],
    removed_participant_key: &str,
    round_number: i64,
) -> Option<NextTurn> {
    let eligible = eligible_turn_participants(participants);
    let current_index = eligible
        .iter()
        .position(|participant| participant.participant_key == removed_participant_key)?;
    let remaining = eligible
        .into_iter()
        .filter(|participant| participant.participant_key != removed_participant_key)
        .collect::<Vec<_>>();
    if remaining.is_empty() {
        return Some(NextTurn {
            participant_key: None,
            round_number,
        });
    }
    let next_index = if current_index >= remaining.len() {
        0
    } else {
        current_index
    };
    Some(NextTurn {
        participant_key: Some(remaining[next_index].participant_key.clone()),
        round_number: if next_index == 0 && current_index >= remaining.len() {
            round_number + 1
        } else {
            round_number
        },
    })
}

fn eligible_turn_participants(participants: &[EncounterParticipant]) -> Vec<&EncounterParticipant> {
    let active = participants
        .iter()
        .filter(|participant| participant.initiative.is_some() && !participant.defeated)
        .collect::<Vec<_>>();
    if !active.is_empty() {
        return active;
    }
    participants
        .iter()
        .filter(|participant| {
            participant.initiative.is_some()
                && (participant.participant_kind == ParticipantKind::Pc
                    || participant.side == ParticipantSide::Pc)
        })
        .collect()
}

fn default_hp(record: &atlas_record::AtlasRecord) -> (Option<i64>, Option<i64>) {
    let metric = |key: &str| {
        record.mechanics.metrics.iter().find_map(|metric| {
            if metric.key == key {
                match metric.value {
                    MetricValue::Number(value) => Some(value.round() as i64),
                    MetricValue::Text(_) | MetricValue::Boolean(_) => None,
                }
            } else {
                None
            }
        })
    };
    let max_hp = atlas_record::metrics::actor::HP_MAX
        .exact_key()
        .and_then(metric);
    let current_hp = atlas_record::metrics::actor::HP_VALUE
        .exact_key()
        .and_then(metric)
        .or(max_hp);
    (max_hp.or(current_hp), current_hp)
}

fn encounter_summary(encounter: Encounter, participant_count: u32) -> EncounterSummaryView {
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

fn participant_view(
    participant: EncounterParticipant,
    records_by_key: &BTreeMap<String, atlas_record::AtlasRecord>,
) -> EncounterParticipantView {
    let record = participant
        .record_key
        .as_ref()
        .and_then(|key| records_by_key.get(key))
        .map(record_summary);
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
    EncounterParticipantView {
        participant_key: participant.participant_key,
        record_key: participant.record_key,
        participant_kind: participant_kind(participant.participant_kind),
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
        hidden: participant.hidden,
        note: participant.note,
        note_hint: note_hint.filter(|value| !value.is_empty()),
        conditions: participant
            .conditions
            .into_iter()
            .map(condition_view)
            .collect(),
        record,
    }
}

fn condition_view(condition: EncounterParticipantCondition) -> EncounterParticipantConditionView {
    EncounterParticipantConditionView {
        condition_id: condition.condition_id,
        condition_key: condition.condition_key,
        name: condition.name,
        value: condition.value,
        source_participant_key: condition.source_participant_key,
        duration_rounds: condition.duration_rounds,
        note: condition.note,
        source_note: condition.source_note,
        created_at: condition.created_at,
        updated_at: condition.updated_at,
    }
}

fn encounter_not_found(encounter_ref: &str) -> AppServiceError {
    AppServiceError::new(
        AppErrorCode::EncounterNotFound,
        format!("encounter `{encounter_ref}` was not found"),
    )
}

fn encounter_status(status: EncounterStatus) -> EncounterStatusView {
    match status {
        EncounterStatus::Draft => EncounterStatusView::Draft,
        EncounterStatus::Running => EncounterStatusView::Running,
        EncounterStatus::Complete => EncounterStatusView::Complete,
        EncounterStatus::Archived => EncounterStatusView::Archived,
    }
}

fn encounter_status_local(status: EncounterStatusView) -> EncounterStatus {
    match status {
        EncounterStatusView::Draft => EncounterStatus::Draft,
        EncounterStatusView::Running => EncounterStatus::Running,
        EncounterStatusView::Complete => EncounterStatus::Complete,
        EncounterStatusView::Archived => EncounterStatus::Archived,
    }
}

fn participant_kind(kind: ParticipantKind) -> EncounterParticipantKindView {
    match kind {
        ParticipantKind::Creature => EncounterParticipantKindView::Creature,
        ParticipantKind::Hazard => EncounterParticipantKindView::Hazard,
        ParticipantKind::Pc => EncounterParticipantKindView::Pc,
    }
}

fn participant_side(side: EncounterParticipantSideView) -> ParticipantSide {
    match side {
        EncounterParticipantSideView::Pc => ParticipantSide::Pc,
        EncounterParticipantSideView::Ally => ParticipantSide::Ally,
        EncounterParticipantSideView::Enemy => ParticipantSide::Enemy,
        EncounterParticipantSideView::Neutral => ParticipantSide::Neutral,
        EncounterParticipantSideView::Hazard => ParticipantSide::Hazard,
    }
}

fn participant_side_view(side: ParticipantSide) -> EncounterParticipantSideView {
    match side {
        ParticipantSide::Pc => EncounterParticipantSideView::Pc,
        ParticipantSide::Ally => EncounterParticipantSideView::Ally,
        ParticipantSide::Enemy => EncounterParticipantSideView::Enemy,
        ParticipantSide::Neutral => EncounterParticipantSideView::Neutral,
        ParticipantSide::Hazard => EncounterParticipantSideView::Hazard,
    }
}

fn reorder_placement(placement: ReorderEncounterParticipantPlacementView) -> ReorderPlacement {
    match placement {
        ReorderEncounterParticipantPlacementView::Before => ReorderPlacement::Before,
        ReorderEncounterParticipantPlacementView::After => ReorderPlacement::After,
    }
}

#[cfg(test)]
mod tests {
    use atlas_app_model::{
        AddEncounterParticipantConditionRequest, CreateEncounterRequest,
        ReorderEncounterParticipantPlacementView, ReorderEncounterParticipantRequest,
        SetEncounterTurnRequest, UpdateEncounterParticipantConditionRequest,
    };
    use atlas_domain::RecordKey;
    use atlas_local_state::{AddEncounterParticipant, ParticipantKind, ParticipantSide};

    use super::*;
    use crate::test_support::fixture_worker;

    #[test]
    fn set_encounter_turn_starts_advances_and_wraps_rounds() {
        let fixture = fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Turn Test".to_string(),
                description: None,
                note: None,
            })
            .expect("encounter should create")
            .encounter;
        let store = fixture
            .worker
            .local_state_store()
            .expect("local state should open");
        store
            .encounters()
            .add_participant(&encounter.slug, pc("Unset", None))
            .expect("unset participant should add");
        let high = store
            .encounters()
            .add_participant(&encounter.slug, pc("High", Some(20)))
            .expect("high participant should add");
        let low = store
            .encounters()
            .add_participant(&encounter.slug, pc("Low", Some(10)))
            .expect("low participant should add");

        let started = fixture
            .worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: encounter.slug.clone(),
                participant_key: None,
            })
            .expect("encounter should start");
        let advanced = fixture
            .worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: encounter.slug.clone(),
                participant_key: None,
            })
            .expect("encounter should advance");
        let wrapped = fixture
            .worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: encounter.slug,
                participant_key: None,
            })
            .expect("encounter should wrap");

        assert_eq!(
            started.current_turn_participant_key.as_deref(),
            Some(high.participant_key.as_str())
        );
        assert_eq!(started.encounter.round_number, 1);
        assert_eq!(
            advanced.current_turn_participant_key.as_deref(),
            Some(low.participant_key.as_str())
        );
        assert_eq!(advanced.encounter.round_number, 1);
        assert_eq!(
            wrapped.current_turn_participant_key.as_deref(),
            Some(high.participant_key.as_str())
        );
        assert_eq!(wrapped.encounter.round_number, 2);
    }

    #[test]
    fn set_encounter_turn_falls_back_to_pcs_when_everyone_is_defeated() {
        let fixture = fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Defeated Test".to_string(),
                description: None,
                note: None,
            })
            .expect("encounter should create")
            .encounter;
        let store = fixture
            .worker
            .local_state_store()
            .expect("local state should open");
        let enemy = store
            .encounters()
            .add_participant(&encounter.slug, creature("Enemy", Some(18)))
            .expect("enemy participant should add");
        let pc = store
            .encounters()
            .add_participant(&encounter.slug, pc("PC", Some(12)))
            .expect("pc participant should add");
        fixture
            .worker
            .update_encounter_participant(&encounter.slug, participant_update(&enemy, true))
            .expect("enemy should update");
        fixture
            .worker
            .update_encounter_participant(&encounter.slug, participant_update(&pc, true))
            .expect("pc should update");

        let started = fixture
            .worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: encounter.slug,
                participant_key: None,
            })
            .expect("encounter should still start on pc");

        assert_eq!(
            started.current_turn_participant_key.as_deref(),
            Some(pc.participant_key.as_str())
        );
    }

    #[test]
    fn removing_current_turn_advances_to_next_participant() {
        let fixture = fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Remove Current".to_string(),
                description: None,
                note: None,
            })
            .expect("encounter should create")
            .encounter;
        let store = fixture
            .worker
            .local_state_store()
            .expect("local state should open");
        let first = store
            .encounters()
            .add_participant(&encounter.slug, pc("First", Some(20)))
            .expect("first should add");
        let second = store
            .encounters()
            .add_participant(&encounter.slug, pc("Second", Some(10)))
            .expect("second should add");
        fixture
            .worker
            .set_encounter_turn(SetEncounterTurnRequest {
                encounter_ref: encounter.slug.clone(),
                participant_key: Some(first.participant_key.clone()),
            })
            .expect("turn should set");

        let detail = fixture
            .worker
            .remove_encounter_participant(&encounter.slug, &first.participant_key)
            .expect("participant should remove");

        assert_eq!(
            detail.current_turn_participant_key.as_deref(),
            Some(second.participant_key.as_str())
        );
    }

    #[test]
    fn encounter_conditions_and_reorder_route_through_app_service() {
        let fixture = fixture_worker();
        let encounter = fixture
            .worker
            .create_encounter(CreateEncounterRequest {
                name: "Condition Test".to_string(),
                description: None,
                note: None,
            })
            .expect("encounter should create")
            .encounter;
        let store = fixture
            .worker
            .local_state_store()
            .expect("local state should open");
        let high = store
            .encounters()
            .add_participant(&encounter.slug, pc("High", Some(20)))
            .expect("high should add");
        let low = store
            .encounters()
            .add_participant(&encounter.slug, pc("Low", Some(10)))
            .expect("low should add");

        let reordered = fixture
            .worker
            .reorder_encounter_participant(
                &encounter.slug,
                ReorderEncounterParticipantRequest {
                    participant_key: high.participant_key.clone(),
                    target_participant_key: low.participant_key.clone(),
                    placement: ReorderEncounterParticipantPlacementView::After,
                },
            )
            .expect("reorder should succeed");
        assert_eq!(
            reordered.participants[0].participant_key,
            low.participant_key
        );
        assert_eq!(
            reordered.participants[1].participant_key,
            high.participant_key
        );

        let added = fixture
            .worker
            .add_encounter_participant_condition(
                &encounter.slug,
                AddEncounterParticipantConditionRequest {
                    participant_key: high.participant_key.clone(),
                    condition_key: None,
                    name: "Frightened".to_string(),
                    value: Some(1),
                    source_participant_key: Some(low.participant_key.clone()),
                    duration_rounds: Some(2),
                    note: Some("spell".to_string()),
                    source_note: Some("source text".to_string()),
                },
            )
            .expect("condition should add");
        let condition = &added.participants[1].conditions[0];
        assert_eq!(condition.name, "Frightened");
        assert_eq!(
            condition.source_participant_key.as_deref(),
            Some(low.participant_key.as_str())
        );

        let updated = fixture
            .worker
            .update_encounter_participant_condition(
                &encounter.slug,
                UpdateEncounterParticipantConditionRequest {
                    condition_id: condition.condition_id,
                    condition_key: None,
                    name: "Frightened".to_string(),
                    value: Some(2),
                    source_participant_key: None,
                    duration_rounds: Some(1),
                    note: None,
                    source_note: None,
                },
            )
            .expect("condition should update");
        assert_eq!(updated.participants[1].conditions[0].value, Some(2));

        let removed = fixture
            .worker
            .remove_encounter_participant_condition(&encounter.slug, condition.condition_id)
            .expect("condition should remove");
        assert!(removed.participants[1].conditions.is_empty());
    }

    fn pc(name: &str, initiative: Option<i64>) -> AddEncounterParticipant {
        AddEncounterParticipant {
            record_key: None,
            participant_kind: ParticipantKind::Pc,
            display_name: name.to_string(),
            record_title_snapshot: None,
            record_kind_snapshot: None,
            side: ParticipantSide::Pc,
            initiative,
            max_hp: Some(10),
            current_hp: Some(10),
            temporary_hp: 0,
            note: None,
        }
    }

    fn creature(name: &str, initiative: Option<i64>) -> AddEncounterParticipant {
        AddEncounterParticipant {
            record_key: Some(RecordKey::parse("actors:testCreature").expect("key should parse")),
            participant_kind: ParticipantKind::Creature,
            display_name: name.to_string(),
            record_title_snapshot: Some(name.to_string()),
            record_kind_snapshot: Some("creature".to_string()),
            side: ParticipantSide::Enemy,
            initiative,
            max_hp: Some(10),
            current_hp: Some(10),
            temporary_hp: 0,
            note: None,
        }
    }

    fn participant_update(
        participant: &EncounterParticipant,
        defeated: bool,
    ) -> UpdateEncounterParticipantRequest {
        UpdateEncounterParticipantRequest {
            participant_key: participant.participant_key.clone(),
            display_name: participant.display_name.clone(),
            side: participant_side_view(participant.side),
            initiative: participant.initiative,
            max_hp: participant.max_hp,
            current_hp: participant.current_hp,
            temporary_hp: participant.temporary_hp,
            defeated,
            hidden: participant.hidden,
            note: participant.note.clone(),
        }
    }
}
