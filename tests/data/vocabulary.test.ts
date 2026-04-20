import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import {
  getSearchCategorySummary,
  getSearchSemanticsBootstrapSummary,
  getSearchVocabulary,
} from "../../src/data/vocabulary.js";

function createVocabularyDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      category TEXT NOT NULL,
      subcategory TEXT,
      rarity TEXT,
      source_category TEXT NOT NULL,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE actor_records (
      record_key TEXT PRIMARY KEY,
      size TEXT
    );
    CREATE TABLE spell_records (
      record_key TEXT PRIMARY KEY,
      traditions_json TEXT NOT NULL,
      spell_kinds_json TEXT NOT NULL
    );
    CREATE TABLE record_traits (
      record_key TEXT NOT NULL,
      trait TEXT NOT NULL
    );
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
  `);
  return db;
}

describe("search vocabulary decoding", () => {
  let db: DatabaseSync | null = null;

  afterEach(() => {
    db?.close();
    db = null;
  });

  it("decodes validated category and spell vocabulary rows plus lightweight summaries", () => {
    db = createVocabularyDb();
    db.prepare(
      `
        INSERT INTO records (record_key, category, subcategory, rarity, source_category, is_search_canonical)
        VALUES
          ('spell:test', 'spell', NULL, 'common', 'core', 1),
          ('creature:test', 'creature', NULL, 'rare', 'rules', 1),
          ('rule:test', 'rule', 'action', 'common', 'rules', 1)
      `,
    ).run();
    db.prepare(`INSERT INTO actor_records (record_key, size) VALUES ('creature:test', 'med')`).run();
    db.prepare(
      `INSERT INTO spell_records (record_key, traditions_json, spell_kinds_json) VALUES ('spell:test', '["Primal"]', '["Focus"]')`,
    ).run();
    db.prepare(
      `
        INSERT INTO record_traits (record_key, trait) VALUES
          ('spell:test', 'fire'),
          ('creature:test', 'undead'),
          ('rule:test', 'concentrate')
      `,
    ).run();
    db.prepare(
      `
        INSERT INTO record_derived_tags (record_key, tag) VALUES
          ('spell:test', 'damage_burst'),
          ('creature:test', 'undead_adjacent'),
          ('rule:test', 'action_economy')
      `,
    ).run();

    const categorySummary = getSearchCategorySummary(db);
    const bootstrapSummary = getSearchSemanticsBootstrapSummary(db, { traitLimitPerCategory: 4 });
    const vocabulary = getSearchVocabulary(db, { traitLimitPerCategory: 4 });

    expect(categorySummary.categories).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ value: "spell" }),
        expect.objectContaining({ value: "creature" }),
        expect.objectContaining({ value: "rule" }),
      ]),
    );
    expect(bootstrapSummary.subcategoryCountsByCategory).toContainEqual({
      category: "rule",
      subcategories: [{ value: "action", count: 1 }],
    });
    expect(bootstrapSummary.commonTraitsByCategory).toContainEqual({
      category: "rule",
      traits: [{ value: "concentrate", count: 1 }],
    });
    expect(bootstrapSummary.commonDerivedTagsByCategory).toContainEqual({
      category: "rule",
      tags: [{ value: "action_economy", count: 1 }],
    });
    expect(bootstrapSummary.derivedTagCatalog.length).toBeGreaterThan(0);
    expect(vocabulary.categories).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ value: "spell" }),
        expect.objectContaining({ value: "creature" }),
        expect.objectContaining({ value: "rule" }),
      ]),
    );
    expect(vocabulary.sourceCategories).toEqual(
      expect.arrayContaining([expect.objectContaining({ value: "core" }), expect.objectContaining({ value: "rules" })]),
    );
    expect(vocabulary.traditions).toContainEqual({ value: "primal", count: 1 });
    expect(vocabulary.spellKinds).toContainEqual({ value: "focus", count: 1 });
  });

  it("rejects invalid decoded source categories", () => {
    db = createVocabularyDb();
    db.prepare(
      `
        INSERT INTO records (record_key, category, subcategory, rarity, source_category, is_search_canonical)
        VALUES ('spell:test', 'spell', NULL, 'common', 'not-real', 1)
      `,
    ).run();
    db.prepare(
      `INSERT INTO spell_records (record_key, traditions_json, spell_kinds_json) VALUES ('spell:test', '[]', '[]')`,
    ).run();

    expect(() => getSearchVocabulary(db!)).toThrow('Invalid source category "not-real"');
  });

  it("rejects malformed traditions arrays", () => {
    db = createVocabularyDb();
    db.prepare(
      `
        INSERT INTO records (record_key, category, subcategory, rarity, source_category, is_search_canonical)
        VALUES ('spell:test', 'spell', NULL, 'common', 'core', 1)
      `,
    ).run();
    db.prepare(
      `INSERT INTO spell_records (record_key, traditions_json, spell_kinds_json) VALUES ('spell:test', '{"bad":true}', '[]')`,
    ).run();

    expect(() => getSearchVocabulary(db!)).toThrow("Expected traditionsJson for search vocabulary spell traditions");
  });
});
