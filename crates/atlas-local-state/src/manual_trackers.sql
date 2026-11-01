-- Explicit encounter countdowns, independent of source rules and condition causes.
CREATE TABLE encounter_manual_trackers (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    tracker_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    PRIMARY KEY(participant_key,tracker_key)
);
-- Rebuild to allow explicit turn timing for every supported duration unit.
-- Preserve deleted-condition high water as well as surviving row identities.
CREATE TEMP TABLE tracker_condition_sequence AS
SELECT COALESCE(MAX(seq),0) AS seq FROM sqlite_sequence WHERE name='encounter_participant_conditions';
CREATE TABLE encounter_participant_conditions_v23 (
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
  duration_remaining_rounds INTEGER CHECK(duration_remaining_rounds>=0),
  CHECK((duration_amount IS NULL) = (duration_unit IS NULL)),
  CHECK((duration_anchor_key IS NULL) = (duration_event IS NULL)),
  CHECK(duration_anchor_key IS NULL OR duration_unit IS NOT NULL)
);
INSERT INTO encounter_participant_conditions_v23 (
  id,participant_id,condition_key,name,value,source_participant_key,
  duration_amount,duration_unit,duration_anchor_key,duration_event,note,
  created_at,updated_at,scope_json,extra_action_json,immunity_override,parameters_json,origin_json
) SELECT id,participant_id,condition_key,name,value,source_participant_key,
  duration_amount,duration_unit,duration_anchor_key,duration_event,note,
  created_at,updated_at,scope_json,extra_action_json,immunity_override,parameters_json,origin_json
FROM encounter_participant_conditions;
DROP TABLE encounter_participant_conditions;
ALTER TABLE encounter_participant_conditions_v23 RENAME TO encounter_participant_conditions;
INSERT INTO sqlite_sequence(name,seq) SELECT 'encounter_participant_conditions',seq
FROM tracker_condition_sequence WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name='encounter_participant_conditions');
UPDATE sqlite_sequence SET seq=MAX(seq,(SELECT seq FROM tracker_condition_sequence)) WHERE name='encounter_participant_conditions';
DROP TABLE tracker_condition_sequence;
