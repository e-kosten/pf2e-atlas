CREATE TABLE encounter_boundary_claims (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','assisted_recovery','turn_start','turn_end')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key)
);
