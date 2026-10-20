ALTER TABLE encounter_participants ADD COLUMN level_override INTEGER;
ALTER TABLE encounter_participants ADD COLUMN base_death_threshold INTEGER NOT NULL DEFAULT 4 CHECK(base_death_threshold>0);
