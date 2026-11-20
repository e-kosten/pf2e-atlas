CREATE TABLE characters (
    character_key TEXT PRIMARY KEY NOT NULL,
    owner_id INTEGER NOT NULL REFERENCES principals(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 255),
    level INTEGER CHECK(level IS NULL OR (typeof(level)='integer' AND level BETWEEN 1 AND 20)),
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0),
    reference_payload TEXT CHECK(reference_payload IS NULL OR json_valid(reference_payload))
);
INSERT INTO characters(character_key,owner_id,name,level)
SELECT m.member_key,p.owner_id,m.name,m.level
FROM party_members m JOIN parties p USING(party_key);

CREATE TABLE party_character_memberships (
    party_key TEXT NOT NULL REFERENCES parties(party_key) ON DELETE CASCADE,
    character_key TEXT NOT NULL REFERENCES characters(character_key) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    PRIMARY KEY(party_key,character_key), UNIQUE(party_key,position)
);
INSERT INTO party_character_memberships SELECT party_key,member_key,position FROM party_members;
CREATE INDEX party_character_memberships_character ON party_character_memberships(character_key);

CREATE TABLE encounter_pc_characters (
    participant_id INTEGER PRIMARY KEY REFERENCES encounter_participants(id) ON DELETE CASCADE,
    character_key TEXT NOT NULL REFERENCES characters(character_key) ON DELETE CASCADE
);
CREATE INDEX encounter_pc_characters_character ON encounter_pc_characters(character_key);
DROP TABLE party_members;

CREATE TABLE character_creation_receipts (
    owner_id INTEGER NOT NULL REFERENCES principals(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL CHECK(length(request_id) BETWEEN 1 AND 128),
    intent_digest TEXT NOT NULL CHECK(length(intent_digest)=64),
    character_key TEXT NOT NULL,
    PRIMARY KEY(owner_id,request_id)
);

CREATE TABLE encounter_command_receipts_new (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    family TEXT NOT NULL CHECK(family IN ('turn','event','pc_addition')),
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    boundary_key TEXT,
    receipt_json TEXT,
    PRIMARY KEY(encounter_id,family,request_id),
    CHECK((family='turn' AND boundary_key IS NOT NULL AND receipt_json IS NULL)
       OR (family IN ('event','pc_addition') AND boundary_key IS NULL AND receipt_json IS NOT NULL))
);
INSERT INTO encounter_command_receipts_new SELECT * FROM encounter_command_receipts;
DROP TABLE encounter_command_receipts;
ALTER TABLE encounter_command_receipts_new RENAME TO encounter_command_receipts;
CREATE UNIQUE INDEX encounter_command_turn_boundary
    ON encounter_command_receipts(boundary_key) WHERE family='turn';
