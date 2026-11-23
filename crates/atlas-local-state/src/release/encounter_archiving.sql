ALTER TABLE encounters ADD COLUMN archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1));
UPDATE encounters SET archived = CASE status
    WHEN 'archived' THEN 1 WHEN 'draft' THEN 0 WHEN 'running' THEN 0 WHEN 'complete' THEN 0 END;
ALTER TABLE encounters DROP COLUMN status;

-- Map each frozen encounter independently. Invalid or mixed metadata aborts
-- the migration before any new representation becomes visible.
CREATE TEMP TABLE checked_turn_archives (
    encounter_id INTEGER PRIMARY KEY,
    archived INTEGER NOT NULL CHECK(archived IN (0,1))
);
INSERT INTO checked_turn_archives
SELECT encounter_id, CASE WHEN json_type(payload_json, '$.version') = 'integer'
    AND json_extract(payload_json, '$.version') = 1
    AND json_type(payload_json, '$.payload.snapshot.encounter.archived') IS NULL
    AND (SELECT count(*) FROM json_each(payload_json, '$.payload.snapshot.encounter') WHERE key='status') = 1
    THEN CASE json_extract(payload_json, '$.payload.snapshot.encounter.status')
      WHEN 'archived' THEN 1 WHEN 'draft' THEN 0 WHEN 'running' THEN 0 WHEN 'complete' THEN 0 END END
FROM encounter_outgoing_turns;
UPDATE encounter_outgoing_turns AS outgoing
SET payload_json = json_set(json_remove(payload_json, '$.payload.snapshot.encounter.status'),
    '$.payload.snapshot.encounter.archived', json(CASE checked.archived WHEN 1 THEN 'true' ELSE 'false' END))
FROM checked_turn_archives AS checked WHERE checked.encounter_id = outgoing.encounter_id;
DROP TABLE checked_turn_archives;
