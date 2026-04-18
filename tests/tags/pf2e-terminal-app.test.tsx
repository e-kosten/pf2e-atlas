import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, NormalizedRecord } from "../../src/types.js";
import {
  Pf2eTerminalApp,
  Pf2eTerminalBootstrap,
} from "../../src/tui/pf2e-app.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import type { SearchCategory } from "../../src/types.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
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
  const defaultRequest = {
    mode: "browse" as const,
    limit: 50,
    queryText: "",
    searchProfile: "balanced" as const,
    sourceLabel: null,
    filters: {
      category: null,
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
      facets: [],
    },
  };
  return {
    config: createTestConfig(),
    catalog: {
      countRecords: vi.fn(async () => ({
        searchProfile: "lexical",
        mode: "lexical",
        total: 1,
      })),
      getRecord: vi.fn(() => record),
      getSearchVocabulary: vi.fn(() => ({}) as never),
      listFilterValues: vi.fn(() => ({ field: "categories", values: [] }) as never),
      listRecords: vi.fn(() => ({
        searchProfile: null,
        mode: "structured",
        sort: "alphabetical",
        total: 1,
        offset: 0,
        limit: 20,
        hasMore: false,
        nextOffset: null,
        records: [record],
      })),
      lookup: vi.fn(() => ({ match: record, alternatives: [] })),
      search: vi.fn(async () => ({
        searchProfile: "balanced",
        mode: "hybrid",
        sort: "ranked",
        total: 1,
        offset: 0,
        limit: 20,
        hasMore: false,
        nextOffset: null,
        records: [record],
      })),
    },
    user: {
      search: {
        createDefaultRequest: vi.fn(() => defaultRequest),
        createRequestFromOntologyQuery: vi.fn(() => defaultRequest),
        getActionCostOptions: vi.fn(() => []),
        getCategoryOptions: vi.fn(() => [
          {
            value: null,
            label: "Any Category",
            description: "Search across the full indexed PF2E corpus.",
          },
          {
            value: "spell" satisfies SearchCategory,
            label: "Spell",
            description: "1 indexed canonical record.",
          },
        ]),
        getProfileOptions: vi.fn(() => [
          {
            value: "balanced",
            label: "Balanced",
            description: "Default hybrid retrieval for concise themed searches.",
          },
        ]),
        getRarityOptions: vi.fn(() => []),
        getSubcategoryOptions: vi.fn(() => []),
        getModeOptions: vi.fn(() => [
          {
            value: "browse",
            label: "Browse",
            description: "Deterministic listing.",
          },
        ]),
        getFacetFieldOptions: vi.fn(() => []),
        getFacetValueOptions: vi.fn(() => []),
        getResultSortOptions: vi.fn(() => [
          {
            value: "alphabetical",
            label: "Alphabetical",
            description: "Read matched results in name order.",
          },
        ]),
        normalizeRequest: vi.fn((request) => request),
        getDefaultSort: vi.fn(() => "alphabetical"),
        countQuery: vi.fn(async () => ({
          searchProfile: "lexical",
          mode: "lexical",
          total: 1,
        })),
        executeQuery: vi.fn(async (request) => ({
          request,
          results: [record],
          resultMode: request.mode === "browse" ? "structured" : "hybrid",
          total: 1,
          loadedCount: 1,
          hasMore: false,
          nextOffset: null,
          searchProfile: request.mode === "lookup" ? null : request.searchProfile,
          sort: "alphabetical",
          sortSeed: null,
        })),
        loadMore: vi.fn(async (session) => session),
        changeSort: vi.fn(async (session) => session),
      },
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
        createSession: vi.fn(async () => {
          throw new Error("not implemented");
        }),
        getQueueItems: vi.fn(() => []),
        promptAndCreateSession: vi.fn(async () => undefined),
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

  it("closes loaded services when the bootstrap unmounts", async () => {
    const services = createFakeServices();
    const loadServices = vi.fn(async () => services);
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalBootstrap
          rootPath={process.cwd()}
          argv={[]}
          onExit={vi.fn()}
          loadServices={loadServices}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    app.unmount();

    expect(loadServices).toHaveBeenCalledTimes(1);
    expect(services.close).toHaveBeenCalledTimes(1);
  });
});
