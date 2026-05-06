import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createPf2eApplicationEntityPageService } from "../../src/app/ontology/entity-page-service.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { SearchFilterNode, SearchRequest } from "../../src/domain/search-request-types.js";
import type { SearchCountResult } from "../../src/domain/search-types.js";
import { Pf2eTerminalAppServicesProvider } from "../../src/tui/app-service-context.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { createNoopTerminalDebugTraceService } from "../../src/tui/debug-trace.js";
import {
  liftSearchFilterNodeAtPath,
  moveSearchFilterNodeToGroupPath,
  toggleSearchFilterRootGroupOperator,
  unwrapSearchFilterNodeAtPath,
  updateSearchFilterNodeAtPath,
} from "../../src/tui/search/query-core.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import type { SearchStructuredDraftEntry } from "../../src/tui/search/structured-draft-session.js";
import { buildStructuredDraftEntries } from "../../src/tui/search-screen/structured-draft/structured-draft-support.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftRootResumeTarget,
  getStructuredDraftSelectionIndexForResumeTarget,
  type StructuredDraftResumeTarget,
} from "../../src/tui/search-screen/structured-draft/structured-draft-state.js";
import { SearchScreen } from "../../src/tui/search-screen/screen.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import {
  actionCostFilter,
  allOfFilter,
  anyOfFilter,
  browseQuery,
  levelFilter,
  metadataPredicateFilter,
  metricCompareFilter,
  packFilter,
  notFilter,
  rarityFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

type SearchServiceDependencies = Parameters<typeof createPf2eTerminalSearchService>[0];
type CountRecordsFn = SearchServiceDependencies["countRecords"];
type ListRecordsFn = SearchServiceDependencies["listRecords"];
type LookupFn = SearchServiceDependencies["lookup"];
type OpenSearchWindowFn = SearchServiceDependencies["openSearchWindow"];
type ReadSearchWindowPageFn = SearchServiceDependencies["readSearchWindowPage"];
type SearchFn = SearchServiceDependencies["search"];

/*
Structured editor continuation host-flow coverage matrix.

| Bucket | Test name |
| --- | --- |
| add-here grouped field continuation, canonical + visible tree | keeps grouped add-here trait continuations canonical and stable across reopen |
| edit-clause grouped field continuation | rehydrates grouped trait edit-clause continuations without nesting the field bucket |
| mixed field-family continuation | keeps trait and family continuations flat in one group-local host flow |
| reopen after continuation | keeps grouped add-here trait continuations canonical and stable across reopen |
| move/lift/unwrap/remove/toggle-root after prior grouped edits | keeps structural follow-through actions stable after grouped edits |
| host-level root toggle after grouped edits | keeps host structural root toggles and unwrap flat after grouped edits and reopen |
| host-level nested unwrap after grouped edits | keeps host structural root toggles and unwrap flat after grouped edits and reopen |
| host-level move after grouped edits | keeps host structural moves flat after grouped edits and reopen |
| host-level lift after grouped edits | keeps host structural lifts flat after grouped edits and reopen |
| host-level remove after grouped edits | keeps host structural removes flat after grouped edits and reopen |
| focus restoration after structural reshapes | keeps structural follow-through actions stable after grouped edits |
| scalar/metric follow-through range entry | recovers from invalid scalar input and then commits an action-cost range |
| scalar/metric cancel/back follow-through | keeps metric q and repeated-back cancellation from mutating grouped trees |
| invalid input recovery | recovers from invalid scalar input and then commits an action-cost range |
| action-cost shared-explorer grouped edit | edits action-cost fields through the shared explorer without query-global replacement |
| pack/rarity/action-cost grouped edit flatness | grouped shared-explorer edits assert visible tree and canonical filter output have no synthetic boolean groups |
| canonical and visible-tree expectations | every host-flow test pairs visible frame assertions with canonical filter assertions where practical |
*/

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

type LegacyOntologyNodeFixture = OntologyDomainModel["rootNodes"][number] & { children?: LegacyOntologyNodeFixture[] };

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

function pressDown(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[B");
}

function pressUp(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[A");
}

function pressLeft(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[D");
}

function pressEnter(app: ReturnType<typeof render>): void {
  app.stdin.write("\r");
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
      return { values: ["unique", "common", "rare", "uncommon"].map((value) => ({ value, count: 1 })) };
    }
    if (field === "actionCost") {
      return { values: ["1", "2", "3"].map((value) => ({ value, count: 1 })) };
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
        sortSeed: null,
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
    closeSearchWindow: vi.fn(),
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
            children: [],
          },
        ],
      },
    ],
  });
}

function createStructuredTraitsExplorerDomain(values: readonly string[]): OntologyDomainModel {
  const domain = createFacetPickerOntologyDomain();
  const metadataFields = domain.rootNodes[0]?.children?.[0];
  metadataFields?.children?.push({
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
    children: values.map((value) => ({
      id: `spell:traits:${value}`,
      kind: "trait",
      label: value,
      filterText: value,
      listLabel: value,
      detailTitle: "Trait Details",
      detailLines: [{ text: value, tone: "section" }],
      groupValues: { family: "traits" },
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

function createCreatureStructuredExplorerDomain(): OntologyDomainModel {
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
                childPresentation: { mode: "grouped", groupBy: "family", render: "inline" },
                children: ["humanoid", "evil"].map((value) => ({
                  id: `creature:traits:${value}`,
                  kind: "trait",
                  label: value,
                  filterText: value,
                  listLabel: value,
                  detailTitle: "Trait Details",
                  detailLines: [{ text: value, tone: "section" }],
                  groupValues: { family: "traits" },
                  selection: {
                    field: "traits",
                    fieldLabel: "Traits",
                    value,
                    allowedStates: ["any", "all", "exclude"],
                  },
                })),
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
            ],
          },
        ],
      },
    ],
  });
}

function createEquipmentStructuredExplorerDomain(): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Equipment",
    description: "Equipment structured explorer test domain",
    rootNodes: [
      {
        id: "searchSemantics:equipment",
        kind: "category",
        label: "Equipment",
        shortLabel: "equipment",
        filterText: "equipment",
        detailTitle: "Equipment",
        detailLines: [{ text: "Equipment", tone: "section" }],
        children: [
          {
            id: "equipment:metadataFields",
            kind: "group",
            label: "Metadata Fields",
            filterText: "metadata fields",
            detailTitle: "Metadata Fields",
            detailLines: [{ text: "Metadata Fields", tone: "section" }],
            children: [
              {
                id: "equipment:field:traits",
                kind: "field",
                label: "Traits",
                filterText: "traits",
                listLabel: "Traits",
                detailTitle: "Metadata Field Details",
                detailLines: [{ text: "Traits", tone: "section" }],
                childPresentation: { mode: "grouped", groupBy: "family", render: "inline" },
                children: ["magical", "consumable", "cursed", "alchemical", "invested"].map((value) => ({
                  id: `equipment:traits:${value}`,
                  kind: "trait",
                  label: value,
                  filterText: value,
                  listLabel: value,
                  detailTitle: "Trait Details",
                  detailLines: [{ text: value, tone: "section" }],
                  groupValues: { family: "traits" },
                  selection: {
                    field: "traits",
                    fieldLabel: "Traits",
                    value,
                    allowedStates: ["any", "all", "exclude"],
                  },
                })),
              },
            ],
          },
          {
            id: "equipment:itemMetrics:discovery",
            kind: "group",
            label: "Equipment Statistics",
            filterText: "equipment statistics metrics",
            listLabel: "Equipment Statistics",
            detailTitle: "Equipment Statistics",
            detailLines: [{ text: "Equipment Statistics", tone: "section" }],
            children: [
              {
                id: "equipment:itemMetrics:bulk.",
                kind: "metricNamespace",
                label: "bulk.",
                filterText: "bulk carried item bulk",
                listLabel: "bulk. | 1 metric",
                detailTitle: "Metric Namespace",
                detailLines: [{ text: "bulk.", tone: "section" }],
                children: [
                  {
                    id: "equipment:itemMetrics:bulk.value",
                    kind: "metric",
                    label: "Bulk Value",
                    filterText: "bulk value",
                    listLabel: "Bulk Value | 3",
                    detailTitle: "Metric Details",
                    detailLines: [{ text: "Bulk Value", tone: "section" }],
                    query: browseQuery("Browse equipment by bulk", {
                      filter: allOfFilter([
                        scopeFilter("equipment"),
                        {
                          kind: "metric",
                          metric: "bulk.value",
                          op: "gte",
                          value: 1,
                        },
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

function createFacetPickerOntologyDomainWithDiscreteFields(): OntologyDomainModel {
  const domain = createFacetPickerOntologyDomain();
  const metadataFields = domain.rootNodes[0]?.children?.[0];
  metadataFields?.children?.push(
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
          id: "spell:field:rarity:value:uncommon",
          kind: "value",
          label: "uncommon",
          filterText: "uncommon",
          listLabel: "uncommon",
          detailTitle: "Value Details",
          detailLines: [{ text: "uncommon", tone: "section" }],
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
        {
          id: "spell:field:rarity:value:unique",
          kind: "value",
          label: "unique",
          filterText: "unique",
          listLabel: "unique",
          detailTitle: "Value Details",
          detailLines: [{ text: "unique", tone: "section" }],
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

function renderSearch(
  services: Pf2eTerminalAppServices,
  initialRequest: SearchRequest,
): ReturnType<typeof render> {
  return render(
    <DerivedTagTerminalProvider>
      <Pf2eTerminalAppServicesProvider services={services}>
        <SearchScreen initialRequest={initialRequest} onBack={vi.fn()} />
      </Pf2eTerminalAppServicesProvider>
    </DerivedTagTerminalProvider>,
  );
}

async function openStructuredQueryEditor(app: ReturnType<typeof render>): Promise<void> {
  await flushInk();
  pressLeft(app);
  await flushInk();
  pressDown(app);
  await flushInk();
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Structured Query Editor", 60);
}

async function returnFromExplorerToStructuredEditor(app: ReturnType<typeof render>): Promise<void> {
  for (let step = 0; step < 6; step += 1) {
    if (app.lastFrame().includes("Structured Query Editor")) {
      return;
    }
    pressLeft(app);
    await flushInk();
  }
  await waitForFrameToContain(app, "Structured Query Editor", 60);
}

async function openTraitsExplorerFromAddHere(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("\r");
  await waitForFrameToContain(app, "Add Clause", 60);
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

async function openMetricExplorerFromAddHere(app: ReturnType<typeof render>, title: string): Promise<void> {
  pressEnter(app);
  await waitForFrameToContain(app, "Add Clause", 60);
  pressDown(app);
  await flushInk();
  pressDown(app);
  await flushInk();
  pressEnter(app);
  await flushInk();
  await flushInk();
  expect(app.lastFrame()).toContain("Metric");
  expect(app.lastFrame()).toContain(title);
  pressEnter(app);
  await flushInk();
  await flushInk();
  await waitForFrameToContain(app, `${title} Explorer`, 60);
}

async function addRootTraitGroup(app: ReturnType<typeof render>): Promise<void> {
  await openTraitsExplorerFromAddHere(app);
  await waitForFrameToContain(app, "archetype", 120);
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
  await returnFromExplorerToStructuredEditor(app);
}

async function executeCurrentBrowseQuery(app: ReturnType<typeof render>): Promise<void> {
  if (app.lastFrame().includes("Structured Query Editor")) {
    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
  }
  app.stdin.write("\t");
  await waitForFrameToContain(app, "[RESULTS]", 60);
}

function lastListRequest(listRecords: ListRecordsFn): SearchRequest {
  const calls = vi.mocked(listRecords).mock.calls;
  const request = calls.at(-1)?.[0];
  if (!request) {
    throw new Error("expected listRecords to have been called");
  }
  return request;
}

async function openStructuredActionMenuMatching(
  app: ReturnType<typeof render>,
  matches: (frame: string) => boolean,
  attempts = 10,
): Promise<string> {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    pressEnter(app);
    await flushInk();
    await flushInk();
    const actionFrame = app.lastFrame();
    if (matches(actionFrame)) {
      return actionFrame;
    }
    pressLeft(app);
    await waitForFrameToContain(app, "Structured Query Editor", 60);
    pressDown(app);
    await flushInk();
  }
  throw new Error(`Could not find matching structured action menu.\n\n${app.lastFrame()}`);
}

async function runCurrentActionByOffset(app: ReturnType<typeof render>, offset: number): Promise<void> {
  for (let step = 0; step < offset; step += 1) {
    pressDown(app);
    await flushInk();
  }
  pressEnter(app);
  await waitForFrameToContain(app, "Structured Query Editor", 60);
}

async function runCurrentPromptActionByLabel(app: ReturnType<typeof render>, label: string): Promise<void> {
  await waitForFrameToContain(app, label, 60);
  for (let step = 0; step < 12; step += 1) {
    if (app.lastFrame().includes(`Selected: ${label}`)) {
      pressEnter(app);
      await waitForFrameToContain(app, "Structured Query Editor", 60);
      return;
    }
    pressDown(app);
    await flushInk();
  }
  throw new Error(`Could not select prompt action "${label}".\n\n${app.lastFrame()}`);
}

function structuredEntriesFor(
  filter: SearchFilterNode | undefined,
  resumeTarget: StructuredDraftResumeTarget,
): SearchStructuredDraftEntry[] {
  return buildStructuredDraftEntries({ mode: "browse", filter }, resumeTarget, {
    groupedFieldValues: new Set(["traits", "families"]),
  });
}

function selectedStructuredEntry(
  filter: SearchFilterNode | undefined,
  resumeTarget: StructuredDraftResumeTarget,
): SearchStructuredDraftEntry {
  const entries = structuredEntriesFor(filter, resumeTarget);
  return entries[getStructuredDraftSelectionIndexForResumeTarget(entries, resumeTarget, 0)]!;
}

function expectVisibleTraitNode(filter: SearchFilterNode | undefined, label: string): void {
  expect(structuredEntriesFor(filter, createStructuredDraftGroupResumeTarget([]))).toContainEqual(
    expect.objectContaining({
      kind: "queryNode",
      label,
    }),
  );
}

function expectNoDuplicateGroupedTraitProjection(frame: string, expectedTraitRows: number): void {
  const visibleTraitRows = frame
    .split("\n")
    .map((line) => line.slice(0, 32))
    .filter((line) => /^[│ ]*[├└]─/.test(line) && line.includes("Traits:"));
  expect(visibleTraitRows).toHaveLength(expectedTraitRows);
  expect(frame.match(/^├─ All of$/m)).toBeNull();
  expect(frame.match(/^├─ Any of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ All of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ Any of$/m)).toBeNull();
}

function expectNoSyntheticGroupedFieldProjection(frame: string): void {
  expect(frame).not.toContain("Filter: Any of (");
  expect(frame).not.toContain("Filter: ! Any of (");
  expect(frame).not.toContain("Filter: All of (");
  expect(frame.match(/^├─ Any of$/m)).toBeNull();
  expect(frame.match(/^├─ ! Any of$/m)).toBeNull();
  expect(frame.match(/^├─ All of$/m)).toBeNull();
  expect(frame.match(/^├─ ! All of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ Any of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ ! Any of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ All of$/m)).toBeNull();
  expect(frame.match(/^│ {2}├─ ! All of$/m)).toBeNull();
}

function createMetricRegressionFilter(): SearchFilterNode {
  return allOfFilter([
    packFilter("pathfinder-equipment-core"),
    scopeFilter("equipment"),
    levelFilter({ kind: "gt", value: 3 }),
    anyOfFilter([
      metadataPredicateFilter({ field: "traits", op: "includes", value: "magical" }),
      metadataPredicateFilter({ field: "traits", op: "includes", value: "consumable" }),
    ]),
    notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "cursed" })),
  ])!;
}

function createMetricRegressionServices(listRecords: ListRecordsFn): Pf2eTerminalAppServices {
  const services = createServices({ listRecords });
  services.user.search.getQueryFieldOptions = vi.fn(() => [
    {
      value: "traits",
      label: "Traits",
      description: "Trait query field for the current browse scope.",
      fieldType: "set",
      editor: "sharedExplorer",
    },
    {
      value: "itemMetric",
      label: "Equipment Statistics",
      description: "Browse live equipment metrics.",
      fieldType: "enumString",
      editor: "sharedExplorer",
    },
  ]);
  services.user.search.getPackLabel = vi.fn((packValue: string) =>
    packValue === "pathfinder-equipment-core" ? "Pathfinder Equipment Core" : packValue,
  );
  services.user.search.loadMetricKeyOptions = vi.fn(async (_query, field) =>
    field === "itemMetric"
      ? [
          {
            value: "bulk.value",
            label: "bulk.value",
            description: "3 matching canonical records.",
            count: 3,
          },
        ]
      : [],
  );
  services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createEquipmentStructuredExplorerDomain());
  return services;
}

function createTraitContinuationServices(listRecords: ListRecordsFn): Pf2eTerminalAppServices {
  const services = createServices({ listRecords });
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
  return services;
}

describe("search structured editor continuation", () => {
  afterEach(() => {
    cleanup();
  });

  it("keeps grouped add-here trait continuations canonical and stable across reopen", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
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

    const app = renderSearch(
      services,
      browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
    );

    await openStructuredQueryEditor(app);
    await addRootTraitGroup(app);

    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame()).toContain("Metadata predicates: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).not.toContain("Filter: Any of (2 filters)");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
      ]),
    );
  });

  it("rehydrates grouped trait edit-clause continuations without nesting the field bucket", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
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
      createStructuredTraitsExplorerDomain(["archetype", "skill", "concentrate"]),
    );

    const app = renderSearch(
      services,
      browseQuery("Browse spells", {
        filter: allOfFilter([
          scopeFilter("spell"),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
        ]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame().toLowerCase()).toContain("concent");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Traits Explorer", 60);
    await waitForFrameToContain(app, "skill", 120);
    await returnFromExplorerToStructuredEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame().toLowerCase()).toContain("concent");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();
    expect(app.lastFrame().match(/^├─ Any of$/m)).toBeNull();

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
      ]),
    );
  });

  it("keeps flat equipment traits flat when edit-clause returns without changes", async () => {
    const initialFilter = allOfFilter([
      packFilter("equipment"),
      scopeFilter("equipment"),
      metadataPredicateFilter({ field: "traits", op: "includes", value: "consumable" }),
      notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "alchemical" })),
      notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "invested" })),
      metadataPredicateFilter({ field: "traits", op: "includes", value: "magical" }),
    ]);
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord({ category: "equipment", type: "equipment", packName: "equipment" })],
    }));
    const services = createServices({ listRecords });
    services.user.search.getQueryFieldOptions = vi.fn(() => [
      {
        value: "traits",
        label: "Traits",
        description: "Trait query field for the current browse scope.",
        fieldType: "set",
        editor: "sharedExplorer",
      },
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createEquipmentStructuredExplorerDomain());

    const app = renderSearch(
      services,
      browseQuery("Browse equipment", {
        filter: initialFilter,
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Pack: equipment");
    expect(app.lastFrame()).toContain("Scope: Equipment");
    expect(app.lastFrame()).toContain("Traits: includes Consumable");
    expect(app.lastFrame()).toContain("! Traits: includes Alchemical");
    expect(app.lastFrame()).toContain("! Traits: includes Invested");
    expect(app.lastFrame()).toContain("Traits: includes Magical");

    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    pressEnter(app);
    await waitForFrameToContain(app, "Query Clause", 60);
    pressEnter(app);
    await waitForFrameToContain(app, "Traits Explorer", 60);
    await waitForFrameToContain(app, "[✓] consumable", 120);

    await returnFromExplorerToStructuredEditor(app);

    const frame = app.lastFrame();
    expect(frame).toContain("Pack: equipment");
    expect(frame).toContain("Scope: Equipment");
    expect(frame).toContain("Traits: includes Consumable");
    expect(frame).toContain("! Traits: includes Alchemical");
    expect(frame).toContain("! Traits: includes Invested");
    expect(frame).toContain("Traits: includes Magical");
    expect(frame.match(/^├─ All of$/m)).toBeNull();
    expect(frame).not.toContain("Filter: All of (4 filters)");

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(initialFilter);
  });

  it("keeps trait and family continuations flat in one group-local host flow", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
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
    ]);
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () => createCreatureStructuredExplorerDomain());

    const app = renderSearch(
      services,
      browseQuery("Browse creatures", {
        filter: allOfFilter([
          scopeFilter("creature"),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
          notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
        ]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Add Clause", 60);
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Families");
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Families Explorer", 60);
    await waitForFrameToContain(app, "ancestry npcs", 120);
    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    await returnFromExplorerToStructuredEditor(app);

    expect(app.lastFrame()).toContain("Traits: includes Humanoid");
    expect(app.lastFrame().match(/^├─ (! Traits: includes Evil|Traits: !evil)/m)).not.toBeNull();
    expect(app.lastFrame()).toContain("Families: includes Ancestry Npcs");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expect(app.lastFrame().match(/^├─ All of$/m)).toBeNull();

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "humanoid" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "evil" })),
        metadataPredicateFilter({ field: "families", op: "includes", value: "ancestry_npcs" }),
      ]),
    );
  });

  it("keeps metric q and repeated-back cancellation from mutating grouped trees", async () => {
    const expectedFilter = createMetricRegressionFilter();
    const createListRecords = () =>
      vi.fn((request: SearchRequest) => ({
        searchProfile: null,
        mode: "structured" as const,
        sort: request.sort ?? "alphabetical",
        total: 1,
        offset: request.offset ?? 0,
        limit: request.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [createRecord({ category: "equipment", type: "equipment", packName: "equipment" })],
      })) satisfies ListRecordsFn;

    const qListRecords = createListRecords();
    const qServices = createMetricRegressionServices(qListRecords);
    const qApp = renderSearch(
      qServices,
      browseQuery("Browse reported equipment", { filter: expectedFilter, limit: 20 }).request,
    );

    await openStructuredQueryEditor(qApp);
    expect(qApp.lastFrame()).toContain("Pack: Pathfinder Equipment Core");
    expect(qApp.lastFrame()).toContain("Scope: Equipment");
    expect(qApp.lastFrame()).toContain("Level: > 3");
    expect(qApp.lastFrame()).toContain("Traits: includes Consumable");
    expect(qApp.lastFrame()).toContain("Traits: includes Magical");
    expect(qApp.lastFrame()).toContain("! Traits: includes Cursed");
    expectNoDuplicateGroupedTraitProjection(qApp.lastFrame(), 3);

    await openMetricExplorerFromAddHere(qApp, "Equipment Statistics");
    await waitForFrameToContain(qApp, "bulk.", 120);
    qApp.stdin.write("q");
    await waitForFrameToContain(qApp, "Structured Query Editor", 60);

    expect(qApp.lastFrame()).toContain("Traits: includes Consumable");
    expect(qApp.lastFrame()).toContain("Traits: includes Magical");
    expect(qApp.lastFrame()).toContain("! Traits: includes Cursed");
    expect(qApp.lastFrame()).not.toContain("Equipment Statistics: bulk");
    expectNoDuplicateGroupedTraitProjection(qApp.lastFrame(), 3);
    await executeCurrentBrowseQuery(qApp);
    expect(lastListRequest(qListRecords).filter).toEqual(expectedFilter);

    cleanup();

    const backListRecords = createListRecords();
    const backServices = createMetricRegressionServices(backListRecords);
    const backApp = renderSearch(
      backServices,
      browseQuery("Browse reported equipment", { filter: expectedFilter, limit: 20 }).request,
    );

    await openStructuredQueryEditor(backApp);
    await openMetricExplorerFromAddHere(backApp, "Equipment Statistics");
    await waitForFrameToContain(backApp, "bulk.", 120);
    for (let step = 0; step < 4 && !backApp.lastFrame().includes("Structured Query Editor"); step += 1) {
      pressLeft(backApp);
      await flushInk();
    }
    await waitForFrameToContain(backApp, "Structured Query Editor", 60);

    expect(backApp.lastFrame()).toContain("Pack: Pathfinder Equipment Core");
    expect(backApp.lastFrame()).toContain("Traits: includes Consumable");
    expect(backApp.lastFrame()).toContain("Traits: includes Magical");
    expect(backApp.lastFrame()).toContain("! Traits: includes Cursed");
    expect(backApp.lastFrame()).not.toContain("Equipment Statistics: bulk");
    expectNoDuplicateGroupedTraitProjection(backApp.lastFrame(), 3);
    await executeCurrentBrowseQuery(backApp);
    expect(lastListRequest(backListRecords).filter).toEqual(expectedFilter);
  });

  it("edits action-cost leaves through the shared explorer without query-global replacement", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = renderSearch(
      services,
      browseQuery("Browse spells", {
        filter: allOfFilter([
          scopeFilter("spell"),
          actionCostFilter({ kind: "eq", value: 2 }),
          rarityFilter({ kind: "eq", value: "common" }),
        ]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Action Cost: 2");
    expect(app.lastFrame()).toContain("Rarity: Common");

    pressUp(app);
    await flushInk();
    pressUp(app);
    await flushInk();
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Query Clause", 60);
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Action Cost Explorer", 60);
    await waitForFrameToContain(app, "1 action", 120);
    app.stdin.write("\r");
    await flushInk();
    await waitForFrameToContain(app, "[x] 1 action", 120);
    pressDown(app);
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    app.stdin.write(" ");
    await flushInk();
    await waitForFrameToContain(app, "[.] 2 actions", 120);
    await returnFromExplorerToStructuredEditor(app);

    expect(app.lastFrame()).toContain("Action Cost: 1");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expectNoSyntheticGroupedFieldProjection(app.lastFrame());

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        actionCostFilter({ kind: "eq", value: 1 }),
        rarityFilter({ kind: "eq", value: "common" }),
      ]),
    );
  });

  it("keeps shared-explorer rarity include and exclude multi-selects flat", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
    services.user.ontology.loadSearchFilterExplorerDomain = vi.fn(async () =>
      createFacetPickerOntologyDomainWithDiscreteFields(),
    );

    const app = renderSearch(
      services,
      browseQuery("Browse spells", {
        filter: allOfFilter([
          scopeFilter("spell"),
          actionCostFilter({ kind: "eq", value: 2 }),
          rarityFilter({ kind: "eq", value: "common" }),
        ]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Action Cost: 2");
    expect(app.lastFrame()).toContain("Rarity: Common");

    pressUp(app);
    await flushInk();
    pressEnter(app);
    await waitForFrameToContain(app, "Query Clause", 60);
    pressEnter(app);
    await waitForFrameToContain(app, "Rarity Explorer", 60);
    await waitForFrameToContain(app, "[✓] common", 120);

    pressDown(app);
    await waitForFrameToContain(app, "uncommon", 120);
    app.stdin.write(" ");
    await waitForFrameToContain(app, "[✓] uncommon", 120);

    pressDown(app);
    await waitForFrameToContain(app, "rare", 120);
    app.stdin.write(" ");
    await waitForFrameToContain(app, "[✓] rare", 120);
    app.stdin.write(" ");
    await waitForFrameToContain(app, "[x] rare", 120);

    pressDown(app);
    await waitForFrameToContain(app, "unique", 120);
    app.stdin.write(" ");
    await waitForFrameToContain(app, "[✓] unique", 120);
    app.stdin.write(" ");
    await waitForFrameToContain(app, "[x] unique", 120);

    await returnFromExplorerToStructuredEditor(app);

    const frame = app.lastFrame();
    expect(frame).toContain("Rarity: Common");
    expect(frame).toContain("Rarity: Uncommon");
    expect(frame).toContain("! Rarity: Rare");
    expect(frame).toContain("! Rarity: Unique");
    expect(frame).toContain("Action Cost: 2");
    expectNoSyntheticGroupedFieldProjection(frame);

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        actionCostFilter({ kind: "eq", value: 2 }),
        rarityFilter({ kind: "eq", value: "common" }),
        rarityFilter({ kind: "eq", value: "uncommon" }),
        notFilter(rarityFilter({ kind: "eq", value: "rare" })),
        notFilter(rarityFilter({ kind: "eq", value: "unique" })),
      ]),
    );
  });

  it("keeps structural follow-through actions stable after grouped edits", () => {
    const groupedFilter = allOfFilter([
      scopeFilter("spell"),
      anyOfFilter([
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
      ]),
      notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
      rarityFilter({ kind: "eq", value: "common" }),
    ])!;

    const toggledRoot = toggleSearchFilterRootGroupOperator(groupedFilter);
    expect(toggledRoot).toMatchObject({ kind: "anyOf" });
    expect(selectedStructuredEntry(toggledRoot, createStructuredDraftRootResumeTarget())).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [],
    });

    const removedExclude = updateSearchFilterNodeAtPath(groupedFilter, [2], () => undefined);
    expect(removedExclude).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        anyOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        ]),
        rarityFilter({ kind: "eq", value: "common" }),
      ]),
    );
    expect(selectedStructuredEntry(removedExclude, createStructuredDraftGroupResumeTarget([]))).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [],
    });
    expectVisibleTraitNode(removedExclude, "Traits: includes Archetype");

    const nestedGroupFilter = allOfFilter([
      scopeFilter("spell"),
      allOfFilter([
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
      ]),
      rarityFilter({ kind: "eq", value: "common" }),
    ])!;
    const unwrappedGroup = unwrapSearchFilterNodeAtPath(nestedGroupFilter, [1]);
    expect(unwrappedGroup).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        rarityFilter({ kind: "eq", value: "common" }),
      ]),
    );
    expectVisibleTraitNode(unwrappedGroup, "Traits: includes Archetype");

    const liftedRarity = liftSearchFilterNodeAtPath(nestedGroupFilter, [1, 1]);
    expect(liftedRarity).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        {
          kind: "allOf",
          children: [metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" })],
        },
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        rarityFilter({ kind: "eq", value: "common" }),
      ]),
    );
    expectVisibleTraitNode(liftedRarity, "Traits: includes Dedication");

    const movedRarityIntoGroup = moveSearchFilterNodeToGroupPath(nestedGroupFilter, [2], [1], "allOf");
    expect(movedRarityIntoGroup).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        allOfFilter([
          metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
          metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
          rarityFilter({ kind: "eq", value: "common" }),
        ]),
      ]),
    );
    expect(selectedStructuredEntry(movedRarityIntoGroup, createStructuredDraftGroupResumeTarget([1]))).toMatchObject({
      kind: "queryTreeRoot",
      treePath: [1],
    });
  });

  it("keeps host structural root toggles and unwrap flat after grouped edits and reopen", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
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

    const app = renderSearch(
      services,
      browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
    );

    await openStructuredQueryEditor(app);
    await addRootTraitGroup(app);

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    app.stdin.write("g");
    await flushInk();
    app.stdin.write("g");
    await flushInk();

    await openStructuredActionMenuMatching(
      app,
      (frame) => frame.includes("Root Group") && frame.includes("Change Root To Any of"),
    );
    await runCurrentActionByOffset(app, 4);
    expect(app.lastFrame()).toContain("Query Logic: Any of (4 filters)");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame()).toContain("Top-level filters: 4");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      anyOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
      ]),
    );
  });

  it("keeps host structural moves flat after grouped edits and reopen", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createTraitContinuationServices(listRecords);
    const app = renderSearch(
      services,
      browseQuery("Browse spells", {
        filter: allOfFilter([scopeFilter("spell"), rarityFilter({ kind: "eq", value: "common" })]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    await addRootTraitGroup(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    await openStructuredActionMenuMatching(
      app,
      (frame) => frame.includes("Move Node"),
      16,
    );
    await runCurrentPromptActionByLabel(app, "Move Node");
    app.stdin.write("G");
    await waitForFrameToContain(app, "Move Here", 60);
    pressEnter(app);
    await waitForFrameToContain(app, "Structured Query Editor", 60);

    expect(app.lastFrame()).toContain("Top-level filters: 5");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        rarityFilter({ kind: "eq", value: "common" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
        scopeFilter("spell"),
      ]),
    );
  });

  it("keeps host structural lifts flat after grouped edits and reopen", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createTraitContinuationServices(listRecords);
    const app = renderSearch(
      services,
      browseQuery("Browse spells", {
        filter: allOfFilter([
          scopeFilter("spell"),
          anyOfFilter([
            allOfFilter([rarityFilter({ kind: "eq", value: "common" }), levelFilter({ kind: "gt", value: 1 })]),
            actionCostFilter({ kind: "eq", value: 2 }),
          ]),
        ]),
        limit: 20,
      }).request,
    );

    await openStructuredQueryEditor(app);
    await addRootTraitGroup(app);
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Level: > 1");
    expect(app.lastFrame()).toContain("Action Cost: 2");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    await openStructuredActionMenuMatching(
      app,
      (frame) => frame.includes("Boolean Group") && frame.includes("Lift Node"),
      18,
    );
    await runCurrentPromptActionByLabel(app, "Lift Node");

    expect(app.lastFrame()).toContain("Top-level filters: 6");
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Level: > 1");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Rarity: Common");
    expect(app.lastFrame()).toContain("Level: > 1");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 3);

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        { kind: "anyOf", children: [actionCostFilter({ kind: "eq", value: 2 })] },
        allOfFilter([rarityFilter({ kind: "eq", value: "common" }), levelFilter({ kind: "gt", value: 1 })]),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
        notFilter(metadataPredicateFilter({ field: "traits", op: "includes", value: "concentrate" })),
      ]),
    );
  });

  it("keeps host structural removes flat after grouped edits and reopen", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createTraitContinuationServices(listRecords);
    const app = renderSearch(
      services,
      browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
    );

    await openStructuredQueryEditor(app);
    await addRootTraitGroup(app);
    expect(app.lastFrame()).toContain("! Traits: includes Concentrat");

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    for (let step = 0; step < 3; step += 1) {
      pressDown(app);
      await flushInk();
    }
    await openStructuredActionMenuMatching(
      app,
      (frame) => frame.includes("Exclude Group") && frame.includes("Remove Group"),
      16,
    );
    await runCurrentPromptActionByLabel(app, "Remove Group");

    expect(app.lastFrame()).toContain("Top-level filters: 3");
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).not.toContain("Concentrat");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 2);

    pressLeft(app);
    await waitForFrameToContain(app, "[EDITOR] Query", 60);
    await openStructuredQueryEditor(app);
    expect(app.lastFrame()).toContain("Traits: includes Archetype");
    expect(app.lastFrame()).toContain("Traits: includes Dedication");
    expect(app.lastFrame()).not.toContain("Concentrat");
    expectNoDuplicateGroupedTraitProjection(app.lastFrame(), 2);

    await executeCurrentBrowseQuery(app);
    expect(lastListRequest(listRecords).filter).toEqual(
      allOfFilter([
        scopeFilter("spell"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "archetype" }),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "dedication" }),
      ]),
    );
  });

  it("opens action-cost additions through the grouped shared explorer route", async () => {
    const listRecords: ListRecordsFn = vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 1,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [createRecord()],
    }));
    const services = createServices({ listRecords });
    services.user.search.getQueryFieldOptions = vi.fn(() => []);

    const app = renderSearch(
      services,
      browseQuery("Browse spells", { filter: scopeFilter("spell"), limit: 20 }).request,
    );

    await openStructuredQueryEditor(app);
    app.stdin.write("\r");
    await waitForFrameToContain(app, "Add Clause", 60);
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    expect(await waitForFrameToContain(app, "Action Cost Explorer", 60)).toContain("matching counts");
  });
});
