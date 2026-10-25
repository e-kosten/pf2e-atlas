-- No past allocation can be inferred for an existing participant.
ALTER TABLE encounter_participants ADD COLUMN turn_resources_json TEXT
    CHECK (turn_resources_json IS NULL OR json_valid(turn_resources_json));
