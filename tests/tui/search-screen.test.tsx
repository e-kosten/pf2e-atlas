import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { EntityPageDocument } from "../../src/app/ontology/entity-page.js";
import type { SearchRequest } from "../../src/domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../src/domain/search-field-domains.js";
import type { SearchCountResult } from "../../src/domain/search-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import { createPf2eApplicationEntityPageService } from "../../src/app/ontology/entity-page-service.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
  type FilterExplorerComposeTarget,
} from "../../src/tui/filter-explorer/index.js";
import { createPf2eTerminalSearchService, type Pf2eTerminalSearchSession } from "../../src/tui/search/service.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import {
  createNoopTerminalDebugTraceService,
  createTerminalDebugTraceService,
} from "../../src/tui/debug-trace.js";
import { SearchFilterExplorerScreen } from "../../src/tui/search-screen/filter-explorer-screen.js";
import { createSearchFilterExplorerLoadingModel } from "../../src/tui/search-screen/filter-explorer-loading-model.js";
import { reconcileSearchFilterExplorerModel } from "../../src/tui/search-screen/filter-explorer-model-reconciliation.js";
import type { SearchFilterExplorerSession } from "../../src/tui/search-screen/query-field-builder-session.js";
import { getOntologyNodeChildren } from "../../src/app/ontology/node-helpers.js";
import { getSearchEditorInteractionActions } from "../../src/tui/search-screen/interactions.js";
import { buildMetricSelectionTargetResolver } from "../../src/tui/search-screen/structured-draft/structured-draft-explorer-actions.js";
import { buildGroupedFieldSeedState } from "../../src/tui/search-screen/structured-draft/structured-draft-grouped-field.js";
import { SearchScreen, parseJumpToResultInput } from "../../src/tui/search-screen/screen.js";
import { createInitialSearchScreenState } from "../../src/tui/search-screen/state.js";
import { ROUTE_TRANSITION_STATUS_KIND } from "../../src/tui/route-transition-status.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import {
  getSearchQueryMetadataTree,
  getSearchQueryPackSelection,
  setSearchQueryActionCostSelection,
  setSearchQueryMetadataTree,
  setSearchQueryRaritySelection,
} from "../../src/tui/search/query-state.js";
import {
  actionCostFilter,
  allOfFilter,
  anyOfFilter,
  browseQuery,
  browseRequest,
  levelFilter,
  metricCompareFilter,
  metricFilter,
  metadataPredicateFilter,
  notFilter,
  rarityFilter,
  scopeFilter,
  searchRequest,
} from "../helpers/search-request-fixture.js";

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

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error?: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });

  return { promise, resolve, reject };
}

type LegacyOntologyNodeFixture = OntologyNode & { children?: LegacyOntologyNodeFixture[] };

function withStaticChildSources(domain: OntologyDomainModel): OntologyDomainModel {
  const normalizeNode = (node: LegacyOntologyNodeFixture): void => {
    if (!node.children) {
      return;
    }
    for (const child of node.children) {
      normalizeNode(child);
    }
    Object.assign(node, {
      childSource: { kind: "static" as const, children: node.children },
    });
  };

  for (const node of domain.rootNodes as LegacyOntologyNodeFixture[]) {
    normalizeNode(node);
  }
  return domain;
}

async function waitForFrameToContain(app: ReturnType<typeof render>, text: string, attempts = 12): Promise<string> {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    const frame = app.lastFrame();
    if (frame.includes(text)) {
      return frame;
    }
    await flushInk();
  }
  return app.lastFrame();
}

async function waitForFrameToExclude(app: ReturnType<typeof render>, text: string, attempts = 12): Promise<void> {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    if (!app.lastFrame().includes(text)) {
      return;
    }
    await flushInk();
  }
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

function pressCtrlE(app: ReturnType<typeof render>): void {
  app.stdin.write("\u0005");
}

function pressCtrlY(app: ReturnType<typeof render>): void {
  app.stdin.write("\u0019");
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

function createSearchSession(overrides: Partial<Pf2eTerminalSearchSession> = {}): Pf2eTerminalSearchSession {
  const query = overrides.query ?? browseRequest({ filter: scopeFilter("spell"), limit: 20 });
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

function createScrollablePreviewPageDocument(): EntityPageDocument {
  const references = Array.from({ length: 18 }, (_, index) => ({
    kind: "record" as const,
    label: `Reference ${String(index + 1).padStart(2, "0")}`,
    recordKey: `spell:reference-${index + 1}`,
    action: "preview" as const,
  }));

  return {
    recordKey: "spell:test-alarm",
    title: "Alarm Ward",
    identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
    traits: [],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused ward that rings when trespassers approach." }],
        targets: [],
      },
      {
        id: "details",
        kind: "details",
        title: "Details",
        blocks: [
          {
            kind: "factList",
            facts: [
              { label: "Traditions", value: "Arcane" },
              { label: "Cast", value: "2 actions" },
              { label: "Range", value: "30 feet" },
              { label: "Duration", value: "1 minute" },
            ],
          },
        ],
        targets: [],
      },
      {
        id: "references",
        kind: "references",
        title: "References",
        blocks: [{ kind: "targetList", targets: references }],
        targets: references,
      },
    ],
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
  const listFilterValues = vi.fn((query: { field?: string; target?: { field: string } }) => {
    const field = query.field ?? query.target?.field;
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
  const lookup: LookupFn =
    overrides.lookup ?? vi.fn(() => ({ match: record, alternatives: [], matchType: "exact" as const }));
  const search: SearchFn =
    overrides.search ??
    vi.fn((request: SearchRequest) =>
      Promise.resolve({
        searchProfile: request.mode === "search" ? (request.search.profile ?? "balanced") : null,
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
  const pageRelations = {
    loadPageRelations: vi.fn(() => ({
      recordKey: record.recordKey,
      outgoing: { records: [], edges: [] },
      incoming: { records: [], edges: [] },
      edges: [],
      incomingGroups: [],
    })),
  };
  const entityPages = createPf2eApplicationEntityPageService(pageRelations);

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
    debug: createNoopTerminalDebugTraceService(),
    user: {
      entityPages,
      search: searchService,
      ontology: {
        loadSearchSemanticsDomain: vi.fn(() => createFacetPickerOntologyDomain()),
        loadSearchFilterExplorerDomain: vi.fn(async () => createFacetPickerOntologyDomain()),
      },
      pageRelations,
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
  return withStaticChildSources({
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
  });
}

function createFacetPickerOntologyDomainWithDiscreteFields(): OntologyDomainModel {
  const domain = createFacetPickerOntologyDomain();
  const categoryNode = domain.rootNodes[0];
  const metadataFieldsNode = categoryNode?.children?.[0];
  if (!metadataFieldsNode?.children) {
    return withStaticChildSources(domain);
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

  return withStaticChildSources(domain);
}

function createCreatureDerivedTagsOntologyDomain(): OntologyDomainModel {
  return withStaticChildSources({
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
  });
}

function createCreatureMetricExplorerModel(): OntologyDomainModel {
  return withStaticChildSources({
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
              filter: allOfFilter([scopeFilter("creature"), metricCompareFilter("hp.value", "gte", "hp.value")]),
              limit: 20,
            }),
          },
        ],
      },
    ],
  });
}

function createLoadingExplorerModel(title: string): OntologyDomainModel {
  return createSearchFilterExplorerLoadingModel(title);
}

function createNamedExplorerDomain(label: string): OntologyDomainModel {
  return withStaticChildSources({
    id: `searchSemantics:${label.toLowerCase().replace(/\s+/g, "-")}`,
    label,
    description: `${label} test domain`,
    rootNodes: [
      {
        id: `${label}:root`,
        kind: "group",
        label,
        listLabel: label,
        filterText: label.toLowerCase(),
        detailTitle: label,
        detailLines: [{ text: label, tone: "section" }],
      },
    ],
  });
}

function createRarityExplorerDomain(values: readonly string[]): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Rarity Explorer",
    description: "Rarity explorer test domain",
    rootNodes: [
      {
        id: "spell:field:rarity",
        kind: "field",
        label: "Rarity",
        filterText: "rarity",
        listLabel: "Rarity",
        detailTitle: "Metadata Field Details",
        detailLines: [{ text: "Rarity", tone: "section" }],
        children: values.map((value) => ({
          id: `spell:field:rarity:value:${value}`,
          kind: "value",
          label: value,
          filterText: value,
          listLabel: value,
          detailTitle: "Value Details",
          detailLines: [{ text: value, tone: "section" }],
          query: browseQuery(`Browse ${value} spells`, {
            filter: allOfFilter([scopeFilter("spell"), rarityFilter({ kind: "eq", value })]),
            limit: 20,
          }),
        })),
      },
    ],
  });
}

function createTraitsExplorerDomain(values: readonly string[]): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Traits Explorer",
    description: "Traits explorer test domain",
    rootNodes: values.map((value) => ({
      id: `spell:traits:${value}`,
      kind: "trait",
      label: value,
      filterText: value,
      listLabel: value,
      detailTitle: "Trait Details",
      detailLines: [{ text: value, tone: "section" }],
    })),
  });
}

function createStructuredTraitsExplorerDomain(
  values: readonly string[],
  options: { categoryKey?: string; categoryLabel?: string } = {},
): OntologyDomainModel {
  const categoryKey = options.categoryKey ?? "spell";
  const categoryLabel = options.categoryLabel ?? "Spell";
  const domain = createFacetPickerOntologyDomain();
  const rewriteNodeIds = (node: OntologyNode): void => {
    if (node.id.startsWith("spell:")) {
      node.id = `${categoryKey}:${node.id.slice("spell:".length)}`;
    }
    node.children?.forEach(rewriteNodeIds);
  };
  const categoryNode = domain.rootNodes[0];
  if (categoryNode) {
    categoryNode.id = `searchSemantics:${categoryKey}`;
    categoryNode.label = categoryLabel;
    categoryNode.listLabel = categoryLabel;
    categoryNode.detailTitle = categoryLabel;
    categoryNode.detailLines = [{ text: categoryLabel, tone: "section" }];
    categoryNode.children?.forEach(rewriteNodeIds);
  }
  const metadataFields = domain.rootNodes[0]?.children?.[0];
  if (!metadataFields?.children) {
    return withStaticChildSources(domain);
  }

  metadataFields.children.unshift({
    id: `${categoryKey}:field:traits`,
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
    children: values.map((value) => ({
      id: `${categoryKey}:traits:${value}`,
      kind: "trait",
      label: value,
      filterText: value,
      listLabel: value,
      detailTitle: "Trait Details",
      detailLines: [{ text: value, tone: "section" }],
      groupValues: {
        family: "traits",
      },
      selection: {
        field: "traits",
        fieldLabel: "Traits",
        value,
        allowedStates: ["any", "all", "exclude"],
      },
    })),
  });

  return withStaticChildSources(domain);
}

function createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain(): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Creature",
    description: "Creature structured explorer test domain",
    rootNodes: [
      {
        id: "searchSemantics:creature",
        kind: "category",
        label: "Creature",
        shortLabel: "creature",
        filterText: "creature",
        detailTitle: "Creature",
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
                id: "creature:field:traits",
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
                    id: "creature:traits:humanoid",
                    kind: "trait",
                    label: "humanoid",
                    filterText: "humanoid",
                    listLabel: "humanoid",
                    detailTitle: "Trait Details",
                    detailLines: [{ text: "humanoid", tone: "section" }],
                    groupValues: { family: "traits" },
                    selection: {
                      field: "traits",
                      fieldLabel: "Traits",
                      value: "humanoid",
                      allowedStates: ["any", "all", "exclude"],
                    },
                  },
                  {
                    id: "creature:traits:evil",
                    kind: "trait",
                    label: "evil",
                    filterText: "evil",
                    listLabel: "evil",
                    detailTitle: "Trait Details",
                    detailLines: [{ text: "evil", tone: "section" }],
                    groupValues: { family: "traits" },
                    selection: {
                      field: "traits",
                      fieldLabel: "Traits",
                      value: "evil",
                      allowedStates: ["any", "all", "exclude"],
                    },
                  },
                ],
              },
              {
                id: "creature:field:families",
                kind: "field",
                label: "Families",
                filterText: "families",
                listLabel: "Families",
                detailTitle: "Metadata Field Details",
                detailLines: [{ text: "Families", tone: "section" }],
                children: [
                  {
                    id: "creature:field:families:value:ancestry_npcs",
                    kind: "value",
                    label: "ancestry npcs",
                    filterText: "ancestry npcs",
                    listLabel: "ancestry npcs",
                    detailTitle: "Value Details",
                    detailLines: [{ text: "ancestry npcs", tone: "section" }],
                  },
                  {
                    id: "creature:field:families:value:humanoid",
                    kind: "value",
                    label: "humanoid",
                    filterText: "humanoid",
                    listLabel: "humanoid",
                    detailTitle: "Value Details",
                    detailLines: [{ text: "humanoid", tone: "section" }],
                  },
                ],
              },
            ],
          },
          {
            id: "creature:actorMetrics:discovery",
            kind: "group",
            label: "Creature Statistics",
            filterText: "creature statistics metrics",
            listLabel: "Creature Statistics",
            detailTitle: "Creature Statistics",
            detailLines: [{ text: "Creature Statistics", tone: "section" }],
            children: [
              {
                id: "creature:actorMetrics:ac.",
                kind: "metricNamespace",
                label: "ac.",
                filterText: "ac armor class",
                listLabel: "ac. | 1 metric",
                detailTitle: "Metric Namespace",
                detailLines: [{ text: "ac.", tone: "section" }],
                children: [
                  {
                    id: "creature:actorMetrics:ac.value",
                    kind: "metric",
                    label: "Armor Class",
                    filterText: "ac armor class",
                    listLabel: "Armor Class | 3",
                    detailTitle: "Metric Details",
                    detailLines: [{ text: "Armor Class", tone: "section" }],
                    query: browseQuery("Browse creatures by armor class", {
                      filter: allOfFilter([
                        scopeFilter("creature"),
                        metricCompareFilter("ac.value", "gte", "ac.value"),
                      ]),
                      limit: 20,
                    }),
                  },
                ],
              },
              {
                id: "creature:actorMetrics:hp.",
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
                    query: browseQuery("Browse creatures by hit points", {
                      filter: allOfFilter([
                        scopeFilter("creature"),
                        metricCompareFilter("hp.value", "gte", "hp.value"),
                      ]),
                      limit: 20,
                    }),
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  });
}

function createStructuredCreatureTraitsRarityExplorerDomain(): OntologyDomainModel {
  const domain = createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain();
  const categoryNode = domain.rootNodes[0];
  const metadataFieldsNode = categoryNode?.children?.[0];
  if (!metadataFieldsNode?.children) {
    return withStaticChildSources(domain);
  }

  metadataFieldsNode.children.unshift({
    id: "creature:field:rarity",
    kind: "field",
    label: "Rarity",
    filterText: "rarity",
    listLabel: "Rarity",
    detailTitle: "Metadata Field Details",
    detailLines: [{ text: "Rarity", tone: "section" }],
    children: [
      {
        id: "creature:field:rarity:value:common",
        kind: "value",
        label: "common",
        filterText: "common",
        listLabel: "common",
        detailTitle: "Value Details",
        detailLines: [{ text: "common", tone: "section" }],
      },
      {
        id: "creature:field:rarity:value:rare",
        kind: "value",
        label: "rare",
        filterText: "rare",
        listLabel: "rare",
        detailTitle: "Value Details",
        detailLines: [{ text: "rare", tone: "section" }],
      },
      {
        id: "creature:field:rarity:value:uncommon",
        kind: "value",
        label: "uncommon",
        filterText: "uncommon",
        listLabel: "uncommon",
        detailTitle: "Value Details",
        detailLines: [{ text: "uncommon", tone: "section" }],
      },
    ],
  });

  return withStaticChildSources(domain);
}

function createStructuredCreatureTraitsFamiliesMetricAndPackExplorerDomain(): OntologyDomainModel {
  const domain = createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain();
  const categoryNode = domain.rootNodes[0];
  if (!categoryNode?.children) {
    return withStaticChildSources(domain);
  }

  categoryNode.children.push({
    id: "creature:pack",
    kind: "field",
    label: "Pack",
    filterText: "pack compendium source",
    listLabel: "Pack",
    detailTitle: "Pack Details",
    detailLines: [{ text: "Pack", tone: "section" }],
    children: [
      {
        id: "creature:pack:pathfinder-npc-core",
        kind: "value",
        label: "Pathfinder NPC Core",
        filterText: "pathfinder npc core",
        listLabel: "Pathfinder NPC Core | 4",
        detailTitle: "Pack Details",
        detailLines: [{ text: "Pathfinder NPC Core", tone: "section" }],
      },
      {
        id: "creature:pack:monster-core",
        kind: "value",
        label: "Monster Core",
        filterText: "monster core",
        listLabel: "Monster Core | 2",
        detailTitle: "Pack Details",
        detailLines: [{ text: "Monster Core", tone: "section" }],
      },
    ],
  });

  return withStaticChildSources(domain);
}

function createStructuredCreatureMetricExplorerDomainWithTextMetric(): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Creature",
    description: "Creature metric explorer test domain with text and numeric metrics",
    rootNodes: [
      {
        id: "searchSemantics:creature",
        kind: "category",
        label: "Creature",
        shortLabel: "creature",
        filterText: "creature",
        detailTitle: "Creature",
        detailLines: [{ text: "Creature", tone: "section" }],
        children: [
          {
            id: "creature:actorMetrics:discovery",
            kind: "group",
            label: "Creature Statistics",
            filterText: "creature statistics metrics",
            listLabel: "Creature Statistics",
            detailTitle: "Creature Statistics",
            detailLines: [{ text: "Creature Statistics", tone: "section" }],
            children: [
              {
                id: "creature:actorMetrics:save.",
                kind: "metricNamespace",
                label: "save.",
                filterText: "save best",
                listLabel: "save. | 1 metric",
                detailTitle: "Metric Namespace",
                detailLines: [{ text: "save.", tone: "section" }],
                children: [
                  {
                    id: "creature:actorMetrics:save.best",
                    kind: "metric",
                    label: "Best Save",
                    filterText: "save best",
                    listLabel: "Best Save | 3",
                    detailTitle: "Metric Details",
                    detailLines: [{ text: "Best Save", tone: "section" }],
                    query: browseQuery("Browse creatures by best save", {
                      filter: scopeFilter("creature"),
                      limit: 20,
                    }),
                  },
                ],
              },
              {
                id: "creature:actorMetrics:hp.",
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
                    query: browseQuery("Browse creatures by hit points", {
                      filter: allOfFilter([
                        scopeFilter("creature"),
                        metricCompareFilter("hp.value", "gte", "hp.value"),
                      ]),
                      limit: 20,
                    }),
                  },
                ],
              },
            ],
          },
        ],
      },
    ],
  });
}

function createPackExplorerModel(): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Pack Explorer",
    description: "Pack explorer test domain",
    rootNodes: [
      {
        id: "creature:pack:pathfinder-npc-core",
        kind: "value",
        label: "Pathfinder NPC Core",
        filterText: "pathfinder npc core",
        listLabel: "Pathfinder NPC Core | 4",
        detailTitle: "Pack Details",
        detailLines: [{ text: "Pathfinder NPC Core", tone: "section" }],
      },
      {
        id: "creature:pack:monster-core",
        kind: "value",
        label: "Monster Core",
        filterText: "monster core",
        listLabel: "Monster Core | 2",
        detailTitle: "Pack Details",
        detailLines: [{ text: "Monster Core", tone: "section" }],
      },
    ],
  });
}

async function openStructuredQueryEditor(app: ReturnType<typeof render>): Promise<void> {
  await flushInk();
  pressLeft(app);
  await flushInk();
  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Structured Query Editor");
}

async function driveRootTraitAddFlow(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metadata");
  expect(app.lastFrame()).toContain("Traits");

  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Traits Explorer", 60);
  await waitForFrameToContain(app, "archetype", 120);
  expect(app.lastFrame()).toContain("archetype");
  expect(app.lastFrame()).toContain("dedication");
  expect(app.lastFrame()).toContain("concentrate");

  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  pressDown(app);
  await flushInk();
  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  pressDown(app);
  await flushInk();
  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  pressLeft(app);
  await flushInk();
  if (app.lastFrame().includes("Metadata")) {
    pressLeft(app);
    await flushInk();
  }
  if (app.lastFrame().includes("Add Clause")) {
    pressLeft(app);
    await flushInk();
  }
  await waitForFrameToContain(app, "Structured Query Editor");
}

async function openTraitsExplorerFromAddHere(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metadata");
  expect(app.lastFrame()).toContain("Traits");

  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Traits Explorer", 60);
}

async function returnFromExplorerToStructuredEditor(app: ReturnType<typeof render>): Promise<void> {
  for (let step = 0; step < 6; step += 1) {
    if (app.lastFrame().includes("Structured Query Editor")) {
      return;
    }
    pressLeft(app);
    await flushInk();
  }
  await waitForFrameToContain(app, "Structured Query Editor");
}

async function openFamiliesExplorerFromAddHere(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metadata");
  expect(app.lastFrame()).toContain("Families");

  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Families Explorer", 60);
}

async function addAncestryNpcsFromCurrentFamiliesSelectionFromExplorer(app: ReturnType<typeof render>): Promise<void> {
  await waitForFrameToContain(app, "Families Explorer", 60);
  await waitForFrameToContain(app, "ancestry npcs", 120);
  expect(app.lastFrame().toLowerCase()).toContain("humanoid");

  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  await returnFromExplorerToStructuredEditor(app);
}

async function openMetricExplorerFromAddHere(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metric");
  expect(app.lastFrame()).toContain("Creature Statistics");

  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Creature Statistics Explorer", 60);
}

async function openRarityExplorerFromAddHere(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metadata");
  expect(app.lastFrame()).toContain("Rarity");

  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Rarity Explorer", 60);
}

async function openRarityExplorerFromAddClause(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  for (let step = 0; step < 4; step += 1) {
    pressDown(app);
    await flushInk();
  }
  expect(app.lastFrame()).toContain("Rarity");
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Rarity Explorer", 60);
}

async function openUnscopedRarityExplorerFromAddClause(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  for (let step = 0; step < 3; step += 1) {
    pressDown(app);
    await flushInk();
  }
  expect(app.lastFrame()).toContain("Rarity");
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Rarity Explorer", 60);
}

async function openUnscopedPackExplorerFromAddClause(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause");

  pressDown(app);
  await flushInk();
  expect(app.lastFrame()).toContain("Pack");
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Pack", 60);
  await waitForFrameToContain(app, "Pathfinder NPC Core", 120);
}

async function addCommonToCurrentRaritySelectionFromExplorer(app: ReturnType<typeof render>): Promise<void> {
  await waitForFrameToContain(app, "Rarity Explorer", 60);
  await waitForFrameToContain(app, "common", 120);
  await waitForFrameToContain(app, "[ ] common", 120);
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "[x] common", 120);
  await flushInk();
  await returnFromExplorerToStructuredEditor(app);
  await waitForFrameToContain(app, "Rarity: Common", 120);
}

async function addPathfinderNpcCoreToCurrentPackSelectionFromExplorer(app: ReturnType<typeof render>): Promise<void> {
  await waitForFrameToContain(app, "Pathfinder NPC Core", 120);
  app.stdin.write(" ");
  await flushInk();
  await flushInk();
  await returnFromExplorerToStructuredEditor(app);
  await waitForFrameToContain(app, "Pack: Pathfinder NPC Core", 120);
}

async function addAcGte10FromCurrentMetricSelectionFromExplorer(app: ReturnType<typeof render>): Promise<void> {
  await waitForFrameToContain(app, "Creature Statistics Explorer", 60);
  await waitForFrameToContain(app, "ac.", 120);
  expect(app.lastFrame()).toContain("hp.");

  app.stdin.write("\r");
  await flushInk();
  if (!app.lastFrame().includes("Enter `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.")) {
    await waitForFrameToContain(app, "Armor Class", 60);
    app.stdin.write(" ");
    await flushInk();
  }
  await waitForFrameToContain(app, "Creature Statistics / Armor Class", 60);
  expect(app.lastFrame()).toContain("Enter `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.");

  for (const character of ">=10") {
    app.stdin.write(character);
    await flushInk();
  }
  app.stdin.write("\r");
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, "Armor Class", 60);

  await returnFromExplorerToStructuredEditor(app);
}

async function _driveFlatFeatTraitSeedFlow(app: ReturnType<typeof render>): Promise<void> {
  await openTraitsExplorerFromAddHere(app);
  for (let attempt = 0; attempt < 3 && !app.lastFrame().includes("archetype"); attempt += 1) {
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
  }
  await waitForFrameToContain(app, "archetype", 120);
  expect(app.lastFrame()).toContain("skill");
  expect(app.lastFrame()).toContain("concentrate");

  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  pressDown(app);
  await flushInk();
  pressDown(app);
  await flushInk();
  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  await returnFromExplorerToStructuredEditor(app);
}

async function addSkillToCurrentTraitSelectionFromExplorer(app: ReturnType<typeof render>): Promise<void> {
  await waitForFrameToContain(app, "Traits Explorer", 60);
  for (let attempt = 0; attempt < 3 && !app.lastFrame().includes("skill"); attempt += 1) {
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
  }
  expect(app.lastFrame()).toContain("archetype");
  expect(app.lastFrame()).toContain("concentrate");
  expect(app.lastFrame()).toContain("skill");

  pressDown(app);
  await flushInk();
  app.stdin.write(" ");
  await flushInk();
  await flushInk();

  await returnFromExplorerToStructuredEditor(app);
}

describe("search screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("does not expose the text-query shortcut while browsing", () => {
    const browseState = createInitialSearchScreenState(browseRequest({ limit: 20 }));
    const searchState = createInitialSearchScreenState(searchRequest({ search: { query: "ghost" }, limit: 20 }));

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

  it("loads page relations through the shared entity-page facade for result previews", async () => {
    const services = createServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    expect(services.user.pageRelations.loadPageRelations).toHaveBeenCalledWith("spell:test-alarm");
    expect(app.lastFrame()).toContain("Alarm Ward");
    expect(app.lastFrame()).toContain("Preview | Alarm Ward | Identity");
  });

  it("routes hybrid page-preview navigation through shared section and target interactions", async () => {
    const services = createServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Identity");
    expect(app.lastFrame()).toContain("↑/↓ section");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Summary");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Description");

    app.stdin.write("G");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Classification");
    expect(app.lastFrame()).toContain("Enter/→ targets");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("↑/↓ target");
    expect(app.lastFrame()).toContain("←/Esc section");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("↑/↓ section");
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Classification");
  });

  it("allows visible header actions to be selected from the result preview", async () => {
    const services = createServices();
    const onActivatePageTarget = vi.fn(() => true);
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            entry="results"
            initialSession={createSearchSession()}
            onActivatePageTarget={onActivatePageTarget}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("Open in Archives of Nethys");
    expect(app.lastFrame()).toContain("Enter/→ targets");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("↑/↓ target");

    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "external",
        label: "AoN: Open in Archives of Nethys",
      }),
    );
  });

  it("uses Ctrl-Y and Ctrl-E to smooth-scroll preview content without leaving section mode", async () => {
    const services = createServices();
    const referenceRecords = Array.from({ length: 18 }, (_, index) =>
      createRecord({
        recordKey: `spell:reference-${index + 1}`,
        id: `reference-${index + 1}`,
        name: `Reference ${String(index + 1).padStart(2, "0")}`,
      }),
    );
    vi.mocked(services.user.pageRelations.loadPageRelations).mockReturnValue({
      recordKey: "spell:test-alarm",
      outgoing: {
        records: referenceRecords,
        edges: referenceRecords.map((record, index) => ({
          fromRecordKey: "spell:test-alarm",
          toRecordKey: record.recordKey,
          displayText: `Reference ${String(index + 1).padStart(2, "0")}`,
          referenceText: `Reference ${String(index + 1).padStart(2, "0")}`,
          sourceCategory: "core",
        })),
      },
      incoming: { records: [], edges: [] },
      edges: [],
      incomingGroups: [],
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Identity");
    expect(app.lastFrame()).toContain("↑/↓ section");
    expect(app.lastFrame()).toContain("Archives of Nethys");
    expect(app.lastFrame()).not.toContain("Reference 01");

    for (let step = 0; step < 20; step += 1) {
      pressCtrlE(app);
      await flushInk();
    }

    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | References");
    expect(app.lastFrame()).toContain("↑/↓ section");
    expect(app.lastFrame()).not.toContain("Archives of Nethys");
    expect(app.lastFrame()).toContain("Reference 01");

    for (let step = 0; step < 20; step += 1) {
      pressCtrlY(app);
      await flushInk();
    }

    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Identity");
    expect(app.lastFrame()).toContain("↑/↓ section");
    expect(app.lastFrame()).toContain("Archives of Nethys");
    expect(app.lastFrame()).not.toContain("Reference 01");
  });

  it("keeps offscreen selected targets visible in the result preview host", async () => {
    const services = createServices();
    const previewDocument = createScrollablePreviewPageDocument();
    services.user.entityPages = {
      ...services.user.entityPages,
      buildDocument: vi.fn(() => previewDocument),
      buildDocumentByRecordKey: vi.fn(() => null),
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | References");
    expect(app.lastFrame()).toContain("Reference 01");
    expect(app.lastFrame()).not.toContain("Reference 18");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("↑/↓ target");

    app.stdin.write("G");
    await flushInk();

    expect(app.lastFrame()).toContain("↑/↓ target");
    expect(app.lastFrame()).toContain("Reference 18");
  });

  it("emits search-pivot page target activation through the shared callback seam", async () => {
    const services = createServices();
    const onActivatePageTarget = vi.fn(() => true);
    const record = createRecord({ traits: ["fire"] });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            entry="results"
            initialSession={createSearchSession({ results: [record] })}
            onActivatePageTarget={onActivatePageTarget}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Identity");
    expect(app.lastFrame()).toContain("Traits: Fire");

    app.stdin.write("\r");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith({
      kind: "searchPivot",
      label: "Trait: Fire",
      request: {
        mode: "browse",
        filter: {
          kind: "allOf",
          children: [
            {
              kind: "scope",
              category: "spell",
              subcategory: { kind: "any" },
            },
            {
              kind: "metadataPredicate",
              predicate: { field: "traits", op: "includes", value: "fire" },
            },
          ],
        },
        sort: { kind: "alphabetical" },
        limit: 50,
      },
    });
  });

  it("activates Classification row pivots from the result preview", async () => {
    const services = createServices();
    const onActivatePageTarget = vi.fn(() => true);
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            entry="results"
            initialSession={createSearchSession()}
            onActivatePageTarget={onActivatePageTarget}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    app.stdin.write("G");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Classification");
    expect(app.lastFrame()).toContain("Category: Spell");

    app.stdin.write("\r");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onActivatePageTarget).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "searchPivot",
        label: "Category: Spell",
      }),
    );
  });

  it("warns when a page pivot is activated without a host navigation callback", async () => {
    const services = createServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    app.stdin.write("G");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Page pivots are unavailable in this host.");
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
    expect(app.lastFrame()).toContain("Choose Search Mode");
    pressRight(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToExclude(app, "Choose Search Mode");
    expect(app.lastFrame()).toContain("Search | Any Category | Enter search text to enable this search.");
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Query Status");
    expect(app.lastFrame()).toContain("Execute Query");
    expect(app.lastFrame()).toContain("Profile | balanced");
    expect(app.lastFrame()).not.toContain("Action Cost |");
    expect(app.lastFrame()).toContain("Filters > | None yet");
    expect(app.lastFrame()).toContain("Mode");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Query");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Text");
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    for (const character of "ghost") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToExclude(app, "Search Text");

    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Exclude");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Exclude Text");
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    for (const character of "skeleton") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToExclude(app, "Exclude Text");

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

  it("uses a blanked standalone mode picker before the workspace is revealed", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} promptForInitialMode />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await waitForFrameToContain(app, "Choose Search Mode");
    expect(app.lastFrame()).toContain("Choose Search Mode");
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
    expect(app.lastFrame()).toContain("[EDITOR] Query");

    pressLeft(app);
    await flushInk();

    expect(onBack).toHaveBeenCalledTimes(1);
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
    expect(app.lastFrame()).toContain("Choose Search Mode");
    expect(app.lastFrame()).toContain("Search");

    app.stdin.write("h");
    await flushInk();
    expect(app.lastFrame()).toContain("Choose Search Mode");
    expect(app.lastFrame()).toContain("Browse");
  });

  it("focuses the shared editor action rail and runs the focused editor action", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Open Focused Row");
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Choose Search Mode");

    expect(app.lastFrame()).toContain("Choose Search Mode");
  });

  it("does not expose discard-results actions in the editor rail before a result session exists", async () => {
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
    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Open Focused Row");
    expect(app.lastFrame()).toContain("Execute Query");
    expect(app.lastFrame()).toContain("Reset Query");
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

  it("opens result actions with sort selection for browse sessions", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen entry="results" initialSession={createSearchSession()} onBack={vi.fn()} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write(":");
    await waitForFrameToContain(app, "Actions:");

    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Jump to Result");
    expect(app.lastFrame()).toContain("Change Sort");
  });

  it("keeps ranked search results on the shared action flow without exposing sort", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen
            entry="results"
            initialSession={createSearchSession({
              query: searchRequest({ search: { query: "alarm ward" }, limit: 20 }),
              resultMode: "hybrid",
              searchProfile: "balanced",
              sort: "ranked",
            })}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write(":");
    const actionsFrame = await waitForFrameToContain(app, "Actions:");

    expect(actionsFrame).toContain("Actions:");
    expect(actionsFrame).toContain("Jump to Result");
    expect(actionsFrame).not.toContain("Change Sort");
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
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  levelFilter({ kind: "between", min: 1, max: 1 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("All of");
    expect(app.lastFrame()).toContain("[+ add here]");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metadata");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metadata");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Derived Tags Explorer > Derived Tags", 60);
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
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
    expect(app.lastFrame()).toContain("coastal_setting");
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
        filter: allOfFilter([
          scopeFilter("spell"),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
        ]),
        limit: 20,
      }),
    );

    expect(request).toEqual(
      browseQuery("Browse records with this trait", {
        filter: allOfFilter([
          scopeFilter("spell"),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
        ]),
        limit: 20,
      }).request,
    );
  });

  it("shows seeded metadata clauses in the query editor when launched from ontology", async () => {
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices()}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse illusion spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
                ]),
                limit: 20,
              }).request
            }
            origin="ontology"
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filters > | 2 active");
    expect(app.lastFrame()).toContain("Scope: Spell");
    expect(app.lastFrame()).toContain("Traits: includes Illusion");
    expect(app.lastFrame()).not.toContain("Filter | Traits: includes Illusion");
  });

  it("does not auto-execute seeded route entry without a prepared session", async () => {
    const openSearchWindow = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={createServices({ openSearchWindow })}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse illusion spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
                ]),
                limit: 20,
              }).request
            }
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
          <SearchScreen entry="results" initialSession={initialSession} origin="ontology" onBack={vi.fn()} />
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
          <SearchScreen entry="results" initialSession={initialSession} origin="ontology" onBack={vi.fn()} />
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
        searchProfile: request.mode === "search" ? (request.search.profile ?? "balanced") : null,
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
        setSearchQueryActionCostSelection(
          setSearchQueryRaritySelection(
            searchRequest({
              limit: 20,
              filter: scopeFilter("spell"),
              search: {
                query: "ghost",
                profile: "balanced",
              },
            }),
            {
              include: ["common"],
              exclude: ["rare"],
            },
          ),
          {
            include: [2],
            exclude: [1],
          },
        ),
        {
          and: [
            {
              field: "traits",
              op: "includes",
              value: "illusion",
            },
            {
              field: "traits",
              op: "includes",
              value: "auditory",
            },
            {
              not: {
                field: "traits",
                op: "includes",
                value: "emotion",
              },
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
            kind: "rarity",
            match: { kind: "eq", value: "common" },
          },
          {
            kind: "rarity",
            match: { kind: "notIn", values: ["rare"] },
          },
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
          {
            kind: "allOf",
            children: [
              {
                kind: "metadataPredicate",
                predicate: { field: "traits", op: "includes", value: "illusion" },
              },
              {
                kind: "metadataPredicate",
                predicate: { field: "traits", op: "includes", value: "auditory" },
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

  it("keeps live query-tree edits when returning from the top-level structured editor", async () => {
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
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  levelFilter({ kind: "between", min: 1, max: 1 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("All of");
    expect(app.lastFrame()).toContain("[+ add here]");
    expect(app.lastFrame()).toContain(": focus actions");
    expect(app.lastFrame()).toContain("return");

    app.stdin.write("?");
    await flushInk();
    expect(app.lastFrame()).toContain("q: return to the main query editor");
    expect(app.lastFrame()).toContain("Press : to open the action rail");

    app.stdin.write("x");
    await waitForFrameToExclude(app, "Press : to open the action rail");
    expect(app.lastFrame()).toContain("Structured Query Editor");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Add Clause");
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metadata");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metadata");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Derived Tags Explorer > Derived Tags", 60);
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
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
    expect(app.lastFrame()).toContain("coastal_setting");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).toContain("Explorer Entries");

    pressLeft(app);
    await flushInk();
    if (app.lastFrame().includes("Derived Tags Explorer > Derived Tags")) {
      pressLeft(app);
      await flushInk();
    }
    if (app.lastFrame().includes("Metadata")) {
      pressLeft(app);
      await flushInk();
    }
    if (app.lastFrame().includes("Add Clause")) {
      pressLeft(app);
      await flushInk();
    }

    if (app.lastFrame().includes("Structured Query Editor")) {
      expect(app.lastFrame()).not.toContain("Browse/Search");
    } else {
      expect(app.lastFrame()).toContain("[EDITOR] Query");
      expect(app.lastFrame()).toContain("Derived Tags: includes Coas");
      return;
    }

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Filters > | 5 active");
    expect(app.lastFrame()).toContain("Top-level filters: 5");
    expect(app.lastFrame()).toContain("Metadata predicates: 1");
    expect(app.lastFrame()).toContain("Derived Tags: includes Coas");
    expect(app.lastFrame()).not.toContain("Structured Query Editor");
  });

  it("uses the shared rarity explorer for live query-tree rows and returns to the structured editor", async () => {
    const services = createServices();
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  levelFilter({ kind: "between", min: 1, max: 1 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Action Cost: 2");

    pressUp(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Rarity: Common");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "common", 60);
    expect(app.lastFrame()).toContain("Rarity Explorer");
    expect(app.lastFrame()).toContain("common");
    await waitForFrameToContain(app, "rare", 60);
    expect(app.lastFrame()).toContain("rare");

    pressLeft(app);
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Action Cost: 2");
  });

  it("returns from a live rarity explorer edit after clearing the focused clause", async () => {
    const services = createServices();
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Rarity: Common");

    pressUp(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "Rarity Explorer", 60);

    await waitForFrameToContain(app, "[✓] common", 60);
    await new Promise((resolve) => setTimeout(resolve, 120));
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "[x] common", 60);
    await new Promise((resolve) => setTimeout(resolve, 120));
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "[.] common", 60);
    await new Promise((resolve) => setTimeout(resolve, 120));
    await flushInk();

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Action Cost: 2");
    expect(app.lastFrame()).not.toContain("Rarity: Common");
  });

  it("applies the latest live rarity explorer state when returning immediately to the structured editor", async () => {
    const services = createServices();
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    await waitForFrameToContain(app, "Structured Query Editor");
    pressUp(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "[✓] common", 60);

    app.stdin.write(" ");
    await flushInk();
    pressLeft(app);
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("! Rarity: Common");
    expect(app.lastFrame()).not.toContain("├─ Rarity: Common");
  });

  it("uses friendly aliases in live clause and exclude-group action menus", async () => {
    const services = createServices();

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([scopeFilter("spell"), rarityFilter({ kind: "eq", value: "common" })]),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("Rarity: Common");

    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    expect(app.lastFrame()).toContain("Wrap In Exclude");
    expect(app.lastFrame()).toContain("Wrap In All of");
    expect(app.lastFrame()).toContain("Wrap In Any of");
    expect(app.lastFrame()).not.toContain("Wrap In NOT");
    expect(app.lastFrame()).not.toContain("Wrap In allOf");
    expect(app.lastFrame()).not.toContain("Wrap In anyOf");

    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("! Scope: Spell");

    app.stdin.write("\u001b");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Exclude Group");
    expect(app.lastFrame()).toContain("Remove Exclude");
    expect(app.lastFrame()).not.toContain("NOT Group");
    expect(app.lastFrame()).not.toContain("Remove NOT");
  });

  it("defaults live query-tree shared explorers to matching counts", async () => {
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
        return withStaticChildSources(domain);
      },
    );
    services.user.ontology.loadSearchFilterExplorerDomain = loadSearchFilterExplorerDomain;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  levelFilter({ kind: "between", min: 1, max: 1 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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

    pressUp(app);
    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await flushInk();

    await waitForFrameToContain(app, "Rarity Explorer", 60);
    await waitForFrameToContain(app, "matching counts", 60);
    expect(app.lastFrame()).toContain("Rarity Explorer");
    expect(app.lastFrame()).toContain("matching counts");
    expect(loadSearchFilterExplorerDomain).toHaveBeenCalledWith(
      expect.objectContaining({
        discoveryMode: "matching",
        request: expect.objectContaining({
          mode: "browse",
        }),
        targetFields: ["rarity"],
      }),
    );
  });

  it("routes live ontology composition through clause-kind and field pickers before opening the shared explorer", async () => {
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

      return withStaticChildSources(domain);
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  actionCostFilter({ kind: "eq", value: 2 }),
                  levelFilter({ kind: "between", min: 1, max: 1 }),
                  rarityFilter({ kind: "eq", value: "common" }),
                ]),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("All of");
    expect(app.lastFrame()).toContain("[+ add here]");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metadata");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metadata");
    expect(app.lastFrame()).toContain("Traits");
    expect(app.lastFrame()).toContain("Derived Tags");
    expect(app.lastFrame()).not.toContain("Filters >");
    expect(app.lastFrame()).not.toContain("Browse/Search");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Traits Explorer > Illusion", 60);
    expect(app.lastFrame()).toContain("Traits Explorer > Illusion");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("illusion");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("illusion");

    pressLeft(app);
    await flushInk();
    if (app.lastFrame().includes("Metadata")) {
      expect(app.lastFrame()).toContain("Traits");
      pressLeft(app);
      await flushInk();
    }
    if (app.lastFrame().includes("Add Clause")) {
      pressLeft(app);
      await flushInk();
    }
    if (app.lastFrame().includes("Structured Query Editor")) {
      expect(app.lastFrame()).not.toContain("Filters >");
    } else {
      expect(app.lastFrame()).toContain("[EDITOR] Query");
      expect(app.lastFrame()).toContain("Traits: includes Illusion");
    }
  });

  it("returns grouped add-clause metadata explorers directly to the structured editor on back", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(() =>
      createStructuredTraitsExplorerDomain(["illusion", "emotion"]),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: scopeFilter("spell"),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await openTraitsExplorerFromAddHere(app);
    await waitForFrameToContain(app, "illusion", 120);

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    pressLeft(app);
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Traits: includes Illusion");
    expect(app.lastFrame()).not.toContain("Add Clause");
  });

  it("steps single-clause shared-explorer flows back through the metadata picker before returning to the query editor", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(() =>
      createStructuredTraitsExplorerDomain(["illusion", "emotion"]),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: scopeFilter("spell"),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Insertion Slot");
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    for (
      let attempt = 0;
      attempt < 4 && !app.lastFrame().includes("Choose the metadata field for the next clause");
      attempt += 1
    ) {
      if (app.lastFrame().includes("Choose the clause kind to insert into the current group")) {
        pressDown(app);
        await flushInk();
        app.stdin.write("\r");
        await flushInk();
        continue;
      }
      await flushInk();
    }

    expect(app.lastFrame()).toContain("Traits");

    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "Traits Explorer", 60);
    await waitForFrameToContain(app, "illusion", 120);

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Metadata");
    expect(app.lastFrame()).toContain("Traits");

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query");
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Traits: includes Illusion");
  });

  it("keeps multi-trait add-here composition canonical when returning from the shared explorer", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredTraitsExplorerDomain(["archetype", "dedication", "concentrate"]),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: scopeFilter("spell"),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await driveRootTraitAddFlow(app);

    if (!app.lastFrame().includes("Structured Query Editor")) {
      await openStructuredQueryEditor(app);
    }
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).not.toContain("Filter: Any of (2 filters)");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^│ {2}├─ Any of$/m)).toBeNull();
  });

  it("keeps the same canonical trait subtree after closing and reopening the structured editor", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredTraitsExplorerDomain(["archetype", "dedication", "concentrate"]),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: scopeFilter("spell"),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await driveRootTraitAddFlow(app);

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[EDITOR] Query");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).not.toContain("Filter: Any of (2 filters)");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^│ {2}├─ Any of$/m)).toBeNull();
  });

  it("renders existing mixed multi-trait buckets without changing canonical shape", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredTraitsExplorerDomain(["archetype", "dedication", "concentrate"]),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  allOfFilter([
                    anyOfFilter([
                      metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
                      metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
                    ]),
                    notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
                  ]),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Top-level filters: 2");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("├─ All of");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrate");
  });

  it("keeps feat trait edit-clause additions flat instead of wrapping them in a nested group", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredTraitsExplorerDomain(["archetype", "skill", "concentrate"], {
        categoryKey: "feat",
        categoryLabel: "Feat",
      }),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse feats", {
                filter: allOfFilter([
                  scopeFilter("feat"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame().toLowerCase()).toContain("concent");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    await flushInk();
    await flushInk();
    await addSkillToCurrentTraitSelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Skill");
    expect(app.lastFrame().toLowerCase()).toContain("concent");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();
  });

  it("keeps feat trait add-here additions flat instead of converting the field into a nested any-of group", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredTraitsExplorerDomain(["archetype", "skill", "concentrate"], {
        categoryKey: "feat",
        categoryLabel: "Feat",
      }),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse feats", {
                filter: allOfFilter([
                  scopeFilter("feat"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame().toLowerCase()).toContain("concent");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    await openTraitsExplorerFromAddHere(app);
    await addSkillToCurrentTraitSelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Skill");
    expect(app.lastFrame().toLowerCase()).toContain("concent");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();
  });

  it("keeps flat grouped trait rows stable when adding a metric scalar clause from add-here", async () => {
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
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live creature metrics.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: allOfFilter([
                  scopeFilter("creature"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("evil");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    await openMetricExplorerFromAddHere(app);
    await addAcGte10FromCurrentMetricSelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().match(/^├─ (! Traits: includes Evil|Traits: !evil)/m)).not.toBeNull();
    expect(app.lastFrame()).toContain("Creature Statistics: ac.va");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();
  });

  it("renders long metric filters and negated trait clauses as readable query-tree rows", async () => {
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
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live creature metrics.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: allOfFilter([
                  scopeFilter("creature"),
                  metricFilter("ability.cha.mod", "gt", 5),
                  metricCompareFilter("ac.value", "gt", "ability.cha.mod"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toMatch(/^├─ Creature Statistics: ability\.cha\.mod > 5\s*│/m);
    expect(app.lastFrame()).toMatch(/^├─ Traits: includes Humanoid\s*│/m);
    expect(app.lastFrame()).toMatch(/^├─ ! Traits: includes Evil\s*│/m);
    expect(app.lastFrame()).toMatch(/^├─ ! Traits: includes Unholy\s*│/m);
    expect(app.lastFrame()).not.toContain("Traits: !evil, !unholy");
  });

  it("keeps mixed trait and rarity add-here additions flat", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "rarity",
        label: "Rarity",
        description: "Browse live rarities for the current scope.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsRarityExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: allOfFilter([
                  scopeFilter("creature"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("evil");
    expect(app.lastFrame().toLowerCase()).toContain("unholy");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();

    await openRarityExplorerFromAddHere(app);
    await addCommonToCurrentRaritySelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("evil");
    expect(app.lastFrame().toLowerCase()).toContain("unholy");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Top-level filters: 5");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^│\s+├─ All of$/m)).toBeNull();
    expect(app.lastFrame().toLowerCase()).toContain("├─ traits: includes humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("├─ ! traits: includes evil");
    expect(app.lastFrame().toLowerCase()).toContain("├─ ! traits: includes unholy");
  });

  it("adds rarity via direct add-clause option when trait fields are also available", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsRarityExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: allOfFilter([
                  scopeFilter("creature"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "unholy" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("evil");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    await openRarityExplorerFromAddClause(app);
    await addCommonToCurrentRaritySelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("evil");
    expect(app.lastFrame().toLowerCase()).toContain("unholy");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Top-level filters: 5");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^│\s+├─ All of$/m)).toBeNull();
    expect(app.lastFrame().toLowerCase()).toContain("├─ traits: includes humanoid");
    expect(app.lastFrame().toLowerCase()).toContain("├─ ! traits: includes evil");
    expect(app.lastFrame().toLowerCase()).toContain("├─ ! traits: includes unholy");
  });

  it("adds rarity before selecting a scope", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => []);
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(async () =>
      createStructuredCreatureTraitsRarityExplorerDomain(),
    );
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => {
      throw new Error("unscoped rarity should not use category-scoped filter explorer loading");
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse all records", {
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await openUnscopedRarityExplorerFromAddClause(app);
    await addCommonToCurrentRaritySelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Top-level filters: 1");
    expect(app.lastFrame()).not.toContain("Scope:");
    expect(services.user.ontology.loadSearchSemanticsDomain).toHaveBeenCalled();
    expect(services.user.ontology.loadSearchFilterExplorerDomain).not.toHaveBeenCalled();
  });

  it("adds pack before selecting a scope", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => []);
    services.user.search.loadPackOptions = vi.fn(async () => []);
    services.user.search.getPackLabel = vi.fn((packValue: string) =>
      packValue === "pathfinder-npc-core" ? "Pathfinder NPC Core" : packValue,
    );
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(async () =>
      createStructuredCreatureTraitsFamiliesMetricAndPackExplorerDomain(),
    );
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => {
      throw new Error("unscoped pack should not use category-scoped filter explorer loading");
    });

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse all records", {
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await openUnscopedPackExplorerFromAddClause(app);
    await addPathfinderNpcCoreToCurrentPackSelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Pack: Pathfinder NPC Core");
    expect(app.lastFrame()).toContain("Top-level filters: 1");
    expect(app.lastFrame()).not.toContain("Scope:");
    expect(services.user.ontology.loadSearchSemanticsDomain).toHaveBeenCalled();
    expect(services.user.ontology.loadSearchFilterExplorerDomain).not.toHaveBeenCalled();
  });

  it("keeps the tree flat after reopening a mixed trait and family query", async () => {
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
        value: "families",
        label: "Families",
        description: "Family query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
      {
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live creature metrics.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: allOfFilter([
                  scopeFilter("creature"),
                  metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
                  notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
                ]),
                limit: 20,
              }).request
            }
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await openStructuredQueryEditor(app);
    await openFamiliesExplorerFromAddHere(app);
    await addAncestryNpcsFromCurrentFamiliesSelectionFromExplorer(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().match(/^├─ (! Traits: includes Evil|Traits: !evil)/m)).not.toBeNull();
    expect(app.lastFrame()).toContain("Families: includes Ancestry Npcs");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    pressLeft(app);
    await flushInk();
    await openStructuredQueryEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().match(/^├─ (! Traits: includes Evil|Traits: !evil)/m)).not.toBeNull();
    expect(app.lastFrame()).toContain("Families: includes Ancestry Npcs");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();
  });

  it("renders grouped exclude buckets inline in the live structured query tree", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse spells", {
                filter: allOfFilter([
                  scopeFilter("spell"),
                  allOfFilter([
                    metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
                    notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "emotion" })),
                  ]),
                ]),
                limit: 20,
              }).request
            }
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
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Traits: includes Illusion");
    expect(app.lastFrame()).toContain("! Traits: includes Emotion");
    expect(app.lastFrame()).not.toContain("├─ Exclude");
    expect(app.lastFrame()).not.toContain("└─ Exclude");
  });

  it("uses left navigation to step back one page within add-clause scope flows", async () => {
    const services = createServices();
    services.user.search.getCategoryOptions = vi.fn(() => [
      { value: null, label: "Any Category", description: "Browse every category." },
      { value: "creature", label: "Creature", description: "Browse creatures." },
    ]);

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
    pressDown(app);
    await flushInk();

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Scope");

    await flushInk();
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "Creature");
    expect(app.lastFrame()).toContain("Scope");
    expect(app.lastFrame()).toContain("Creature");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Scope");
  });

  it("scopes ontology-backed query fields from the current tree category instead of the live query", async () => {
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
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createCreatureDerivedTagsOntologyDomain(),
    );

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
    expect(app.lastFrame()).toContain("All of");
    expect(app.lastFrame()).toContain("[+ add here]");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Scope");

    await flushInk();
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "Creature");
    expect(app.lastFrame()).toContain("Scope");
    expect(app.lastFrame()).toContain("Creature");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Subcategory Mode");

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Scope: Creature");
    expect(app.lastFrame()).toContain("All of");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metadata");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metadata");
    expect(app.lastFrame()).toContain("Derived Tags");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await waitForFrameToContain(app, "Derived Tags Explorer > Derived Tags", 60);
    expect(app.lastFrame()).toContain("Derived Tags Explorer");
    expect(app.lastFrame()).toContain("Explorer Entries");
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags");
    expect(app.lastFrame()).not.toContain("Choose a category before editing a discoverable query field.");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Derived Tags Explorer > Derived Tags > Undead");
    expect(app.lastFrame()).toContain("undead | 1 tag");
    expect(app.lastFrame()).toContain("undead");
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
    services.user.search.loadMetricKeyOptions = vi.fn(async (query, field, discoveryMode) =>
      field === "actorMetric"
        ? [
            {
              value: "hp.value",
              label: "hp.value",
              description:
                discoveryMode === "matching" ? "2 matching canonical records." : "4 applicable canonical records.",
              count: 2,
            },
            {
              value: "ac.value",
              label: "ac.value",
              description:
                discoveryMode === "matching" ? "1 matching canonical record." : "3 applicable canonical records.",
              count: 1,
            },
          ]
        : [],
    );
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: scopeFilter("creature"),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metric");
    expect(app.lastFrame()).toContain("Metric comparison");

    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metric comparison");
    expect(app.lastFrame()).toContain("Creature Statistics");

    pressLeft(app);
    await waitForFrameToContain(app, "Add Clause");
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metric comparison");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Metric comparison");
    expect(app.lastFrame()).toContain("Creature Statistics");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Creature Statistics Explorer", 60);
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("Creature Statistics");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "ac.", 120);
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("ac.");
    expect(app.lastFrame()).toContain("hp.");
    expect(app.lastFrame()).toContain("matching counts");

    pressLeft(app);
    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("Creature Statistics");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "ac.", 120);
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("ac.");
    expect(app.lastFrame()).toContain("hp.");

    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Hit Points");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Comparison Operator");

    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Creature Statistics Explorer", 120);
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "ac.", 120);
    expect(app.lastFrame()).toContain("ac.");
    expect(app.lastFrame()).toContain("hp.");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Armor Class");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Structured Query Editor");
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Creature Statistics: hp.value >= ac.value");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Metric comparison");
  });

  it("shows opt-in debug trace status in the filter explorer footer", async () => {
    let now = 100;
    const debug = createTerminalDebugTraceService({
      enabled: true,
      now: () => now,
    });
    const span = debug.startSpan("filterExplorer.loadModel", { mode: "matching" });
    now = 225;
    span.end({ rootNodes: 1 });
    const services = {
      ...createServices(),
      debug,
    };
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const traitFieldOption = {
      value: "traits",
      label: "Traits",
      description: "Browse traits.",
      fieldType: "enumString" as const,
      editor: "sharedExplorer" as const,
    };
    const session: SearchFilterExplorerSession = {
      title: "Traits",
      model: buildSearchFilterExplorerModel(createFacetPickerOntologyDomain(), {
        category: "spell",
        subcategory: null,
        fieldOptions: [traitFieldOption],
        singleFieldBehavior: "list",
      }),
      query: browseQuery("Browse spells", {
        filter: scopeFilter("spell"),
        limit: 20,
      }).request,
      fieldOptions: [traitFieldOption],
      onEvent: vi.fn(),
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("debug | last filterExplorer.loadModel 125ms");
  });

  it("does not allow non-numeric metric keys to resolve as metric-compare select targets", async () => {
    const services = createServices();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const onSelectTarget = vi.fn();
    const metricFieldOption = {
      value: "actorMetric" as const,
      label: "Creature Statistics",
      description: "Browse live statistic keys for the current scope.",
      fieldType: "enumString" as const,
      editor: "sharedExplorer" as const,
    };
    const session: SearchFilterExplorerSession = {
      title: "Left Metric",
      model: buildSearchFilterExplorerModel(createStructuredCreatureMetricExplorerDomainWithTextMetric(), {
        category: "creature",
        subcategory: null,
        fieldOptions: [metricFieldOption],
        singleFieldBehavior: "list",
      }),
      query: browseQuery("Browse creatures", {
        filter: scopeFilter("creature"),
        limit: 20,
      }).request,
      fieldOptions: [metricFieldOption],
      onEvent: (event) => {
        if (event.kind === "selectTarget") {
          onSelectTarget(event.outcome);
        }
      },
      resolveSelectionTarget: buildMetricSelectionTargetResolver("actorMetric", "Creature Statistics", {
        numericOnly: true,
      }),
      selectTargetMode: true,
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "save.", 60);
    expect(app.lastFrame()).toContain("hp.");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Best Save", 60);
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(onSelectTarget).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Best Save");

    pressLeft(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Hit Points", 60);
    app.stdin.write("\r");
    await flushInk();
    expect(onSelectTarget).toHaveBeenCalledTimes(1);
    expect(onSelectTarget.mock.calls[0]?.[0].result.target).toMatchObject({
      kind: "scalar",
      key: "actorMetric:hp.value",
    });
  });

  it("commits and re-edits pack clauses through the shared explorer from the structured editor flow", async () => {
    const services = createServices();
    services.user.search.getQueryFieldOptions = vi.fn(() => []);
    services.user.search.loadPackOptions = vi.fn(async () => []);
    services.user.search.getPackLabel = vi.fn((packValue: string) =>
      packValue === "pathfinder-npc-core"
        ? "Pathfinder NPC Core"
        : packValue === "monster-core"
          ? "Monster Core"
          : packValue,
    );
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createStructuredCreatureTraitsFamiliesMetricAndPackExplorerDomain(),
    );

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: scopeFilter("creature"),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Pack");

    await flushInk();
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Pathfinder NPC Core");
    expect(app.lastFrame()).toContain("Pack");
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");
    expect(app.lastFrame()).toContain("Monster Core");
    expect(app.lastFrame()).toContain("matching counts");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await returnFromExplorerToStructuredEditor(app);
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("Pack: Pathfinder NPC Core");
    expect(app.lastFrame()).not.toContain("Any of");
    expect(app.lastFrame()).not.toContain("! Any of");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Query Clause");
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Current clauses", 60);
    await waitForFrameToContain(app, "Pathfinder NPC Core", 120);
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await returnFromExplorerToStructuredEditor(app);
    expect(app.lastFrame()).toContain("Structured Query Editor");
    expect(app.lastFrame()).toContain("! Pack: Pathfinder NPC Core");
    expect(app.lastFrame()).not.toContain("Any of");
    expect(app.lastFrame()).not.toContain("! Any of");
  });

  it("keeps discovery-mode actions available in select-target metric explorers", async () => {
    const services = createServices();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const metricFieldOption = {
      value: "actorMetric" as const,
      label: "Creature Statistics",
      description: "Browse live statistic keys and author exact or numeric literal filters for the current scope.",
      fieldType: "enumString" as const,
      editor: "sharedExplorer" as const,
    };
    const buildMetricExplorerModel = () =>
      buildSearchFilterExplorerModel(createStructuredCreatureTraitsFamiliesAndMetricExplorerDomain(), {
        category: "creature",
        subcategory: null,
        fieldOptions: [metricFieldOption],
        singleFieldBehavior: "list",
      });
    const loadModelForDiscoveryMode = vi.fn(async () => buildMetricExplorerModel());
    const session: SearchFilterExplorerSession = {
      title: "Left Metric",
      model: buildMetricExplorerModel(),
      query: browseQuery("Browse creatures", {
        filter: scopeFilter("creature"),
        limit: 20,
      }).request,
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
      fieldOptions: [metricFieldOption],
      onEvent: vi.fn(),
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([metricFieldOption]),
      selectTargetMode: true,
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics");

    app.stdin.write(":");
    await waitForFrameToContain(app, "Use Catalog Counts", 60);
    expect(app.lastFrame()).toContain("Use Catalog Counts");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Use Matching Counts", 60);
    expect(app.lastFrame()).toContain("Use Matching Counts");

    app.stdin.write(":");
    await flushInk();
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(app.lastFrame()).toContain("catalog counts");
    expect(loadModelForDiscoveryMode).toHaveBeenCalledWith("catalog");
  });

  it("keeps discovery-mode actions available in direct-value pack explorers", async () => {
    const services = createServices();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const loadModelForDiscoveryMode = vi.fn(async () => createPackExplorerModel());
    const session: SearchFilterExplorerSession = {
      title: "Pack Explorer",
      model: createPackExplorerModel(),
      query: browseQuery("Browse creatures", {
        filter: scopeFilter("creature"),
        limit: 20,
      }).request,
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
      fieldOptions: [
        {
          value: "pack",
          label: "Pack",
          description: "Browse live packs for the current scope and stage canonical pack clauses.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "pack",
          label: "Pack",
          description: "Browse live packs for the current scope and stage canonical pack clauses.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ]),
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");

    app.stdin.write(":");
    await waitForFrameToContain(app, "Use Catalog Counts", 60);
    expect(app.lastFrame()).toContain("Use Catalog Counts");

    app.stdin.write("\r");
    await waitForFrameToContain(app, "Use Matching Counts", 60);
    expect(app.lastFrame()).toContain("Use Matching Counts");

    app.stdin.write(":");
    await flushInk();
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(app.lastFrame()).toContain("catalog counts");
    expect(loadModelForDiscoveryMode).toHaveBeenCalledWith("catalog");
  });

  it("cycles pack selections in direct-value pack explorers and updates the query", async () => {
    const services = createServices();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const onQueryChange = vi.fn();
    const session: SearchFilterExplorerSession = {
      title: "Pack Explorer",
      model: createPackExplorerModel(),
      query: browseQuery("Browse creatures", {
        filter: scopeFilter("creature"),
        limit: 20,
      }).request,
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode: vi.fn(async () => createPackExplorerModel()),
      fieldOptions: [
        {
          value: "pack",
          label: "Pack",
          description: "Browse live packs for the current scope and stage canonical pack clauses.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ],
      onEvent: (event) => {
        if (event.kind === "change") {
          onQueryChange(event.query, event.fieldState);
        }
      },
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "pack",
          label: "Pack",
          description: "Browse live packs for the current scope and stage canonical pack clauses.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ]),
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Pathfinder NPC Core");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    expect(onQueryChange).toHaveBeenCalled();
    expect(getSearchQueryPackSelection(onQueryChange.mock.calls.at(-1)?.[0])).toEqual({
      include: ["pathfinder-npc-core"],
      exclude: [],
    });
  });

  it("opens the numeric scalar editor when compose-mode creature statistics focus a metric key", async () => {
    const services = createServices();
    const model = createCreatureMetricExplorerModel();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const session: SearchFilterExplorerSession = {
      title: "Creature Statistics Explorer",
      model,
      query: browseQuery("Browse creatures", {
        filter: scopeFilter("creature"),
        limit: 20,
      }).request,
      fieldOptions: [
        {
          value: "actorMetric",
          label: "Creature Statistics",
          description: "Browse live creature metrics.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
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
    };
    const searchFilterExplorerElement = React.createElement(SearchFilterExplorer, { session });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          {searchFilterExplorerElement}
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics Explorer");
    expect(app.lastFrame()).toContain("hp.");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Hit Points");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Creature Statistics / Hit Points");
    expect(app.lastFrame()).toContain("Enter `5`, `!=5`, `>5`, `>=5`, `<5`, `<=5`, or `3-8`.");

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

  it("emits live query changes before returning from the shared search explorer", async () => {
    const services = createServices();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const onQueryChange = vi.fn();
    const session: SearchFilterExplorerSession = {
      title: "Rarity Explorer",
      model: withStaticChildSources({
        id: "searchSemantics",
        label: "Rarity Explorer",
        description: "Rarity explorer test domain",
        rootNodes: [
          {
            id: "spell:field:rarity",
            kind: "field",
            label: "Rarity",
            filterText: "rarity",
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
                query: browseQuery("Browse common spells", {
                  filter: allOfFilter([scopeFilter("spell"), rarityFilter({ kind: "eq", value: "common" })]),
                  limit: 20,
                }),
              },
            ],
          },
        ],
      }),
      query: browseQuery("Browse spells", {
        filter: scopeFilter("spell"),
        limit: 20,
      }).request,
      fieldOptions: [
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ],
      onEvent: (event) => {
        if (event.kind === "change") {
          onQueryChange(event.query, event.fieldState);
        }
      },
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ]),
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("common");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    expect(onQueryChange).toHaveBeenCalled();
    expect(onQueryChange.mock.calls.at(-1)?.[0]).toMatchObject({
      filter: allOfFilter([scopeFilter("spell"), rarityFilter({ kind: "eq", value: "common" })]),
    });
  });

  it("shows the add-clause picker before pack and metric discovery checks resolve", async () => {
    const services = createServices();
    const metricOptionsDeferred = createDeferred<
      {
        value: string;
        label: string;
        description: string;
        count: number;
      }[]
    >();
    const packOptionsDeferred = createDeferred<
      {
        value: string;
        label: string;
        description: string;
        count: number;
      }[]
    >();
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live creature metrics.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);
    services.user.search.loadMetricKeyOptions = vi.fn(() => metricOptionsDeferred.promise);
    services.user.search.loadPackOptions = vi.fn(() => packOptionsDeferred.promise);

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            initialRequest={
              browseQuery("Browse creatures", {
                filter: scopeFilter("creature"),
                limit: 20,
              }).request
            }
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
    expect(app.lastFrame()).toContain("Add Clause");
    expect(app.lastFrame()).toContain("Metric comparison");
    expect(app.lastFrame()).toContain("Pack");
    expect(services.user.search.loadMetricKeyOptions).not.toHaveBeenCalled();
    expect(services.user.search.loadPackOptions).not.toHaveBeenCalled();
  });

  it("renders the explorer immediately while the initial model refresh is still loading", async () => {
    const services = createServices();
    const loadModelForDiscoveryMode = vi.fn(() => Promise.resolve(createFacetPickerOntologyDomain()));
    const session: SearchFilterExplorerSession = {
      title: "Derived Tags Explorer",
      model: createLoadingExplorerModel("Derived Tags Explorer"),
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "derivedTags",
          label: "Derived Tags",
          description: "Browse live derived tags for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: () => undefined,
    };

    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toMatch(/Loading explorer entries|Spell/);
    expect(loadModelForDiscoveryMode).toHaveBeenCalledWith("matching", { targetFields: ["derivedTags"] });

    await waitForFrameToContain(app, "Spell");
    expect(app.lastFrame()).toContain("matching counts");
  });

  it("scopes initial loading-model hydration to the session field set", async () => {
    const services = createServices();
    const loadModelForDiscoveryMode = vi.fn(() => Promise.resolve(createRarityExplorerDomain(["rare"])));
    const session: SearchFilterExplorerSession = {
      title: "Rarity Explorer",
      model: createLoadingExplorerModel("Rarity Explorer"),
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
        {
          value: "traits",
          label: "Traits",
          description: "Browse live traits for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
        {
          value: "traits",
          label: "Traits",
          description: "Browse live traits for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ]),
    };
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;

    render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    expect(loadModelForDiscoveryMode).toHaveBeenCalledWith("matching", {
      targetFields: ["rarity", "traits"],
    });
  });

  it("ignores stale explorer loads after the session changes", async () => {
    const services = createServices();
    const firstDeferred = createDeferred<OntologyDomainModel>();
    const secondDeferred = createDeferred<OntologyDomainModel>();
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const firstSession: SearchFilterExplorerSession = {
      title: "First Explorer",
      model: createLoadingExplorerModel("First Explorer"),
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode: vi.fn(() => firstDeferred.promise),
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "derivedTags",
          label: "Derived Tags",
          description: "Browse live derived tags for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: () => undefined,
    };
    const secondSession: SearchFilterExplorerSession = {
      title: "Second Explorer",
      model: createLoadingExplorerModel("Second Explorer"),
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode: vi.fn(() => secondDeferred.promise),
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "derivedTags",
          label: "Derived Tags",
          description: "Browse live derived tags for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: () => undefined,
    };

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={firstSession} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );
    const rerender = app.rerender as ((tree: React.ReactNode) => void) | undefined;

    await flushInk();
    expect(app.lastFrame()).toContain("Loading explorer entries...");

    rerender?.(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={secondSession} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );
    await flushInk();
    expect(app.lastFrame()).toContain("Loading explorer entries...");

    firstDeferred.resolve(createNamedExplorerDomain("First Result"));
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).not.toContain("First Result");

    secondDeferred.resolve(createNamedExplorerDomain("Second Result"));
    await waitForFrameToContain(app, "Second Result");
    expect(app.lastFrame()).not.toContain("First Result");
  });

  it("completes matching and catalog refresh transitions through the live explorer action rail", async () => {
    const services = createServices();
    const catalogDeferred = createDeferred<OntologyDomainModel>();
    const loadModelForDiscoveryMode = vi.fn((mode: "matching" | "catalog") =>
      mode === "catalog" ? catalogDeferred.promise : Promise.resolve(createNamedExplorerDomain("Matching Result")),
    );
    const session: SearchFilterExplorerSession = {
      title: "Derived Tags Explorer",
      model: createNamedExplorerDomain("Matching Result"),
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "derivedTags",
          label: "Derived Tags",
          description: "Browse live derived tags for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: () => undefined,
    };
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Matching Result");
    expect(app.lastFrame()).toContain("matching counts");

    app.stdin.write(":");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Use Catalog Counts");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Use Matching Counts");
    expect(app.lastFrame()).not.toContain("Use Catalog Counts");

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("matching counts | refreshing catalog");
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(loadModelForDiscoveryMode).toHaveBeenCalledWith("catalog");

    catalogDeferred.resolve(createNamedExplorerDomain("Catalog Result"));
    await waitForFrameToContain(app, "Catalog Result", 80);
    expect(app.lastFrame()).toContain("catalog counts");
    expect(app.lastFrame()).not.toContain("refreshing");

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Use Matching Counts");
    expect(app.lastFrame()).not.toContain("Use Catalog Counts");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Matching Result");
    expect(app.lastFrame()).toContain("matching counts");
    expect(app.lastFrame()).not.toContain("refreshing");
    expect(loadModelForDiscoveryMode).toHaveBeenCalledTimes(1);

    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(app.lastFrame()).toContain("Matching Result");
    expect(app.lastFrame()).not.toContain("Catalog Result");
  });

  it("preserves live discrete selections across parent query rerenders", async () => {
    const services = createServices();
    const fieldOptions = [
      {
        value: "rarity",
        label: "Rarity",
        description: "Browse live rarities for the current scope.",
        fieldType: "enumString" as const,
        editor: "sharedExplorer" as const,
      },
    ];
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;

    function Harness(): React.JSX.Element {
      const [query, setQuery] = React.useState(
        browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      );
      const model = React.useMemo(() => createRarityExplorerDomain(["common"]), []);

      const session = React.useMemo<SearchFilterExplorerSession>(
        () => ({
          title: "Rarity Explorer",
          model,
          query,
          fieldOptions,
          onEvent: (event) => {
            if (event.kind === "change") {
              setQuery(setSearchQueryRaritySelection(event.query, { include: [], exclude: [] }));
            }
          },
          resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        }),
        [model, query],
      );

      return <SearchFilterExplorer session={session} />;
    }

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <Harness />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await waitForFrameToContain(app, "Rarity Explorer");
    app.stdin.write("\r");
    await waitForFrameToContain(app, "common", 60);
    expect(app.lastFrame()).toContain("common");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("[✓] common");
    expect(app.lastFrame()).toContain("Current clauses");
    expect(app.lastFrame()).not.toContain("No filter values selected yet.");
  });

  it("preserves sibling same-field groups when grouped explorer edits seed a live traits session", async () => {
    const services = createServices();
    const fieldOptions = [
      {
        value: "traits",
        label: "Traits",
        description: "Browse live traits for the current scope.",
        fieldType: "set" as const,
        editor: "sharedExplorer" as const,
      },
    ];
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const sourceQuery = browseQuery("Browse spells", {
      filter: allOfFilter([
        scopeFilter("spell"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
          anyOfFilter([
            metadataPredicateFilter({ field: "traits", op: "includes", value: "auditory" }),
            metadataPredicateFilter({ field: "traits", op: "includes", value: "emotion" }),
          ]),
          { kind: "pack", value: "monster-core" },
        ]),
      ]),
      limit: 20,
    }).request;
    const seededState = buildGroupedFieldSeedState(sourceQuery, [1], {
      field: "traits",
      fieldMemberPaths: [[1, 0]],
    });
    const latestQueryRef = {
      current: seededState.seedQuery,
    };

    function Harness(): React.JSX.Element {
      const [query, setQuery] = React.useState(seededState.seedQuery);
      const model = React.useMemo(() => createTraitsExplorerDomain(["illusion", "humanoid"]), []);

      const session = React.useMemo<SearchFilterExplorerSession>(
        () => ({
          title: "Traits Explorer",
          model,
          query,
          initialFieldState: seededState.initialFieldState,
          preservedMetadata: seededState.preservedMetadata,
          fieldOptions,
          onEvent: (event) => {
            if (event.kind === "change") {
              latestQueryRef.current = event.query;
              setQuery(event.query);
            }
          },
          resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
        }),
        [model, query],
      );

      return <SearchFilterExplorer session={session} />;
    }

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <Harness />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await waitForFrameToContain(app, "Traits Explorer");
    expect(app.lastFrame()).toContain("[✓] illusion");

    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("[✓] humanoid");
    expect(getSearchQueryMetadataTree(latestQueryRef.current)).toEqual({
      and: [
        {
          or: [
            { field: "traits", op: "includes", value: "auditory" },
            { field: "traits", op: "includes", value: "emotion" },
          ],
        },
        { field: "traits", op: "includes", value: "humanoid" },
        { field: "traits", op: "includes", value: "illusion" },
      ],
    });
  });

  it("keeps excluded values visible while matching counts refresh in place", async () => {
    const services = createServices();
    const fieldOptions = [
      {
        value: "rarity",
        label: "Rarity",
        description: "Browse live rarities for the current scope.",
        fieldType: "enumString" as const,
        editor: "sharedExplorer" as const,
      },
    ];
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;

    function Harness(): React.JSX.Element {
      const [query, setQuery] = React.useState(
        browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      );
      const queryRef = React.useRef(query);

      React.useEffect(() => {
        queryRef.current = query;
      }, [query]);

      const loadModelForDiscoveryMode = React.useCallback(async () => {
        const filter = JSON.stringify(queryRef.current.filter);
        return filter.includes('"notIn"')
          ? createRarityExplorerDomain(["common"])
          : createRarityExplorerDomain(["common", "rare"]);
      }, []);

      const session = React.useMemo<SearchFilterExplorerSession>(
        () => ({
          title: "Rarity Explorer",
          model: createRarityExplorerDomain(["common", "rare"]),
          query,
          fieldOptions,
          onEvent: (event) => {
            if (event.kind === "change") {
              queryRef.current = event.query;
              setQuery(event.query);
            }
          },
          resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
          refreshOnQueryChange: true,
          initialDiscoveryMode: "matching",
          loadModelForDiscoveryMode,
        }),
        [loadModelForDiscoveryMode, query],
      );

      return <SearchFilterExplorer session={session} />;
    }

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <Harness />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("common");
    expect(app.lastFrame()).toContain("rare");

    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("common");
    expect(app.lastFrame()).toContain("rare");
    expect(app.lastFrame()).toContain("[x] rare");
    expect(app.lastFrame()).toContain("Current clauses");
    expect(app.lastFrame()).not.toContain("No filter values selected yet.");
  });

  it("overlays only selected missing values when live counts refresh", () => {
    const fieldOptions = [
      {
        value: "rarity",
        label: "Rarity",
        description: "Browse live rarities for the current scope.",
        fieldType: "enumString" as const,
        editor: "sharedExplorer" as const,
      },
    ];
    const model = reconcileSearchFilterExplorerModel({
      currentModel: createRarityExplorerDomain(["common", "rare", "unique"]),
      refreshedModel: createRarityExplorerDomain(["common"]),
      fieldState: {
        discreteSelections: {
          rarity: {
            include: [],
            exclude: ["rare"],
          },
        },
        scalarClauses: {},
      },
      fieldOptions,
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
    });

    const rarityField = model.rootNodes[0]!;
    const values = getOntologyNodeChildren(rarityField);
    expect(values.map((node) => node.label)).toEqual(["common", "rare"]);
    expect(values.find((node) => node.label === "rare")?.listLabel).toBe("rare | 0");
    expect(getOntologyNodeChildren(values.find((node) => node.label === "rare"))).toEqual([]);
  });

  it("replaces stale member children for refreshed live rows", () => {
    const fieldOptions = [
      {
        value: "rarity",
        label: "Rarity",
        description: "Browse live rarities for the current scope.",
        fieldType: "enumString" as const,
        editor: "sharedExplorer" as const,
      },
    ];
    const currentModel = createRarityExplorerDomain(["rare"]);
    const refreshedModel = createRarityExplorerDomain(["rare"]);
    const currentRare = getOntologyNodeChildren(currentModel.rootNodes[0])[0];
    const refreshedRare = getOntologyNodeChildren(refreshedModel.rootNodes[0])[0];
    Object.assign(currentRare, {
      childSource: {
        kind: "static" as const,
        children: [
          {
            id: "record:old",
            kind: "record",
            label: "Old Member",
            filterText: "old member",
            detailLines: [{ text: "Old Member" }],
          },
        ],
      },
    });
    Object.assign(refreshedRare, {
      listLabel: "rare | 1",
      childSource: {
        kind: "static" as const,
        children: [
          {
            id: "record:fresh",
            kind: "record",
            label: "Fresh Member",
            filterText: "fresh member",
            detailLines: [{ text: "Fresh Member" }],
          },
        ],
      },
    });

    const model = reconcileSearchFilterExplorerModel({
      currentModel,
      refreshedModel,
      fieldState: { discreteSelections: {}, scalarClauses: {} },
      fieldOptions,
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver(fieldOptions),
    });

    const rare = getOntologyNodeChildren(model.rootNodes[0])[0];
    expect(rare.listLabel).toBe("rare | 1");
    expect(getOntologyNodeChildren(rare).map((node) => node.label)).toEqual(["Fresh Member"]);
  });

  it("keeps the picker responsive while refreshing live counts after a query edit", async () => {
    const services = createServices();
    const loadModelForDiscoveryMode = vi.fn(async () => createRarityExplorerDomain(["rare"]));
    const session: SearchFilterExplorerSession = {
      title: "Rarity Explorer",
      model: createRarityExplorerDomain(["common", "rare"]),
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
        {
          value: "traits",
          label: "Traits",
          description: "Browse live traits for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
        {
          value: "traits",
          label: "Traits",
          description: "Browse live traits for the current scope.",
          fieldType: "set",
          editor: "sharedExplorer",
        },
      ]),
      refreshOnQueryChange: true,
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
    };
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await flushInk();

    expect(loadModelForDiscoveryMode).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("common");
    expect(app.lastFrame()).toContain("rare");
    expect(app.lastFrame()).toContain("[✓] rare");

    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();

    expect(loadModelForDiscoveryMode).toHaveBeenCalledTimes(1);
    expect(loadModelForDiscoveryMode).toHaveBeenLastCalledWith("matching", { targetFields: ["rarity"] });
    expect(app.lastFrame()).not.toContain("common");
    expect(app.lastFrame()).toContain("rare");
    expect(app.lastFrame()).toContain("[✓] rare");
  });

  it("ignores stale live count refreshes after rapid query edits", async () => {
    const services = createServices();
    const firstRefresh = createDeferred<OntologyDomainModel>();
    const secondRefresh = createDeferred<OntologyDomainModel>();
    const loadModelForDiscoveryMode = vi
      .fn<(_: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>>()
      .mockReturnValueOnce(firstRefresh.promise)
      .mockReturnValueOnce(secondRefresh.promise);
    const session: SearchFilterExplorerSession = {
      title: "Rarity Explorer",
      model: createRarityExplorerDomain(["common", "rare", "unique"]),
      query: browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
      fieldOptions: [
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ],
      onEvent: vi.fn(),
      resolveSelectionTarget: buildSearchFilterExplorerTargetResolver([
        {
          value: "rarity",
          label: "Rarity",
          description: "Browse live rarities for the current scope.",
          fieldType: "enumString",
          editor: "sharedExplorer",
        },
      ]),
      refreshOnQueryChange: true,
      initialDiscoveryMode: "matching",
      loadModelForDiscoveryMode,
    };
    const SearchFilterExplorer = SearchFilterExplorerScreen as React.ComponentType<{
      session: SearchFilterExplorerSession;
    }>;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchFilterExplorer session={session} />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(loadModelForDiscoveryMode).toHaveBeenCalledTimes(1);

    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await new Promise((resolve) => {
      setTimeout(resolve, 120);
    });
    await flushInk();
    expect(loadModelForDiscoveryMode).toHaveBeenCalledTimes(2);

    firstRefresh.resolve(createNamedExplorerDomain("First Result"));
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).not.toContain("First Result");

    secondRefresh.resolve(createNamedExplorerDomain("Second Result"));
    await waitForFrameToContain(app, "Second Result", 80);
    expect(app.lastFrame()).toContain("Second Result");
    expect(app.lastFrame()).not.toContain("First Result");
  });

  it("supports pane clicks and hovered-pane wheel routing in the live search host", async () => {
    const services = createServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalAppServicesProvider services={services}>
          <SearchScreen
            entry="results"
            initialSession={createSearchSession({
              results: [
                createRecord(),
                createRecord({
                  recordKey: "spell:test-barrier",
                  id: "test-barrier",
                  name: "Barrier Ward",
                  normalizedName: "barrier ward",
                  descriptionText: "Shields a threshold against passage.",
                  descriptionSnippet: "Shields a threshold against passage.",
                  sourcePath: "packs/spells/barrier-ward.json",
                }),
              ],
            })}
            onBack={vi.fn()}
          />
        </Pf2eTerminalAppServicesProvider>
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("\u001b[<0;70;6M");
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Identity");

    app.stdin.write("\u001b[<0;10;6M");
    await flushInk();
    expect(app.lastFrame()).toContain("Preview | Alarm Ward | Identity");

    app.stdin.write("\u001b[<65;10;6M");
    await flushInk();
    expect(app.lastFrame()).toContain("Preview | Barrier Ward | Identity");
  });
});
