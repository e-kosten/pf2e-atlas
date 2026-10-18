CREATE TABLE principals (
  id INTEGER PRIMARY KEY,
  principal_key TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
);
INSERT INTO principals VALUES (1, 'local', 'Local owner');

CREATE TABLE saved_lists (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL DEFAULT 1 REFERENCES principals(id) ON DELETE RESTRICT,
  revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0),
  list_key TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(owner_id, slug)
);

CREATE TABLE saved_list_items (
  list_id INTEGER NOT NULL,
  record_key TEXT NOT NULL REFERENCES record_identity(key) ON DELETE RESTRICT,
  position INTEGER NOT NULL,
  note TEXT,
  record_title_snapshot TEXT NOT NULL,
  record_kind_snapshot TEXT,
  added_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (list_id, record_key),
  UNIQUE (list_id, position),
  FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
  );

CREATE INDEX saved_list_items_position_idx
  ON saved_list_items(list_id, position);

CREATE TABLE saved_list_tags (
  list_id INTEGER NOT NULL,
  tag TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (list_id, tag),
  FOREIGN KEY (list_id) REFERENCES saved_lists(id) ON DELETE CASCADE
  );

CREATE INDEX saved_list_tags_tag_idx
  ON saved_list_tags(tag, list_id);

CREATE TABLE encounters (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL DEFAULT 1 REFERENCES principals(id) ON DELETE RESTRICT,
  encounter_key TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  note TEXT,
  status TEXT NOT NULL,
  round_number INTEGER NOT NULL DEFAULT 1,
  current_turn_participant_key TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL DEFAULT 0 CHECK(typeof(revision) = 'integer' AND revision >= 0),
  UNIQUE(owner_id, slug)
);

CREATE TABLE encounter_participants (
  id INTEGER PRIMARY KEY,
  encounter_id INTEGER NOT NULL,
  participant_key TEXT NOT NULL UNIQUE,
  record_key TEXT REFERENCES record_identity(key) ON DELETE RESTRICT,
  participant_kind TEXT NOT NULL,
  participant_variant TEXT NOT NULL DEFAULT 'normal',
  position INTEGER NOT NULL,
  display_name TEXT NOT NULL,
  record_title_snapshot TEXT,
  record_kind_snapshot TEXT,
  side TEXT NOT NULL DEFAULT 'enemy',
  initiative INTEGER,
  initiative_order INTEGER NOT NULL,
  max_hp INTEGER,
  current_hp INTEGER,
  temporary_hp INTEGER NOT NULL DEFAULT 0,
  defeated INTEGER NOT NULL DEFAULT 0,
  hidden INTEGER NOT NULL DEFAULT 0,
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  hp_origin TEXT NOT NULL DEFAULT 'explicit' CHECK(hp_origin IN ('derived_pristine','derived_edited','explicit','unknown')),
  variant_origin TEXT NOT NULL DEFAULT 'explicit' CHECK(variant_origin IN ('default_unadjusted','inherited_known','inherited_unknown','explicit')),
  revision INTEGER NOT NULL DEFAULT 0 CHECK(typeof(revision) = 'integer' AND revision >= 0),
  UNIQUE (encounter_id, position),
  UNIQUE (encounter_id, initiative, initiative_order),
  FOREIGN KEY (encounter_id) REFERENCES encounters(id) ON DELETE CASCADE
  );

CREATE INDEX encounter_participants_order_idx
  ON encounter_participants(encounter_id, initiative, initiative_order);

CREATE TABLE encounter_participant_adjustments (
  participant_id INTEGER NOT NULL,
  adjustment_key TEXT NOT NULL,
  kind TEXT NOT NULL,
  value TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (participant_id, adjustment_key),
  FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE
  );

CREATE TABLE encounter_participant_conditions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  participant_id INTEGER NOT NULL,
  condition_key TEXT,
  name TEXT NOT NULL,
  value INTEGER,
  source_participant_key TEXT,
  duration_amount INTEGER,
  duration_unit TEXT CHECK(duration_unit IN ('rounds','minutes','hours','days')),
  duration_anchor_key TEXT,
  duration_event TEXT CHECK(duration_event IN ('start','end')),
  note TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (participant_id) REFERENCES encounter_participants(id) ON DELETE CASCADE,
  CHECK((duration_amount IS NULL) = (duration_unit IS NULL)),
  CHECK((duration_anchor_key IS NULL) = (duration_event IS NULL)),
  CHECK(duration_unit = 'rounds' OR duration_anchor_key IS NULL)
  );

CREATE TABLE encounter_spell_resources (
  participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
  resource_key TEXT NOT NULL,
  capacity INTEGER NOT NULL CHECK(typeof(capacity)='integer' AND capacity>=0),
  remaining INTEGER NOT NULL CHECK(typeof(remaining)='integer' AND remaining BETWEEN 0 AND capacity),
  undo_remaining INTEGER CHECK(undo_remaining IS NULL OR (typeof(undo_remaining)='integer' AND undo_remaining BETWEEN 0 AND capacity)),
  PRIMARY KEY(participant_id, resource_key)
  );

CREATE TABLE encounter_expended_spells (
  participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
  spell_id TEXT NOT NULL,
  PRIMARY KEY(participant_id,spell_id)
  );

CREATE TABLE encounter_roll_receipts (
  encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
  request_id TEXT NOT NULL,
  request_json TEXT NOT NULL,
  receipt_json TEXT NOT NULL,
  PRIMARY KEY(encounter_id,request_id)
  );
