import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { SearchRequest } from "../../src/domain/search-request-types.js";
import type { SearchCountResult } from "../../src/domain/search-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import type { FilterExplorerComposeTarget } from "../../src/tui/filter-explorer/index.js";
import {
  createPf2eTerminalSearchService,
  type Pf2eTerminalSearchSession,
} from "../../src/tui/search/service.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { SearchFilterExplorerScreen } from "../../src/tui/search-screen/filter-explorer-screen.js";
import type { SearchFilterExplorerSession } from "../../src/tui/search-screen/query-field-builder-session.js";
import { getSearchEditorInteractionActions } from "../../src/tui/search-screen/interactions.js";
import { SearchScreen, parseJumpToResultInput } from "../../src/tui/search-screen/screen.js";
import { createInitialSearchScreenState } from "../../src/tui/search-screen/state.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import {
  setSearchQueryActionCostPolicy,
  setSearchQueryMetadataTree,
  setSearchQueryRarityPolicy,
} from "../../src/tui/search/query-state.js";
import { browseQuery, browseRequest, searchRequest } from "../helpers/search-request-fixture.js";

type SearchServiceDependencies = Parameters<typeof createPf2eTerminalSearchService>[0];
type CloseSearchWindowFn = SearchServiceDependencies["closeSearchWindow"];
type CountRecordsFn = SearchServiceDependencies["countRecords"];
type ListRecordsFn = SearchServiceDependencies["listRecords"];
type LookupFn = SearchServiceDependencies["lookup"];
type OpenSearchWindowFn = SearchServiceDependencies["openSearchWindow"];
type ReadSearchWindowPageFn = SearchServiceDependencies["readSearchWindowPage"];
type SearchFn = SearchServiceDependencies["search"];

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

function createSearchSession(
  overrides: Partial<Pf2eTerminalSearchSession> = {},
): Pf2eTerminalSearchSession {
  const query = overrides.query ?? browseRequest({ category: "spell", limit: 20 });
  const results = overrides.results ?? [createRecord()];

  return {
    windowId: "window-1",
    query,
    results,
    windowOffset: 0,
    resultMode: "browse",
    total: results.length,
    loadedCount: results.length,
    hasMore: false,
    nextOffset: null,
    searchProfile: null,
    sort: "alphabetical",
    sortSeed: null,
    ...overrides,
  };
}

function createServices(
  overrides: {
    closeSearchWindow?: CloseSearchWindowFn;
    countRecords?: CountRecordsFn;
    listRecords?: ListRecordsFn;
    lookup?: LookupFn;
    openSearchWindow?: OpenSearchWindowFn;
    readSearchWindowPage?: ReadSearchWindowPageFn;
    search?: SearchFn;
  } = {},
): Pf2eTerminalAppServices {
  const record = createRecord();
  const listFilterValues = vi.fn(({ field }) => {
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
  });
  const discovery = createPf2eApplicationSearchDiscoveryService({
    discoverFilterValues: vi.fn(async (query) => listFilterValues(query)),
    getPack: vi.fn(() => undefined),
    listFilterValues,
  });
  const countRecords: CountRecordsFn =
    overrides.countRecords ??
    vi.fn(() =>
      Promise.resolve({
        searchProfile: "lexical",
        mode: "lexical" as const,
        total: 1,
      } satisfies SearchCountResult),
    );
  const listRecords: ListRecordsFn =
    overrides.listRecords ??
    vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [record],
    }));
  const lookup: LookupFn = overrides.lookup ?? vi.fn(() => ({ match: record, alternatives: [], matchType: "exact" as const }));
  const search: SearchFn =
    overrides.search ??
    vi.fn((request: SearchRequest) =>
      Promise.resolve({
        searchProfile: request.mode === "search" ? request.search.profile ?? "balanced" : null,
        mode: "hybrid" as const,
        sort: "ranked" as const,
        total: 1,
        offset: request.offset ?? 0,
        limit: request.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [record],
      }),
    );
  const openSearchWindow: OpenSearchWindowFn =
    overrides.openSearchWindow ??
    vi.fn(async (request: SearchRequest) => {
      const result = request.mode === "browse" ? listRecords(request) : await search(request);
      return {
        id: "window-1",
        searchProfile: result.searchProfile,
        mode: result.mode,
        sort: result.sort,
        sortSeed: request.mode === "browse" && request.sort?.kind === "random" ? (request.sort.seed ?? null) : null,
        total: result.total,
        offset: result.offset,
        limit: result.limit,
        hasMore: result.hasMore,
        nextOffset: result.nextOffset,
        records: result.records,
      };
    });
  const readSearchWindowPage: ReadSearchWindowPageFn =
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
  const closeSearchWindow: CloseSearchWindowFn = overrides.closeSearchWindow ?? vi.fn();

  const searchService = createPf2eTerminalSearchService({
    closeSearchWindow,
    countRecords,
    discovery,
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
    lookup,
    listRecords,
    openSearchWindow,
    readSearchWindowPage,
    search,
  });

  return {
    config: createTestConfig(),
    user: {
      search: searchService,
      ontology: {
        loadSearchSemanticsDomain: vi.fn(() => createFacetPickerOntologyDomain()),
        loadSearchFilterExplorerDomain: vi.fn(async () => createFacetPickerOntologyDomain()),
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
                label: "Derived Tags",
                filterText: "derived tags",
                listLabel: "Derived Tags",
                detailTitle: "Metadata Field Details",
                detailLines: [{ text: "Derived Tags", tone: "section" }],
                childPresentation: {
                  mode: "grouped",
                  groupBy: "axis",
                  render: "inline",
                },
                children: [
                  {
                    id: "spell:family:coast",
                    kind: "family",
                    label: "coast",
                    filterText: "coast coastal setting",
                    listLabel: "coast | 1 tag",
                    detailTitle: "Family Details",
                    detailLines: [{ text: "coast", tone: "section" }],
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

function createFacetPickerOntologyDomainWithDiscreteFields(): OntologyDomainModel {
  const domain = createFacetPickerOntologyDomain();
  const categoryNode = domain.rootNodes[0];
  const metadataFieldsNode = categoryNode?.children?.[0];
  if (!metadataFieldsNode?.children) {
    return domain;
  }

  metadataFieldsNode.children.unshift(
    {
      id: "spell:field:rarity",
      kind: "field",
      label: "Rarity",
      filterText: "rarity",
      listLabel: "Rarity",
      detailTitle: "Metadata Field Details",
      detailLines: [{ text: "Rarity", tone: "section" }],
      children: [
        {
          id: "spell:field:rarity:value:common",
          kind: "value",
          label: "common",
          filterText: "common",
          listLabel: "common",
          detailTitle: "Value Details",
          detailLines: [{ text: "common", tone: "section" }],
        },
        {
          id: "spell:field:rarity:value:rare",
          kind: "value",
          label: "rare",
          filterText: "rare",
          listLabel: "rare",
          detailTitle: "Value Details",
          detailLines: [{ text: "rare", tone: "section" }],
        },
      ],
    },
    {
      id: "spell:field:actionCost",
      kind: "field",
      label: "actionCost",
      filterText: "action cost",
      listLabel: "actionCost",
      detailTitle: "Metadata Field Details",
      detailLines: [{ text: "actionCost", tone: "section" }],
      children: [
        {
          id: "spell:field:actionCost:value:1",
          kind: "value",
          label: "1 action",
          filterText: "1 action",
          listLabel: "1 action",
          detailTitle: "Value Details",
          detailLines: [{ text: "1 action", tone: "section" }],
        },
        {
          id: "spell:field:actionCost:value:2",
          kind: "value",
          label: "2 actions",
          filterText: "2 actions",
          listLabel: "2 actions",
          detailTitle: "Value Details",
          detailLines: [{ text: "2 actions", tone: "section" }],
        },
      ],
    },
  );

  return domain;
}

function createCreatureDerivedTagsOntologyDomain(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Creature facet picker domain",
    rootNodes: [
      {
        id: "searchSemantics:creature",
        kind: "category",
        label: "Creature",
        shortLabel: "creature",
        filterText: "creature",
        detailTitle: "Search Semantics",
        detailLines: [{ text: "Creature", tone: "section" }],
        children: [
          {
            id: "creature:metadataFields",
            kind: "group",
            label: "Metadata Fields",
            filterText: "metadata fields",
            detailTitle: "Metadata Fields",
            detailLines: [{ text: "Metadata Fields", tone: "section" }],
            children: [
              {
                id: "creature:field:derivedTags",
                kind: "field",
                label: "Derived Tags",
                filterText: "derived tags",
                listLabel: "Derived Tags",
                detailTitle: "Metadata Field Details",
                detailLines: [{ text: "Derived Tags", tone: "section" }],
                childPresentation: {
                  mode: "grouped",
                  groupBy: "axis",
                  render: "inline",
                },
                children: [
                  {
                    id: "creature:family:undead",
                    kind: "family",
                    label: "undead",
                    filterText: "undead creature-type",
                    listLabel: "undead | 1 tag",
                    detailTitle: "Family Details",
                    detailLines: [{ text: "undead", tone: "section" }],
                    groupValues: {
                      axis: "creature-type",
                    },
                    children: [
                      {
                        id: "creature:derivedTags:undead",
                        kind: "tag",
                        label: "undead",
                        filterText: "undead",
                        listLabel: "undead",
                        detailTitle: "Tag Details",
                        detailLines: [{ text: "undead", tone: "section" }],
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

function createCreatureMetricExplorerModel(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Creature Statistics Explorer",
    description: "Metric explorer test domain",
    rootNodes: [
      {
        id: "creature:actorMetrics:namespace:hp.",
        kind: "metricNamespace",
        label: "hp.",
        filterText: "hp hit points",
        listLabel: "hp. | 1 metric",
        detailTitle: "Metric Namespace",
        detailLines: [{ text: "hp.", tone: "section" }],
        children: [
          {
            id: "creature:actorMetrics:hp.value",
            kind: "metric",
            label: "Hit Points",
            filterText: "hp hit points",
            listLabel: "Hit Points | 4",
            detailTitle: "Metric Details",
            detailLines: [{ text: "Hit Points", tone: "section" }],
            query: browseQuery("Browse records with Hit Points", {
                category: "creature",
                metadata: {
                  field: "actorMetricCompare",
                  leftMetric: "hp.value",
                  op: ">=",
                  rightMetric: "hp.value",
                },
                limit: 20,
              }),
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

  it("does not expose the text-query shortcut while browsing", () => {
    const browseState = createInitialSearchScreenState(browseRequest({ limit: 20 }));
    const searchState = createInitialSearchScreenState(searchRequest({ query: "ghost", limit: 20 }));

    expect(getSearchEditorInteractionActions(browseState).some((action) => action.id === "search")).toBe(false);
    expect(getSearchEditorInteractionActions(searchState).some((action) => action.id === "search")).toBe(true);
  });

  it("does not append shared transition footer state twice in the result-reader host", async () => {
    const services = createServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            onBack={vi.fn()}
            transitionStatus={{
              kind: ROUTE_TRANSITION_STATUS_KIND.PENDING,
              message: "Loading results",
              frame: 0,
            }}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    const matches = app.lastFrame().match(/Loading next view \|/g) ?? [];
    expect(matches).toHaveLength(1);
  });

  it("supports arrow-driven navigation for editing and executing the query workspace", async () => {
    const search = vi.fn((request: SearchRequest) =>
      Promise.resolve({
        searchProfile: request.searchProfile ?? "balanced",
        mode: "hybrid" as const,
        sort: request.sort ?? "ranked",
        total: 1,
        offset: 0,
        limit: request.limit ?? 50,
        hasMore: false,
        nextOffset: null,
        records: [createRecord()],
      }),
    );
    const services = createServices({ search });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Browse | Any Category | Counting matches...");
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Query Status");
    expect(app.lastFrame()).toContain("Execute Query");
    expect(app.lastFrame()).not.toContain("Profile |");
    expect(app.lastFrame()).not.toContain("Action Cost |");
    expect(app.lastFrame()).toContain("Filters > | None yet");
    expect(app.lastFrame()).toContain("Mode");
    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Query Mode");
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
    expect(app.lastFrame()).toContain("Exclude");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Exclude Text");
    for (const character of "skeleton") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    app.stdin.write("\t");
    await flushInk();
    await flushInk();

    expect(search).toHaveBeenCalledWith(
      expect.objectContaining({
        mode: "search",
        offset: 0,
        search: {
          exclude: "skeleton",
          query: "ghost",
          profile: "balanced",
        },
      }),
    );
    expect(search.mock.calls[0]?.[0]?.limit).toBeGreaterThan(50);
    expect(app.lastFrame()).toContain("Current editor matches applied query");
    expect(app.lastFrame()).toContain("1/1 | Buf 1 | Win 1-1");
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 | Buf 1 | Ranked");
    expect(app.lastFrame()).toContain("Alarm Ward | L1 | Spells");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward");
    expect(app.lastFrame()).not.toContain("Left staged");

    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[RESULTS] 1/1 | Buf 1 | Ranked");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward");
    expect(app.lastFrame()).not.toContain("[PREVIEW] Alarm Ward");
    expect(app.lastFrame()).toContain("Preview is already visible.");

    app.stdin.write("\u001b[D");
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).not.toContain("[RESULTS] 1/1 | Buf 1 | Ranked");
  });

  it("uses left to back out of the query editor instead of opening the selected item", async () => {
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
    expect(app.lastFrame()).not.toContain("Query Mode");
  });

  it("treats vim horizontal keys as the same editor and subprompt navigation semantics", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("l");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Mode");

    app.stdin.write("h");
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
  });

  it("opens the shared query editor command palette and runs the selected editor action", async () => {
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
    expect(app.lastFrame()).toContain("Query Editor Commands");

    for (const character of "mode") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Query Mode");
  });

  it("hides unavailable editor commands from the palette while leaving the editor row visible", async () => {
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
    expect(app.lastFrame()).toContain("Query Editor Commands");

    for (const character of "subcategory") {
      app.stdin.write(character);
    }
    await flushInk();
    expect(app.lastFrame()).toContain("No commands match the current filter.");
    expect(app.lastFrame()).not.toContain("Subcategory Scope");
    expect(app.lastFrame()).not.toContain("This command is currently unavailable.");

    pressLeft(app);
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filters > | None yet");
    expect(app.lastFrame()).not.toContain("Subcategory |");
  });

  it("does not treat old page-specific letters as live editor commands", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");

    app.stdin.write("c");
    await flushInk();

    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).not.toContain("Category Scope");
  });

  it("uses space to open add-query-part and keeps derived-tag composition on the shared explorer path", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "derivedTags",
        label: "Derived Tags",
        description: "Derived-tag query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createFacetPickerOntologyDomain());

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse spells", {
              actionCost: 2,
              category: "spell",
              limit: 20,
              levelMax: 1,
              levelMin: 1,
              rarity: "common",
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    for (let step = 0; step < 1; step += 1) {
      pressDown(app);
      await flushInk();
    }
    expect(app.lastFrame()).toContain("Filters >");

    app.stdin.write(" ");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("allOf");
    expect(app.lastFrame()).toContain("[+ add here]");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Field filter");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Field Filter");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).toContain("Focused node is not selectable.");
    expect(app.lastFrame()).not.toContain("Query Field\n");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags > Coast");
    expect(app.lastFrame()).toContain("coast | 1 tag");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("coastal_setting");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("include any");
  });

  it("loads the next result page through the window reader instead of rerunning the search", async () => {
    const firstPageRecords = [
      createRecord({ recordKey: "spell:a", id: "a", name: "Alarm Ward" }),
      createRecord({ recordKey: "spell:b", id: "b", name: "Arcane Echo" }),
    ];
    const secondPageRecords = [createRecord({ recordKey: "spell:c", id: "c", name: "Beacon Sigil" })];
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    const search = vi.fn(() =>
      Promise.resolve({
        searchProfile: "balanced" as const,
        mode: "hybrid" as const,
        sort: "ranked" as const,
        total: 0,
        offset: 0,
        limit: 50,
        hasMore: false,
        nextOffset: null,
        records: [],
      }),
    );
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
    pressUp(app);
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
      openSearchWindow: vi.fn(() =>
        Promise.resolve({
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
        }),
      ),
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
    expect(app.lastFrame()).toContain("[RESULTS] 3/3");
    expect(app.lastFrame()).toContain("Preview | Beacon Sigil");
    expect(app.lastFrame()).not.toContain("[PREVIEW] Beacon Sigil");
    expect(app.lastFrame()).toContain("Preview is already visible.");

    app.stdin.write("g");
    await flushInk();
    app.stdin.write("g");
    await flushInk();
    pressRight(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[RESULTS] 1/3");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward");
    expect(app.lastFrame()).not.toContain("[PREVIEW] Alarm Ward");
    expect(app.lastFrame()).toContain("Preview is already visible.");
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
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    const readSearchWindowPage = vi.fn(() => ({
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
    }));
    const services = createServices({
      openSearchWindow: vi.fn(() =>
        Promise.resolve({
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
        }),
      ),
      readSearchWindowPage,
    });

    const session = await services.user.search.executeQuery(services.user.search.createDefaultQuery(), {
      sort: "alphabetical",
      limit: 120,
    });
    await services.user.search.loadMore(session);

    expect(session.query.limit).toBe(120);
    expect(readSearchWindowPage).toHaveBeenCalledWith("window-1", 120, 120);
  });

  it("loads enough future pages to restore a wider local result buffer", async () => {
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
      records: Array.from({ length: limit }, (_, index) =>
        createRecord({
          recordKey: `spell:${offset + index}`,
          id: `${offset + index}`,
          name: `Spell ${offset + index}`,
        }),
      ),
    }));
    const services = createServices({
      openSearchWindow: vi.fn(() =>
        Promise.resolve({
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
        }),
      ),
      readSearchWindowPage,
    });

    const session = await services.user.search.executeQuery(services.user.search.createDefaultQuery(), {
      sort: "alphabetical",
      limit: 120,
    });
    const buffered = await services.user.search.loadMore(session, { minimumLoadedCount: 301 });

    expect(buffered.loadedCount).toBe(360);
    expect(buffered.nextOffset).toBe(360);
    expect(readSearchWindowPage).toHaveBeenNthCalledWith(1, "window-1", 120, 120);
    expect(readSearchWindowPage).toHaveBeenNthCalledWith(2, "window-1", 240, 120);
  });

  it("jumps G to the last true result page rather than the end of the loaded prefix", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    expect(app.lastFrame()).toContain("[RESULTS] 1,000/1,000");
    expect(app.lastFrame()).toContain("Preview | Spell 999");
    expect(app.lastFrame()).not.toContain("[PREVIEW] Spell 999");
    expect(app.lastFrame()).toContain("Preview is already visible.");
  });

  it("treats rightward result navigation as a no-op when no result is selected", async () => {
    const initialSession = createSearchSession({
      results: [],
      total: 0,
      loadedCount: 0,
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen entry="results" initialSession={initialSession} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    const before = app.lastFrame();

    pressRight(app);
    await flushInk();

    expect(app.lastFrame()).toBe(before);
    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).not.toContain("[PREVIEW]");
  });

  it("does not keep rereading the same terminal window while moving inside the last page", async () => {
    const firstPageRecords = Array.from({ length: 120 }, (_, index) =>
      createRecord({
        recordKey: `spell:${index}`,
        id: `${index}`,
        name: `Spell ${index}`,
      }),
    );
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    expect(readSearchWindowPage.mock.calls.every((call) => call[2] > 100)).toBe(true);
    const finalWindowSize = readSearchWindowPage.mock.calls.at(-1)?.[2];
    expect(app.lastFrame()).toContain(`| Buf ${finalWindowSize} | Alphabetical`);
  });

  it("coalesces rapid Ctrl-D jumps into a single latest window read", async () => {
    const openSearchWindow = vi.fn(() =>
      Promise.resolve({
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
      }),
    );
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
    const request = services.user.search.createQueryFromOntologyQuery(
      browseQuery("Browse records with this trait", {
        category: "spell",
        metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
        limit: 20,
      }),
    );

    expect(request).toEqual(
      browseQuery("Browse records with this trait", {
        category: "spell",
        metadata: {
          field: "traits",
          op: "includesAny",
          values: ["illusion"],
        },
        limit: 20,
      }).request,
    );
  });

  it("shows seeded metadata clauses in the query editor when launched from ontology", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen
            initialRequest={browseQuery("Browse illusion spells", {
              category: "spell",
              metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
              limit: 20,
            }).request}
            origin="ontology"
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filters > | 2 active");
    expect(app.lastFrame()).toContain("Filter | Scope: Spell");
    expect(app.lastFrame()).toContain("Filter | Traits: includes any Illusion");
  });

  it("does not auto-execute seeded route entry without a prepared session", async () => {
    const openSearchWindow = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices({ openSearchWindow })}>
          <SearchScreen
            initialRequest={browseQuery("Browse illusion spells", {
              category: "spell",
              metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
              limit: 20,
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).not.toContain("[RESULTS]");
    expect(openSearchWindow).not.toHaveBeenCalled();
  });

  it("opens preloaded ontology direct-result sessions immediately", async () => {
    const openSearchWindow = vi.fn();
    const initialSession = createSearchSession({
      results: [createRecord({ recordKey: "spell:veil", id: "veil", name: "Illusory Veil" })],
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices({ openSearchWindow })}>
          <SearchScreen
            entry="results"
            initialSession={initialSession}
            origin="ontology"
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).toContain("Illusory Veil");
    expect(app.lastFrame()).not.toContain("[EDITOR] Query");
    expect(openSearchWindow).not.toHaveBeenCalled();
  });

  it("does not re-execute ontology direct-result launches when a prepared session is already present", async () => {
    const openSearchWindow = vi.fn();
    const search = vi.fn();
    const initialSession = createSearchSession({
      results: [createRecord({ recordKey: "spell:veil", id: "veil", name: "Illusory Veil" })],
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices({ openSearchWindow, search })}>
          <SearchScreen
            entry="results"
            initialSession={initialSession}
            origin="ontology"
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();
    await flushDebouncedWindowRead();
    await flushInk();

    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).toContain("Illusory Veil");
    expect(openSearchWindow).not.toHaveBeenCalled();
    expect(search).not.toHaveBeenCalled();
  });

  it("translates policy-based query filters into metadata clauses", async () => {
    const search = vi.fn((request: SearchRequest) =>
      Promise.resolve({
        searchProfile: request.mode === "search" ? request.search.profile ?? "balanced" : null,
        mode: "hybrid" as const,
        sort: "ranked" as const,
        total: 1,
        offset: 0,
        limit: request.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [createRecord()],
      }),
    );
    const services = createServices({ search });

    await services.user.search.executeQuery(
      setSearchQueryMetadataTree(
        setSearchQueryActionCostPolicy(
          setSearchQueryRarityPolicy(
            searchRequest({
              category: "spell",
              limit: 20,
              query: "ghost",
              searchProfile: "balanced",
            }),
            {
              any: ["common"],
              all: [],
              exclude: ["rare"],
            },
          ),
          {
            any: [2],
            all: [],
            exclude: [1],
          },
        ),
        {
          and: [
            {
              field: "traits",
              op: "includesAny",
              values: ["illusion"],
            },
            {
              field: "traits",
              op: "includesAll",
              values: ["auditory"],
            },
            {
              field: "traits",
              op: "excludesAny",
              values: ["emotion"],
            },
          ],
        },
      ),
    );

    expect(search).toHaveBeenCalledWith({
      mode: "search",
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "spell",
            subcategory: { kind: "any" },
          },
          {
            kind: "allOf",
            children: [
              {
                kind: "rarity",
                match: { kind: "eq", value: "common" },
              },
              {
                kind: "not",
                child: {
                  kind: "rarity",
                  match: { kind: "eq", value: "rare" },
                },
              },
            ],
          },
          {
            kind: "allOf",
            children: [
              {
                kind: "actionCost",
                match: { kind: "eq", value: 2 },
              },
              {
                kind: "not",
                child: {
                  kind: "actionCost",
                  match: { kind: "eq", value: 1 },
                },
              },
            ],
          },
          {
            kind: "allOf",
            children: [
              {
                kind: "metadataPredicate",
                predicate: { field: "traits", op: "includes", value: "illusion" },
              },
              {
                kind: "metadataPredicate",
                kind: "allOf",
                children: [
                  {
                    kind: "metadataPredicate",
                    predicate: { field: "traits", op: "includes", value: "auditory" },
                  },
                ],
              },
              {
                kind: "not",
                child: {
                  kind: "metadataPredicate",
                  predicate: { field: "traits", op: "includes", value: "emotion" },
                },
              },
            ],
          },
        ],
      },
      limit: 20,
      offset: 0,
      search: {
        query: "ghost",
        profile: "balanced",
      },
      explain: false,
    });
  });

  it("applies the staged query when returning from the top-level structured editor", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "derivedTags",
        label: "Derived Tags",
        description: "Derived-tag query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createFacetPickerOntologyDomain());

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse spells", {
              actionCost: 2,
              category: "spell",
              limit: 20,
              levelMax: 1,
              levelMin: 1,
              rarity: "common",
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    for (let step = 0; step < 1; step += 1) {
      pressDown(app);
      await flushInk();
    }

    expect(app.lastFrame()).toContain("Filters >");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("allOf");
    expect(app.lastFrame()).toContain("[+ add here]");
    expect(app.lastFrame()).toContain("q return");

    app.stdin.write("?");
    await flushInk();
    expect(app.lastFrame()).toContain("q: apply the staged structured query and return to the live editor");
    expect(app.lastFrame()).toContain("Use Left or Esc to apply the staged query and return to the top editor.");

    app.stdin.write("x");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Field filter");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Field Filter");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).toContain("Focused node is not selectable.");
    expect(app.lastFrame()).not.toContain("Query Field\n");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags > Coast");
    expect(app.lastFrame()).toContain("coast | 1 tag");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("coastal_setting");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("include any");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).toContain("Selected fields");
    expect(app.lastFrame()).toContain("coastal_setting");

    pressLeft(app);
    await flushInk();
    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).not.toContain("Browse/Search");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filters > | 5 active");
    expect(app.lastFrame()).toContain("Top-level filters: 5");
    expect(app.lastFrame()).toContain("Metadata predicates: 1");
    expect(app.lastFrame()).toContain("Filter | Derived Tags: includes any Coas");
    expect(app.lastFrame()).not.toContain("Structured Query Editor");
  });

  it("opens the shared explorer for staged rarity and action-cost rows", async () => {
    const services = createServices();
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(
      async () => createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse spells", {
              actionCost: 2,
              category: "spell",
              limit: 20,
              levelMax: 1,
              levelMin: 1,
              rarity: "common",
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    for (let step = 0; step < 1; step += 1) {
      pressDown(app);
      await flushInk();
    }

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Action Cost: 2");

    for (let step = 0; step < 2; step += 1) {
      pressUp(app);
      await flushInk();
    }
    expect(app.lastFrame()).toContain("Rarity: Common");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Rarity Explorer");
    expect(app.lastFrame()).toContain("common");
    expect(app.lastFrame()).toContain("rare");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");

    expect(app.lastFrame()).toContain("Action Cost: 2");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Action Cost Explorer");
    expect(app.lastFrame()).toContain("1 action");
    expect(app.lastFrame()).toContain("2 actions");
  });

  it("defaults structured-draft shared explorers to matching counts and can switch to catalog counts", async () => {
    const services = createServices();
    const loadSearchFilterExplorerDomain = vi.fn(
      async ({ discoveryMode }: { discoveryMode: "matching" | "catalog" }) => {
        const domain = createFacetPickerOntologyDomainWithDiscreteFields();
        const metadataFields = domain.rootNodes[0]?.children?.[0];
        const rarityField = metadataFields?.children?.find((node) => node.id === "spell:field:rarity");
        if (rarityField?.children?.[0]) {
          rarityField.children[0] = {
            ...rarityField.children[0],
            listLabel: discoveryMode === "matching" ? "Common | 1" : "Common | 3",
            detailLines: [{ text: discoveryMode === "matching" ? "Matching records: 1" : "Applicable records: 3" }],
          };
        }
        return domain;
      },
    );
    services.user.ontology.loadSearchFilterExplorerDomain = loadSearchFilterExplorerDomain;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse spells", {
              actionCost: 2,
              category: "spell",
              limit: 20,
              levelMax: 1,
              levelMin: 1,
              rarity: "common",
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    pressDown(app);
    await flushInk();

    app.stdin.write("\r");
    await flushInk();

    for (let step = 0; step < 2; step += 1) {
      pressUp(app);
      await flushInk();
    }

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Rarity Explorer");
    expect(app.lastFrame()).toContain("matching counts");
    expect(loadSearchFilterExplorerDomain).toHaveBeenCalledWith({
      discoveryMode: "matching",
      request: expect.objectContaining({
        mode: "browse",
      }),
    });

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Rarity Explorer Commands");
    for (const character of "catalog") {
      app.stdin.write(character);
      await flushInk();
    }
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("catalog counts");
    expect(loadSearchFilterExplorerDomain).toHaveBeenCalledWith({
      discoveryMode: "catalog",
      request: expect.objectContaining({
        mode: "browse",
      }),
    });
  });

  it("routes staged ontology composition through clause-kind and field pickers before opening the shared explorer", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
      {
        value: "derivedTags",
        label: "Derived Tags",
        description: "Derived-tag query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => {
      const domain = createFacetPickerOntologyDomain();
      const metadataFields = domain.rootNodes[0]?.children?.[0];
      if (metadataFields?.children) {
        metadataFields.children.unshift({
          id: "spell:field:traits",
          kind: "field",
          label: "Traits",
          filterText: "traits",
          listLabel: "Traits",
          detailTitle: "Metadata Field Details",
          detailLines: [{ text: "Traits", tone: "section" }],
          childPresentation: {
            mode: "grouped",
            groupBy: "family",
            render: "inline",
          },
          children: [
            {
              id: "spell:traits:illusion",
              kind: "trait",
              label: "illusion",
              filterText: "illusion",
              listLabel: "illusion",
              detailTitle: "Trait Details",
              detailLines: [{ text: "illusion", tone: "section" }],
              groupValues: {
                family: "magic",
              },
              selection: {
                field: "traits",
                fieldLabel: "Traits",
                value: "illusion",
                allowedStates: ["any", "all", "exclude"],
              },
            },
          ],
        });
      }

      return domain;
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse spells", {
              actionCost: 2,
              category: "spell",
              limit: 20,
              levelMax: 1,
              levelMin: 1,
              rarity: "common",
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    for (let step = 0; step < 1; step += 1) {
      pressDown(app);
      await flushInk();
    }

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("allOf");
    expect(app.lastFrame()).toContain("[+ add here]");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Field filter");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Field Filter");
    expect(app.lastFrame()).toContain("Traits");
    expect(app.lastFrame()).toContain("Derived Tags");
    expect(app.lastFrame()).not.toContain("Filters >");
    expect(app.lastFrame()).not.toContain("Browse/Search");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Traits Explorer > Illusion");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("illusion");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("include any");
    expect(app.lastFrame()).toContain("Traits:");
    expect(app.lastFrame()).toContain("illusion");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Filter: Traits: includes any Illusion");
    expect(app.lastFrame()).not.toContain("Filters >");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filter | Traits: includes any Illusion");
  });

  it("scopes ontology-backed query fields from the staged category instead of the live query", async () => {
    const services = createServices();
    services.user.search.getCategoryOptions = vi.fn(() => [
      { value: null, label: "Any Category", description: "Browse every category." },
      { value: "creature", label: "Creature", description: "Browse creatures." },
    ]);
    services.user.search.getQueryFieldOptions = vi.fn((category) =>
      category === "creature"
        ? [
            {
              value: "derivedTags",
              label: "Derived Tags",
              description: "Derived-tag query field for the current creature scope.",
              fieldType: "set",
              editor: "sharedExplorer",
            },
          ]
        : [],
    );
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createCreatureDerivedTagsOntologyDomain());

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    for (let step = 0; step < 1; step += 1) {
      pressDown(app);
      await flushInk();
    }

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("allOf");
    expect(app.lastFrame()).toContain("[+ add here]");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Scope");

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Scope");
    expect(app.lastFrame()).toContain("Creature");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Subcategory Mode");

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Scope: Creature");
    expect(app.lastFrame()).toContain("allOf");

    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Field filter");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Field Filter");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).toContain("Focused node is not selectable.");
    expect(app.lastFrame()).not.toContain("Choose a category before editing a discoverable query field.");
  });

  it("covers metric comparison through the dedicated structured-editor clause-kind flow", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live statistic keys and author exact or numeric literal filters for the current scope.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);
    services.user.search.getMetricKeyOptions = vi.fn((category, subcategory, field) =>
      field === "actorMetric"
        ? [
            {
              value: "hp.value",
              label: "hp.value",
              description: "2 indexed canonical records in the current scope.",
              count: 2,
            },
            {
              value: "ac.value",
              label: "ac.value",
              description: "1 indexed canonical record in the current scope.",
              count: 1,
            },
          ]
        : [],
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse creatures", {
              category: "creature",
              limit: 20,
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    pressDown(app);
    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metric filter");
    expect(app.lastFrame()).toContain("Metric comparison");

    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metric Comparison");
    expect(app.lastFrame()).toContain("Creature Statistics");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Left Metric");
    expect(app.lastFrame()).toContain("hp.value");
    expect(app.lastFrame()).toContain("ac.value");

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Comparison Operator");

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Right Metric");

    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Creature Statistics: hp.value gte ac.value");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Metric Comparison");
  });

  it("covers pack clauses through the dedicated structured-editor clause-kind flow", async () => {
    const services = createServices();
    services.user.search.getPackOptions = vi.fn(() => [
      {
        value: "pathfinder-npc-core",
        label: "Pathfinder NPC Core",
        description: "4 indexed canonical records in this pack.",
        count: 4,
      },
      {
        value: "monster-core",
        label: "Monster Core",
        description: "2 indexed canonical records in this pack.",
        count: 2,
      },
    ]);
    services.user.search.getPackLabel = vi.fn((packValue: string) =>
      packValue === "pathfinder-npc-core"
        ? "Pathfinder NPC Core"
        : packValue === "monster-core"
          ? "Monster Core"
          : packValue,
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={browseQuery("Browse creatures", {
              category: "creature",
              limit: 20,
            }).request}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    pressLeft(app);
    await flushInk();
    pressDown(app);
    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Insertion Slot");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Pack");

    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Pack");
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");
    expect(app.lastFrame()).toContain("Monster Core");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Pack: Pathfinder NPC Core");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Pack");
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");
  });

  it("opens the numeric scalar editor when compose-mode creature statistics focus a metric key", async () => {
    const model = createCreatureMetricExplorerModel();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const session: SearchFilterExplorerSession = {
      title: "Creature Statistics Explorer",
      model,
      draft: {
        selection: {},
        scalarClauses: {},
      },
      resolveSelectionTarget: (node: OntologyNode | undefined): FilterExplorerComposeTarget | undefined =>
        node?.kind === "metric"
          ? {
              kind: "scalar",
              key: "actorMetric:hp.value",
              fieldLabel: "Creature Statistics",
              subjectLabel: node.label,
              valueType: "number",
              editorLabel: `Creature Statistics / ${node.label}`,
            }
          : undefined,
      onApply: () => {},
    };
    const searchFilterExplorerElement = React.createElement(SearchFilterExplorer, { session });
    const app = render(<DerivedTagTerminalProvider>{searchFilterExplorerElement}</DerivedTagTerminalProvider>);

    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("hp.");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Hit Points");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics / Hit Points");
    expect(app.lastFrame()).toContain("Selected: Equals");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Enter a numeric value. Leave blank to clear.");
    await flushInk();

    for (const character of "12") {
      app.stdin.write(character);
      await flushInk();
    }
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Hit Points");
    expect(app.lastFrame()).toContain("= 12");
  });
});
