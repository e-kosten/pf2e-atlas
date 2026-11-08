ALTER TABLE artifact_context ADD COLUMN query_catalog_sha256 BLOB
  CHECK(length(query_catalog_sha256)=32);

CREATE TABLE search_profiles_with_model(
 profile_id TEXT PRIMARY KEY,
 model_id INTEGER NOT NULL UNIQUE REFERENCES embedding_models
) STRICT;
INSERT INTO search_profiles_with_model
 SELECT p.profile_id,m.model_id
 FROM search_profiles p JOIN embedding_models m ON m.identity_json=p.identity_json;
DROP TABLE search_profiles;
ALTER TABLE search_profiles_with_model RENAME TO search_profiles;

DROP INDEX lexical_units_root;
CREATE UNIQUE INDEX lexical_units_root ON lexical_units(record_id,unit_id);
DROP INDEX semantic_locations_root;
CREATE UNIQUE INDEX semantic_locations_root ON semantic_locations(record_id,location_id);

CREATE TABLE core_lexical_units_owned(
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(record_id,unit_id) REFERENCES lexical_units(record_id,unit_id) ON DELETE CASCADE
) STRICT;
INSERT INTO core_lexical_units_owned SELECT unit_id,record_id FROM core_lexical_units;
DROP TABLE core_lexical_units;
ALTER TABLE core_lexical_units_owned RENAME TO core_lexical_units;
CREATE INDEX core_lexical_units_root ON core_lexical_units(record_id,unit_id);

CREATE TABLE core_semantic_locations_owned(
 location_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(record_id,location_id) REFERENCES semantic_locations(record_id,location_id) ON DELETE CASCADE
) STRICT;
INSERT INTO core_semantic_locations_owned SELECT location_id,record_id FROM core_semantic_locations;
DROP TABLE core_semantic_locations;
ALTER TABLE core_semantic_locations_owned RENAME TO core_semantic_locations;
CREATE INDEX core_semantic_locations_root ON core_semantic_locations(record_id,location_id);

CREATE TABLE semantic_units_owned(
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 profile_id TEXT NOT NULL REFERENCES search_profiles,
 location_id INTEGER NOT NULL,
 input_id INTEGER NOT NULL REFERENCES embedding_inputs,
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('identity','passage')),
 UNIQUE(location_id,profile_id),
 FOREIGN KEY(record_id,location_id) REFERENCES semantic_locations(record_id,location_id)
) STRICT;
INSERT INTO semantic_units_owned
 SELECT unit_id,record_id,profile_id,location_id,input_id,unit_kind FROM semantic_units;
DROP TABLE semantic_units;
ALTER TABLE semantic_units_owned RENAME TO semantic_units;
CREATE INDEX semantic_units_root ON semantic_units(record_id,profile_id,unit_id);
CREATE INDEX semantic_units_input ON semantic_units(input_id,unit_id);
CREATE UNIQUE INDEX semantic_identity_root ON semantic_units(record_id,profile_id) WHERE unit_kind='identity';
