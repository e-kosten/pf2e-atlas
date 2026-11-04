-- Referenced-table reconstruction; atlas-storage disables enforcement before BEGIN.

CREATE TEMP TABLE saved_integrity_adjustments_guard (
    row_count INTEGER NOT NULL CONSTRAINT historical_adjustments_require_preservation CHECK(row_count = 0)
);
INSERT INTO saved_integrity_adjustments_guard SELECT count(*) FROM encounter_participant_adjustments;
DROP TABLE saved_integrity_adjustments_guard;

CREATE TABLE encounters_rebuilt (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL DEFAULT 1 REFERENCES principals(id) ON DELETE RESTRICT,
  encounter_key TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  note TEXT,
  status TEXT NOT NULL CHECK(status IN ('draft','running','complete','archived')),
  round_number INTEGER NOT NULL DEFAULT 1,
  current_turn_participant_key TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL DEFAULT 0 CHECK(typeof(revision) = 'integer' AND revision >= 0),
  difficulty_party_level INTEGER
    CHECK(difficulty_party_level IS NULL OR (typeof(difficulty_party_level) = 'integer' AND difficulty_party_level BETWEEN 1 AND 20)),
  difficulty_party_size INTEGER
    CHECK(difficulty_party_size IS NULL OR (typeof(difficulty_party_size) = 'integer' AND difficulty_party_size BETWEEN 1 AND 100)),
  round_duration_seconds INTEGER NOT NULL DEFAULT 6 CHECK(round_duration_seconds BETWEEN 1 AND 86400),
  UNIQUE(owner_id, slug),
  FOREIGN KEY(id,current_turn_participant_key) REFERENCES encounter_participants(encounter_id,participant_key) DEFERRABLE INITIALLY DEFERRED
);

INSERT INTO encounters_rebuilt SELECT * FROM encounters;

CREATE TABLE encounter_participants_rebuilt (
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

INSERT INTO encounter_participants_rebuilt SELECT * FROM encounter_participants;

CREATE TABLE encounter_pending_consequences_rebuilt (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    consequence_key TEXT NOT NULL,
    participant_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id, consequence_key),
  FOREIGN KEY(encounter_id,participant_key) REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);

INSERT INTO encounter_pending_consequences_rebuilt SELECT * FROM encounter_pending_consequences;

CREATE TABLE encounter_boundary_claims_rebuilt (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','turn_start','turn_end','prepared_persistent_damage','prepared_persistent_recovery')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key),
  FOREIGN KEY(encounter_id,participant_key) REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);

INSERT INTO encounter_boundary_claims_rebuilt SELECT * FROM encounter_boundary_claims;

CREATE TABLE encounter_condition_applications_rebuilt (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    application_id TEXT NOT NULL,
    grant_slot TEXT NOT NULL,
    condition_id INTEGER,
    PRIMARY KEY(encounter_id, participant_key, application_id, grant_slot),
  FOREIGN KEY(encounter_id,participant_key) REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);

INSERT INTO encounter_condition_applications_rebuilt SELECT * FROM encounter_condition_applications;

DROP TABLE encounter_condition_applications;

DROP TABLE encounter_boundary_claims;

DROP TABLE encounter_pending_consequences;

DROP TABLE encounter_participants;

DROP TABLE encounters;

ALTER TABLE encounters_rebuilt RENAME TO encounters;

ALTER TABLE encounter_participants_rebuilt RENAME TO encounter_participants;

ALTER TABLE encounter_pending_consequences_rebuilt RENAME TO encounter_pending_consequences;

ALTER TABLE encounter_boundary_claims_rebuilt RENAME TO encounter_boundary_claims;

ALTER TABLE encounter_condition_applications_rebuilt RENAME TO encounter_condition_applications;

CREATE INDEX encounter_participants_order_idx
  ON encounter_participants(encounter_id, initiative, initiative_order);

CREATE TRIGGER hazard_participation_insert BEFORE INSERT ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;

CREATE TRIGGER hazard_participation_update BEFORE UPDATE ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;

DROP TABLE encounter_participant_adjustments;
