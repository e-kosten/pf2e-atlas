import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  buildDerivedTagOntologyExplorerModel,
  filterOntologyExplorerNodes,
  writeDerivedTagOntologyExplorerDbCache,
} from "../../src/tui/ontology-explorer/data.js";
import {
  buildDerivedTagOntologyExplorerListLines,
  createDerivedTagOntologyExplorerState,
  drillIntoDerivedTagOntologyExplorer,
  jumpDerivedTagOntologyExplorerSelection,
  moveDerivedTagOntologyExplorerDetailScroll,
  moveDerivedTagOntologyExplorerDetailScrollToBoundary,
  moveDerivedTagOntologyExplorerSelection,
  moveDerivedTagOntologyExplorerSelectionToBoundary,
  normalizeDerivedTagOntologyExplorerState,
  popDerivedTagOntologyExplorerDepth,
  setDerivedTagOntologyExplorerFilter,
} from "../../src/tui/ontology-explorer/ui.js";
import {
  getRenderedTerminalLineCount,
  sliceRenderedTerminalLines,
  normalizeTerminalTwoPaneLayoutMode,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
} from "../../src/tui/terminal-ui.js";

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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const revelationFamily = spellCategory?.families.find((family) => family.family === "revelation");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");
    const truthRevealTag = revelationFamily?.tags.find((tag) => tag.tag === "truth_reveal");

    expect(spellCategory?.taggedRecordCount).toBe(3);
    expect(securityFamily?.axis).toBeTruthy();
    expect(securityFamily?.liveRecordCount).toBe(2);
    expect(revelationFamily?.liveRecordCount).toBe(2);
    expect(alarmTag?.liveRecordCount).toBe(2);
    expect(truthRevealTag?.liveRecordCount).toBe(2);
    expect(alarmTag?.records.map((record) => record.record.name)).toEqual([
      "Alarm Ward",
      "Watchful Truth",
    ]);
  });

  it("orders explorer categories by the managed category policy and tags alphabetically", () => {
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
    });
    insertRecord(db, {
      recordKey: "equipment:one",
      name: "Silent Toolkit",
      category: "equipment",
      subcategory: "gear",
      tags: ["alarm"],
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");

    expect(model.categories.map((category) => category.category)).toEqual([
      "affliction",
      "creature",
      "equipment",
      "hazard",
      "spell",
    ]);
    expect(securityFamily?.tags.map((tag) => tag.tag)).toEqual([
      "alarm",
      "scrying_protection",
      "security",
    ]);
  });

  it("reuses the persisted ontology explorer cache instead of rebuilding from live index tables", () => {
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

    writeDerivedTagOntologyExplorerDbCache(db);

    db.exec(`
      DELETE FROM record_derived_tags;
      DELETE FROM spell_records;
      DELETE FROM records;
    `);

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    expect(alarmTag?.liveRecordCount).toBe(2);
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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: securityFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:one",
    });

    state = setDerivedTagOntologyExplorerFilter(model, state, "reveals lies");

    expect(state.selectedRecordKey).toBe("spell:three");
  });

  it("matches family depth against descendant tag metadata", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Zone of Truth",
      category: "spell",
      tags: ["truth_reveal"],
      descriptionText: "Forces honesty.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const targetFamily = spellCategory?.families.find((family) =>
      family.tags.some((tag) => tag.tag === "truth_reveal"));
    const nonMatchingFamily = spellCategory?.families.find((family) => family.key !== targetFamily?.key);

    expect(targetFamily).toBeTruthy();
    expect(nonMatchingFamily).toBeTruthy();

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "family",
      selectedCategoryKey: "spell",
      selectedFamilyKey: nonMatchingFamily?.key,
    });

    state = setDerivedTagOntologyExplorerFilter(model, state, "truth_reveal");

    const lines = buildDerivedTagOntologyExplorerListLines(model, state, 20);
    expect(lines.some((line) => line.text.startsWith(`${targetFamily?.family} |`))).toBe(true);
  });

  it("does not match family depth against descendant record text", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Watchful Truth",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Amber bell chime.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const communicationFamily = spellCategory?.families.find((family) => family.family === "communication");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "family",
      selectedCategoryKey: "spell",
      selectedFamilyKey: communicationFamily?.key,
    });

    state = setDerivedTagOntologyExplorerFilter(model, state, "amber bell");

    expect(buildDerivedTagOntologyExplorerListLines(model, state, 10)).toEqual([
      { text: "No nodes match the current filter.", tone: "dim" },
    ]);
  });

  it("chooses the nearest surviving selection when filtering removes the current row", () => {
    const db = createExplorerDb();
    insertRecord(db, {
      recordKey: "spell:one",
      name: "Alarm Ward",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Amber warning light.",
    });
    insertRecord(db, {
      recordKey: "spell:two",
      name: "Breach Alarm",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Middle sentinel.",
    });
    insertRecord(db, {
      recordKey: "spell:three",
      name: "Watch Bell",
      category: "spell",
      tags: ["alarm"],
      descriptionText: "Amber bell chime.",
    });

    const model = buildDerivedTagOntologyExplorerModel(db);
    const spellCategory = model.categories.find((category) => category.category === "spell");
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: securityFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:two",
    });

    state = setDerivedTagOntologyExplorerFilter(model, state, "amber");

    expect(state.selectedRecordKey).toBe("spell:one");
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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: securityFamily?.key,
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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: securityFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:two",
    });

    state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "start");
    expect(state.selectedRecordKey).toBe("spell:one");

    state = moveDerivedTagOntologyExplorerSelectionToBoundary(model, state, "end");
    expect(state.selectedRecordKey).toBe("spell:three");
  });

  it("clamps jump-style record movement instead of wrapping", () => {
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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const alarmTag = securityFamily?.tags.find((tag) => tag.tag === "alarm");

    let state = createDerivedTagOntologyExplorerState(model);
    state = normalizeDerivedTagOntologyExplorerState(model, {
      ...state,
      depth: "record",
      selectedCategoryKey: "spell",
      selectedFamilyKey: securityFamily?.key,
      selectedTagKey: alarmTag?.key,
      selectedRecordKey: "spell:two",
    });

    state = jumpDerivedTagOntologyExplorerSelection(model, state, 10);
    expect(state.selectedRecordKey).toBe("spell:three");

    state = jumpDerivedTagOntologyExplorerSelection(model, state, -10);
    expect(state.selectedRecordKey).toBe("spell:one");
  });

  it("scrolls detail focus with clamping and boundary jumps", () => {
    let state = {
      depth: "record" as const,
      filter: "",
      detailScroll: 3,
    };

    state = moveDerivedTagOntologyExplorerDetailScroll(state, 5, 10);
    expect(state.detailScroll).toBe(8);

    state = moveDerivedTagOntologyExplorerDetailScroll(state, 5, 10);
    expect(state.detailScroll).toBe(10);

    state = moveDerivedTagOntologyExplorerDetailScroll(state, -20, 10);
    expect(state.detailScroll).toBe(0);

    state = moveDerivedTagOntologyExplorerDetailScrollToBoundary(state, "end", 14);
    expect(state.detailScroll).toBe(14);

    state = moveDerivedTagOntologyExplorerDetailScrollToBoundary(state, "start", 14);
    expect(state.detailScroll).toBe(0);
  });

  it("toggles focused detail layout only while detail has focus", () => {
    expect(toggleTerminalTwoPaneLayoutMode("split", "list")).toBe("split");
    expect(toggleTerminalTwoPaneLayoutMode("split", "detail")).toBe("detail-only");
    expect(toggleTerminalTwoPaneLayoutMode("detail-only", "detail")).toBe("split");
  });

  it("falls back to split layout when detail focus is left", () => {
    expect(normalizeTerminalTwoPaneLayoutMode("detail-only", "list")).toBe("split");
    expect(normalizeTerminalTwoPaneLayoutMode("detail-only", "detail")).toBe("detail-only");
  });

  it("toggles two-pane focus generically", () => {
    expect(toggleTerminalTwoPaneFocus("list")).toBe("detail");
    expect(toggleTerminalTwoPaneFocus("detail")).toBe("list");
  });

  it("counts and slices wrapped detail rows using rendered terminal width", () => {
    const lines = [
      { text: "Short heading", tone: "section" as const },
      { text: "This is a long wrapped detail line that should span multiple rendered terminal rows when the pane is narrow." },
    ];

    expect(getRenderedTerminalLineCount(lines, 18)).toBeGreaterThan(lines.length);

    const visibleLines = sliceRenderedTerminalLines(lines, 18, 1, 3);
    expect(visibleLines).toHaveLength(3);
    expect(visibleLines.every((line) => line.noWrap)).toBe(true);
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
    const securityFamily = spellCategory?.families.find((family) => family.family === "security");
    const filtered = filterOntologyExplorerNodes(
      spellCategory?.families ?? [],
      securityFamily?.axis ?? "",
    );

    expect(filtered.map((family) => family.family)).toContain("security");
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
