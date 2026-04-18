import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import { buildDerivedTagOntologyExplorerModel } from "../../src/tui/ontology-explorer/data.js";

function createOntologyExplorerDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      pack_name TEXT,
      name TEXT NOT NULL,
      record_type TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      document_type TEXT NOT NULL,
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
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
    CREATE TABLE actor_records (
      record_key TEXT PRIMARY KEY,
      size TEXT,
      languages_json TEXT,
      speed_types_json TEXT,
      senses_json TEXT,
      immunities_json TEXT,
      resistances_json TEXT,
      weaknesses_json TEXT,
      disable_text TEXT,
      disable_skills_json TEXT,
      is_complex INTEGER
    );
    CREATE TABLE item_records (
      record_key TEXT PRIMARY KEY,
      item_category TEXT,
      base_item TEXT,
      price_cp INTEGER,
      usage_text TEXT,
      hands INTEGER,
      damage_types_json TEXT,
      weapon_group TEXT,
      armor_group TEXT
    );
    CREATE TABLE spell_records (
      record_key TEXT PRIMARY KEY,
      traditions_json TEXT,
      spell_kinds_json TEXT,
      save_type TEXT,
      area_type TEXT,
      range_text TEXT,
      duration_text TEXT,
      target_text TEXT,
      area_value INTEGER,
      sustained INTEGER,
      basic_save INTEGER
    );
  `);
  return db;
}

describe("ontology explorer data decoding", () => {
  let db: DatabaseSync | null = null;

  afterEach(() => {
    db?.close();
    db = null;
  });

  it("rejects invalid live-count categories before building the model", () => {
    db = createOntologyExplorerDb();
    db.prepare(
      `
        INSERT INTO records (
          record_key, pack_name, name, record_type, category, subcategory, document_type, level, rarity,
          traits_json, derived_tags_json, families_json, description_text, blurb_text, source_category,
          publication_title, publication_remaster, is_unique, is_search_canonical
        )
        VALUES (?, 'packs', ?, 'spell', ?, NULL, 'Item', NULL, NULL, '[]', '[]', '[]', NULL, NULL, 'core', NULL, 0, 0, 1)
      `,
    ).run("bad:record", "Bad Record", "not-a-real-category");
    db.prepare(`INSERT INTO record_derived_tags (record_key, tag) VALUES ('bad:record', 'alarm')`).run();

    expect(() => buildDerivedTagOntologyExplorerModel(db!)).toThrow(
      'Invalid search category "not-a-real-category" for ontology explorer live count row "bad:record".',
    );
  });
});
