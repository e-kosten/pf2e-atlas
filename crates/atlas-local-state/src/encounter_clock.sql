-- Configurable round length; elapsed-time countdowns preserve exact seconds.
ALTER TABLE encounters ADD COLUMN round_duration_seconds INTEGER NOT NULL DEFAULT 6 CHECK(round_duration_seconds BETWEEN 1 AND 86400);
ALTER TABLE encounter_participant_conditions ADD COLUMN duration_remaining_seconds INTEGER CHECK(duration_remaining_seconds>=0);
UPDATE encounter_participant_conditions SET duration_remaining_seconds=duration_remaining_rounds*6 WHERE duration_remaining_rounds IS NOT NULL;
ALTER TABLE encounter_participant_conditions DROP COLUMN duration_remaining_rounds;
