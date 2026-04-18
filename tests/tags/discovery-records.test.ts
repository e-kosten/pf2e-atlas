import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import { loadDiscoveryRecords } from "../../src/tags/discovery/discovery-records.js";

function vectorBlob(values: number[]): Uint8Array {
  return new Uint8Array(Float32Array.from(values).buffer.slice(0));
}

function createDiscoveryDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      pack_name TEXT,
      publication_title TEXT,
      folder_id TEXT,
      source_path TEXT,
      name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      variant_family_key TEXT,
      variant_base_name TEXT,
      variant_label TEXT,
      variant_axes_json TEXT,
      level INTEGER,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT,
      description_text TEXT,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE embeddings (
      record_key TEXT PRIMARY KEY,
      vector_blob BLOB NOT NULL
    );
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
    CREATE TABLE reference_edges (
      from_record_key TEXT NOT NULL,
      to_record_key TEXT NOT NULL,
      display_text TEXT,
      reference_text TEXT NOT NULL,
      from_pack_name TEXT NOT NULL,
      from_record_type TEXT NOT NULL,
      from_document_type TEXT NOT NULL,
      from_source_category TEXT NOT NULL
    );
  `);
  return db;
}

function insertRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    category: string;
    subcategory?: string | null;
    variantAxesJson?: string | null;
    traitsJson?: string;
    derivedTagsJson?: string | null;
  },
): void {
  db.prepare(
    `
      INSERT INTO records (
        record_key, pack_name, publication_title, folder_id, source_path, name, category, subcategory,
        variant_family_key, variant_base_name, variant_label, variant_axes_json,
        level, traits_json, derived_tags_json, description_text, is_search_canonical
      )
      VALUES (?, 'packs', NULL, NULL, NULL, ?, ?, ?, NULL, NULL, NULL, ?, NULL, ?, ?, NULL, 1)
    `,
  ).run(
    input.recordKey,
    input.name,
    input.category,
    input.subcategory ?? null,
    input.variantAxesJson ?? "[]",
    input.traitsJson ?? '["arcane"]',
    input.derivedTagsJson ?? '["sample_tag"]',
  );
  db.prepare("INSERT INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(input.recordKey, vectorBlob([1, 2]));
}

describe("discovery record loading", () => {
  let db: DatabaseSync | null = null;

  afterEach(() => {
    db?.close();
    db = null;
  });

  it("decodes typed discovery rows without blind casts", () => {
    db = createDiscoveryDb();
    insertRecord(db, {
      recordKey: "spell:test-spell",
      name: "Test Spell",
      category: "spell",
      variantAxesJson: '["tradition"]',
      traitsJson: '["arcane","attack"]',
      derivedTagsJson: '["damage_burst"]',
    });

    const records = loadDiscoveryRecords(db, { includeVectors: true });

    expect(records).toHaveLength(1);
    expect(records[0]).toMatchObject({
      recordKey: "spell:test-spell",
      category: "spell",
      subcategory: null,
      variantAxes: ["tradition"],
      traits: ["arcane", "attack"],
      derivedTags: ["damage_burst"],
    });
    expect(records[0]?.vector.length).toBe(2);
  });

  it("rejects invalid discovery categories from the database", () => {
    db = createDiscoveryDb();
    insertRecord(db, {
      recordKey: "weird:test-record",
      name: "Weird Record",
      category: "not-a-real-category",
    });

    expect(() => loadDiscoveryRecords(db!, {})).toThrow('Invalid discovery category "not-a-real-category"');
  });

  it("rejects subcategories that do not belong to the decoded category", () => {
    db = createDiscoveryDb();
    insertRecord(db, {
      recordKey: "spell:test-record",
      name: "Wrong Subcategory",
      category: "spell",
      subcategory: "action",
    });

    expect(() => loadDiscoveryRecords(db!, {})).toThrow('Invalid discovery subcategory "action" for spell record');
  });
});
