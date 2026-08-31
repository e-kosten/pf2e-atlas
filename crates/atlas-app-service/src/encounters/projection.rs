use std::collections::BTreeMap;

use atlas_app_model::{
    AppErrorCode, EncounterDetailView, EncounterParticipantKindView, EncounterParticipantSideView,
    EncounterParticipantStatusView, EncounterParticipantVariantView, EncounterParticipantView,
    EncounterStatusView, EncounterSummaryView, RecordSurfaceProfileView,
    ReorderEncounterParticipantPlacementView, SurfaceUnavailableReasonView,
};
use atlas_local_state::{
    Encounter, EncounterParticipant, EncounterStatus, ParticipantKind, ParticipantSide,
    ParticipantVariant, ReorderPlacement,
};

use crate::error::{AppServiceError, AppServiceResult};
use crate::service::AtlasAppService;
use crate::surface::{record_surface, unavailable_participant_surface};

use super::hydration::hydrate_participant_records;
use super::mechanics::{manual_encounter_runtime, participant_encounter_runtime};

pub(super) fn encounter_detail_view(
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
    records_by_key: &BTreeMap<String, atlas_record::RetrievedRecord>,
) -> EncounterParticipantView {
    let retrieved = participant
        .record_key
        .as_ref()
        .and_then(|key| records_by_key.get(key));
    let encounter_runtime = retrieved
        .and_then(|retrieved| participant_encounter_runtime(&participant, retrieved))
        .unwrap_or_else(|| manual_encounter_runtime(&participant));
    let status = if participant.participant_kind == ParticipantKind::Pc {
        EncounterParticipantStatusView::Manual
    } else if retrieved.is_some() {
        EncounterParticipantStatusView::Active
    } else {
        EncounterParticipantStatusView::Unresolved
    };
    let note_hint = participant
        .note
        .as_ref()
        .map(|note| note.chars().take(40).collect::<String>().trim().to_string());
    let surface = if let Some(retrieved) = retrieved {
        record_surface(
            retrieved,
            RecordSurfaceProfileView::EncounterParticipant,
            Some(encounter_runtime),
        )
    } else {
        let reason = if participant.participant_kind == ParticipantKind::Pc {
            SurfaceUnavailableReasonView::ManualParticipant
        } else {
            SurfaceUnavailableReasonView::RecordUnavailable
        };
        let kind = match participant.participant_kind {
            ParticipantKind::Creature => "creature",
            ParticipantKind::Hazard => "hazard",
            ParticipantKind::Pc => "character",
        }
        .to_string();
        unavailable_participant_surface(
            participant.record_key.clone(),
            participant.display_name.clone(),
            kind,
            RecordSurfaceProfileView::EncounterParticipant,
            reason,
            encounter_runtime,
        )
    };
    EncounterParticipantView {
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
        defeated: participant.defeated,
        hidden: participant.hidden,
        note: participant.note,
        note_hint: note_hint.filter(|value| !value.is_empty()),
        surface,
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
