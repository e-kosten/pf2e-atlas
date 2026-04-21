import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig } from "../../src/domain/config-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import { Pf2eTerminalApp, Pf2eTerminalBootstrap } from "../../src/tui/pf2e-app.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createDeferred<T>(): {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (reason?: unknown) => void;
} {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });
  return { promise, resolve, reject };
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

function createFakeServices(overrides: Partial<Pf2eTerminalAppServices> = {}): Pf2eTerminalAppServices {
  const record = createRecord();
  const closeSearchWindow = vi.fn();
  const countRecords = vi.fn(() =>
    Promise.resolve({
      searchProfile: "lexical",
      mode: "lexical",
      total: 1,
    }),
  );
  const listRecords = vi.fn(() => ({
    searchProfile: null,
    mode: "structured" as const,
    sort: "alphabetical" as const,
    total: 1,
    offset: 0,
    limit: 20,
    hasMore: false,
    nextOffset: null,
    records: [record],
  }));
  const lookup = vi.fn(() => ({ match: record, alternatives: [] }));
  const openSearchWindow = vi.fn(() =>
    Promise.resolve({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 1,
      offset: 0,
      limit: 20,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }),
  );
  const readSearchWindowPage = vi.fn(() => ({
    id: "window-1",
    searchProfile: null,
    mode: "structured" as const,
    sort: "alphabetical" as const,
    sortSeed: null,
    total: 1,
    offset: 0,
    limit: 20,
    hasMore: false,
    nextOffset: null,
    records: [record],
  }));
  const search = vi.fn(() =>
    Promise.resolve({
      searchProfile: "balanced" as const,
      mode: "hybrid" as const,
      sort: "ranked" as const,
      total: 1,
      offset: 0,
      limit: 20,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }),
  );
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
            { value: "common", count: 1 },
            { value: "rare", count: 1 },
            { value: "unique", count: 1 },
            { value: "uncommon", count: 1 },
          ],
        };
      }
      if (field === "actionCost") {
        return {
          values: [
            { value: "1", count: 1 },
            { value: "2", count: 1 },
            { value: "3", count: 1 },
          ],
        };
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
      closeSearchWindow,
      countRecords,
      getRecord: vi.fn(() => record),
      getSearchCategorySummary: vi.fn(() => ({
        categories: [{ value: "spell", count: 1 }],
      })),
      getSearchVocabulary: vi.fn(() => ({}) as never),
      listFilterValues: vi.fn(() => ({ field: "categories", values: [] }) as never),
      listRecords,
      lookup,
      openSearchWindow,
      readSearchWindowPage,
      search,
    },
    user: {
      search: searchService,
      ontology: {
        loadSearchSemanticsDomain: vi.fn(() => ({
          id: "searchSemantics",
          label: "Search Semantics",
          description: "Search semantics ontology",
          rootNodes: [
            {
              id: "searchSemantics:spell",
              kind: "category",
              label: "Spell",
              filterText: "spell",
              listLabel: "spell | 1 group",
              detailTitle: "Search Semantics",
              detailLines: [{ text: "Spell", tone: "section" }],
              children: [
                {
                  id: "spell:metadataFields",
                  kind: "group",
                  label: "Metadata Fields",
                  filterText: "metadata fields",
                  listLabel: "Metadata fields | 1",
                  detailTitle: "Metadata Fields",
                  detailLines: [{ text: "Metadata Fields", tone: "section" }],
                  children: [
                    {
                      id: "spell:field:publicationTitle",
                      kind: "field",
                      label: "publicationTitle",
                      filterText: "publication title",
                      listLabel: "publicationTitle",
                      detailTitle: "Metadata Field Details",
                      detailLines: [{ text: "publicationTitle", tone: "section" }],
                      children: [
                        {
                          id: "spell:publicationTitle:pathfinder-player-core",
                          kind: "value",
                          label: "Pathfinder Player Core",
                          filterText: "pathfinder player core",
                          listLabel: "Pathfinder Player Core | 1",
                          detailTitle: "Filter Value",
                          detailLines: [{ text: "Pathfinder Player Core", tone: "section" }],
                          query: {
                            kind: "listRecords",
                            label: "Browse records with this value",
                            filters: {
                              category: "spell",
                              limit: 20,
                            },
                          },
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        })),
      },
    },
    dev: {
      tagRefinement: {
        createSession: vi.fn(() => Promise.reject(new Error("not implemented"))),
        getQueueItems: vi.fn(() => []),
        promptAndCreateSession: vi.fn(() => Promise.resolve(undefined)),
      },
    },
    close: vi.fn(),
    ...overrides,
  };
}

describe("pf2e terminal app", () => {
  afterEach(() => {
    cleanup();
  });

  it("routes from the top level into search and back out", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Choose a first-class TUI area");

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Browse/Search");

    app.stdin.write("q");
    await flushInk();

    expect(app.lastFrame()).toContain("Choose a first-class TUI area");
  });

  it("opens the ontology browser directly in search semantics", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");

    expect(services.user.ontology.loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).toContain("Spell");
  });

  it("uses the dedicated search-semantics ontology loader", async () => {
    const services = createFakeServices();
    const loadSearchSemanticsDomain = vi.fn(() => ({
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          listLabel: "spell | 0 groups",
          detailTitle: "Search Semantics",
          detailLines: [{ text: "Spell", tone: "section" }],
          children: [],
        },
      ],
    }));
    services.user.ontology.loadSearchSemanticsDomain = loadSearchSemanticsDomain;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);
  });

  it("passes shared prompt adapters into custom workbench session creation", async () => {
    const promptAndCreateSession = vi.fn(() => Promise.resolve(undefined));
    const services = createFakeServices();
    services.dev.tagRefinement.promptAndCreateSession = promptAndCreateSession;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Tag Refinement");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(promptAndCreateSession).toHaveBeenCalledWith(process.cwd(), "legacy_seed", expect.any(Object));
    const prompts = promptAndCreateSession.mock.calls[0]![2];
    expect(prompts).toEqual(
      expect.objectContaining({
        promptOptionalSelectOption: expect.any(Function),
        promptSelectOption: expect.any(Function),
        promptTextInput: expect.any(Function),
        pauseForAnyKey: expect.any(Function),
      }),
    );
    expect("exitApp" in prompts).toBe(false);
  });

  it("preserves cancel and error handling for custom workbench session creation", async () => {
    const promptAndCreateSession = vi
      .fn<Pf2eTerminalAppServices["dev"]["tagRefinement"]["promptAndCreateSession"]>()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error("bad scope"));
    const services = createFakeServices();
    services.dev.tagRefinement.promptAndCreateSession = promptAndCreateSession;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Tag Refinement");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Tag Refinement");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Could not create the legacy seed session.");
    expect(app.lastFrame()).toContain("bad scope");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Tag Refinement");
  });

  it("renders grouped return wording on the direct ontology explorer entry", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("\u2190/Esc back");

    app.stdin.write("?");
    await flushInk();

    expect(app.lastFrame()).toContain("\u2190 or h / Backspace / Escape: return to the previous level");
    expect(app.lastFrame()).toContain("q: leave the explorer");
  });

  it("opens the selected top-level area with right-arrow style confirm", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("l");
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");
  });

  it("returns from ontology-launched search to the exact ontology snapshot", async () => {
    const services = createFakeServices();
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => ({
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [
        {
          id: "creature:publicationTitle:monster-core",
          kind: "value",
          label: "Pathfinder Monster Core",
          filterText: "pathfinder monster core monster",
          listLabel: "Pathfinder Monster Core | 320",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Monster Core", tone: "section" }],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              limit: 20,
            },
          },
        },
        {
          id: "creature:publicationTitle:rage-of-elements",
          kind: "value",
          label: "Pathfinder Rage of Elements",
          filterText: "pathfinder rage of elements rage",
          listLabel: "Pathfinder Rage of Elements | 81",
          detailTitle: "Filter Value",
          detailLines: [
            { text: "Pathfinder Rage of Elements", tone: "section" },
            ...Array.from({ length: 30 }, (_, index) => ({ text: `Detail line ${index + 1}` })),
          ],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              limit: 20,
            },
          },
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Monster Core");

    app.stdin.write("j");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");

    app.stdin.write(":");
    await flushInk();
    for (const character of "open query") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    const searchFrame = app.lastFrame();
    expect(searchFrame).toContain("Browse/Search");
    expect(searchFrame).toContain("Category | Creature");
    expect(searchFrame).toContain("Seeded from: Browse records with this value");

    pressLeft(app);
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");
    expect(app.lastFrame()).toContain("Pathfinder Rage of Elements | 81");
  });

  it("opens concrete ontology leaves directly in the shared result reader and returns to the same ontology leaf", async () => {
    const services = createFakeServices();
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => ({
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [
        {
          id: "creature:publicationTitle:monster-core",
          kind: "value",
          label: "Pathfinder Monster Core",
          filterText: "pathfinder monster core monster",
          listLabel: "Pathfinder Monster Core | 320",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Monster Core", tone: "section" }],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              limit: 20,
            },
          },
        },
        {
          id: "creature:publicationTitle:rage-of-elements",
          kind: "value",
          label: "Pathfinder Rage of Elements",
          filterText: "pathfinder rage of elements rage",
          listLabel: "Pathfinder Rage of Elements | 81",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Rage of Elements", tone: "section" }],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              limit: 20,
            },
          },
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Monster Core");

    app.stdin.write("j");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Browse/Search");
    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).not.toContain("[EDITOR] Query");

    pressLeft(app);
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");
    expect(app.lastFrame()).toContain("Pathfinder Rage of Elements | 81");
  });

  it("keeps the ontology explorer mounted while direct-result transitions prepare", async () => {
    const services = createFakeServices();
    const pendingSession = createDeferred<Awaited<ReturnType<typeof services.user.search.executeQuery>>>();
    const preparedQuery = {
      kind: "listRecords" as const,
      label: "Browse records with this value",
      filters: {
        category: "creature" as const,
        limit: 20,
      },
    };
    services.user.search.executeQuery = vi.fn(() =>
      pendingSession.promise,
    ) as typeof services.user.search.executeQuery;
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => ({
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [
        {
          id: "creature:publicationTitle:monster-core",
          kind: "value",
          label: "Pathfinder Monster Core",
          filterText: "pathfinder monster core monster",
          listLabel: "Pathfinder Monster Core | 320",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Monster Core", tone: "section" }],
          query: preparedQuery,
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Pathfinder Monster Core | 320");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    const pendingFrame = app.lastFrame();
    expect(pendingFrame).toContain("Search Semantics");
    expect(pendingFrame).toContain("Explorer Entries");
    expect(pendingFrame).toContain("Loading next view | Loading results for Browse records with this value...");
    expect(pendingFrame).not.toContain("Browse/Search");
    expect(services.user.search.executeQuery).toHaveBeenCalledTimes(1);

    pendingSession.resolve({
      windowId: "window-1",
      query: services.user.search.createQueryFromOntologyQuery(preparedQuery),
      results: [createRecord()],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    });

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Browse/Search");
    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).toContain("Alarm Ward");
  });

  it("keeps the ontology explorer mounted and clears pending status when a direct-result transition fails", async () => {
    const services = createFakeServices();
    const pendingSession = createDeferred<Awaited<ReturnType<typeof services.user.search.executeQuery>>>();
    services.user.search.executeQuery = vi.fn(() =>
      pendingSession.promise,
    ) as typeof services.user.search.executeQuery;
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => ({
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Search semantics ontology",
      rootNodes: [
        {
          id: "creature:publicationTitle:monster-core",
          kind: "value",
          label: "Pathfinder Monster Core",
          filterText: "pathfinder monster core monster",
          listLabel: "Pathfinder Monster Core | 320",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Monster Core", tone: "section" }],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              limit: 20,
            },
          },
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Loading next view | Loading results for Browse records with this value...");

    pendingSession.reject(new Error("index offline"));

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Query execution failed.");
    expect(app.lastFrame()).toContain("index offline");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    const recoveredFrame = app.lastFrame();
    expect(recoveredFrame).toContain("Search Semantics");
    expect(recoveredFrame).toContain("Pathfinder Monster Core | 320");
    expect(recoveredFrame).not.toContain("Loading next view |");
  });

  it("closes loaded services when the bootstrap unmounts", async () => {
    const services = createFakeServices();
    const loadServices = vi.fn(() => Promise.resolve(services));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalBootstrap rootPath={process.cwd()} argv={[]} onExit={vi.fn()} loadServices={loadServices} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    app.unmount();

    expect(loadServices).toHaveBeenCalledTimes(1);
    expect(services.close).toHaveBeenCalledTimes(1);
  });

  it("shows a single exit affordance on startup errors", async () => {
    const loadServices = vi.fn(() => Promise.reject(new Error("boom")));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalBootstrap rootPath={process.cwd()} argv={[]} onExit={vi.fn()} loadServices={loadServices} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Could not load the PF2E app services.");
    expect(app.lastFrame()).toContain("Esc/q exit");
    expect(app.lastFrame()).not.toContain("exit  q exit");
  });
});
