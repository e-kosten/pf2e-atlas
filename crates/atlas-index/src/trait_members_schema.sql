CREATE TABLE trait_members(
    value TEXT NOT NULL,
    record_key TEXT NOT NULL REFERENCES records(key) ON DELETE CASCADE,
    owners_json TEXT NOT NULL CHECK(json_valid(owners_json) AND json_type(owners_json)='array'),
    lookup_member INTEGER NOT NULL CHECK(lookup_member IN (0,1)),
    source_label TEXT,
    CHECK(lookup_member=1 OR source_label IS NULL),
    PRIMARY KEY(value,record_key,owners_json)
) WITHOUT ROWID,STRICT;
CREATE INDEX trait_members_owner ON trait_members(record_key,owners_json,value);
CREATE INDEX trait_members_root_value ON trait_members(value,record_key) WHERE owners_json='[]';
