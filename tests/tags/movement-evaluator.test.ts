import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import { evaluateDerivedTagMovement } from "../../src/tags/evaluation/movement-evaluator.js";

function createMovementDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      category TEXT NOT NULL,
      is_search_canonical INTEGER NOT NULL
    );
    CREATE TABLE record_derived_tags (
      record_key TEXT NOT NULL,
      tag TEXT NOT NULL
    );
  `);
  return db;
}

function insertMovementRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name?: string;
    category: string;
    tags?: string[];
    isSearchCanonical?: boolean;
  },
): void {
  db.prepare(`
    INSERT INTO records (record_key, name, category, is_search_canonical)
    VALUES (?, ?, ?, ?)
  `).run(
    input.recordKey,
    input.name ?? input.recordKey,
    input.category,
    input.isSearchCanonical === false ? 0 : 1,
  );

  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

describe("derived tag movement evaluator", () => {
  it("reports category and tag movement between two index snapshots", () => {
    const baselineDb = createMovementDb();
    const currentDb = createMovementDb();

    try {
      insertMovementRecord(baselineDb, { recordKey: "creature:1", category: "creature", tags: ["urban_setting", "temple_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:2", category: "creature", tags: ["urban_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:3", category: "creature" });
      insertMovementRecord(baselineDb, { recordKey: "spell:1", category: "spell", tags: ["mobility"] });

      insertMovementRecord(currentDb, { recordKey: "creature:1", category: "creature", tags: ["temple_setting"] });
      insertMovementRecord(currentDb, { recordKey: "creature:2", category: "creature", tags: ["temple_setting"] });
      insertMovementRecord(currentDb, { recordKey: "creature:3", category: "creature" });
      insertMovementRecord(currentDb, { recordKey: "spell:1", category: "spell", tags: ["mobility"] });

      const result = evaluateDerivedTagMovement(baselineDb, currentDb, {
        category: "creature",
        sampleLimit: 3,
      });

      expect(result.categories).toEqual([
        {
          category: "creature",
          baselineTotal: 3,
          currentTotal: 3,
          baselineTagged: 2,
          currentTagged: 2,
          baselineCoveragePercent: (2 / 3) * 100,
          currentCoveragePercent: (2 / 3) * 100,
          deltaTagged: 0,
          deltaCoveragePoints: 0,
        },
      ]);

      expect(result.tags).toEqual([
        {
          category: "creature",
          tag: "urban_setting",
          baselineCount: 2,
          currentCount: 0,
          baselineCoveragePercent: (2 / 3) * 100,
          currentCoveragePercent: 0,
          deltaCount: -2,
          deltaCoveragePoints: -(2 / 3) * 100,
          gainedRecords: [],
          lostRecords: [
            { recordKey: "creature:1", name: "creature:1" },
            { recordKey: "creature:2", name: "creature:2" },
          ],
        },
        {
          category: "creature",
          tag: "temple_setting",
          baselineCount: 1,
          currentCount: 2,
          baselineCoveragePercent: (1 / 3) * 100,
          currentCoveragePercent: (2 / 3) * 100,
          deltaCount: 1,
          deltaCoveragePoints: (1 / 3) * 100,
          gainedRecords: [
            { recordKey: "creature:2", name: "creature:2" },
          ],
          lostRecords: [],
        },
      ]);
    } finally {
      baselineDb.close();
      currentDb.close();
    }
  });

  it("supports explicit tag scopes and warning thresholds", () => {
    const baselineDb = createMovementDb();
    const currentDb = createMovementDb();

    try {
      insertMovementRecord(baselineDb, { recordKey: "creature:1", category: "creature", tags: ["underground_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:2", category: "creature", tags: ["underground_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:3", category: "creature", tags: ["urban_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:4", category: "creature" });

      insertMovementRecord(currentDb, { recordKey: "creature:1", category: "creature", tags: ["urban_setting"] });
      insertMovementRecord(currentDb, { recordKey: "creature:2", category: "creature" });
      insertMovementRecord(currentDb, { recordKey: "creature:3", category: "creature" });
      insertMovementRecord(currentDb, { recordKey: "creature:4", category: "creature" });

      const result = evaluateDerivedTagMovement(baselineDb, currentDb, {
        category: "creature",
        tags: [" underground setting ", "urban_setting"],
        sampleLimit: 2,
        warnCategoryGainBelowPoints: 10,
        warnCategoryDropPoints: 20,
        warnTagGainBelowCount: 2,
        warnTagDropCount: 2,
        warnTagDropPoints: 40,
      });

      expect(result.tags).toEqual([
        {
          category: "creature",
          tag: "underground_setting",
          baselineCount: 2,
          currentCount: 0,
          baselineCoveragePercent: 50,
          currentCoveragePercent: 0,
          deltaCount: -2,
          deltaCoveragePoints: -50,
          gainedRecords: [],
          lostRecords: [
            { recordKey: "creature:1", name: "creature:1" },
            { recordKey: "creature:2", name: "creature:2" },
          ],
        },
        {
          category: "creature",
          tag: "urban_setting",
          baselineCount: 1,
          currentCount: 1,
          baselineCoveragePercent: 25,
          currentCoveragePercent: 25,
          deltaCount: 0,
          deltaCoveragePoints: 0,
          gainedRecords: [
            { recordKey: "creature:1", name: "creature:1" },
          ],
          lostRecords: [
            { recordKey: "creature:3", name: "creature:3" },
          ],
        },
      ]);

      expect(result.warnings).toEqual([
        {
          kind: "category_drop",
          category: "creature",
          deltaCoveragePoints: -50,
          message: "creature dropped -50.0 coverage points.",
        },
        {
          kind: "category_gain_below",
          category: "creature",
          deltaCoveragePoints: -50,
          minimumExpectedGain: 10,
          message: "creature net gain was only -50.0 coverage points; expected at least +10.0.",
        },
        {
          kind: "tag_gain_below",
          category: "creature",
          tag: "underground_setting",
          deltaCount: -2,
          minimumExpectedGain: 2,
          message: "creature/underground_setting net gain was only -2 records; expected at least +2.",
        },
        {
          kind: "tag_drop",
          category: "creature",
          tag: "underground_setting",
          deltaCount: -2,
          deltaCoveragePoints: -50,
          message: "creature/underground_setting dropped -2 records and -50.0 coverage points.",
        },
        {
          kind: "tag_gain_below",
          category: "creature",
          tag: "urban_setting",
          deltaCount: 0,
          minimumExpectedGain: 2,
          message: "creature/urban_setting net gain was only 0 records; expected at least +2.",
        },
      ]);
    } finally {
      baselineDb.close();
      currentDb.close();
    }
  });

  it("captures gained samples for small positive expansion movement", () => {
    const baselineDb = createMovementDb();
    const currentDb = createMovementDb();

    try {
      insertMovementRecord(baselineDb, { recordKey: "creature:1", name: "Baseline Hunter", category: "creature", tags: ["swamp_setting"] });
      insertMovementRecord(baselineDb, { recordKey: "creature:2", name: "Common Scout", category: "creature" });
      insertMovementRecord(baselineDb, { recordKey: "creature:3", name: "Common Guard", category: "creature" });

      insertMovementRecord(currentDb, { recordKey: "creature:1", name: "Baseline Hunter", category: "creature", tags: ["swamp_setting"] });
      insertMovementRecord(currentDb, { recordKey: "creature:2", name: "Common Scout", category: "creature", tags: ["swamp_setting"] });
      insertMovementRecord(currentDb, { recordKey: "creature:3", name: "Common Guard", category: "creature" });

      const result = evaluateDerivedTagMovement(baselineDb, currentDb, {
        category: "creature",
        tags: ["swamp_setting"],
        sampleLimit: 2,
        warnTagGainBelowCount: 3,
      });

      expect(result.tags).toEqual([
        {
          category: "creature",
          tag: "swamp_setting",
          baselineCount: 1,
          currentCount: 2,
          baselineCoveragePercent: (1 / 3) * 100,
          currentCoveragePercent: (2 / 3) * 100,
          deltaCount: 1,
          deltaCoveragePoints: (1 / 3) * 100,
          gainedRecords: [
            { recordKey: "creature:2", name: "Common Scout" },
          ],
          lostRecords: [],
        },
      ]);

      expect(result.warnings).toEqual([
        {
          kind: "tag_gain_below",
          category: "creature",
          tag: "swamp_setting",
          deltaCount: 1,
          minimumExpectedGain: 3,
          message: "creature/swamp_setting net gain was only +1 records; expected at least +3.",
        },
      ]);
    } finally {
      baselineDb.close();
      currentDb.close();
    }
  });

  it("rejects tag-specific comparisons without a category scope", () => {
    const baselineDb = createMovementDb();
    const currentDb = createMovementDb();

    try {
      expect(() => evaluateDerivedTagMovement(baselineDb, currentDb, {
        tags: ["urban_setting"],
      })).toThrow(/requires a category scope/i);
    } finally {
      baselineDb.close();
      currentDb.close();
    }
  });
});
