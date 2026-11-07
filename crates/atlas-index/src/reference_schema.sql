ALTER TABLE trait_definitions RENAME TO reference_entries;
CREATE TABLE owned_ability_evidence(
    record_key TEXT NOT NULL REFERENCES records(key) ON DELETE CASCADE,
    owners_json TEXT NOT NULL CHECK(json_type(owners_json)='array'),
    help_json TEXT CHECK(help_json IS NULL OR json_type(help_json)='object'),
    has_declaration INTEGER NOT NULL CHECK(has_declaration IN (0,1)),
    action_keys_json TEXT NOT NULL CHECK(json_type(action_keys_json)='array'),
    PRIMARY KEY(record_key,owners_json)
) WITHOUT ROWID, STRICT;
CREATE TABLE reference_notices(
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL,
    text TEXT NOT NULL,
    source_url TEXT
) STRICT;
ALTER TABLE actor_senses ADD COLUMN acuity_state TEXT NOT NULL DEFAULT 'missing' CHECK(acuity_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_senses ADD COLUMN acuity TEXT CHECK((acuity_state='value' AND acuity IS NOT NULL) OR (acuity_state<>'value' AND acuity IS NULL));
ALTER TABLE actor_senses ADD COLUMN range_state TEXT NOT NULL DEFAULT 'missing' CHECK(range_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_senses ADD COLUMN range ANY CHECK((range_state='value' AND range IS NOT NULL AND typeof(range) IN ('integer','real')) OR (range_state<>'value' AND range IS NULL));
ALTER TABLE armor_projection ADD COLUMN armor_group_state TEXT NOT NULL DEFAULT 'missing' CHECK(armor_group_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE armor_projection ADD COLUMN armor_group TEXT CHECK((armor_group_state='value' AND armor_group IS NOT NULL) OR (armor_group_state<>'value' AND armor_group IS NULL));
