CREATE TABLE encounters (
  id INTEGER PRIMARY KEY,
  owner_id INTEGER NOT NULL REFERENCES principals(id) ON DELETE RESTRICT,
  encounter_key TEXT NOT NULL UNIQUE,
  slug TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT,
  note TEXT,
  status TEXT NOT NULL CHECK(status IN ('draft','running','complete','archived')),
  round_number INTEGER NOT NULL DEFAULT 1,
  current_turn_participant_key TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revision INTEGER NOT NULL DEFAULT 0 CHECK(typeof(revision) = 'integer' AND revision >= 0),
  difficulty_party_level INTEGER
    CHECK(difficulty_party_level IS NULL OR (typeof(difficulty_party_level) = 'integer' AND difficulty_party_level BETWEEN 1 AND 20)),
  difficulty_party_size INTEGER
    CHECK(difficulty_party_size IS NULL OR (typeof(difficulty_party_size) = 'integer' AND difficulty_party_size BETWEEN 1 AND 100)),
  round_duration_seconds INTEGER NOT NULL DEFAULT 6 CHECK(round_duration_seconds BETWEEN 1 AND 86400),
  UNIQUE(owner_id, slug),
  FOREIGN KEY(id,current_turn_participant_key) REFERENCES encounter_participants(encounter_id,participant_key) DEFERRABLE INITIALLY DEFERRED
);
