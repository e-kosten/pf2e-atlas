CREATE TABLE encounter_action_attempts (
    attempt_key TEXT PRIMARY KEY NOT NULL,
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK(revision >= 0),
    status TEXT NOT NULL CHECK(status IN ('awaiting_check','ready','disrupted','abandoned')),
    payload_json TEXT NOT NULL
);
CREATE UNIQUE INDEX one_pending_action_check_per_participant ON encounter_action_attempts(participant_id) WHERE status='awaiting_check';
CREATE TABLE encounter_action_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id,request_id)
);
