CREATE TABLE verified_duplicate_records (
    duplicate_record_id INTEGER PRIMARY KEY REFERENCES records(record_id) ON DELETE CASCADE,
    canonical_record_id INTEGER NOT NULL REFERENCES records(record_id) ON DELETE CASCADE,
    duplicate_sha256 TEXT NOT NULL,
    canonical_sha256 TEXT NOT NULL,
    CHECK (duplicate_record_id <> canonical_record_id)
) STRICT;
CREATE INDEX verified_duplicate_records_canonical
    ON verified_duplicate_records(canonical_record_id);
