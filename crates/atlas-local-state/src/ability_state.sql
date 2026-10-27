CREATE TABLE encounter_ability_states (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    owner_key TEXT NOT NULL,
    rule_ordinal INTEGER NOT NULL CHECK(rule_ordinal >= -1),
    payload_json TEXT NOT NULL,
    PRIMARY KEY(participant_key, owner_key, rule_ordinal)
);
