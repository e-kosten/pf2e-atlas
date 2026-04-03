import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  discoverSemanticCandidates,
  rankSemanticDiscoveryCandidates,
  type SemanticDiscoveryRecord,
} from "../../src/tags/semantic-discovery.js";

function vector(values: number[]): Float32Array {
  return Float32Array.from(values);
}

function blob(values: number[]): Uint8Array {
  return new Uint8Array(vector(values).buffer.slice(0));
}

function createSemanticDiscoveryDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      normalized_name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      level INTEGER,
      traits_json TEXT NOT NULL,
      derived_tags_json TEXT NOT NULL,
      description_text TEXT,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE embeddings (
      record_key TEXT PRIMARY KEY,
      vector_blob BLOB NOT NULL
    );
    CREATE TABLE record_aliases (
      canonical_record_key TEXT NOT NULL,
      alias_text TEXT NOT NULL,
      normalized_alias TEXT NOT NULL,
      source_kind TEXT NOT NULL,
      source_ref TEXT NOT NULL
    );
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
  `);
  return db;
}

function insertDiscoveryRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    normalizedName?: string;
    category: string;
    subcategory?: string | null;
    level?: number | null;
    traits?: string[];
    derivedTags?: string[];
    descriptionText?: string | null;
    vector: number[];
    aliases?: string[];
    tags?: string[];
  },
): void {
  db.prepare(`
    INSERT INTO records (record_key, name, normalized_name, category, subcategory, level, traits_json, derived_tags_json, description_text, is_search_canonical)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
  `).run(
    input.recordKey,
    input.name,
    input.normalizedName ?? input.name.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
    input.category,
    input.subcategory ?? null,
    input.level ?? null,
    JSON.stringify(input.traits ?? []),
    JSON.stringify(input.derivedTags ?? []),
    input.descriptionText ?? null,
  );
  db.prepare("INSERT INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(
    input.recordKey,
    blob(input.vector),
  );
  for (const alias of input.aliases ?? []) {
    db.prepare(`
      INSERT INTO record_aliases (canonical_record_key, alias_text, normalized_alias, source_kind, source_ref)
      VALUES (?, ?, ?, 'test', ?)
    `).run(
      input.recordKey,
      alias,
      alias.toLowerCase().replace(/[^a-z0-9]+/g, " ").trim().replace(/\s+/g, " "),
      alias,
    );
  }
  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

describe("semantic discovery evaluator", () => {
  it("discovers exemplar-near candidates, summarizes evidence, and surfaces contrast records", () => {
    const db = createSemanticDiscoveryDb();
    try {
      insertDiscoveryRecord(db, {
        recordKey: "creature:ghost-commoner",
        name: "Ghost Commoner",
        category: "creature",
        level: 4,
        traits: ["undead", "human"],
        derivedTags: ["undead_adjacent"],
        descriptionText: "Mournful spirits haunt abandoned halls with whispered laments.",
        vector: [1, 0, 0],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:ghost-pirate-captain",
        name: "Ghost Pirate Captain",
        category: "creature",
        level: 7,
        traits: ["undead", "human"],
        derivedTags: ["undead_adjacent", "nautical_setting"],
        descriptionText: "A mournful undead captain haunts abandoned ships with whispered laments and cursed crews.",
        vector: [0.98, 0.02, 0],
        aliases: ["Pirate Ghost Captain"],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:haunted-bosun",
        name: "Haunted Bosun",
        category: "creature",
        level: 6,
        traits: ["undead", "human"],
        derivedTags: ["undead_adjacent"],
        descriptionText: "This undead bosun haunts abandoned decks with whispered laments and doomed crews.",
        vector: [0.97, 0.03, 0],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:shipwreck-wraith",
        name: "Shipwreck Wraith",
        category: "creature",
        level: 8,
        traits: ["undead", "spirit"],
        derivedTags: ["haunt_theme"],
        descriptionText: "A mournful wraith circles abandoned shipwrecks and drowned crews.",
        vector: [0.96, 0.04, 0],
        tags: ["haunt_theme"],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:dockside-ruffian",
        name: "Dockside Ruffian",
        category: "creature",
        level: 5,
        traits: ["human"],
        descriptionText: "A dockside thug bullies sailors and crews around abandoned wharves.",
        vector: [0.91, 0.09, 0],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:sunny-guard",
        name: "Sunny Guard",
        category: "creature",
        level: 5,
        traits: ["human"],
        descriptionText: "A cheerful guard patrols bright plazas.",
        vector: [0, 1, 0],
      });

      const result = discoverSemanticCandidates(db, {
        category: "creature",
        exemplarNames: ["Ghost Commoner", "Pirate Ghost Captain"],
        excludeDerivedTag: "haunt_theme",
        limit: 2,
        contrastLimit: 2,
      });

      expect(result.category).toBe("creature");
      expect(result.exemplarCount).toBe(2);
      expect(result.candidateCount).toBe(3);
      expect(result.resolvedExemplars).toEqual(expect.arrayContaining([
        expect.objectContaining({ query: "Ghost Commoner", matchedBy: "name", recordKey: "creature:ghost-commoner" }),
        expect.objectContaining({ query: "Pirate Ghost Captain", matchedBy: "alias", recordKey: "creature:ghost-pirate-captain" }),
      ]));
      expect(result.commonTraits).toEqual(expect.arrayContaining(["undead", "human"]));
      expect(result.sharedTokens.map((entry) => entry.value)).toEqual(expect.arrayContaining(["abandoned", "mournful", "whispered"]));
      expect(result.candidates.map((entry) => entry.name)).toEqual(["Haunted Bosun", "Dockside Ruffian"]);
      expect(result.candidates[0]?.sharedTraits).toEqual(expect.arrayContaining(["undead", "human"]));
      expect(result.contrastRecords.map((entry) => entry.name)).toContain("Sunny Guard");
      expect(result.similarityBuckets.find((bucket) => bucket.minSimilarity === 0.9)?.count).toBe(2);
    } finally {
      db.close();
    }
  });

  it("fails cleanly when an exemplar name is ambiguous", () => {
    const db = createSemanticDiscoveryDb();
    try {
      insertDiscoveryRecord(db, {
        recordKey: "creature:watcher-1",
        name: "Watcher",
        category: "creature",
        level: 3,
        traits: ["human"],
        descriptionText: "A vigilant observer.",
        vector: [1, 0, 0],
      });
      insertDiscoveryRecord(db, {
        recordKey: "creature:watcher-2",
        name: "Watcher",
        category: "creature",
        level: 5,
        traits: ["construct"],
        descriptionText: "An eerie machine observer.",
        vector: [0.9, 0.1, 0],
      });

      expect(() => discoverSemanticCandidates(db, {
        category: "creature",
        exemplarNames: ["Watcher"],
      })).toThrow(/ambiguous/i);
    } finally {
      db.close();
    }
  });

  it("ignores malformed vectors while still ranking valid records", () => {
    const exemplars: SemanticDiscoveryRecord[] = [
      {
        recordKey: "creature:1",
        name: "Valid Exemplar",
        category: "creature",
        subcategory: null,
        level: 5,
        traits: ["undead"],
        derivedTags: [],
        descriptionText: "Haunts abandoned halls.",
        vector: vector([1, 0, 0]),
      },
      {
        recordKey: "creature:2",
        name: "Broken Exemplar",
        category: "creature",
        subcategory: null,
        level: 4,
        traits: ["undead"],
        derivedTags: [],
        descriptionText: "Should be ignored.",
        vector: new Float32Array(0),
      },
    ];
    const candidates: SemanticDiscoveryRecord[] = [
      {
        recordKey: "creature:3",
        name: "Likely Candidate",
        category: "creature",
        subcategory: null,
        level: 6,
        traits: ["undead"],
        derivedTags: [],
        descriptionText: "Haunts abandoned crypts.",
        vector: vector([0.98, 0.02, 0]),
      },
      {
        recordKey: "creature:4",
        name: "Broken Candidate",
        category: "creature",
        subcategory: null,
        level: 6,
        traits: ["construct"],
        derivedTags: [],
        descriptionText: "Should be ignored.",
        vector: new Float32Array(0),
      },
    ];

    const result = rankSemanticDiscoveryCandidates(exemplars, candidates, {
      category: "creature",
      limit: 2,
    });

    expect(result.exemplarCount).toBe(1);
    expect(result.candidateCount).toBe(1);
    expect(result.candidates.map((entry) => entry.name)).toEqual(["Likely Candidate"]);
  });
});
