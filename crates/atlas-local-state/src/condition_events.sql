ALTER TABLE encounter_participants ADD COLUMN track_dying INTEGER NOT NULL DEFAULT 0 CHECK(track_dying IN (0,1));
UPDATE encounter_participants SET track_dying=1 WHERE participant_kind='pc';
ALTER TABLE encounter_participants ADD COLUMN mortality TEXT NOT NULL DEFAULT 'alive' CHECK(mortality IN ('alive','dead'));
ALTER TABLE encounter_participant_conditions ADD COLUMN parameters_json TEXT NOT NULL DEFAULT '{"kind":"none"}';
ALTER TABLE encounter_participant_conditions ADD COLUMN origin_json TEXT;
CREATE TABLE encounter_condition_applications (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    application_id TEXT NOT NULL,
    grant_slot TEXT NOT NULL,
    condition_id INTEGER NOT NULL,
    PRIMARY KEY(encounter_id, participant_key, application_id, grant_slot)
);
CREATE TABLE encounter_event_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id, request_id)
);
CREATE TABLE encounter_pending_consequences (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    consequence_key TEXT NOT NULL,
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    payload_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id, consequence_key)
);
