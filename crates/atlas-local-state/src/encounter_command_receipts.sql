CREATE TABLE encounter_command_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    family TEXT NOT NULL CHECK(family IN ('turn','event')),
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    boundary_key TEXT,
    receipt_json TEXT,
    PRIMARY KEY(encounter_id, family, request_id),
    CHECK((family='turn' AND boundary_key IS NOT NULL AND receipt_json IS NULL)
       OR (family='event' AND boundary_key IS NULL AND receipt_json IS NOT NULL))
);
CREATE UNIQUE INDEX encounter_command_turn_boundary
    ON encounter_command_receipts(boundary_key) WHERE family='turn';

INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,boundary_key)
SELECT encounter_id,'turn',request_id,intent_json,boundary_key FROM encounter_turn_receipts;

-- Preparation only saved drafts. A committed claim always takes precedence over
-- draft identification. Guard JSON inspection so unfamiliar receipts carry forward.
WITH claim_families AS (
    SELECT encounter_id, request_id,
           max(kind NOT IN ('prepared_persistent_damage','prepared_persistent_recovery')) AS committed
    FROM encounter_boundary_claims
    GROUP BY encounter_id, request_id
)
INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,receipt_json)
SELECT r.encounter_id,'event',r.request_id,r.intent_json,r.receipt_json
FROM encounter_event_receipts r
LEFT JOIN claim_families c USING(encounter_id,request_id)
WHERE c.committed IS 1 OR (c.committed IS NULL AND NOT (
    CASE WHEN json_valid(r.intent_json)
         THEN json_extract(r.intent_json,'$.action') IS 'prepare_persistent_damage'
         ELSE 0 END
    OR CASE WHEN json_valid(r.receipt_json)
            THEN json_extract(r.receipt_json,'$.version') IS 1
             AND json_array_length(r.receipt_json,'$.payload.outcomes') IS 1
             AND json_extract(r.receipt_json,'$.payload.outcomes[0].kind') IS 'persistent_prepared'
            ELSE 0 END
));

CREATE TABLE encounter_outgoing_turns_current (
    encounter_id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    requires_review INTEGER NOT NULL CHECK(requires_review IN (0,1)),
    receipt_family TEXT NOT NULL DEFAULT 'turn' CHECK(receipt_family='turn'),
    FOREIGN KEY(encounter_id,receipt_family,request_id)
        REFERENCES encounter_command_receipts(encounter_id,family,request_id) ON DELETE CASCADE
);
INSERT INTO encounter_outgoing_turns_current(encounter_id,request_id,payload_json,requires_review)
SELECT encounter_id,request_id,payload_json,requires_review FROM encounter_outgoing_turns;
DROP TABLE encounter_outgoing_turns;
ALTER TABLE encounter_outgoing_turns_current RENAME TO encounter_outgoing_turns;

CREATE TABLE encounter_boundary_claims_current (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','turn_start','turn_end')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key),
    FOREIGN KEY(encounter_id,participant_key)
        REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE
);
INSERT INTO encounter_boundary_claims_current
SELECT * FROM encounter_boundary_claims
WHERE kind NOT IN ('prepared_persistent_damage','prepared_persistent_recovery');
DROP TABLE encounter_boundary_claims;
ALTER TABLE encounter_boundary_claims_current RENAME TO encounter_boundary_claims;

DROP TABLE encounter_turn_receipts;
DROP TABLE encounter_event_receipts;
DROP TABLE encounter_roll_receipts;
DROP TABLE encounter_recovery_receipts;
