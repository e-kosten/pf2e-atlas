CREATE TABLE party_member_references (
    member_key TEXT PRIMARY KEY NOT NULL REFERENCES party_members(member_key) ON DELETE CASCADE,
    payload TEXT NOT NULL CHECK(json_valid(payload))
);

CREATE TABLE encounter_pc_party_members (
    participant_id INTEGER PRIMARY KEY REFERENCES encounter_participants(id) ON DELETE CASCADE,
    member_key TEXT NOT NULL REFERENCES party_members(member_key) ON DELETE CASCADE
);

CREATE INDEX encounter_pc_party_members_member ON encounter_pc_party_members(member_key);
