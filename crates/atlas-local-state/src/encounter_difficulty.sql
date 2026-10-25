ALTER TABLE encounters ADD COLUMN difficulty_party_level INTEGER
    CHECK(difficulty_party_level IS NULL OR (typeof(difficulty_party_level) = 'integer' AND difficulty_party_level BETWEEN 1 AND 20));
ALTER TABLE encounters ADD COLUMN difficulty_party_size INTEGER
    CHECK(difficulty_party_size IS NULL OR (typeof(difficulty_party_size) = 'integer' AND difficulty_party_size BETWEEN 1 AND 100));
