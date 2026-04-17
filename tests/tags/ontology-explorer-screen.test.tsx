import React from "react";
import { DatabaseSync } from "node:sqlite";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import { DerivedTagOntologyExplorerScreen } from "../../src/tags/migration/ontology-explorer-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tags/migration/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

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

  db.prepare(`
    INSERT INTO records (
      record_key, pack_name, name, record_type, category, subcategory, document_type, level, rarity,
      traits_json, derived_tags_json, families_json, description_text, blurb_text, is_search_canonical
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)
  `).run(
    "spell:test-alarm",
    "spell",
    "Alarm Ward",
    "spell",
    "spell",
    null,
    "Item",
    1,
    null,
    JSON.stringify([]),
    JSON.stringify(["alarm"]),
    JSON.stringify(["security"]),
    "Warns against intruders.",
    null,
  );

  db.prepare(`
    INSERT INTO spell_records (
      record_key, traditions_json, spell_kinds_json, save_type, area_type,
      range_text, duration_text, target_text, area_value, sustained, basic_save
    )
    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `).run(
    "spell:test-alarm",
    JSON.stringify(["arcane"]),
    JSON.stringify(["spell"]),
    null,
    null,
    "30 feet",
    "1 minute",
    "creature",
    null,
    0,
    0,
  );

  db.prepare("INSERT INTO record_derived_tags (record_key, tag) VALUES (?, ?)").run("spell:test-alarm", "alarm");
  return db;
}

describe("ontology explorer screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("treats q as search input while inline search is active", async () => {
    const db = createExplorerDb();
    const onExit = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <DerivedTagOntologyExplorerScreen db={db} onExit={onExit} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("/");
    await flushInk();
    expect(app.lastFrame()).toContain("Search /");

    app.stdin.write("q");
    await flushInk();

    expect(onExit).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Search /q");

    app.unmount();
    db.close();
  });
});
