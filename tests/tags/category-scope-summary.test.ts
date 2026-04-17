import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import {
  getCurrentDerivedTagMigrationAuthoredState,
  setCurrentDerivedTagMigrationAuthoredState,
} from "../../src/tags/migration/authored-state.js";
import { summarizeDerivedTagCategoryScopes } from "../../src/tags/migration/category-scope-summary.js";
import { buildDerivedTagMigrationSession } from "../../src/tags/migration/session-builder.js";

function createMigrationDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      pack_name TEXT,
      name TEXT NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      level INTEGER,
      traits_json TEXT NOT NULL DEFAULT '[]',
      families_json TEXT,
      derived_tags_json TEXT NOT NULL DEFAULT '[]',
      description_text TEXT,
      blurb_text TEXT,
      is_search_canonical INTEGER NOT NULL
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
    tags?: string[];
  },
): void {
  db.prepare(`
    INSERT INTO records (
      record_key, pack_name, name, category, subcategory, level,
      traits_json, families_json, derived_tags_json, description_text, blurb_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, NULL, '[]', NULL, ?, NULL, NULL, 1)
  `).run(
    input.recordKey,
    input.recordKey.split(":")[0] ?? "",
    input.name,
    input.category,
    input.subcategory ?? null,
    JSON.stringify(input.tags ?? []),
  );

  for (const tag of input.tags ?? []) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

const initialState = getCurrentDerivedTagMigrationAuthoredState();

afterEach(() => {
  setCurrentDerivedTagMigrationAuthoredState(initialState);
});

describe("derived tag category scope summaries", () => {
  it("shows pending review counts by category for review-queue sessions", () => {
    const db = createMigrationDb();
    const nextState = structuredClone(initialState);
    nextState.assignmentReviews.creature = {
      category: "creature",
      decisions: [
        {
          name: "Scout",
          recordKey: "creature:scout",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "Pending creature assignment review.",
        },
      ],
    };
    nextState.exemplarReviews.spell = {
      category: "spell",
      decisions: [
        {
          name: "Alarm Ward",
          recordKey: "spell:alarm-ward",
          tag: "alarm",
          proposedPolarity: "positive",
          status: "needs_review",
          rationale: "Pending spell exemplar review.",
        },
      ],
    };
    setCurrentDerivedTagMigrationAuthoredState(nextState);

    const summary = summarizeDerivedTagCategoryScopes(db, "review_queue");

    expect(summary.allCategoriesDetailLines).toEqual([
      "2 pending review changes",
      "1 assignment + 1 exemplar",
      "2 queue slices",
    ]);
    expect(summary.categories.find((entry) => entry.category === "creature")?.detailLines).toEqual([
      "1 pending review change",
      "1 assignment + 0 exemplar",
      "1 queue slice",
    ]);
    expect(summary.categories.find((entry) => entry.category === "spell")?.detailLines).toEqual([
      "1 pending review change",
      "0 assignment + 1 exemplar",
      "1 queue slice",
    ]);
  });

  it("shows pending LLM proposal counts for proposal-review scope selection", () => {
    const db = createMigrationDb();
    const nextState = structuredClone(initialState);
    nextState.assignmentReviews.creature = {
      category: "creature",
      decisions: [
        {
          name: "Creature Scout",
          recordKey: "creature:one",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "LLM proposal.",
          source: "llm",
        },
        {
          name: "Creature Guide",
          recordKey: "creature:two",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "Human proposal.",
          source: "human",
        },
      ],
    };
    nextState.exemplarReviews.spell = {
      category: "spell",
      decisions: [
        {
          name: "Arcane Bell",
          recordKey: "spell:one",
          tag: "alarm",
          proposedPolarity: "positive",
          status: "needs_review",
          rationale: "LLM exemplar proposal.",
          source: "llm",
        },
      ],
    };
    setCurrentDerivedTagMigrationAuthoredState(nextState);

    const summary = summarizeDerivedTagCategoryScopes(db, "proposal_review");
    expect(summary.allCategoriesDetailLines).toEqual([
      "2 pending review changes",
      "1 assignment + 1 exemplar",
      "LLM-origin proposals only",
    ]);
    expect(summary.categories.find((entry) => entry.category === "creature")?.detailLines).toEqual([
      "1 pending review change",
      "1 assignment + 0 exemplar",
      "LLM-origin proposals only",
    ]);
    expect(summary.categories.find((entry) => entry.category === "spell")?.detailLines).toEqual([
      "1 pending review change",
      "0 assignment + 1 exemplar",
      "LLM-origin proposals only",
    ]);
  });

  it("builds proposal-review sessions from LLM pending review items", () => {
    const db = createMigrationDb();
    insertRecord(db, {
      recordKey: "creature:one",
      name: "Creature Scout",
      category: "creature",
    });
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Arcane Bell",
      category: "spell",
    });
    insertRecord(db, {
      recordKey: "equipment:one",
      name: "Tagged Toolkit",
      category: "equipment",
    });

    const nextState = structuredClone(initialState);
    nextState.assignmentReviews.creature = {
      category: "creature",
      decisions: [
        {
          name: "Creature Scout",
          recordKey: "creature:one",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "LLM proposal.",
          source: "llm",
        },
        {
          name: "Tagged Toolkit",
          recordKey: "equipment:one",
          family: "alarm",
          tag: "alarm",
          mode: "include",
          rationale: "Human proposal in wrong category slice.",
          source: "human",
        },
      ],
    };
    nextState.exemplarReviews.spell = {
      category: "spell",
      decisions: [
        {
          name: "Arcane Bell",
          recordKey: "spell:one",
          tag: "alarm",
          proposedPolarity: "positive",
          status: "needs_review",
          rationale: "LLM exemplar proposal.",
          source: "llm",
        },
      ],
    };
    setCurrentDerivedTagMigrationAuthoredState(nextState);

    const session = buildDerivedTagMigrationSession(db, {
      mode: "proposal_review",
    });

    expect(session.manifest.category).toBeUndefined();
    expect(session.records.map((record) => record.recordKey)).toEqual([
      "spell:one",
      "creature:one",
    ]);
    expect(new Set(session.records.map((record) => record.category))).toEqual(new Set(["creature", "spell"]));
    expect(session.decisions.map((record) => record.decisions).flat().every((decision) => decision.source === "llm")).toBe(true);
  });
});
