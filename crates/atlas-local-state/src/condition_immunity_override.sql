ALTER TABLE encounter_participant_conditions ADD COLUMN immunity_override INTEGER NOT NULL DEFAULT 0 CHECK(immunity_override IN (0, 1));
UPDATE encounter_participant_conditions
SET immunity_override = 1,
    note = CASE WHEN note IS NULL OR trim(note) = '' THEN gm_override_reason
                WHEN note = gm_override_reason THEN note
                ELSE note || char(10) || gm_override_reason END
WHERE gm_override_reason IS NOT NULL AND trim(gm_override_reason) <> '';
UPDATE encounter_participant_conditions
SET scope_json = json_set(scope_json, '$.kind', 'selected_participants')
WHERE json_extract(scope_json, '$.kind') = 'selected_opponents';
ALTER TABLE encounter_participant_conditions DROP COLUMN gm_override_reason;
