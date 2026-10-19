ALTER TABLE encounter_participants ADD COLUMN hazard_disabled INTEGER NOT NULL DEFAULT 0 CHECK (hazard_disabled IN (0, 1) AND (hazard_disabled = 0 OR participant_kind = 'hazard'));
