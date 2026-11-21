CREATE TABLE encounter_command_receipts_new (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    family TEXT NOT NULL CHECK(family IN ('turn','event','pc_addition','record_addition')),
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    boundary_key TEXT,
    receipt_json TEXT,
    PRIMARY KEY(encounter_id,family,request_id),
    CHECK((family='turn' AND boundary_key IS NOT NULL AND receipt_json IS NULL)
       OR (family='record_addition' AND boundary_key IS NULL AND receipt_json IS NULL)
       OR (family IN ('event','pc_addition') AND boundary_key IS NULL AND receipt_json IS NOT NULL))
);
INSERT INTO encounter_command_receipts_new SELECT * FROM encounter_command_receipts;
DROP TABLE encounter_command_receipts;
ALTER TABLE encounter_command_receipts_new RENAME TO encounter_command_receipts;
CREATE UNIQUE INDEX encounter_command_turn_boundary
    ON encounter_command_receipts(boundary_key) WHERE family='turn';
