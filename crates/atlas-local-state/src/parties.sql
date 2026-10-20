CREATE TABLE parties (
    party_key TEXT PRIMARY KEY,
    owner_id INTEGER NOT NULL DEFAULT 1 REFERENCES principals(id),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 255),
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0)
);
CREATE TABLE party_members (
    member_key TEXT PRIMARY KEY,
    party_key TEXT NOT NULL REFERENCES parties(party_key) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK(position >= 0),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 255),
    UNIQUE(party_key, position)
);
