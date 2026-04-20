import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it, vi } from "vitest";

import { Pf2eRecordCatalog } from "../../src/data/backend/record-catalog.js";
import type {
  SearchCategorySummaryResult,
  SearchSemanticsBootstrapSummaryResult,
} from "../../src/data/vocabulary.js";
import {
  getSearchCategorySummary as getSearchCategorySummaryRuntime,
  getSearchSemanticsBootstrapSummary as getSearchSemanticsBootstrapSummaryRuntime,
} from "../../src/data/vocabulary.js";

vi.mock("../../src/data/vocabulary.js", async () => {
  const actual = await vi.importActual<typeof import("../../src/data/vocabulary.js")>("../../src/data/vocabulary.js");
  return {
    ...actual,
    getSearchCategorySummary: vi.fn(),
    getSearchSemanticsBootstrapSummary: vi.fn(),
  };
});

function createCatalogDb(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      name TEXT NOT NULL
    );
    CREATE TABLE record_aliases (
      canonical_record_key TEXT NOT NULL,
      alias_text TEXT NOT NULL
    );
    CREATE TABLE record_legacy_links (
      canonical_record_key TEXT NOT NULL,
      legacy_record_key TEXT NOT NULL
    );
  `);
  return db;
}

describe("Pf2eRecordCatalog summary caches", () => {
  let db: DatabaseSync | null = null;

  afterEach(() => {
    db?.close();
    db = null;
    vi.resetAllMocks();
  });

  it("caches repeated category summary reads", () => {
    db = createCatalogDb();
    const categorySummary: SearchCategorySummaryResult = {
      categories: [{ value: "spell", count: 12 }],
    };
    vi.mocked(getSearchCategorySummaryRuntime).mockReturnValue(categorySummary);

    const catalog = new Pf2eRecordCatalog(db, [], null);

    expect(catalog.getSearchCategorySummary()).toBe(categorySummary);
    expect(catalog.getSearchCategorySummary()).toBe(categorySummary);
    expect(getSearchCategorySummaryRuntime).toHaveBeenCalledTimes(1);
    expect(getSearchCategorySummaryRuntime).toHaveBeenCalledWith(db);
  });

  it("caches bootstrap summaries per normalized trait limit", () => {
    db = createCatalogDb();
    const defaultSummary: SearchSemanticsBootstrapSummaryResult = {
      categories: [{ value: "spell", count: 12 }],
      subcategoryCountsByCategory: [],
      commonTraitsByCategory: [],
      commonDerivedTagsByCategory: [],
      derivedTagCatalog: [],
    };
    const narrowSummary: SearchSemanticsBootstrapSummaryResult = {
      ...defaultSummary,
      commonTraitsByCategory: [{ category: "spell", traits: [{ value: "fire", count: 4 }] }],
    };
    vi.mocked(getSearchSemanticsBootstrapSummaryRuntime).mockImplementation((_db, options = {}) =>
      options.traitLimitPerCategory === 5 ? narrowSummary : defaultSummary,
    );

    const catalog = new Pf2eRecordCatalog(db, [], null);

    expect(catalog.getSearchSemanticsBootstrapSummary()).toBe(defaultSummary);
    expect(catalog.getSearchSemanticsBootstrapSummary({ traitLimitPerCategory: 12 })).toBe(defaultSummary);
    expect(catalog.getSearchSemanticsBootstrapSummary({ traitLimitPerCategory: 5 })).toBe(narrowSummary);
    expect(catalog.getSearchSemanticsBootstrapSummary({ traitLimitPerCategory: 5 })).toBe(narrowSummary);

    expect(getSearchSemanticsBootstrapSummaryRuntime).toHaveBeenCalledTimes(2);
    expect(getSearchSemanticsBootstrapSummaryRuntime).toHaveBeenNthCalledWith(1, db, { traitLimitPerCategory: 12 });
    expect(getSearchSemanticsBootstrapSummaryRuntime).toHaveBeenNthCalledWith(2, db, { traitLimitPerCategory: 5 });
  });
});
