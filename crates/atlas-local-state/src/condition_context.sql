ALTER TABLE encounter_participant_conditions ADD COLUMN scope_json TEXT NOT NULL DEFAULT '{"kind":"everyone"}';
ALTER TABLE encounter_participant_conditions ADD COLUMN extra_action_json TEXT NOT NULL DEFAULT '{"kind":"unspecified"}';
ALTER TABLE encounter_participant_conditions ADD COLUMN gm_override_reason TEXT;
