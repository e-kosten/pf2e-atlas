-- Creation facts cannot be reconstructed for pre-existing participants.
CREATE TABLE encounter_participant_baselines (
    participant_key TEXT PRIMARY KEY REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    payload_json TEXT NOT NULL CHECK(json_valid(payload_json))
);
CREATE TABLE encounter_recovery_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL CHECK(json_valid(intent_json)),
    PRIMARY KEY(encounter_id,request_id)
);
