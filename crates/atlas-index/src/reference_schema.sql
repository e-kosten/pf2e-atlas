ALTER TABLE trait_definitions RENAME TO reference_entries;
ALTER TABLE trait_references RENAME TO legacy_trait_references;
CREATE TABLE definition_reference_occurrences(
    occurrence_id TEXT PRIMARY KEY,
    source_record_key TEXT REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,
    source_definition_key TEXT REFERENCES reference_entries(key) ON DELETE CASCADE,
    target_record_key TEXT REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,
    target_definition_key TEXT REFERENCES reference_entries(key) ON DELETE CASCADE,
    visible INTEGER NOT NULL CHECK(visible IN (0,1)),
    evidence TEXT NOT NULL CHECK(json_valid(evidence)),
    CHECK((source_record_key IS NULL) != (source_definition_key IS NULL)),
    CHECK((target_record_key IS NULL) != (target_definition_key IS NULL))
) STRICT;
CREATE INDEX definition_references_source_record ON definition_reference_occurrences(source_record_key,target_definition_key);
CREATE INDEX definition_references_source_definition ON definition_reference_occurrences(source_definition_key,target_record_key);
CREATE INDEX definition_references_record_target ON definition_reference_occurrences(target_record_key,source_record_key);
CREATE INDEX definition_references_definition_target ON definition_reference_occurrences(target_definition_key,source_record_key);
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
ALTER TABLE actor_senses ADD COLUMN source_state TEXT NOT NULL DEFAULT 'missing' CHECK(source_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_senses ADD COLUMN source TEXT CHECK((source_state='value' AND source IS NOT NULL) OR (source_state<>'value' AND source IS NULL));
ALTER TABLE armor_projection ADD COLUMN armor_group_state TEXT NOT NULL DEFAULT 'missing' CHECK(armor_group_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE armor_projection ADD COLUMN armor_group TEXT CHECK((armor_group_state='value' AND armor_group IS NOT NULL) OR (armor_group_state<>'value' AND armor_group IS NULL));
