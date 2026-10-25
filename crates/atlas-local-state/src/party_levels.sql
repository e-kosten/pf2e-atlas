ALTER TABLE party_members ADD COLUMN level INTEGER
    CHECK(level IS NULL OR (typeof(level)='integer' AND level BETWEEN 1 AND 20));
