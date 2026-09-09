CREATE TABLE canonical_journal_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_roll_table_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

ALTER TABLE record_content RENAME TO record_content_v8;
ALTER TABLE reference_occurrences RENAME TO reference_occurrences_v8;

CREATE TABLE record_content (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  identity_stability TEXT NOT NULL CHECK (identity_stability IN (
    'stable_source_identity', 'unstable_authored_ordinal'
  )),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN (
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence', 'child'
  )),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
  owner_hazard_entity_id TEXT,
  owner_hazard_occurrence_id TEXT,
  owner_hazard_occurrence_authored_order INTEGER,
  role TEXT NOT NULL CHECK (role IN (
    'primary_description', 'summary', 'supplemental_rules', 'embedded_capability',
    'journal_page', 'table_result', 'generated_narrative', 'provenance'
  )),
  origin_json TEXT NOT NULL,
  visibility TEXT NOT NULL CHECK (visibility IN ('public', 'gm_only', 'private', 'internal')),
  provenance_json TEXT NOT NULL,
  source_kind TEXT NOT NULL CHECK (source_kind IN ('description', 'blurb', 'disable', 'routine', 'reset', 'stealth_details', 'details_field_description', 'public_notes', 'gm_notes', 'private_notes', 'embedded_item_description', 'embedded_gm_description', 'embedded_spell_description', 'generated_affliction', 'journal_page', 'table_result')),
  contributes_to_search INTEGER NOT NULL CHECK (contributes_to_search IN (0, 1)),
  contributes_to_references INTEGER NOT NULL CHECK (contributes_to_references IN (0, 1)),
  label TEXT,
  content_json TEXT NOT NULL,
  content_hash TEXT NOT NULL,
  duplicate_status_json TEXT NOT NULL,
  diagnostics_json TEXT NOT NULL,
  PRIMARY KEY (record_key, content_key, authored_order),
  UNIQUE (record_key, authored_order),
  CHECK (
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'child' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id) REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order) REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_hazard_entity_id) REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_hazard_occurrence_id, owner_hazard_occurrence_authored_order) REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order)
);

INSERT INTO record_content SELECT * FROM record_content_v8;

CREATE TABLE reference_occurrences (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  content_authored_order INTEGER NOT NULL CHECK (content_authored_order >= 0),
  occurrence_ordinal INTEGER NOT NULL CHECK (occurrence_ordinal >= 0),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN (
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence', 'child'
  )),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
  owner_hazard_entity_id TEXT,
  owner_hazard_occurrence_id TEXT,
  owner_hazard_occurrence_authored_order INTEGER,
  role TEXT NOT NULL CHECK (role IN (
    'primary_description', 'summary', 'supplemental_rules', 'embedded_capability',
    'journal_page', 'table_result', 'generated_narrative', 'provenance'
  )),
  origin_json TEXT NOT NULL,
  visibility TEXT NOT NULL CHECK (visibility IN ('public', 'gm_only', 'private', 'internal')),
  provenance_json TEXT NOT NULL,
  target_kind TEXT NOT NULL CHECK (target_kind IN ('record', 'record_child', 'local_content', 'external', 'unresolved')),
  target_record_key TEXT,
  target_json TEXT NOT NULL,
  label TEXT,
  relation_kind TEXT NOT NULL CHECK (relation_kind IN ('reference', 'embed')),
  PRIMARY KEY (record_key, content_key, content_authored_order, occurrence_ordinal),
  CHECK (
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'child' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL)
  ),
  CHECK (
    (target_kind IN ('record', 'record_child') AND target_record_key IS NOT NULL)
    OR (target_kind NOT IN ('record', 'record_child') AND target_record_key IS NULL)
  ),
  FOREIGN KEY (record_key, content_key, content_authored_order) REFERENCES record_content(record_key, content_key, authored_order) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id) REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order) REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_hazard_entity_id) REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_hazard_occurrence_id, owner_hazard_occurrence_authored_order) REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (target_record_key) REFERENCES records(record_key)
);

INSERT INTO reference_occurrences SELECT * FROM reference_occurrences_v8;
DROP TABLE reference_occurrences_v8;
DROP TABLE record_content_v8;

ALTER TABLE reference_edges RENAME TO reference_edges_v8;

CREATE TABLE reference_edges (
  from_record_key TEXT NOT NULL,
  to_record_key TEXT NOT NULL,
  display_text TEXT,
  reference_text TEXT NOT NULL,
  relation_kind TEXT NOT NULL DEFAULT 'reference' CHECK (relation_kind IN ('reference', 'embed')),
  source_kind TEXT NOT NULL CHECK (source_kind IN ('description', 'blurb', 'disable', 'routine', 'reset', 'stealth_details', 'details_field_description', 'public_notes', 'gm_notes', 'private_notes', 'embedded_item_description', 'embedded_spell_description', 'generated_affliction', 'journal_page', 'table_result')),
  visibility TEXT NOT NULL CHECK (visibility IN ('public', 'gm_only', 'private', 'internal')),
  source_child_locator TEXT NOT NULL DEFAULT '',
  target_child_locator TEXT NOT NULL DEFAULT '',
  PRIMARY KEY (
    from_record_key, to_record_key, reference_text, relation_kind, source_kind,
    source_child_locator, target_child_locator
  ),
  FOREIGN KEY (from_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (to_record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

INSERT INTO reference_edges (
  from_record_key, to_record_key, display_text, reference_text, relation_kind, source_kind,
  visibility, source_child_locator, target_child_locator
)
SELECT
  from_record_key, to_record_key, display_text, reference_text, relation_kind, source_kind,
  visibility, '', ''
FROM reference_edges_v8;

DROP TABLE reference_edges_v8;
