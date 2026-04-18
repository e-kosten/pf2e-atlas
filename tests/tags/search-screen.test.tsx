import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type {
  AppConfig,
  NormalizedRecord,
  OntologyDomainModel,
  SearchCountResult,
  SearchFilters,
} from "../../src/types.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search-service.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { SearchScreen, parseJumpToResultInput } from "../../src/tui/search-screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function flushDebouncedWindowRead(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 60);
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

function pressLeft(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[D");
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
    closeSearchWindow?: ReturnType<typeof vi.fn>;
    countRecords?: ReturnType<typeof vi.fn>;
    listRecords?: ReturnType<typeof vi.fn>;
    lookup?: ReturnType<typeof vi.fn>;
    openSearchWindow?: ReturnType<typeof vi.fn>;
    readSearchWindowPage?: ReturnType<typeof vi.fn>;
    search?: ReturnType<typeof vi.fn>;
  } = {},
): Pf2eTerminalAppServices {
  const record = createRecord();
  const countRecords =
    overrides.countRecords ??
    vi.fn(
      async () =>
        ({
          searchProfile: "lexical",
          mode: "lexical" as const,
          total: 1,
        }) satisfies SearchCountResult,
    );
  const listRecords =
    overrides.listRecords ??
    vi.fn((filters: SearchFilters) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: filters.sort ?? "alphabetical",
      total: 1,
      offset: filters.offset ?? 0,
      limit: filters.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }));
  const lookup = overrides.lookup ?? vi.fn(() => ({ match: record, alternatives: [] }));
  const search =
    overrides.search ??
    vi.fn(async (filters: SearchFilters) => ({
      searchProfile: filters.searchProfile ?? "balanced",
      mode: "hybrid" as const,
      sort: filters.sort ?? "ranked",
      total: 1,
      offset: filters.offset ?? 0,
      limit: filters.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }));
  const openSearchWindow =
    overrides.openSearchWindow ??
    vi.fn(async (filters: SearchFilters, options?: { mode?: "browse" | "search" | "lookup" }) => {
      const result = options?.mode === "browse" ? listRecords(filters) : await search(filters);
      return {
        id: "window-1",
        searchProfile: result.searchProfile,
        mode: result.mode,
        sort: result.sort,
        sortSeed: filters.sortSeed ?? null,
        total: result.total,
        offset: result.offset,
        limit: result.limit,
        hasMore: result.hasMore,
        nextOffset: result.nextOffset,
        records: result.records,
      };
    });
  const readSearchWindowPage =
    overrides.readSearchWindowPage ??
    vi.fn((windowId: string, offset: number, limit: number) => ({
      id: windowId,
      searchProfile: "balanced" as const,
      mode: "hybrid" as const,
      sort: "ranked" as const,
      sortSeed: null,
      total: 1,
      offset,
      limit,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }));
  const closeSearchWindow = overrides.closeSearchWindow ?? vi.fn();

  const searchService = createPf2eTerminalSearchService({
    closeSearchWindow,
    countRecords,
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
    openSearchWindow,
    readSearchWindowPage,
    search,
  });

  return {
    config: createTestConfig(),
    catalog: {
      countRecords,
      getRecord: vi.fn(() => record),
      getSearchVocabulary: vi.fn(() => ({}) as never),
      listFilterValues: vi.fn(() => ({ field: "categories", values: [] }) as never),
      listRecords,
      lookup,
      openSearchWindow,
      readSearchWindowPage,
      search,
      closeSearchWindow,
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

function createFacetPickerOntologyDomain(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Test facet picker domain",
    rootNodes: [
      {
        id: "searchSemantics:spell",
        kind: "category",
        label: "Spell",
        shortLabel: "spell",
        filterText: "spell",
        detailTitle: "Search Semantics",
        detailLines: [{ text: "Spell", tone: "section" }],
        children: [
          {
            id: "spell:metadataFields",
            kind: "group",
            label: "Metadata Fields",
            filterText: "metadata fields",
            detailTitle: "Metadata Fields",
            detailLines: [{ text: "Metadata Fields", tone: "section" }],
            children: [
              {
                id: "spell:field:derivedTags",
                kind: "field",
                label: "derivedTags",
                filterText: "derived tags",
                listLabel: "derivedTags",
                detailTitle: "Metadata Field Details",
                detailLines: [{ text: "derivedTags", tone: "section" }],
                childPresentation: {
                  mode: "grouped",
                  groupBy: "axis",
                  render: "inline",
                },
                children: [
                  {
                    id: "spell:family:regional-setting",
                    kind: "family",
                    label: "regional_setting",
                    filterText: "regional setting coastal setting",
                    listLabel: "regional_setting | 1 tag",
                    detailTitle: "Family Details",
                    detailLines: [{ text: "regional_setting", tone: "section" }, { text: "Axis: environment" }],
                    groupValues: {
                      axis: "environment",
                    },
                    children: [
                      {
                        id: "spell:derivedTags:coastal_setting",
                        kind: "tag",
                        label: "coastal_setting",
                        filterText: "coastal setting",
                        listLabel: "coastal_setting",
                        detailTitle: "Tag Details",
                        detailLines: [
                          { text: "coastal_setting", tone: "section" },
                          { text: "Live canonical records: 1" },
                        ],
                      },
                    ],
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  };
}

describe("search screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("supports arrow-driven navigation for editing and executing the query workspace", async () => {
    const search = vi.fn(async (filters: SearchFilters) => ({
      searchProfile: filters.searchProfile ?? "balanced",
      mode: "hybrid" as const,
      sort: filters.sort ?? "ranked",
      total: 1,
      offset: 0,
      limit: filters.limit ?? 50,
      hasMore: false,
      nextOffset: null,
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
    expect(app.lastFrame()).toContain("[SETUP] Scope & Filters");
    expect(app.lastFrame()).toContain("Query Status");
    expect(app.lastFrame()).toContain("Execute Query");
    expect(app.lastFrame()).not.toContain("Profile |");
    expect(app.lastFrame()).not.toContain("Action Cost |");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Mode");
    pressRight(app);
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
    expect(app.lastFrame()).toContain("Query Text");
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
    expect(app.lastFrame()).toContain("Execute Query");
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    expect(search).toHaveBeenCalledWith(
      expect.objectContaining({
        actionCost: undefined,
        category: "spell",
        levelMax: undefined,
        levelMin: undefined,
        metadata: undefined,
        offset: 0,
        query: "ghost",
        rarity: undefined,
        searchProfile: "lexical",
        sort: "ranked",
        sortSeed: undefined,
        subcategory: undefined,
      }),
    );
    expect(search.mock.calls[0]?.[0]?.limit).toBeGreaterThan(50);
    expect(app.lastFrame()).toContain("Current setup matches applied query");
    expect(app.lastFrame()).toContain("1/1 | Buf 1 | Win 1-1");
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 | Buf 1 | Ranked");
    expect(app.lastFrame()).toContain("Alarm Ward | spell | lvl 1");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward");
    expect(app.lastFrame()).not.toContain("Left draft");

    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward");

    app.stdin.write("\u001b[D");
    await flushInk();
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 | Buf 1 | Ranked");

    app.stdin.write("\u001b[D");
    await flushInk();
    expect(app.lastFrame()).toContain("[SETUP] Scope & Filters");
    expect(app.lastFrame()).not.toContain("[RESULTS] 1/1 | Buf 1 | Ranked");
  });

  it("uses left to back out of the setup workspace instead of opening the selected item", async () => {
    const onBack = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={onBack} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Mode");

    pressLeft(app);
    await flushInk();

    expect(onBack).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).not.toContain("Workspace Mode");
  });

  it("treats vim horizontal keys as the same setup and subprompt navigation semantics", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressDown(app);
    await flushInk();

    app.stdin.write("l");
    await flushInk();
    expect(app.lastFrame()).toContain("Workspace Mode");

    app.stdin.write("h");
    await flushInk();
    expect(app.lastFrame()).toContain("[SETUP] Scope & Filters");
  });

  it("opens the shared setup command palette and runs the selected setup action", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Setup Commands");

    for (const character of "mode") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Workspace Mode");
  });

  it("does not treat old page-specific letters as live setup commands", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("[SETUP] Scope & Filters");

    app.stdin.write("c");
    await flushInk();

    expect(app.lastFrame()).toContain("[SETUP] Scope & Filters");
    expect(app.lastFrame()).not.toContain("Category Scope");
  });

  it("uses space to open the selected setup item and carries facet edits back into the workspace", async () => {
    const services = createServices();
    services.user.search.getFacetFieldOptions = vi.fn(() => [
      {
        value: "derivedTags",
        label: "Derived Tags",
        description: "Derived-tag facet for the current browse scope.",
        fieldType: "set",
      },
    ]);
    services.user.ontology.loadDomain = vi.fn((id: string) => {
      if (id === "searchSemantics") {
        return createFacetPickerOntologyDomain();
      }
      return {
        id: "derivedTags",
        label: "Derived Tags",
        description: "Unused test domain",
        rootNodes: [],
      };
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialQuery={{
              kind: "listRecords",
              label: "Browse spells",
              filters: {
                category: "spell",
                limit: 20,
              },
            }}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();

    for (let step = 0; step < 7; step += 1) {
      pressDown(app);
      await flushInk();
    }
    expect(app.lastFrame()).toContain("Edit Facet Filter | 0 active");

    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Facet Picker");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Environment");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Current filters");
    expect(app.lastFrame()).toContain("derivedTags: any=coastal_setting");

    app.stdin.write("q");
    await flushInk();

    expect(app.lastFrame()).toContain("Edit Facet Filter | 1 active");
    expect(app.lastFrame()).toContain("Derived Tags: any: Coastal Setting");
  });

  it("loads the next result page through the window reader instead of rerunning the search", async () => {
    const firstPageRecords = [
      createRecord({ recordKey: "spell:a", id: "a", name: "Alarm Ward" }),
      createRecord({ recordKey: "spell:b", id: "b", name: "Arcane Echo" }),
    ];
    const secondPageRecords = [createRecord({ recordKey: "spell:c", id: "c", name: "Beacon Sigil" })];
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 3,
      offset: 0,
      limit: 2,
      hasMore: true,
      nextOffset: 2,
      records: firstPageRecords,
    }));
    const readSearchWindowPage = vi.fn(() => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 3,
      offset: 0,
      limit: 3,
      hasMore: false,
      nextOffset: null,
      records: [...firstPageRecords, ...secondPageRecords],
    }));
    const search = vi.fn(async () => ({
      searchProfile: "balanced" as const,
      mode: "hybrid" as const,
      sort: "ranked" as const,
      total: 0,
      offset: 0,
      limit: 50,
      hasMore: false,
      nextOffset: null,
      records: [],
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage, search });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    expect(openSearchWindow).toHaveBeenCalledTimes(1);
    expect(readSearchWindowPage).toHaveBeenCalledTimes(1);
    expect(search).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Beacon Sigil");
    expect(app.lastFrame()).toContain("[RESULTS] 1/3 | Buf 3 | Alphabetical");
  });

  it("closes the previous backend window when a new applied session replaces it", async () => {
    const openSearchWindow = vi
      .fn()
      .mockResolvedValueOnce({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 1,
        offset: 0,
        limit: 120,
        hasMore: false,
        nextOffset: null,
        records: [createRecord({ recordKey: "spell:a", id: "a", name: "Alarm Ward" })],
      })
      .mockResolvedValueOnce({
        id: "window-2",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 1,
        offset: 0,
        limit: 120,
        hasMore: false,
        nextOffset: null,
        records: [createRecord({ recordKey: "spell:b", id: "b", name: "Beacon Sigil" })],
      });
    const closeSearchWindow = vi.fn();
    const services = createServices({ openSearchWindow, closeSearchWindow });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    pressLeft(app);
    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    expect(closeSearchWindow).toHaveBeenCalledTimes(1);
    expect(closeSearchWindow).toHaveBeenCalledWith("window-1");
    expect(app.lastFrame()).toContain("Beacon Sigil");
  });

  it("closes the active backend window when applied results are discarded", async () => {
    const closeSearchWindow = vi.fn();
    const services = createServices({ closeSearchWindow });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    pressLeft(app);
    await flushInk();
    app.stdin.write("G");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(closeSearchWindow).toHaveBeenCalledTimes(1);
    expect(closeSearchWindow).toHaveBeenCalledWith("window-1");
    expect(app.lastFrame()).toContain("No applied query yet");
  });

  it("closes the active backend window when the search screen exits", async () => {
    const closeSearchWindow = vi.fn();
    const services = createServices({ closeSearchWindow });
    const onBack = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={onBack} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    app.stdin.write("q");
    await flushInk();

    expect(onBack).toHaveBeenCalledTimes(1);
    expect(closeSearchWindow).toHaveBeenCalledTimes(1);
    expect(closeSearchWindow).toHaveBeenCalledWith("window-1");

    app.unmount();
  });

  it("supports shared gg and G navigation in the result reader", async () => {
    const records = [
      createRecord({ recordKey: "spell:a", id: "a", name: "Alarm Ward" }),
      createRecord({ recordKey: "spell:b", id: "b", name: "Arcane Echo" }),
      createRecord({ recordKey: "spell:c", id: "c", name: "Beacon Sigil" }),
    ];
    const services = createServices({
      openSearchWindow: vi.fn(async () => ({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 3,
        offset: 0,
        limit: 120,
        hasMore: false,
        nextOffset: null,
        records,
      })),
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    app.stdin.write("G");
    await flushInk();
    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Beacon Sigil");

    app.stdin.write("\u001b[D");
    await flushInk();
    app.stdin.write("g");
    await flushInk();
    app.stdin.write("g");
    await flushInk();
    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward");
  });

  it("prefetches the full result set for small totals so paging stays invisible", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const secondPageRecords = Array.from({ length: 80 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index + 120}`,
        id: `${index + 120}`,
        name: `Spell ${index + 120}`,
      }),
    );
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 200,
      offset: 0,
      limit: 120,
      hasMore: true,
      nextOffset: 120,
      records: firstPageRecords,
    }));
    const readSearchWindowPage = vi.fn(() => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 200,
      offset: 0,
      limit: 200,
      hasMore: false,
      nextOffset: null,
      records: [...firstPageRecords, ...secondPageRecords],
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    expect(openSearchWindow).toHaveBeenCalledTimes(1);
    expect(readSearchWindowPage).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).toContain("[RESULTS] 1/200 | Buf 200 | Alphabetical");
  });

  it("carries the applied execution window size forward into later page loads", async () => {
    const services = createServices({
      openSearchWindow: vi.fn(async () => ({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 200,
        offset: 0,
        limit: 120,
        hasMore: true,
        nextOffset: 120,
        records: [createRecord()],
      })),
      readSearchWindowPage: vi.fn(() => ({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 200,
        offset: 120,
        limit: 120,
        hasMore: false,
        nextOffset: null,
        records: [createRecord({ recordKey: "spell:b", id: "b", name: "Beacon Sigil" })],
      })),
    });

    const session = await services.user.search.executeQuery(services.user.search.createDefaultRequest(), {
      sort: "alphabetical",
      limit: 120,
    });
    await services.user.search.loadMore(session);

    expect(session.request.limit).toBe(120);
    expect(services.catalog.readSearchWindowPage).toHaveBeenCalledWith("window-1", 120, 120);
  });

  it("loads enough future pages to restore a wider local result buffer", async () => {
    const services = createServices({
      openSearchWindow: vi.fn(async () => ({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 1000,
        offset: 0,
        limit: 120,
        hasMore: true,
        nextOffset: 120,
        records: Array.from({ length: 120 }, (_, index) =>
          createRecord({
            recordKey: `spell:${index}`,
            id: `${index}`,
            name: `Spell ${index}`,
          }),
        ),
      })),
      readSearchWindowPage: vi.fn((windowId: string, offset: number, limit: number) => ({
        id: windowId,
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 1000,
        offset,
        limit,
        hasMore: offset + limit < 1000,
        nextOffset: offset + limit < 1000 ? offset + limit : null,
        records: Array.from({ length: limit }, (_, index) =>
          createRecord({
            recordKey: `spell:${offset + index}`,
            id: `${offset + index}`,
            name: `Spell ${offset + index}`,
          }),
        ),
      })),
    });

    const session = await services.user.search.executeQuery(services.user.search.createDefaultRequest(), {
      sort: "alphabetical",
      limit: 120,
    });
    const buffered = await services.user.search.loadMore(session, { minimumLoadedCount: 301 });

    expect(buffered.loadedCount).toBe(360);
    expect(buffered.nextOffset).toBe(360);
    expect(services.catalog.readSearchWindowPage).toHaveBeenNthCalledWith(1, "window-1", 120, 120);
    expect(services.catalog.readSearchWindowPage).toHaveBeenNthCalledWith(2, "window-1", 240, 120);
  });

  it("jumps G to the last true result page rather than the end of the loaded prefix", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset: 0,
      limit: 120,
      hasMore: true,
      nextOffset: 120,
      records: firstPageRecords,
    }));
    const readSearchWindowPage = vi.fn((windowId: string, offset: number, limit: number) => ({
      id: windowId,
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset,
      limit,
      hasMore: false,
      nextOffset: null,
      records: Array.from({ length: Math.min(limit, 1000 - offset) }, (_, index) =>
        createRecord({
          recordKey: `spell:${offset + index}`,
          id: `${offset + index}`,
          name: `Spell ${offset + index}`,
        }),
      ),
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    app.stdin.write("G");
    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    const finalWindowCall = readSearchWindowPage.mock.calls.at(-1);
    expect(finalWindowCall?.[0]).toBe("window-1");
    expect((finalWindowCall?.[1] ?? 0) + (finalWindowCall?.[2] ?? 0)).toBe(1000);

    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toMatch(/(\[PREVIEW\]|Preview \|) Spell 999/);
  });

  it("does not keep rereading the same terminal window while moving inside the last page", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset: 0,
      limit: 120,
      hasMore: true,
      nextOffset: 120,
      records: firstPageRecords,
    }));
    const readSearchWindowPage = vi.fn((windowId: string, offset: number, limit: number) => ({
      id: windowId,
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset,
      limit,
      hasMore: false,
      nextOffset: null,
      records: Array.from({ length: Math.min(limit, 1000 - offset) }, (_, index) =>
        createRecord({
          recordKey: `spell:${offset + index}`,
          id: `${offset + index}`,
          name: `Spell ${offset + index}`,
        }),
      ),
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    app.stdin.write("G");
    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    const initialReadCount = readSearchWindowPage.mock.calls.length;

    for (let index = 0; index < 5; index += 1) {
      pressUp(app);
      await flushInk();
    }
    await flushInk();
    await flushInk();

    expect(readSearchWindowPage.mock.calls.length).toBe(initialReadCount);
  });

  it("slides the result window instead of growing it without bound", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset: 0,
      limit: 120,
      hasMore: true,
      nextOffset: 120,
      records: firstPageRecords,
    }));
    const readSearchWindowPage = vi.fn((windowId: string, offset: number, limit: number) => ({
      id: windowId,
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset,
      limit,
      hasMore: true,
      nextOffset: offset + limit,
      records: Array.from({ length: limit }, (_, index) =>
        createRecord({
          recordKey: `spell:${offset + index}`,
          id: `${offset + index}`,
          name: `Spell ${offset + index}`,
        }),
      ),
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    for (let index = 0; index < 30; index += 1) {
      app.stdin.write("\u0004");
      await flushInk();
    }
    await flushDebouncedWindowRead();
    await flushInk();

    expect(readSearchWindowPage.mock.calls.length).toBeGreaterThan(0);
    expect(readSearchWindowPage.mock.calls.every((call) => call[0] === "window-1")).toBe(true);
    expect(readSearchWindowPage.mock.calls.every((call) => (call[2] as number) > 100)).toBe(true);
    const finalWindowSize = readSearchWindowPage.mock.calls.at(-1)?.[2];
    expect(app.lastFrame()).toContain(`| Buf ${finalWindowSize} | Alphabetical`);
  });

  it("coalesces rapid Ctrl-D jumps into a single latest window read", async () => {
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset: 0,
      limit: 120,
      hasMore: true,
      nextOffset: 120,
      records: Array.from({ length: 120 }, (_, index) =>
        createRecord({
          recordKey: `spell:${index}`,
          id: `${index}`,
          name: `Spell ${index}`,
        }),
      ),
    }));
    const readSearchWindowPage = vi.fn((windowId: string, offset: number, limit: number) => ({
      id: windowId,
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1000,
      offset,
      limit,
      hasMore: offset + limit < 1000,
      nextOffset: offset + limit < 1000 ? offset + limit : null,
      records: Array.from({ length: Math.min(limit, 1000 - offset) }, (_, index) =>
        createRecord({
          recordKey: `spell:${offset + index}`,
          id: `${offset + index}`,
          name: `Spell ${offset + index}`,
        }),
      ),
    }));
    const services = createServices({ openSearchWindow, readSearchWindowPage });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    for (let index = 0; index < 80; index += 1) {
      app.stdin.write("\u0004");
    }

    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    expect(readSearchWindowPage).toHaveBeenCalledTimes(1);
    expect(readSearchWindowPage.mock.calls[0]?.[0]).toBe("window-1");
    expect((readSearchWindowPage.mock.calls[0]?.[1] as number) > 0).toBe(true);
  });

  it("parses jump-to-position input into an absolute result index", () => {
    expect(parseJumpToResultInput("600", 1000)).toBe(599);
    expect(parseJumpToResultInput("6,000", 10000)).toBe(5999);
    expect(parseJumpToResultInput("0", 1000)).toBe("Result numbers start at 1.");
    expect(parseJumpToResultInput("1200", 1000)).toBe("Result 1200 is out of range. Valid positions are 1-1000.");
    expect(parseJumpToResultInput("six hundred", 1000)).toBe("Enter a result number such as `6000`.");
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
    expect(
      services.user.search.getFacetFieldOptions("spell", null).some((option) => option.value === "actionCost"),
    ).toBe(true);
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
      sort: filters.sort ?? "ranked",
      total: 1,
      offset: 0,
      limit: filters.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ search });

    await services.user.search.executeQuery({
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
      offset: 0,
      query: "ghost",
      rarity: undefined,
      searchProfile: "balanced",
      sort: "ranked",
      sortSeed: undefined,
      subcategory: undefined,
    });
  });

  it("opens the shared ontology picker for derived-tag facet editing and preserves axis grouping", async () => {
    const services = createServices();
    services.user.search.getFacetFieldOptions = vi.fn(() => [
      {
        value: "derivedTags",
        label: "Derived Tags",
        description: "Derived-tag facet for the current browse scope.",
        fieldType: "set",
      },
    ]);
    services.user.ontology.loadDomain = vi.fn((id: string) => {
      if (id === "searchSemantics") {
        return createFacetPickerOntologyDomain();
      }
      return {
        id: "derivedTags",
        label: "Derived Tags",
        description: "Unused test domain",
        rootNodes: [],
      };
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialQuery={{
              kind: "listRecords",
              label: "Browse spells",
              filters: {
                category: "spell",
                limit: 20,
              },
            }}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();

    expect(app.lastFrame()).toContain("Edit Facet Filter | 0 active");
    for (let step = 0; step < 7; step += 1) {
      pressDown(app);
      await flushInk();
    }
    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Facet Picker");
    expect(app.lastFrame()).toContain("Spell Facet Filters");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Environment");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("q");
    await flushInk();

    expect(app.lastFrame()).toContain("Edit Facet Filter | 1 active");
    expect(app.lastFrame()).toContain("Derived Tags: any: Coastal Setting");
  });
});
