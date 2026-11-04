CREATE TABLE encounter_turn_receipts (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL,
    boundary_key TEXT NOT NULL UNIQUE,
    intent_json TEXT NOT NULL,
    PRIMARY KEY(encounter_id, request_id)
);
INSERT INTO encounter_turn_receipts(encounter_id,request_id,boundary_key,intent_json)
SELECT encounter_id,request_id,boundary_key,intent_json FROM encounter_outgoing_turns;

CREATE TABLE encounter_current_outgoing_turns (
    encounter_id INTEGER PRIMARY KEY,
    request_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    requires_review INTEGER NOT NULL CHECK(requires_review IN (0,1)),
    FOREIGN KEY(encounter_id,request_id) REFERENCES encounter_turn_receipts(encounter_id,request_id) ON DELETE CASCADE
);
INSERT INTO encounter_current_outgoing_turns(encounter_id,request_id,payload_json,requires_review)
SELECT t.encounter_id,t.request_id,t.payload_json,t.requires_review
FROM encounter_outgoing_turns t
WHERE t.id=(SELECT max(newest.id) FROM encounter_outgoing_turns newest WHERE newest.encounter_id=t.encounter_id);

DROP TABLE encounter_outgoing_turns;
ALTER TABLE encounter_current_outgoing_turns RENAME TO encounter_outgoing_turns;
