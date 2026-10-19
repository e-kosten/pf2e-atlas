-- Preserve old rows until the versioned lexical backfill can publish atomically.
DROP TRIGGER lexical_insert;
DROP TRIGGER lexical_delete;
DROP TRIGGER lexical_update;
DROP TABLE lexical_fts;
DROP INDEX lexical_units_root;
ALTER TABLE lexical_units RENAME TO lexical_units_previous;
CREATE TABLE lexical_units(
 input_hash TEXT NOT NULL UNIQUE,
 unit_id INTEGER PRIMARY KEY, record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT, section_json TEXT CHECK(section_json IS NULL OR json_valid(section_json)),
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('root_name','root_alias','owned_name','heading','definition_label')),
 identity_terms TEXT NOT NULL, alias_terms TEXT NOT NULL,
 structured_terms TEXT NOT NULL, definition_terms TEXT NOT NULL
) STRICT;
INSERT INTO lexical_units SELECT * FROM lexical_units_previous;
DROP TABLE lexical_units_previous;
CREATE INDEX lexical_units_root ON lexical_units(record_id,unit_id);
CREATE VIRTUAL TABLE lexical_fts USING fts5(
 identity_terms,alias_terms,structured_terms,definition_terms,
 content='lexical_units',content_rowid='unit_id',
 tokenize='unicode61 remove_diacritics 2'
);
INSERT INTO lexical_fts(lexical_fts) VALUES('rebuild');
CREATE TRIGGER lexical_insert AFTER INSERT ON lexical_units BEGIN
 INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms);
END;
CREATE TRIGGER lexical_delete AFTER DELETE ON lexical_units BEGIN
 INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms);
END;
CREATE TRIGGER lexical_update AFTER UPDATE ON lexical_units BEGIN
 INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms);
 INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms);
END;
