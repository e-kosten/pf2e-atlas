CREATE TABLE canonical_consumable_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_consumable_entities (
  owner_record_key TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  target_record_key TEXT,
  canonical_json TEXT NOT NULL,
  PRIMARY KEY (owner_record_key, entity_id),
  FOREIGN KEY (owner_record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (target_record_key) REFERENCES canonical_consumable_records(record_key)
);

CREATE TABLE canonical_consumable_occurrences (
  owner_record_key TEXT NOT NULL,
  occurrence_id TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  canonical_json TEXT NOT NULL,
  PRIMARY KEY (owner_record_key, occurrence_id),
  UNIQUE (owner_record_key, authored_order),
  UNIQUE (owner_record_key, occurrence_id, authored_order),
  FOREIGN KEY (owner_record_key, entity_id)
    REFERENCES canonical_consumable_entities(owner_record_key, entity_id) ON DELETE CASCADE
);

CREATE TABLE consumable_query_records (
  record_key TEXT PRIMARY KEY,
  category TEXT,
  usage TEXT,
  base_item TEXT,
  bulk_value REAL,
  hands_requirement TEXT CHECK (hands_requirement IN ('one_hand', 'one_plus_hands', 'two_hands') OR hands_requirement IS NULL),
  price_cp INTEGER CHECK (price_cp IS NULL OR price_cp >= 0),
  damage_types_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES canonical_consumable_records(record_key) ON DELETE CASCADE
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
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence',
    'consumable_occurrence'
  )),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
  owner_hazard_entity_id TEXT,
  owner_hazard_occurrence_id TEXT,
  owner_hazard_occurrence_authored_order INTEGER,
  owner_consumable_occurrence_id TEXT,
  owner_consumable_occurrence_authored_order INTEGER,
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
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'consumable_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NOT NULL AND owner_consumable_occurrence_authored_order IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id)
    REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_hazard_entity_id)
    REFERENCES canonical_hazard_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_hazard_occurrence_id, owner_hazard_occurrence_authored_order)
    REFERENCES canonical_hazard_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, owner_consumable_occurrence_id, owner_consumable_occurrence_authored_order)
    REFERENCES canonical_consumable_occurrences(owner_record_key, occurrence_id, authored_order)
);

CREATE TABLE reference_occurrences (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  content_authored_order INTEGER NOT NULL CHECK (content_authored_order >= 0),
  occurrence_ordinal INTEGER NOT NULL CHECK (occurrence_ordinal >= 0),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN (
    'record', 'creature_entity', 'creature_occurrence', 'hazard_entity', 'hazard_occurrence',
    'consumable_occurrence'
  )),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
  owner_hazard_entity_id TEXT,
  owner_hazard_occurrence_id TEXT,
  owner_hazard_occurrence_authored_order INTEGER,
  owner_consumable_occurrence_id TEXT,
  owner_consumable_occurrence_authored_order INTEGER,
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
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_entity' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NOT NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'hazard_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NOT NULL AND owner_hazard_occurrence_authored_order IS NOT NULL AND owner_consumable_occurrence_id IS NULL AND owner_consumable_occurrence_authored_order IS NULL)
    OR (owner_kind = 'consumable_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL AND owner_hazard_entity_id IS NULL AND owner_hazard_occurrence_id IS NULL AND owner_hazard_occurrence_authored_order IS NULL AND owner_consumable_occurrence_id IS NOT NULL AND owner_consumable_occurrence_authored_order IS NOT NULL)
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
  FOREIGN KEY (record_key, owner_consumable_occurrence_id, owner_consumable_occurrence_authored_order)
    REFERENCES canonical_consumable_occurrences(owner_record_key, occurrence_id, authored_order),
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
