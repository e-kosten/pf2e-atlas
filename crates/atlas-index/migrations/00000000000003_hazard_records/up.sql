CREATE TABLE canonical_hazard_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  family TEXT NOT NULL CHECK (family = 'hazard'),
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_hazard_entities (
  record_key TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  family TEXT NOT NULL CHECK (family IN (
    'action', 'strike', 'condition', 'effect', 'unsupported_child'
  )),
  label TEXT NOT NULL,
  image_json TEXT NOT NULL,
  source_identity_json TEXT NOT NULL,
  capability_json TEXT NOT NULL,
  PRIMARY KEY (record_key, entity_id),
  FOREIGN KEY (record_key) REFERENCES canonical_hazard_records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_hazard_occurrences (
  record_key TEXT NOT NULL,
  occurrence_id TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  identity_stability TEXT NOT NULL CHECK (identity_stability IN (
    'stable_source_identity', 'unstable_authored_ordinal'
  )),
  family TEXT NOT NULL CHECK (family IN (
    'action', 'strike', 'condition', 'effect', 'unsupported_child'
  )),
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  source_sort_json TEXT NOT NULL,
  source_folder_json TEXT NOT NULL,
  source_ordinal INTEGER NOT NULL CHECK (source_ordinal >= 0),
  contextual_label_json TEXT NOT NULL,
  PRIMARY KEY (record_key, occurrence_id, authored_order),
  UNIQUE (record_key, authored_order),
  FOREIGN KEY (record_key) REFERENCES canonical_hazard_records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, entity_id)
    REFERENCES canonical_hazard_entities(record_key, entity_id)
);

CREATE TABLE canonical_hazard_relationships (
  record_key TEXT NOT NULL,
  relationship_id TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  source_occurrence_id TEXT,
  source_occurrence_authored_order INTEGER,
  relationship_kind TEXT NOT NULL CHECK (relationship_kind = 'contains'),
  target_kind TEXT NOT NULL CHECK (target_kind IN ('entity', 'occurrence')),
  target_entity_id TEXT,
  target_occurrence_id TEXT,
  target_occurrence_authored_order INTEGER,
  PRIMARY KEY (record_key, relationship_id),
  UNIQUE (record_key, authored_order),
  CHECK (
    (source_occurrence_id IS NULL AND source_occurrence_authored_order IS NULL)
    OR (source_occurrence_id IS NOT NULL AND source_occurrence_authored_order IS NOT NULL)
  ),
  CHECK (
    (target_kind = 'entity' AND target_entity_id IS NOT NULL AND target_occurrence_id IS NULL AND target_occurrence_authored_order IS NULL)
    OR (target_kind = 'occurrence' AND target_entity_id IS NULL AND target_occurrence_id IS NOT NULL AND target_occurrence_authored_order IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES canonical_hazard_records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, source_occurrence_id, source_occurrence_authored_order)
    REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, target_entity_id)
    REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, target_occurrence_id, target_occurrence_authored_order)
    REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order)
);

DROP TABLE reference_occurrences;
DROP TABLE record_content;
DROP TABLE record_content_exclusions;

CREATE TABLE record_content (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  identity_stability TEXT NOT NULL CHECK (identity_stability IN (
    'stable_source_identity', 'unstable_authored_ordinal'
  )),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN (
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence'
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
  source_kind TEXT NOT NULL CHECK (source_kind IN ('description', 'blurb', 'disable', 'routine', 'reset', 'stealth_details', 'details_field_description', 'public_notes', 'gm_notes', 'private_notes', 'embedded_item_description', 'embedded_gm_description', 'embedded_spell_description', 'generated_affliction')),
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
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id)
    REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_hazard_entity_id)
    REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_hazard_occurrence_id, owner_hazard_occurrence_authored_order)
    REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order)
);

CREATE TABLE reference_occurrences (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  content_authored_order INTEGER NOT NULL CHECK (content_authored_order >= 0),
  occurrence_ordinal INTEGER NOT NULL CHECK (occurrence_ordinal >= 0),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN (
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence'
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
  target_kind TEXT NOT NULL CHECK (target_kind IN ('record', 'local_content', 'external', 'unresolved')),
  target_record_key TEXT,
  target_json TEXT NOT NULL,
  label TEXT,
  relation_kind TEXT NOT NULL CHECK (relation_kind IN ('reference', 'embed')),
  PRIMARY KEY (record_key, content_key, content_authored_order, occurrence_ordinal),
  CHECK (
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL)
  ),
  CHECK (
    (target_kind = 'record' AND target_record_key IS NOT NULL)
    OR (target_kind <> 'record' AND target_record_key IS NULL)
  ),
  FOREIGN KEY (record_key, content_key, content_authored_order)
    REFERENCES record_content(record_key, content_key, authored_order) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id)
    REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_hazard_entity_id)
    REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_hazard_occurrence_id, owner_hazard_occurrence_authored_order)
    REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (target_record_key) REFERENCES records(record_key)
);

CREATE TABLE record_content_exclusions (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  relative_source_path TEXT NOT NULL,
  label TEXT,
  reason TEXT NOT NULL CHECK (reason IN ('deferred_entity_family', 'missing_typed_owner')),
  PRIMARY KEY (record_key, content_key),
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_spell_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_consumable_spell_children (
  parent_record_key TEXT NOT NULL,
  child_id TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  standalone_target_record_key TEXT,
  canonical_json TEXT NOT NULL,
  PRIMARY KEY (parent_record_key, child_id),
  UNIQUE (parent_record_key, authored_order),
  FOREIGN KEY (parent_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (standalone_target_record_key) REFERENCES canonical_spell_records(record_key)
);

CREATE TABLE spell_traditions (
  record_key TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  tradition TEXT NOT NULL,
  PRIMARY KEY (record_key, authored_order),
  FOREIGN KEY (record_key) REFERENCES canonical_spell_records(record_key) ON DELETE CASCADE
);

CREATE TABLE spell_damage_types (
  record_key TEXT NOT NULL,
  damage_key TEXT NOT NULL,
  damage_authored_order INTEGER NOT NULL CHECK (damage_authored_order >= 0),
  type_authored_order INTEGER NOT NULL CHECK (type_authored_order >= 0),
  damage_type TEXT NOT NULL,
  PRIMARY KEY (record_key, damage_key, damage_authored_order, type_authored_order),
  FOREIGN KEY (record_key) REFERENCES canonical_spell_records(record_key) ON DELETE CASCADE
);

ALTER TABLE spell_records ADD COLUMN rank INTEGER CHECK (rank BETWEEN 0 AND 10);
ALTER TABLE spell_records ADD COLUMN range_kind TEXT
  CHECK (range_kind IN ('touch_melee', 'distance'));
ALTER TABLE spell_records ADD COLUMN range_rule TEXT;
