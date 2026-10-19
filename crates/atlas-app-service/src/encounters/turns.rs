use atlas_local_state::{EncounterParticipant, EncounterTurnState, EncounterWithParticipants};

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
    let current_index = detail
        .participants
        .iter()
        .position(|p| p.participant_key == current_key);
    let following = current_index.and_then(|index| {
        detail.participants[index + 1..].iter().find(|p| {
            eligible
                .iter()
                .any(|e| e.participant_key == p.participant_key)
        })
    });
    EncounterTurnState {
        mark_running: true,
        participant_key: Some(following.unwrap_or(eligible[0]).participant_key.clone()),
        round_number: detail.encounter.round_number
            + i64::from(current_index.is_some() && following.is_none()),
    }
}

pub(super) fn next_turn_after_removed(
    participants: &[EncounterParticipant],
    removed_participant_key: &str,
    round_number: i64,
) -> Option<EncounterTurnState> {
    let current_index = participants
        .iter()
        .position(|p| p.participant_key == removed_participant_key)?;
    let eligible = eligible_turn_participants(participants)
        .into_iter()
        .filter(|p| p.participant_key != removed_participant_key)
        .collect::<Vec<_>>();
    let following = participants[current_index + 1..].iter().find(|p| {
        eligible
            .iter()
            .any(|e| e.participant_key == p.participant_key)
    });
    Some(EncounterTurnState {
        mark_running: true,
        participant_key: following
            .or_else(|| eligible.first().copied())
            .map(|p| p.participant_key.clone()),
        round_number: round_number + i64::from(!eligible.is_empty() && following.is_none()),
    })
}

fn eligible_turn_participants(participants: &[EncounterParticipant]) -> Vec<&EncounterParticipant> {
    participants
        .iter()
        .filter(|p| p.initiative.is_some() && !p.defeated && !p.hazard_disabled)
        .collect()
}
