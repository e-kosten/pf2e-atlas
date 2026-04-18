import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  evaluateDerivedTagGaps,
  rankDerivedTagGapCandidates,
  type DerivedTagGapRecord,
} from "../../src/tags/evaluation/gap-evaluator.js";

function vector(values: number[]): Float32Array {
  return Float32Array.from(values);
}

function blob(values: number[]): Uint8Array {
  return new Uint8Array(vector(values).buffer.slice(0));
}

function createGapEvaluatorDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      level INTEGER,
      traits_json TEXT NOT NULL,
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
  `);
  return db;
}

function insertGapRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    category: string;
    subcategory?: string | null;
    level?: number | null;
    traits?: string[];
    descriptionText?: string | null;
    vector: number[];
    tags?: string[];
  },
): void {
  db.prepare(
    `
    INSERT INTO records (record_key, name, category, subcategory, level, traits_json, description_text, is_search_canonical)
    VALUES (?, ?, ?, ?, ?, ?, ?, 1)
  `,
  ).run(
    input.recordKey,
    input.name,
    input.category,
    input.subcategory ?? null,
    input.level ?? null,
    JSON.stringify(input.traits ?? []),
    input.descriptionText ?? null,
  );
  db.prepare("INSERT INTO embeddings (record_key, vector_blob) VALUES (?, ?)").run(input.recordKey, blob(input.vector));
  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

describe("derived tag gap evaluator", () => {
  it("ranks untagged records by similarity to tagged exemplars and surfaces common traits", () => {
    const exemplars: DerivedTagGapRecord[] = [
      {
        recordKey: "equipment:1",
        name: "Swallow-Spike",
        category: "equipment",
        subcategory: "gear",
        level: 6,
        traits: ["magical", "armor"],
        descriptionText: "Break free when you become Grabbed.",
        vector: vector([1, 0, 0]),
      },
      {
        recordKey: "equipment:2",
        name: "Implacable",
        category: "equipment",
        subcategory: "gear",
        level: 8,
        traits: ["magical", "armor"],
        descriptionText: "Makes you difficult to hold back.",
        vector: vector([0.9, 0.1, 0]),
      },
      {
        recordKey: "equipment:3",
        name: "Miniaturization Module",
        category: "equipment",
        subcategory: "gear",
        level: 5,
        traits: ["magical", "clockwork"],
        descriptionText: "Gives a bonus to Escape.",
        vector: vector([0.95, 0.05, 0]),
      },
    ];

    const candidates: DerivedTagGapRecord[] = [
      {
        recordKey: "equipment:4",
        name: "Likely Gap",
        category: "equipment",
        subcategory: "gear",
        level: 7,
        traits: ["magical", "armor"],
        descriptionText: "A likely missed restraint-escape tool.",
        vector: vector([0.98, 0.02, 0]),
      },
      {
        recordKey: "equipment:5",
        name: "Unrelated",
        category: "equipment",
        subcategory: "gear",
        level: 7,
        traits: ["magical", "illusion"],
        descriptionText: "A thematically unrelated item.",
        vector: vector([0, 1, 0]),
      },
    ];

    const evaluation = rankDerivedTagGapCandidates(exemplars, candidates, {
      tag: "restraint_escape",
      category: "equipment",
      subcategory: "gear",
      limit: 2,
      exemplarLimit: 2,
      commonTraitLimit: 4,
    });

    expect(evaluation.tag).toBe("restraint_escape");
    expect(evaluation.exemplarCount).toBe(3);
    expect(evaluation.candidateCount).toBe(2);
    expect(evaluation.candidateCategory).toBe("equipment");
    expect(evaluation.candidateSubcategory).toBe("gear");
    expect(evaluation.exemplarCategory).toBe("equipment");
    expect(evaluation.exemplarSubcategory).toBe("gear");
    expect(evaluation.commonTraits).toEqual(expect.arrayContaining(["magical", "armor"]));
    expect(evaluation.candidates[0]?.name).toBe("Likely Gap");
    expect(evaluation.candidates[0]?.sharedTraits).toEqual(expect.arrayContaining(["magical", "armor"]));
    expect(evaluation.candidates[0]?.similarity).toBeGreaterThan(evaluation.candidates[1]?.similarity ?? 0);
    expect(evaluation.exemplars).toHaveLength(2);
    expect(evaluation.discriminativeTokens.length).toBeGreaterThan(0);
    expect(evaluation.discriminativePhrases.length).toBeGreaterThan(0);
    expect(evaluation.candidateCohorts.length).toBeGreaterThan(0);
  });

  it("supports cross-category exemplar seeding when explicitly requested", () => {
    const db = createGapEvaluatorDb();
    try {
      insertGapRecord(db, {
        recordKey: "equipment:1",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        level: 5,
        traits: ["illusion", "magical"],
        descriptionText: "A disguise item for social infiltration.",
        vector: [1, 0, 0],
        tags: ["disguise"],
      });
      insertGapRecord(db, {
        recordKey: "equipment:2",
        name: "Quick-Change Outfit",
        category: "equipment",
        subcategory: "gear",
        level: 3,
        traits: ["illusion", "magical"],
        descriptionText: "A quick-change disguise kit.",
        vector: [0.95, 0.05, 0],
        tags: ["disguise"],
      });
      insertGapRecord(db, {
        recordKey: "spell:1",
        name: "Illusory Disguise",
        category: "spell",
        level: 1,
        traits: ["illusion"],
        descriptionText: "A disguise spell.",
        vector: [0.98, 0.02, 0],
      });
      insertGapRecord(db, {
        recordKey: "spell:2",
        name: "Focus Burst",
        category: "spell",
        level: 3,
        traits: ["focus"],
        descriptionText: "An unrelated spell.",
        vector: [0, 1, 0],
      });

      const evaluation = evaluateDerivedTagGaps(db, {
        tag: "disguise",
        category: "spell",
        exemplarCategory: "equipment",
        exemplarSubcategory: "gear",
        limit: 2,
        exemplarLimit: 2,
      });

      expect(evaluation.exemplarCategory).toBe("equipment");
      expect(evaluation.exemplarSubcategory).toBe("gear");
      expect(evaluation.candidateCategory).toBe("spell");
      expect(evaluation.candidateSubcategory).toBeNull();
      expect(evaluation.exemplarCount).toBe(2);
      expect(evaluation.candidateCount).toBe(2);
      expect(evaluation.candidates[0]?.name).toBe("Illusory Disguise");
      expect(evaluation.candidates[0]?.similarity).toBeGreaterThan(evaluation.candidates[1]?.similarity ?? 0);
      expect(evaluation.contrastRecords).toBeDefined();
    } finally {
      db.close();
    }
  });

  it("returns an empty candidate list when the target scope has no untagged records", () => {
    const db = createGapEvaluatorDb();
    try {
      insertGapRecord(db, {
        recordKey: "equipment:1",
        name: "Masquerade Scarf",
        category: "equipment",
        subcategory: "gear",
        level: 5,
        traits: ["illusion", "magical"],
        descriptionText: "A disguise item for social infiltration.",
        vector: [1, 0, 0],
        tags: ["disguise"],
      });
      insertGapRecord(db, {
        recordKey: "spell:1",
        name: "Illusory Disguise",
        category: "spell",
        level: 1,
        traits: ["illusion"],
        descriptionText: "A disguise spell.",
        vector: [0.98, 0.02, 0],
        tags: ["disguise"],
      });

      const evaluation = evaluateDerivedTagGaps(db, {
        tag: "disguise",
        category: "spell",
        exemplarCategory: "equipment",
        exemplarSubcategory: "gear",
      });

      expect(evaluation.candidateCount).toBe(0);
      expect(evaluation.candidates).toEqual([]);
    } finally {
      db.close();
    }
  });

  it("names the exemplar scope when no tagged exemplars exist there", () => {
    const db = createGapEvaluatorDb();
    try {
      insertGapRecord(db, {
        recordKey: "spell:1",
        name: "Illusory Disguise",
        category: "spell",
        level: 1,
        traits: ["illusion"],
        descriptionText: "A disguise spell.",
        vector: [0.98, 0.02, 0],
      });

      expect(() =>
        evaluateDerivedTagGaps(db, {
          tag: "disguise",
          category: "spell",
          exemplarCategory: "equipment",
          exemplarSubcategory: "gear",
        }),
      ).toThrow(/matched exemplar scope "equipment\/gear"/i);
    } finally {
      db.close();
    }
  });
});
