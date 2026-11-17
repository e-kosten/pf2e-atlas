CREATE TABLE encounter_adjustments (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    adjustment_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    position INTEGER NOT NULL CHECK(position > 0),
    PRIMARY KEY(participant_key, adjustment_key),
    UNIQUE(participant_key, position)
);
