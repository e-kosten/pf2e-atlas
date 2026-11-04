CREATE TABLE selected_sections(
 section_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 selector TEXT NOT NULL UNIQUE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json) AND json_type(owners_json)='array'),
 field_path TEXT NOT NULL,
 section_ordinal INTEGER NOT NULL CHECK(section_ordinal>=0),
 body_hash TEXT NOT NULL CHECK(length(body_hash)=64),
 address_kind TEXT NOT NULL CHECK(address_kind IN ('html','plain')),
 prepared_html_sha256 TEXT CHECK(prepared_html_sha256 IS NULL OR length(prepared_html_sha256)=64),
 selection_version TEXT NOT NULL,
 section_label TEXT,
 byte_length INTEGER NOT NULL CHECK(byte_length>0),
 role TEXT NOT NULL,
 owner_label TEXT NOT NULL,
 recipe TEXT NOT NULL,
 UNIQUE(record_id,owners_json,field_path,section_ordinal),
 UNIQUE(section_id,record_id),
 CHECK((address_kind='html')=(prepared_html_sha256 IS NOT NULL))
) STRICT;
CREATE INDEX selected_sections_role ON selected_sections(role,record_id);
