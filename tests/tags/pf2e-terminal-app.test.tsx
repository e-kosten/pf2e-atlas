import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, NormalizedRecord } from "../../src/domain/index.js";
import { Pf2eTerminalApp, Pf2eTerminalBootstrap } from "../../src/tui/pf2e-app.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
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
        listDomains: vi.fn(() => [
          {
            id: "derivedTags",
            label: "Derived Tags",
            description: "Derived tag ontology",
          },
          {
            id: "catalogCategories",
            label: "Categories",
            description: "Category ontology",
          },
        ]),
        loadDomain: vi.fn(() => ({
          id: "derivedTags",
          label: "Derived Tags",
          description: "Derived tag ontology",
          rootNodes: [
            {
              id: "spell",
              kind: "category",
              label: "Spell",
              filterText: "spell",
              listLabel: "spell | 1 family | 1 tag | 1 record",
              detailTitle: "Category Details",
              detailLines: [{ text: "Spell", tone: "section" }],
              children: [
                {
                  id: "spell:security",
                  kind: "family",
                  label: "security",
                  filterText: "security",
                  listLabel: "security | 1 tag | 1 live record",
                  detailTitle: "Family Details",
                  detailLines: [{ text: "security", tone: "section" }],
                  groupValues: { axis: "utility" },
                  children: [
                    {
                      id: "spell:alarm",
                      kind: "tag",
                      label: "alarm",
                      filterText: "alarm",
                      listLabel: "alarm | editorial | 1 live record",
                      detailTitle: "Tag Details",
                      detailLines: [{ text: "alarm", tone: "section" }],
                      children: [
                        {
                          id: record.recordKey,
                          kind: "record",
                          label: record.name,
                          filterText: "alarm ward",
                          listLabel: `${record.name} | spell | lvl 1`,
                          detailTitle: "Record Details",
                          detailLines: [{ text: record.name, tone: "section" }],
                        },
                      ],
                    },
                  ],
                },
              ],
              childPresentation: {
                mode: "grouped",
                groupBy: "axis",
                render: "inline",
              },
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

  it("opens the ontology domain picker and loads the selected domain", async () => {
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
    expect(app.lastFrame()).toContain("Choose an ontology-backed browse domain");

    app.stdin.write("\r");
    await flushInk();

    expect(services.user.ontology.listDomains).toHaveBeenCalled();
    expect(services.user.ontology.loadDomain).toHaveBeenCalledWith("derivedTags");
    expect(app.lastFrame()).toContain("Derived Tags");
  });

  it("renders grouped return wording on the ontology domain picker", async () => {
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

    expect(app.lastFrame()).toContain("Esc/Backspace/\u2190/q return");

    app.stdin.write("?");
    await flushInk();

    expect(app.lastFrame()).toContain("Escape / q / \u2190 or h / Backspace: return to the previous area");
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

    expect(app.lastFrame()).toContain("Choose an ontology-backed browse domain");
  });

  it("returns from ontology-launched search to the exact ontology snapshot", async () => {
    const services = createFakeServices();
    services.user.ontology.listDomains = vi.fn(() => [
      {
        id: "searchSemantics",
        label: "Search Semantics",
        description: "Search semantics ontology",
      },
    ]);
    services.user.ontology.loadDomain = vi.fn(() => ({
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
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Monster Core | depth 0");

    app.stdin.write("j");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements | depth 0");

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

    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements | depth 0");
    expect(app.lastFrame()).toContain("Pathfinder Rage of Elements | 81");
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
