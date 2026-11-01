CREATE TABLE reference_component_digests (
 record_id INTEGER PRIMARY KEY REFERENCES records(record_id) ON DELETE CASCADE,
 recipe TEXT NOT NULL CHECK(recipe='closed-reference-components/v1'),
 digests_json TEXT NOT NULL CHECK(json_valid(digests_json) AND json_type(digests_json)='object'
 AND json_type(digests_json,'$.snapshot')='text' AND length(json_extract(digests_json,'$.snapshot'))=64
 AND json_type(digests_json,'$.query')='text' AND length(json_extract(digests_json,'$.query'))=64
 AND json_type(digests_json,'$.content')='text' AND length(json_extract(digests_json,'$.content'))=64
 AND json_type(digests_json,'$.relationships')='text' AND length(json_extract(digests_json,'$.relationships'))=64
 AND json_type(digests_json,'$.sections')='text' AND length(json_extract(digests_json,'$.sections'))=64
 AND json_type(digests_json,'$.lexical')='text' AND length(json_extract(digests_json,'$.lexical'))=64
 AND json_type(digests_json,'$.trait_associations')='text' AND length(json_extract(digests_json,'$.trait_associations'))=64
 AND json_type(digests_json,'$.core')='text' AND length(json_extract(digests_json,'$.core'))=64)
) STRICT;

-- Root-owned association reconciliation and deferred parent-FK checks use these
-- paths; trait-oriented lookup continues to use each existing primary key.
CREATE INDEX trait_memberships_record_key ON trait_memberships(record_key,trait_key,owners_json);
CREATE INDEX trait_references_record_key ON trait_references(record_key,trait_key,evidence);
