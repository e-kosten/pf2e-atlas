ALTER TABLE records ADD COLUMN record_role TEXT NOT NULL DEFAULT 'source'
  CHECK (record_role IN ('source', 'canonical', 'source_instance'));
ALTER TABLE records ADD COLUMN retrieval_disposition TEXT NOT NULL DEFAULT 'ordinary'
  CHECK (retrieval_disposition IN ('ordinary', 'direct_only', 'inspection_only'));
ALTER TABLE records ADD COLUMN retrieval_rationale TEXT NOT NULL DEFAULT 'source_record'
  CHECK (retrieval_rationale IN (
    'source_record',
    'generated_canonical',
    'duplicate_source_instance',
    'canonical_edition_duplicate',
    'tooling_no_addressable_product_meaning'
  ));

CREATE TABLE canonical_creature_records (
  record_key TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  name TEXT NOT NULL,
  family TEXT NOT NULL CHECK (family IN ('npc')),
  canonical_json TEXT NOT NULL,
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_creature_resources (
  record_key TEXT NOT NULL,
  resource_id TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  resource_kind TEXT NOT NULL,
  resource_json TEXT NOT NULL,
  PRIMARY KEY (record_key, resource_id),
  UNIQUE (record_key, authored_order),
  FOREIGN KEY (record_key) REFERENCES canonical_creature_records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_creature_entities (
  record_key TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  family TEXT NOT NULL CHECK (family IN (
    'strike', 'action', 'spellcasting-entry', 'spell', 'equipment', 'lore',
    'affliction', 'armor', 'backpack', 'book', 'condition', 'consumable',
    'effect', 'shield', 'treasure', 'weapon', 'unsupported'
  )),
  label TEXT NOT NULL,
  source_identity_json TEXT NOT NULL,
  PRIMARY KEY (record_key, entity_id),
  FOREIGN KEY (record_key) REFERENCES canonical_creature_records(record_key) ON DELETE CASCADE
);

CREATE TABLE canonical_creature_occurrences (
  record_key TEXT NOT NULL,
  occurrence_id TEXT NOT NULL,
  identity_stability TEXT NOT NULL CHECK (identity_stability IN (
    'stable_nested_source_id', 'unstable_owner_family_ordinal'
  )),
  family TEXT NOT NULL CHECK (family IN (
    'strike', 'action', 'spellcasting-entry', 'spell', 'equipment', 'lore',
    'affliction', 'armor', 'backpack', 'book', 'condition', 'consumable',
    'effect', 'shield', 'treasure', 'weapon', 'unsupported'
  )),
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  source_sort_json TEXT NOT NULL,
  source_folder_json TEXT NOT NULL,
  source_identity_json TEXT NOT NULL,
  parent_kind TEXT NOT NULL CHECK (parent_kind IN ('creature', 'spellcasting_entry')),
  parent_occurrence_id TEXT,
  parent_occurrence_authored_order INTEGER,
  target_kind TEXT NOT NULL CHECK (target_kind IN ('canonical_record', 'actor_owned')),
  target_record_key TEXT,
  target_entity_id TEXT,
  context_json TEXT NOT NULL,
  capability_json TEXT NOT NULL,
  deltas_json TEXT NOT NULL,
  PRIMARY KEY (record_key, occurrence_id, authored_order),
  UNIQUE (record_key, authored_order),
  CHECK (
    (parent_kind = 'creature' AND parent_occurrence_id IS NULL AND parent_occurrence_authored_order IS NULL)
    OR (parent_kind = 'spellcasting_entry' AND parent_occurrence_id IS NOT NULL AND parent_occurrence_authored_order IS NOT NULL)
  ),
  CHECK (
    (target_kind = 'canonical_record' AND target_record_key IS NOT NULL AND target_entity_id IS NULL)
    OR (target_kind = 'actor_owned' AND target_record_key IS NULL AND target_entity_id IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES canonical_creature_records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, parent_occurrence_id, parent_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order)
    DEFERRABLE INITIALLY DEFERRED,
  FOREIGN KEY (target_record_key) REFERENCES records(record_key),
  FOREIGN KEY (record_key, target_entity_id)
    REFERENCES canonical_creature_entities(record_key, entity_id)
);

CREATE TABLE canonical_creature_relationships (
  record_key TEXT NOT NULL,
  relationship_order INTEGER NOT NULL CHECK (relationship_order >= 0),
  source_occurrence_id TEXT NOT NULL,
  source_occurrence_authored_order INTEGER NOT NULL CHECK (source_occurrence_authored_order >= 0),
  relationship_kind TEXT NOT NULL CHECK (relationship_kind IN (
    'granted_by', 'item_grant', 'linked_weapon', 'prepared_spell'
  )),
  target_kind TEXT NOT NULL CHECK (target_kind IN ('occurrence', 'unresolved_nested_source_id')),
  target_occurrence_id TEXT,
  target_occurrence_authored_order INTEGER,
  target_source_id TEXT,
  source_path TEXT NOT NULL,
  contextual_label_json TEXT NOT NULL,
  lifecycle_json TEXT NOT NULL,
  execution TEXT NOT NULL CHECK (execution IN ('provenance_only')),
  PRIMARY KEY (record_key, relationship_order),
  CHECK (
    (target_kind = 'occurrence' AND target_occurrence_id IS NOT NULL AND target_occurrence_authored_order IS NOT NULL AND target_source_id IS NULL)
    OR (target_kind = 'unresolved_nested_source_id' AND target_occurrence_id IS NULL AND target_occurrence_authored_order IS NULL AND target_source_id IS NOT NULL)
  ),
  FOREIGN KEY (record_key, source_occurrence_id, source_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order),
  FOREIGN KEY (record_key, target_occurrence_id, target_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order)
);

DROP TABLE record_content;
CREATE TABLE record_content (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  authored_order INTEGER NOT NULL CHECK (authored_order >= 0),
  identity_stability TEXT NOT NULL CHECK (identity_stability IN (
    'stable_source_identity', 'unstable_authored_ordinal'
  )),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN ('record', 'creature_entity', 'creature_occurrence')),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
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
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL)
  ),
  FOREIGN KEY (record_key) REFERENCES records(record_key) ON DELETE CASCADE,
  FOREIGN KEY (record_key, owner_entity_id)
    REFERENCES canonical_creature_entities(record_key, entity_id),
  FOREIGN KEY (record_key, owner_occurrence_id, owner_occurrence_authored_order)
    REFERENCES canonical_creature_occurrences(record_key, occurrence_id, authored_order)
);

DROP TABLE reference_occurrences;
CREATE TABLE reference_occurrences (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  content_authored_order INTEGER NOT NULL CHECK (content_authored_order >= 0),
  occurrence_ordinal INTEGER NOT NULL CHECK (occurrence_ordinal >= 0),
  owner_kind TEXT NOT NULL CHECK (owner_kind IN ('record', 'creature_entity', 'creature_occurrence')),
  owner_record_key TEXT,
  owner_entity_id TEXT,
  owner_occurrence_id TEXT,
  owner_occurrence_authored_order INTEGER,
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
    (owner_kind = 'record' AND owner_record_key = record_key AND owner_entity_id IS NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_entity' AND owner_record_key IS NULL AND owner_entity_id IS NOT NULL AND owner_occurrence_id IS NULL AND owner_occurrence_authored_order IS NULL)
    OR (owner_kind = 'creature_occurrence' AND owner_record_key IS NULL AND owner_entity_id IS NULL AND owner_occurrence_id IS NOT NULL AND owner_occurrence_authored_order IS NOT NULL)
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
  FOREIGN KEY (target_record_key) REFERENCES records(record_key)
);

CREATE TABLE record_content_exclusions (
  record_key TEXT NOT NULL,
  content_key TEXT NOT NULL,
  relative_source_path TEXT NOT NULL,
  label TEXT,
  reason TEXT NOT NULL CHECK (reason IN ('deferred_entity_family', 'missing_typed_owner')),
  PRIMARY KEY (record_key, content_key),
  FOREIGN KEY (record_key) REFERENCES canonical_creature_records(record_key) ON DELETE CASCADE
);
