import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  buildDerivedTagOntologyExplorerModel,
  filterOntologyExplorerNodes,
} from "../../src/tags/migration/ontology-explorer-data.js";
import {
  createDerivedTagOntologyExplorerState,
  drillIntoDerivedTagOntologyExplorer,
  normalizeDerivedTagOntologyExplorerState,
  popDerivedTagOntologyExplorerDepth,
  setDerivedTagOntologyExplorerFilter,
} from "../../src/tags/migration/ontology-explorer-ui.js";

function createExplorerDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
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

function insertRecord(db: DatabaseSync, recordKey: string, category: string, tags: string[]): void {
  db.prepare("INSERT INTO records (record_key, category, is_search_canonical) VALUES (?, ?, 1)").run(recordKey, category);
  for (const tag of tags) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(recordKey, tag);
  }
}

describe("derived tag ontology explorer", () => {
  it("builds category, family, and tag live counts from canonical records", () => {
    const db = createExplorerDb();
    insertRecord(db, "spell:one", "spell", ["alarm"]);
    insertRecord(db, "spell:two", "spell", ["truth_reveal"]);
    insertRecord(db, "spell:three", "spell", ["alarm", "truth_reveal"]);

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const alarmTag = communicationFamily?.tags.find((tag) => tag.tag === "alarm");
    const truthRevealTag = communicationFamily?.tags.find((tag) => tag.tag === "truth_reveal");

    expect(spellCategory?.taggedRecordCount).toBe(3);
    expect(communicationFamily?.liveRecordCount).toBe(3);
    expect(alarmTag?.liveRecordCount).toBe(2);
    expect(truthRevealTag?.liveRecordCount).toBe(2);
  });

  it("filters the active depth and clamps selection to visible ontology nodes", () => {
    const db = createExplorerDb();
    insertRecord(db, "spell:one", "spell", ["alarm"]);
    insertRecord(db, "spell:two", "spell", ["truth_reveal"]);

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");

    let state = createDerivedTagOntologyExplorerState(model);
    state = {
      ...state,
      depth: "tag",
      selectedCategoryKey: "spell",
      selectedFamilyKey: communicationFamily?.key,
      selectedTagKey: `${communicationFamily?.category}:alarm`,
    };

    state = setDerivedTagOntologyExplorerFilter(model, state, "truthful answers");

    expect(state.selectedTagKey).toBe("spell:truth_reveal");
  });

  it("navigates down and back up the ontology hierarchy without losing the current slice", () => {
    const db = createExplorerDb();
    insertRecord(db, "spell:one", "spell", ["alarm"]);

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      selectedCategoryKey: spellCategory?.key,
    });

    state = drillIntoDerivedTagOntologyExplorer(model, state);
    expect(state.depth).toBe("family");

    state = drillIntoDerivedTagOntologyExplorer(model, state);
    expect(state.depth).toBe("tag");

    state = popDerivedTagOntologyExplorerDepth(state);
    expect(state.depth).toBe("family");

    state = popDerivedTagOntologyExplorerDepth(state);
    expect(state.depth).toBe("category");
    expect(state.selectedCategoryKey).toBe("spell");
  });

  it("filters ontology node lists by normalized search text", () => {
    const db = createExplorerDb();
    insertRecord(db, "spell:one", "spell", ["alarm"]);
    insertRecord(db, "creature:one", "creature", ["urban_setting"]);

    const model = buildDerivedTagOntologyExplorerModel(db);
    const filtered = filterOntologyExplorerNodes(model.categories, "site and scene-placement");

    expect(filtered.map((entry) => entry.category)).toContain("creature");
    expect(filtered.some((entry) => entry.category === "spell")).toBe(false);
  });
});
