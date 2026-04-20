import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import {
  countDerivedTagMigrationRecords,
  loadDerivedTagMigrationRecords,
} from "../../src/tags/editorial/sessions/record-loader.js";

function createMigrationDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      pack_name TEXT,
      name TEXT NOT NULL,
      record_type TEXT,
      category TEXT NOT NULL,
      subcategory TEXT,
      document_type TEXT,
      level INTEGER,
      rarity TEXT,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT NOT NULL,
      families_json TEXT,
      description_text TEXT,
      blurb_text TEXT,
      source_category TEXT NOT NULL,
      publication_title TEXT,
      publication_remaster INTEGER NOT NULL,
      is_unique INTEGER NOT NULL,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE reference_edges (
      from_record_key TEXT NOT NULL,
      to_record_key TEXT NOT NULL
    );
  `);
  return db;
}

function insertMigrationRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    category: string;
    subcategory?: string | null;
    traitsJson?: string;
    sourceCategory?: string;
  },
): void {
  db.prepare(
    `
      INSERT INTO records (
        record_key, pack_name, name, record_type, category, subcategory, document_type, level, rarity,
        traits_json, derived_tags_json, families_json, description_text, blurb_text, source_category,
        publication_title, publication_remaster, is_unique, is_search_canonical
      )
      VALUES (?, 'packs', ?, 'spell', ?, ?, 'Item', NULL, NULL, ?, '[]', '[]', NULL, NULL, ?, NULL, 0, 0, 1)
    `,
  ).run(
    input.recordKey,
    input.name,
    input.category,
    input.subcategory ?? null,
    input.traitsJson ?? '["arcane"]',
    input.sourceCategory ?? "core",
  );
}

describe("migration record loader decoding", () => {
  let db: DatabaseSync | null = null;

  afterEach(() => {
    db?.close();
    db = null;
  });

  it("loads typed migration references", () => {
    db = createMigrationDb();
    insertMigrationRecord(db, {
      recordKey: "spell:primary",
      name: "Primary",
      category: "spell",
    });
    insertMigrationRecord(db, {
      recordKey: "action:linked",
      name: "Linked Action",
      category: "rule",
      subcategory: "action",
      traitsJson: '["concentrate"]',
    });
    db.prepare(
      `INSERT INTO reference_edges (from_record_key, to_record_key) VALUES ('spell:primary', 'action:linked')`,
    ).run();

    const records = loadDerivedTagMigrationRecords(db, { recordKeys: ["spell:primary"] });

    expect(records).toHaveLength(1);
    expect(records[0]?.references).toEqual([
      expect.objectContaining({
        recordKey: "action:linked",
        category: "rule",
        subcategory: "action",
        traits: ["concentrate"],
      }),
    ]);
    expect(countDerivedTagMigrationRecords(db, {})).toBe(2);
  });

  it("rejects invalid decoded reference subcategories", () => {
    db = createMigrationDb();
    insertMigrationRecord(db, {
      recordKey: "spell:primary",
      name: "Primary",
      category: "spell",
    });
    insertMigrationRecord(db, {
      recordKey: "spell:bad-ref",
      name: "Bad Ref",
      category: "spell",
      subcategory: "action",
    });
    db.prepare(
      `INSERT INTO reference_edges (from_record_key, to_record_key) VALUES ('spell:primary', 'spell:bad-ref')`,
    ).run();

    expect(() => loadDerivedTagMigrationRecords(db!, {})).toThrow(
      'Invalid search subcategory "action" for spell migration reference',
    );
  });

  it("rejects malformed reference trait arrays", () => {
    db = createMigrationDb();
    insertMigrationRecord(db, {
      recordKey: "spell:primary",
      name: "Primary",
      category: "spell",
    });
    insertMigrationRecord(db, {
      recordKey: "rule:bad-traits",
      name: "Bad Traits",
      category: "rule",
      subcategory: "action",
      traitsJson: '{"bad":true}',
    });
    db.prepare(
      `INSERT INTO reference_edges (from_record_key, to_record_key) VALUES ('spell:primary', 'rule:bad-traits')`,
    ).run();

    expect(() => loadDerivedTagMigrationRecords(db!, {})).toThrow(
      'Expected targetTraitsJson for migration reference "rule:bad-traits" to be a JSON string array.',
    );
  });
});
