CREATE TABLE canonical_lexical_units(
 input_hash TEXT NOT NULL UNIQUE,
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 section_id INTEGER,
 owners_json TEXT CHECK(owners_json IS NULL OR (json_valid(owners_json) AND json_type(owners_json)='array')),
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('root_name','root_alias','owned_name','heading','definition_label')),
 identity_terms TEXT NOT NULL,
 alias_terms TEXT NOT NULL,
 structured_terms TEXT NOT NULL,
 definition_terms TEXT NOT NULL,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) DEFERRABLE INITIALLY DEFERRED,
 CHECK((unit_kind IN ('heading','definition_label') AND section_id IS NOT NULL AND owners_json IS NULL) OR (unit_kind IN ('root_name','root_alias','owned_name') AND section_id IS NULL AND owners_json IS NOT NULL))
) STRICT;
CREATE TABLE canonical_semantic_locations(
 location_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 section_id INTEGER,
 chunk_ordinal INTEGER CHECK(chunk_ordinal IS NULL OR chunk_ordinal>=0),
 chunk_start INTEGER,
 chunk_end INTEGER,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) DEFERRABLE INITIALLY DEFERRED,
 UNIQUE(section_id,chunk_ordinal,chunk_start,chunk_end),
 CHECK((section_id IS NOT NULL AND chunk_ordinal IS NOT NULL AND chunk_start IS NOT NULL AND chunk_end IS NOT NULL AND chunk_start>=0 AND chunk_end>chunk_start) OR (section_id IS NULL AND chunk_start IS NULL AND chunk_end IS NULL AND (chunk_ordinal IS NULL OR chunk_ordinal=0)))
) STRICT;
CREATE TABLE canonical_core_parts(
 section_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) ON DELETE CASCADE
) STRICT;
