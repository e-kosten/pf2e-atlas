CREATE TABLE saved_lists (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL REFERENCES principals(id) ON DELETE RESTRICT,
  revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0),
  list_key TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(owner_id, slug)
);
