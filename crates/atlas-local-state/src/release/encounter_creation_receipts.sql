CREATE TABLE encounter_creation_receipts (
    owner_id INTEGER NOT NULL REFERENCES principals(id) ON DELETE CASCADE,
    request_id TEXT NOT NULL CHECK(length(CAST(request_id AS BLOB)) BETWEEN 1 AND 128),
    intent_digest TEXT NOT NULL CHECK(length(intent_digest) = 64 AND intent_digest NOT GLOB '*[^0-9a-f]*'),
    encounter_key TEXT NOT NULL UNIQUE,
    PRIMARY KEY(owner_id, request_id)
);
