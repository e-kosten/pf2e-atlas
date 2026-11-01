-- Convert main's unpublished recovery values directly to the final disposable cache.
-- The native backfill verifies every old value before removing its prior storage.
ALTER TABLE pending_embedding_values RENAME TO legacy_pending_embedding_values;
CREATE TABLE pending_embedding_values(
 input_id INTEGER PRIMARY KEY REFERENCES embedding_inputs ON DELETE CASCADE,
 vector BLOB NOT NULL,
 vector_hash BLOB NOT NULL CHECK(length(vector_hash)=32),
 cached_at INTEGER NOT NULL DEFAULT(unixepoch())
) STRICT;
CREATE INDEX pending_embedding_oldest ON pending_embedding_values(cached_at,input_id);
