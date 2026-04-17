import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, NormalizedRecord, SearchFilters } from "../../src/types.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search-service.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { SearchScreen } from "../../src/tui/search-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function pressDown(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[B");
}

function pressUp(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[A");
}

function pressRight(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[C");
}

function createTestConfig(): AppConfig {
  return {
    dataPath: "vendor/pf2e",
    rootPath: "vendor/pf2e",
    manifestPath: "vendor/pf2e/system.pf2e.json",
    indexPath: ".cache/pf2e-index.sqlite",
    embeddings: {
      provider: "hash",
      modelId: "test-model",
      modelRevision: null,
      cachePath: ".cache/models",
      localModelPath: null,
    },
    ranking: {
      configPath: "pf2e-ranking.json",
    },
  };
}

function createRecord(overrides: Partial<NormalizedRecord> = {}): NormalizedRecord {
  return {
    recordKey: "spell:test-alarm",
    id: "test-alarm",
    name: "Alarm Ward",
    normalizedName: "alarm ward",
    type: "spell",
    category: "spell",
    subcategory: null,
    packName: "spell",
    packLabel: "Spells",
    documentType: "Item",
    level: 1,
    rarity: null,
    traits: [],
    derivedTags: ["alarm"],
    publicationTitle: null,
    publicationRemaster: false,
    descriptionText: "Warns against intruders.",
    blurbText: null,
    hasDescription: true,
    descriptionSnippet: "Warns against intruders.",
    sourceCategory: "core",
    folderId: null,
    families: ["security"],
    variantFamilyKey: null,
    variantBaseName: null,
    variantLabel: null,
    variantAxes: [],
    variantConfidence: null,
    variantSource: "none",
    sourcePath: "packs/spells/alarm-ward.json",
    isUnique: false,
    size: null,
    itemCategory: null,
    baseItem: null,
    priceCp: null,
    bulkValue: null,
    actionCost: 2,
    usage: null,
    hands: null,
    damageTypes: [],
    weaponGroup: null,
    armorGroup: null,
    traditions: ["arcane"],
    spellKinds: ["spell"],
    saveType: null,
    areaType: null,
    rangeText: "30 feet",
    durationText: "1 minute",
    durationUnit: "minute",
    targetText: "creature",
    areaValue: null,
    sustained: false,
    basicSave: false,
    languages: [],
    speedTypes: [],
    senses: [],
    immunities: [],
    resistances: [],
    weaknesses: [],
    disableText: null,
    disableSkills: [],
    isComplex: false,
    actorMetrics: {},
    itemMetrics: {},
    rangeValue: 30,
    aliases: [],
    legacyRecordLinks: [],
    raw: {},
    ...overrides,
  };
}

function createServices(
  overrides: {
    listRecords?: ReturnType<typeof vi.fn>;
    lookup?: ReturnType<typeof vi.fn>;
    search?: ReturnType<typeof vi.fn>;
  } = {},
): Pf2eTerminalAppServices {
  const record = createRecord();
  const listRecords = overrides.listRecords ?? vi.fn((filters: SearchFilters) => ({
    searchProfile: null,
    mode: "structured" as const,
    total: 1,
    offset: filters.offset ?? 0,
    limit: filters.limit ?? 20,
    records: [record],
  }));
  const lookup = overrides.lookup ?? vi.fn(() => ({ match: record, alternatives: [] }));
  const search = overrides.search ?? vi.fn(async (filters: SearchFilters) => ({
    searchProfile: filters.searchProfile ?? "balanced",
    mode: "hybrid" as const,
    total: 1,
    offset: filters.offset ?? 0,
    limit: filters.limit ?? 20,
    records: [record],
  }));

  const searchService = createPf2eTerminalSearchService({
    getSearchVocabulary: () => ({
      categories: [{ value: "spell", count: 1 }],
      subcategories: [],
      rarities: [{ value: "common", count: 1 }],
      sizes: [],
      traditions: [{ value: "arcane", count: 1 }],
      spellKinds: [{ value: "spell", count: 1 }],
      sourceCategories: [{ value: "core", count: 1 }],
      commonTraitsByCategory: [],
      commonDerivedTagsByCategory: [],
      derivedTagOntologyFamilies: [],
      derivedTagOntologyTags: [],
      derivedTagCatalog: [],
    }),
    listFilterValues: vi.fn(({ field }) => {
      if (field === "rarity") {
        return {
          values: [
            { value: "unique", count: 1 },
            { value: "common", count: 1 },
            { value: "rare", count: 1 },
            { value: "uncommon", count: 1 },
          ],
        };
      }
      if (field === "actionCost") {
        return {
          values: [
            { value: "3", count: 1 },
            { value: "1", count: 1 },
            { value: "2", count: 1 },
          ],
        };
      }
      if (field === "traits") {
        return { values: [{ value: "illusion", count: 1 }] };
      }
      return { values: [] };
    }),
    lookup,
    listRecords,
    search,
  });

  return {
    config: createTestConfig(),
    catalog: {
      getRecord: vi.fn(() => record),
      getSearchVocabulary: vi.fn(() => ({}) as never),
      listFilterValues: vi.fn(() => ({ field: "categories", values: [] }) as never),
      listRecords,
      lookup,
      search,
    },
    user: {
      search: searchService,
      ontology: {
        listDomains: vi.fn(() => []),
        loadDomain: vi.fn(() => ({
          id: "catalogCategories",
          label: "Categories",
          description: "Test domain",
          rootNodes: [],
        })),
      },
    },
    dev: {
      tagRefinement: {
        createSession: vi.fn(async () => {
          throw new Error("not implemented");
        }),
        getQueueItems: vi.fn(() => []),
        promptAndCreateSession: vi.fn(async () => undefined),
      },
    },
    close: vi.fn(),
  };
}

describe("search screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("supports arrow-driven navigation for editing and executing the draft workspace", async () => {
    const search = vi.fn(async (filters: SearchFilters) => ({
      searchProfile: filters.searchProfile ?? "balanced",
      mode: "hybrid" as const,
      total: 1,
      offset: 0,
      limit: filters.limit ?? 20,
      records: [createRecord()],
    }));
    const services = createServices({ search });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Browse/Search");
    expect(app.lastFrame()).toContain("Scope & Filters");
    expect(app.lastFrame()).toContain("Preview | Run Draft Query");
    expect(app.lastFrame()).not.toContain("Results | No applied session");
    expect(app.lastFrame()).not.toContain("Profile |");
    expect(app.lastFrame()).not.toContain("Action Cost |");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Mode");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Workspace Mode");
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Query");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Draft Query");
    for (const character of "ghost") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Profile");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Profile");
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Category");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Category Scope");
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Run Draft Query");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(search).toHaveBeenCalledWith({
      actionCost: undefined,
      category: "spell",
      levelMax: undefined,
      levelMin: undefined,
      limit: 20,
      metadata: undefined,
      query: "ghost",
      rarity: undefined,
      searchProfile: "lexical",
      subcategory: undefined,
    });
    expect(app.lastFrame()).toContain("Draft matches applied query");
    expect(app.lastFrame()).toContain("1/1 shown");
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 shown");
    expect(app.lastFrame()).toContain("Alarm Ward | spell | lvl 1");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward");
    expect(app.lastFrame()).not.toContain("Result 1 |");

    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward");

    app.stdin.write("\u001b[D");
    await flushInk();
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 shown");

    app.stdin.write("\u001b[D");
    await flushInk();
    expect(app.lastFrame()).toContain("[WORKSPACE] Scope & Filters");
    expect(app.lastFrame()).not.toContain("[RESULTS] 1/1 shown");
  });

  it("orders filter values from declarative field policies and exposes action cost through facet editing", () => {
    const services = createServices();

    expect(services.user.search.getRarityOptions("spell", null).map((option) => option.value)).toEqual([
      "common",
      "uncommon",
      "rare",
      "unique",
    ]);
    expect(services.user.search.getFacetValueOptions("rarity", "spell", null).map((option) => option.value)).toEqual([
      "common",
      "uncommon",
      "rare",
      "unique",
    ]);
    expect(services.user.search.getActionCostOptions("spell", null).map((option) => option.value)).toEqual([
      "1",
      "2",
      "3",
    ]);
    expect(services.user.search.getFacetFieldOptions("spell", null).some((option) => option.value === "actionCost")).toBe(true);
  });

  it("maps simple ontology browse queries into seeded workspace requests", () => {
    const services = createServices();
    const request = services.user.search.createRequestFromOntologyQuery({
      kind: "listRecords",
      label: "Browse records with this trait",
      filters: {
        category: "spell",
        metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
        limit: 20,
      },
    });

    expect(request).toEqual({
      mode: "browse",
      limit: 20,
      queryText: "",
      searchProfile: "balanced",
      sourceLabel: "Browse records with this trait",
      filters: {
        category: "spell",
        subcategory: null,
        levelMin: null,
        levelMax: null,
        rarity: {
          any: [],
          all: [],
          exclude: [],
        },
        actionCost: {
          any: [],
          all: [],
          exclude: [],
        },
        facets: [
          {
            field: "traits",
            policy: {
              any: ["illusion"],
              all: [],
              exclude: [],
            },
          },
        ],
      },
    });
  });

  it("translates policy-based draft filters into metadata clauses", async () => {
    const search = vi.fn(async (filters: SearchFilters) => ({
      searchProfile: filters.searchProfile ?? "balanced",
      mode: "hybrid" as const,
      total: 1,
      offset: 0,
      limit: filters.limit ?? 20,
      records: [createRecord()],
    }));
    const services = createServices({ search });

    await services.user.search.runQuery({
      mode: "search",
      limit: 20,
      queryText: "ghost",
      searchProfile: "balanced",
      sourceLabel: null,
      filters: {
        category: "spell",
        subcategory: null,
        levelMin: null,
        levelMax: null,
        rarity: {
          any: ["common"],
          all: [],
          exclude: ["rare"],
        },
        actionCost: {
          any: [2],
          all: [],
          exclude: [1],
        },
        facets: [
          {
            field: "traits",
            policy: {
              any: ["illusion"],
              all: ["auditory"],
              exclude: ["emotion"],
            },
          },
        ],
      },
    });

    expect(search).toHaveBeenCalledWith({
      actionCost: undefined,
      category: "spell",
      levelMax: undefined,
      levelMin: undefined,
      limit: 20,
      metadata: {
        and: [
          { field: "rarity", op: "eq", value: "common" },
          { field: "rarity", op: "notIn", values: ["rare"] },
          { field: "actionCost", op: "eq", value: 2 },
          { not: { field: "actionCost", op: "eq", value: 1 } },
          {
            and: [
              { field: "traits", op: "includesAny", values: ["illusion"] },
              { field: "traits", op: "includesAll", values: ["auditory"] },
              { field: "traits", op: "excludesAny", values: ["emotion"] },
            ],
          },
        ],
      },
      query: "ghost",
      rarity: undefined,
      searchProfile: "balanced",
      subcategory: undefined,
    });
  });
});
