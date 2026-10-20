-- Final condition-system schema, applied directly after the published party migration.
ALTER TABLE encounter_participant_conditions ADD COLUMN scope_json TEXT NOT NULL DEFAULT '{"kind":"everyone"}';
ALTER TABLE encounter_participant_conditions ADD COLUMN extra_action_json TEXT NOT NULL DEFAULT '{"kind":"unspecified"}';
ALTER TABLE encounter_participant_conditions ADD COLUMN immunity_override INTEGER NOT NULL DEFAULT 0 CHECK(immunity_override IN (0,1));

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

CREATE TABLE encounter_persistent_selections (
    participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    episode_id TEXT NOT NULL,
    selected_source_id INTEGER NOT NULL,
    membership_json TEXT NOT NULL,
    PRIMARY KEY (participant_id, episode_id)
);

CREATE TABLE encounter_boundary_claims (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','turn_start','turn_end')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key)
);

ALTER TABLE encounter_participants ADD COLUMN level_override INTEGER;
ALTER TABLE encounter_participants ADD COLUMN base_death_threshold INTEGER NOT NULL DEFAULT 4 CHECK(base_death_threshold>0);

CREATE TABLE encounter_outgoing_turns (
    id INTEGER PRIMARY KEY,
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    boundary_key TEXT NOT NULL UNIQUE,
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    requires_review INTEGER NOT NULL DEFAULT 1 CHECK(requires_review IN (0,1)),
    UNIQUE(encounter_id, request_id)
);
