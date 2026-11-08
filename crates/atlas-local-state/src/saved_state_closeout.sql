CREATE TEMP TABLE repaired_initiative_orders AS
SELECT p.id, p.initiative,
       row_number() OVER (
         PARTITION BY p.encounter_id, p.initiative
         ORDER BY p.initiative_order, p.position, p.participant_key
       ) AS initiative_order
FROM encounter_participants p
WHERE EXISTS (
    SELECT 1 FROM encounter_participants bad
    WHERE bad.encounter_id=p.encounter_id AND bad.initiative IS p.initiative
      AND bad.initiative_order<=0
);

-- Park the affected buckets at NULL, which does not participate in SQLite
-- UNIQUE comparisons. This avoids transient collisions with retained orders.
UPDATE encounter_participants SET initiative=NULL
WHERE initiative IS NOT NULL AND id IN (SELECT id FROM repaired_initiative_orders);

UPDATE encounter_participants AS participant
SET initiative=repair.initiative, initiative_order=repair.initiative_order
FROM repaired_initiative_orders AS repair
WHERE participant.id=repair.id;

DROP TABLE repaired_initiative_orders;

DROP INDEX saved_list_items_position_idx;
DROP INDEX saved_list_tags_tag_idx;
DROP INDEX encounter_participants_order_idx;

CREATE TABLE encounter_boundary_claims_with_receipt (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL,
    round_number INTEGER NOT NULL CHECK(round_number>=0),
    kind TEXT NOT NULL CHECK(kind IN ('persistent_damage','persistent_recovery','turn_start','turn_end')),
    scope_key TEXT NOT NULL,
    request_id TEXT NOT NULL,
    receipt_family TEXT NOT NULL DEFAULT 'event' CHECK(receipt_family='event'),
    PRIMARY KEY(encounter_id,participant_key,round_number,kind,scope_key),
    FOREIGN KEY(encounter_id,participant_key)
        REFERENCES encounter_participants(encounter_id,participant_key) ON DELETE CASCADE,
    FOREIGN KEY(encounter_id,receipt_family,request_id)
        REFERENCES encounter_command_receipts(encounter_id,family,request_id)
        DEFERRABLE INITIALLY DEFERRED
);

INSERT INTO encounter_boundary_claims_with_receipt(
    encounter_id,participant_key,round_number,kind,scope_key,request_id
)
SELECT encounter_id,participant_key,round_number,kind,scope_key,request_id
FROM encounter_boundary_claims;
DROP TABLE encounter_boundary_claims;
ALTER TABLE encounter_boundary_claims_with_receipt RENAME TO encounter_boundary_claims;
CREATE INDEX encounter_boundary_claims_receipt_idx
    ON encounter_boundary_claims(encounter_id,request_id);
