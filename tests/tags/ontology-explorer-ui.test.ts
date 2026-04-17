import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  buildDerivedTagOntologyExplorerModel,
  filterOntologyExplorerNodes,
} from "../../src/tags/migration/ontology-explorer-data.js";
import {
  createDerivedTagOntologyExplorerState,
  drillIntoDerivedTagOntologyExplorer,
  moveDerivedTagOntologyExplorerSelection,
  moveDerivedTagOntologyExplorerSelectionToBoundary,
  normalizeDerivedTagOntologyExplorerState,
  popDerivedTagOntologyExplorerDepth,
  setDerivedTagOntologyExplorerFilter,
} from "../../src/tags/migration/ontology-explorer-ui.js";

function createExplorerDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      pack_name TEXT,
      name TEXT NOT NULL,
      record_type TEXT NOT NULL DEFAULT 'creature',
      category TEXT NOT NULL,
      subcategory TEXT,
      document_type TEXT NOT NULL DEFAULT 'Actor',
      level INTEGER,
      rarity TEXT,
      traits_json TEXT NOT NULL DEFAULT '[]',
      derived_tags_json TEXT NOT NULL DEFAULT '[]',
      families_json TEXT,
      description_text TEXT,
      blurb_text TEXT,
      source_category TEXT NOT NULL DEFAULT 'core',
      publication_title TEXT,
      publication_remaster INTEGER NOT NULL DEFAULT 0,
      is_unique INTEGER NOT NULL DEFAULT 0,
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
      bulk_value REAL,
      usage_text TEXT,
      hands INTEGER,
      damage_types_json TEXT NOT NULL DEFAULT '[]',
      weapon_group TEXT,
      armor_group TEXT,
      action_cost INTEGER
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

function insertRecord(
  db: DatabaseSync,
  input: {
    recordKey: string;
    name: string;
    category: string;
    subcategory?: string | null;
    type?: string;
    documentType?: string;
    level?: number | null;
    rarity?: string | null;
    traits?: string[];
    tags: string[];
    families?: string[];
    descriptionText?: string | null;
    blurbText?: string | null;
  },
): void {
  db.prepare(`
    INSERT INTO records (
      record_key, pack_name, name, record_type, category, subcategory, document_type, level, rarity,
      traits_json, derived_tags_json, families_json, description_text, blurb_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
  `).run(
    input.recordKey,
    input.recordKey.split(":")[0] ?? "",
    input.name,
    input.type ?? input.category,
    input.category,
    input.subcategory ?? null,
    input.documentType ?? "Actor",
    input.level ?? null,
    input.rarity ?? null,
    JSON.stringify(input.traits ?? []),
    JSON.stringify(input.tags),
    input.families ? JSON.stringify(input.families) : null,
    input.descriptionText ?? null,
    input.blurbText ?? null,
  );

  if (input.category === "creature" || input.category === "hazard" || input.category === "affliction") {
    db.prepare(`
      INSERT INTO actor_records (
        record_key, size, languages_json, speed_types_json, senses_json, immunities_json,
        resistances_json, weaknesses_json, disable_text, disable_skills_json, is_complex
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      input.recordKey,
      "medium",
      JSON.stringify(["common"]),
      JSON.stringify(["land"]),
      JSON.stringify(["darkvision"]),
      JSON.stringify([]),
      JSON.stringify([]),
      JSON.stringify([]),
      null,
      JSON.stringify([]),
      0,
    );
  }

  if (input.category === "spell") {
    db.prepare(`
      INSERT INTO spell_records (
        record_key, traditions_json, spell_kinds_json, save_type, area_type,
        range_text, duration_text, target_text, area_value, sustained, basic_save
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      input.recordKey,
      JSON.stringify(["arcane"]),
      JSON.stringify(["spell"]),
      null,
      null,
      "60 feet",
      "1 minute",
      "creature",
      null,
      0,
      0,
    );
  }

  if (input.category === "equipment") {
    db.prepare(`
      INSERT INTO item_records (
        record_key, item_category, base_item, price_cp, bulk_value, usage_text,
        hands, damage_types_json, weapon_group, armor_group, action_cost
      )
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      input.recordKey,
      "gear",
      null,
      500,
      null,
      "held in 1 hand",
      1,
      JSON.stringify([]),
      null,
      null,
      null,
    );
  }

  for (const tag of input.tags) {
    db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run(input.recordKey, tag);
  }
}

describe("derived tag ontology explorer", () => {
  it("builds category, family, tag, and record nodes from canonical records", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Warns against intruders.",
    });
    insertRecord(db, {
      recordKey: "spell:two",
      name: "Zone of Truth",
      category: "spell",
      tags: ["truth_reveal"],
      descriptionText: "Forces honesty.",
    });
    insertRecord(db, {
      recordKey: "spell:three",
      name: "Watchful Truth",
      category: "spell",
      tags: ["alarm", "truth_reveal"],
      descriptionText: "Both warns and reveals lies.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const alarmTag = communicationFamily?.tags.find((tag) => tag.tag === "alarm");
    const truthRevealTag = communicationFamily?.tags.find((tag) => tag.tag === "truth_reveal");

    expect(spellCategory?.taggedRecordCount).toBe(3);
    expect(communicationFamily?.axis).toBeTruthy();
    expect(communicationFamily?.liveRecordCount).toBe(3);
    expect(alarmTag?.liveRecordCount).toBe(2);
    expect(truthRevealTag?.liveRecordCount).toBe(2);
    expect(alarmTag?.records.map((record) => record.record.name)).toEqual([
      "Alarm Ward",
      "Watchful Truth",
    ]);
  });

  it("filters record depth and clamps selection to visible records", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Warns against intruders.",
    });
    insertRecord(db, {
      recordKey: "spell:three",
      name: "Watchful Truth",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Both warns and reveals lies.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const alarmTag = communicationFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: communicationFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:one",
    });

    state = setDerivedTagOntologyExplorerFilter(model, state, "reveals lies");

    expect(state.selectedRecordKey).toBe("spell:three");
  });

  it("navigates category -> family -> tag -> record and back without losing the current slice", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
    });

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

    state = drillIntoDerivedTagOntologyExplorer(model, state);
    expect(state.depth).toBe("record");

    state = popDerivedTagOntologyExplorerDepth(state);
    expect(state.depth).toBe("tag");

    state = popDerivedTagOntologyExplorerDepth(state);
    expect(state.depth).toBe("family");

    state = popDerivedTagOntologyExplorerDepth(state);
    expect(state.depth).toBe("category");
    expect(state.selectedCategoryKey).toBe("spell");
  });

  it("moves record selection within a tag", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
    });
    insertRecord(db, {
      recordKey: "spell:two",
      name: "Breach Alarm",
      category: "spell",
      tags: ["alarm"],
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const alarmTag = communicationFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: communicationFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:one",
    });

    state = moveDerivedTagOntologyExplorerSelection(model, state, 1);
    expect(state.selectedRecordKey).toBe("spell:two");
  });

  it("jumps record selection to the list boundaries", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
    });
    insertRecord(db, {
      recordKey: "spell:two",
      name: "Breach Alarm",
      category: "spell",
      tags: ["alarm"],
    });
    insertRecord(db, {
      recordKey: "spell:three",
      name: "Watch Bell",
      category: "spell",
      tags: ["alarm"],
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const alarmTag = communicationFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: communicationFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:two",
    });

    state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "start");
    expect(state.selectedRecordKey).toBe("spell:one");

    state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "end");
    expect(state.selectedRecordKey).toBe("spell:three");
  });

  it("filters ontology node lists by normalized search text", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
    });
    insertRecord(db, {
      recordKey: "creature:one",
      name: "Sewer Stalker",
      category: "creature",
      tags: ["urban_setting"],
      descriptionText: "A city sewer hunter.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const filtered = filterOntologyExplorerNodes(model.categories, "site and scene-placement");

    expect(filtered.map((entry) => entry.category)).toContain("creature");
    expect(filtered.some((entry) => entry.category === "spell")).toBe(false);
  });

  it("includes axis text in family filtering", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");
    const filtered = filterOntologyExplorerNodes(
      spellCategory?.families ?? [],
      communicationFamily?.axis ?? "",
    );

    expect(filtered.map((family) => family.family)).toContain("communication");
  });

  it("loads equipment record details through the live item schema columns", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "equipment:one",
      name: "Silent Toolkit",
      category: "equipment",
      subcategory: "gear",
      tags: ["concealable"],
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const equipmentCategory = model.categories.find((category) => category.category === "equipment");
    const stealthFamily = equipmentCategory?.families.find((family) => family.tags.some((tag) => tag.tag === "concealable"));
    const concealableTag = stealthFamily?.tags.find((tag) => tag.tag === "concealable");
    const equipmentRecord = concealableTag?.records.find((record) => record.record.recordKey === "equipment:one");

    expect(equipmentRecord?.record.usage).toBe("held in 1 hand");
    expect(equipmentRecord?.record.itemCategory).toBe("gear");
  });
});
