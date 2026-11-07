DROP TABLE remaster_alias_support;
DROP TABLE remaster_terms;
DROP TABLE remaster_bindings;
DROP TABLE remaster_citations;
DROP TABLE remaster_declarations;
DROP TABLE remaster_source_inputs;
DROP TABLE remaster_aliases;
DROP TABLE remaster_metadata;
CREATE TABLE verified_aliases(
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE, alias TEXT NOT NULL,
 alias_lookup_key TEXT NOT NULL,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 PRIMARY KEY(record_id,alias)
) WITHOUT ROWID,STRICT;
CREATE TABLE remaster_pairs(
 legacy_record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 remaster_record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 CHECK(legacy_record_id<>remaster_record_id)
) STRICT;
CREATE TABLE record_remaster_context(
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 ordinal INTEGER NOT NULL,
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
 PRIMARY KEY(record_id,ordinal)
) WITHOUT ROWID,STRICT;
