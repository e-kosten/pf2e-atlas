-- Saved-domain schema at the release checkpoint.

CREATE TABLE encounter_ability_states (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    owner_key TEXT NOT NULL,
    rule_ordinal INTEGER NOT NULL CHECK(rule_ordinal >= -1),
    payload_json TEXT NOT NULL,
    PRIMARY KEY(participant_key, owner_key, rule_ordinal)
);

CREATE TABLE "encounter_boundary_claims" (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','turn_start','turn_end')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    receipt_family TEXT NOT NULL DEFAULT 'event' CHECK(receipt_family='event'),
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key),
    FOREIGN KEY(encounter_id,participant_key)
        REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE,
    FOREIGN KEY(encounter_id,receipt_family,request_id)
        REFERENCES encounter_command_receipts(encounter_id,family,request_id)
        DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE encounter_command_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    family TEXT NOT NULL CHECK(family IN ('turn','event')),
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    boundary_key TEXT,
    receipt_json TEXT,
    PRIMARY KEY(encounter_id, family, request_id),
    CHECK((family='turn' AND boundary_key IS NOT NULL AND receipt_json IS NULL)
       OR (family='event' AND boundary_key IS NULL AND receipt_json IS NOT NULL))
);

CREATE TABLE "encounter_condition_applications" (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    application_id TEXT NOT NULL,
    grant_slot TEXT NOT NULL,
    condition_id INTEGER,
    PRIMARY KEY(encounter_id, participant_key, application_id, grant_slot),
  FOREIGN KEY(encounter_id,participant_key) REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);

CREATE TABLE encounter_expended_spells (
  participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
  spell_id TEXT NOT NULL,
  PRIMARY KEY(participant_id,spell_id)
  );

CREATE TABLE "encounter_manual_trackers" (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    tracker_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    position INTEGER NOT NULL CHECK(position > 0),
    PRIMARY KEY(participant_key, tracker_key),
    UNIQUE(participant_key, position)
);

CREATE TABLE "encounter_outgoing_turns" (
    encounter_id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    requires_review INTEGER NOT NULL CHECK(requires_review IN (0,1)),
    receipt_family TEXT NOT NULL DEFAULT 'turn' CHECK(receipt_family='turn'),
    FOREIGN KEY(encounter_id,receipt_family,request_id)
        REFERENCES encounter_command_receipts(encounter_id,family,request_id) ON DELETE CASCADE
);

CREATE TABLE encounter_participant_baselines (
    participant_key TEXT PRIMARY KEY REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    payload_json TEXT NOT NULL CHECK(json_valid(payload_json))
);

CREATE TABLE "encounter_participant_conditions" (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
  condition_key TEXT,
  name TEXT NOT NULL,
  value INTEGER,
  source_participant_key TEXT,
  duration_amount INTEGER,
  duration_unit TEXT CHECK(duration_unit IN ('rounds','minutes','hours','days')),
  duration_anchor_key TEXT,
  duration_event TEXT CHECK(duration_event IN ('start','end')),
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  scope_json TEXT NOT NULL DEFAULT '{"kind":"everyone"}',
  extra_action_json TEXT NOT NULL DEFAULT '{"kind":"unspecified"}',
  immunity_override INTEGER NOT NULL DEFAULT 0 CHECK(immunity_override IN (0,1)),
  parameters_json TEXT NOT NULL DEFAULT '{"kind":"none"}',
  origin_json TEXT,
  duration_remaining_seconds INTEGER CHECK(duration_remaining_seconds>=0),
  CHECK((duration_amount IS NULL) = (duration_unit IS NULL)),
  CHECK((duration_anchor_key IS NULL) = (duration_event IS NULL)),
  CHECK(duration_anchor_key IS NULL OR duration_unit IS NOT NULL)
);

CREATE TABLE "encounter_participants" (
  id INTEGER PRIMARY KEY,
  encounter_id INTEGER NOT NULL,
  participant_key TEXT NOT NULL UNIQUE,
  record_key TEXT REFERENCES record_identity(key) ON DELETE RESTRICT,
  participant_kind TEXT NOT NULL CHECK(participant_kind IN ('creature','hazard','pc')),
  participant_variant TEXT NOT NULL DEFAULT 'normal' CHECK(participant_variant IN ('normal','elite','weak')),
  position INTEGER NOT NULL,
  display_name TEXT NOT NULL,
  record_title_snapshot TEXT,
  record_kind_snapshot TEXT,
  side TEXT NOT NULL DEFAULT 'enemy' CHECK(side IN ('pc','ally','enemy','neutral','hazard')),
  initiative INTEGER,
  initiative_order INTEGER NOT NULL,
  max_hp INTEGER,
  current_hp INTEGER,
  temporary_hp INTEGER NOT NULL DEFAULT 0,
  defeated INTEGER NOT NULL DEFAULT 0,
  hidden INTEGER NOT NULL DEFAULT 0,
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  hp_origin TEXT NOT NULL DEFAULT 'explicit' CHECK(hp_origin IN ('derived_pristine','derived_edited','explicit','unknown')),
  variant_origin TEXT NOT NULL DEFAULT 'explicit' CHECK(variant_origin IN ('default_unadjusted','inherited_known','inherited_unknown','explicit')),
  revision INTEGER NOT NULL DEFAULT 0 CHECK(typeof(revision) = 'integer' AND revision >= 0),
  hazard_disabled INTEGER NOT NULL DEFAULT 0 CHECK (hazard_disabled IN (0, 1) AND (hazard_disabled = 0 OR participant_kind = 'hazard')),
  track_dying INTEGER NOT NULL DEFAULT 0 CHECK(track_dying IN (0,1)),
  mortality TEXT NOT NULL DEFAULT 'alive' CHECK(mortality IN ('alive','dead')),
  level_override INTEGER,
  base_death_threshold INTEGER NOT NULL DEFAULT 4 CHECK(base_death_threshold>0),
  turn_resources_json TEXT
    CHECK (turn_resources_json IS NULL OR json_valid(turn_resources_json)),
  UNIQUE (encounter_id, position),
  UNIQUE (encounter_id, initiative, initiative_order),
  FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE,
  UNIQUE(encounter_id,participant_key)
);

CREATE TABLE "encounter_pending_consequences" (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    consequence_key TEXT NOT NULL,
    participant_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id, consequence_key),
  FOREIGN KEY(encounter_id,participant_key) REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);

CREATE TABLE encounter_persistent_precedence (
    participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    episode_id TEXT NOT NULL,
    source_order_json TEXT NOT NULL,
    PRIMARY KEY (participant_id, episode_id)
);

CREATE TABLE encounter_spell_resources (
  participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
  resource_key TEXT NOT NULL,
  capacity INTEGER NOT NULL CHECK(typeof(capacity)='integer' AND capacity>=0),
  remaining INTEGER NOT NULL CHECK(typeof(remaining)='integer' AND remaining BETWEEN 0 AND capacity),
  undo_remaining INTEGER CHECK(undo_remaining IS NULL OR (typeof(undo_remaining)='integer' AND undo_remaining BETWEEN 0 AND capacity)),
  PRIMARY KEY(participant_id, resource_key)
  );

CREATE TABLE party_members (
    member_key TEXT PRIMARY KEY,
    party_key TEXT NOT NULL REFERENCES parties(party_key) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 255), level INTEGER
    CHECK(level IS NULL OR (typeof(level)='integer' AND level BETWEEN 1 AND 20)),
    UNIQUE(party_key, position)
);

CREATE TABLE principals (
  id INTEGER PRIMARY KEY,
  principal_key TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);

CREATE TABLE saved_list_items (
  list_id INTEGER NOT NULL,
  record_key TEXT NOT NULL REFERENCES record_identity(key) ON DELETE RESTRICT,
  position INTEGER NOT NULL,
  note TEXT,
  record_title_snapshot TEXT NOT NULL,
  record_kind_snapshot TEXT,
  added_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (list_id, record_key),
  UNIQUE (list_id, position),
  FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
  );

CREATE TABLE saved_list_tags (
  list_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (list_id, tag),
  FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
  );

CREATE INDEX encounter_boundary_claims_receipt_idx
    ON encounter_boundary_claims(encounter_id,request_id);

CREATE UNIQUE INDEX encounter_command_turn_boundary
    ON encounter_command_receipts(boundary_key) WHERE family='turn';

CREATE TRIGGER hazard_participation_insert BEFORE INSERT ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;

CREATE TRIGGER hazard_participation_update BEFORE UPDATE ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;

INSERT INTO principals(id, principal_key, name) VALUES (1, 'local', 'Local owner');
