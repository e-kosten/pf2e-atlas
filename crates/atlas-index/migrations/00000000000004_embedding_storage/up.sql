-- One final schema replaces the redundant checkpoint corpus and wide occurrence keys.
ALTER TABLE semantic_units RENAME TO legacy_semantic_units;
DROP INDEX semantic_units_root;
DROP INDEX semantic_units_input_hash;
DROP INDEX semantic_identity_root;
DROP INDEX semantic_reuse_input;
ALTER TABLE trait_vectors RENAME TO legacy_trait_vectors;
CREATE TABLE embedding_models(
 model_id INTEGER PRIMARY KEY,
 identity_json TEXT NOT NULL UNIQUE CHECK(json_valid(identity_json))
) STRICT;
CREATE TABLE embedding_inputs(
 input_id INTEGER PRIMARY KEY,
 model_id INTEGER NOT NULL REFERENCES embedding_models,
 input_hash BLOB NOT NULL CHECK(length(input_hash)=32),
 token_count INTEGER NOT NULL CHECK(token_count>0),
 published_hash BLOB CHECK(length(published_hash)=32),
 UNIQUE(model_id,input_hash)
) STRICT;
CREATE TABLE pending_embedding_values(
 input_id INTEGER PRIMARY KEY REFERENCES embedding_inputs ON DELETE CASCADE,
 vector BLOB NOT NULL,
 vector_hash BLOB NOT NULL CHECK(length(vector_hash)=32),
 touched_job TEXT NOT NULL
) STRICT;
CREATE TABLE trait_native_values(
 input_id INTEGER PRIMARY KEY REFERENCES embedding_inputs ON DELETE CASCADE,
 vector BLOB NOT NULL
) STRICT;
CREATE TABLE semantic_locations(
 location_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 location_hash BLOB NOT NULL UNIQUE CHECK(length(location_hash)=32),
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT,
 section_json TEXT NOT NULL CHECK(json_valid(section_json)),
 chunk_ordinal INTEGER NOT NULL CHECK(chunk_ordinal>=0)
) STRICT;
CREATE INDEX semantic_locations_root ON semantic_locations(record_id,location_id);
CREATE TABLE semantic_units(
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 profile_id TEXT NOT NULL REFERENCES search_profiles,
 location_id INTEGER NOT NULL REFERENCES semantic_locations,
 input_id INTEGER NOT NULL REFERENCES embedding_inputs,
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('identity','passage')),
 UNIQUE(location_id,profile_id)
) STRICT;
CREATE INDEX semantic_units_root ON semantic_units(record_id,profile_id,unit_id);
CREATE INDEX semantic_units_input ON semantic_units(input_id,unit_id);
CREATE UNIQUE INDEX semantic_identity_root ON semantic_units(record_id,profile_id) WHERE unit_kind='identity';
CREATE TABLE trait_vectors(
 trait_key TEXT NOT NULL REFERENCES trait_definitions(key) ON DELETE CASCADE,
 profile_id TEXT NOT NULL REFERENCES search_profiles,
 input_hash TEXT NOT NULL,
 vector BLOB NOT NULL,
 vector_hash BLOB NOT NULL CHECK(length(vector_hash)=32),
 PRIMARY KEY(trait_key,profile_id)
) WITHOUT ROWID,STRICT;
CREATE TABLE trait_seed_chunks(
 trait_key TEXT NOT NULL,
 profile_id TEXT NOT NULL,
 ordinal INTEGER NOT NULL CHECK(ordinal>=0),
 input_id INTEGER NOT NULL REFERENCES embedding_inputs,
 input TEXT NOT NULL,
 address_json TEXT NOT NULL CHECK(json_valid(address_json)),
 PRIMARY KEY(trait_key,profile_id,ordinal),
 FOREIGN KEY(trait_key,profile_id) REFERENCES trait_vectors ON DELETE CASCADE
) WITHOUT ROWID,STRICT;
CREATE INDEX trait_seed_chunks_input ON trait_seed_chunks(input_id);
