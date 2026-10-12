use atlas_local_state::{
    EncounterParticipant, EncounterTurnState, EncounterWithParticipants, ParticipantKind,
    ParticipantSide,
};

pub(super) fn next_turn(detail: &EncounterWithParticipants) -> EncounterTurnState {
    let current_key = detail.encounter.current_turn_participant_key.as_deref();
    let eligible = eligible_turn_participants(&detail.participants);
    if eligible.is_empty() {
        return EncounterTurnState {
            mark_running: true,
            participant_key: None,
            round_number: detail.encounter.round_number,
        };
    }
    let Some(current_key) = current_key else {
        return EncounterTurnState {
            mark_running: true,
            participant_key: Some(eligible[0].participant_key.clone()),
            round_number: detail.encounter.round_number,
        };
    };
    let Some(current_index) = eligible
        .iter()
        .position(|participant| participant.participant_key == current_key)
    else {
        return EncounterTurnState {
            mark_running: true,
            participant_key: Some(eligible[0].participant_key.clone()),
            round_number: detail.encounter.round_number,
        };
    };
    let next_index = (current_index + 1) % eligible.len();
    EncounterTurnState {
        mark_running: true,
        participant_key: Some(eligible[next_index].participant_key.clone()),
        round_number: if next_index == 0 {
            detail.encounter.round_number + 1
        } else {
            detail.encounter.round_number
        },
    }
}

pub(super) fn next_turn_after_removed(
    participants: &[EncounterParticipant],
    removed_participant_key: &str,
    round_number: i64,
) -> Option<EncounterTurnState> {
    let eligible = eligible_turn_participants(participants);
    let current_index = eligible
        .iter()
        .position(|participant| participant.participant_key == removed_participant_key)?;
    let remaining = eligible
        .into_iter()
        .filter(|participant| participant.participant_key != removed_participant_key)
        .collect::<Vec<_>>();
    if remaining.is_empty() {
        return Some(EncounterTurnState {
            mark_running: true,
            participant_key: None,
            round_number,
        });
    }
    let next_index = if current_index >= remaining.len() {
        0
    } else {
        current_index
    };
    Some(EncounterTurnState {
        mark_running: true,
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
