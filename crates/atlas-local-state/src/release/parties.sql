CREATE TABLE parties (
    party_key TEXT PRIMARY KEY,
    owner_id INTEGER NOT NULL REFERENCES principals(id),
    name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 255),
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0)
);
