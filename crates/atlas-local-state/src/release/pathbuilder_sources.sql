ALTER TABLE characters ADD COLUMN pathbuilder_source TEXT
    CHECK(pathbuilder_source IS NULL OR
          (reference_payload IS NOT NULL AND json_valid(pathbuilder_source)));
ALTER TABLE character_creation_receipts ADD COLUMN intent_version INTEGER NOT NULL DEFAULT 1
    CHECK(intent_version IN (1, 2));
