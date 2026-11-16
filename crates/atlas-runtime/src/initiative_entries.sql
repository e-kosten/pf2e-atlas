CREATE TABLE encounter_participants_v44 (
  id INTEGER PRIMARY KEY,
  encounter_id INTEGER NOT NULL,
  participant_key TEXT NOT NULL UNIQUE,
  record_key TEXT REFERENCES record_identity(key) ON DELETE RESTRICT,
  participant_kind TEXT NOT NULL CHECK(participant_kind IN ('creature','hazard','pc','initiative_entry')),
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
  UNIQUE(encounter_id,participant_key),
  CHECK(participant_kind != 'initiative_entry' OR (
    record_key IS NULL AND side = 'neutral' AND initiative IS NOT NULL
    AND participant_variant = 'normal' AND track_dying = 0
    AND max_hp IS NULL AND current_hp IS NULL AND temporary_hp = 0
    AND defeated = 0 AND mortality = 'alive'
    AND level_override IS NULL AND turn_resources_json IS NULL
  ))
);

INSERT INTO encounter_participants_v44 (
    id, encounter_id, participant_key, record_key,
    participant_kind, participant_variant, position, display_name,
    record_title_snapshot, record_kind_snapshot, side, initiative,
    initiative_order, max_hp, current_hp, temporary_hp,
    defeated, hidden, note, created_at,
    updated_at, hp_origin, variant_origin, revision,
    hazard_disabled, track_dying, mortality, level_override,
    base_death_threshold, turn_resources_json
)
SELECT
    id, encounter_id, participant_key, record_key,
    participant_kind, participant_variant, position, display_name,
    record_title_snapshot, record_kind_snapshot, side, initiative,
    initiative_order, max_hp, current_hp, temporary_hp,
    defeated, hidden, note, created_at,
    updated_at, hp_origin, variant_origin, revision,
    hazard_disabled, track_dying, mortality, level_override,
    base_death_threshold, turn_resources_json
FROM encounter_participants;
DROP TABLE encounter_participants;
ALTER TABLE encounter_participants_v44 RENAME TO encounter_participants;

CREATE TRIGGER hazard_participation_insert BEFORE INSERT ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;

CREATE TRIGGER hazard_participation_update BEFORE UPDATE ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;
