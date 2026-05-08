import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { EntityPageDocument } from "../../src/app/ontology/entity-page.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import type { SearchRequest } from "../../src/domain/search-request-types.js";
import { createPf2eApplicationEntityPageService } from "../../src/app/ontology/entity-page-service.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import { Pf2eTerminalApp, Pf2eTerminalBootstrap } from "../../src/tui/pf2e-app.js";
import {
  createPf2eOntologyRoute,
  createPf2ePageRoute,
  createPf2eSearchResultsRoute,
  type Pf2eAppRoute,
} from "../../src/tui/pf2e-app-state.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { createNoopTerminalDebugTraceService } from "../../src/tui/debug-trace.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery, scopeFilter } from "../helpers/search-request-fixture.js";

type WorkbenchPrompts = Parameters<Pf2eTerminalAppServices["dev"]["tagRefinement"]["promptAndCreateSession"]>[2];

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

async function flushFrames(count = 1): Promise<void> {
  for (let index = 0; index < count; index += 1) {
    await flushInk();
  }
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

function pressLeft(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[D");
}

function pressDown(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[B");
}

function pressCtrlE(app: ReturnType<typeof render>): void {
  app.stdin.write("\u0005");
}

function pressCtrlY(app: ReturnType<typeof render>): void {
  app.stdin.write("\u0019");
}

async function openOntologyBrowser(app: ReturnType<typeof render>): Promise<void> {
  app.stdin.write("j");
  await flushFrames();
  app.stdin.write("\r");
  await flushFrames(3);
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
      watch: false,
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

function createSearchSemanticsModel(): OntologyDomainModel {
  return withStaticChildSources({
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
                    query: browseQuery("Browse records with this value", {
                      filter: scopeFilter("spell"),
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

function createRecordOntologyModel(): OntologyDomainModel {
  return withStaticChildSources({
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "spell:test-fireball",
        kind: "record",
        label: "Fireball",
        filterText: "fireball",
        listLabel: "Fireball",
        detailTitle: "Record",
        detailLines: [{ text: "Record detail fallback" }],
        query: browseQuery("Browse fireball references", {
          filter: scopeFilter("spell"),
          limit: 20,
        }),
      },
    ],
  });
}

function createDerivedTagModeSearchSemanticsModel(discoveryMode: "matching" | "catalog"): OntologyDomainModel {
  const createDerivedTagLeaf = (options: { id: string; label: string; count: number }): OntologyNode => ({
    id: `spell:derivedTag:${options.id}`,
    kind: "value",
    label: options.label,
    filterText: `${options.label.toLowerCase()} derived tag`,
    listLabel: `${options.label} | ${options.count}`,
    detailTitle: "Derived Tag",
    detailLines: [
      { text: options.label, tone: "section" },
      { text: options.count === 0 ? "No matching records right now." : `${options.count} matching records.` },
    ],
    query: browseQuery(`Browse spells with ${options.label}`, {
      filter: scopeFilter("spell"),
      limit: 20,
    }),
  });

  const nonzeroLeaf = createDerivedTagLeaf({
    id: "ember-ward",
    label: "Ember Ward",
    count: 3,
  });
  const zeroCountLeaf = createDerivedTagLeaf({
    id: "ghost-ward",
    label: "Ghost Ward",
    count: 0,
  });
  const children = discoveryMode === "matching" ? [nonzeroLeaf] : [nonzeroLeaf, zeroCountLeaf];

  return withStaticChildSources({
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "spell:derivedTagFamily:defense-wards",
        kind: "group",
        label: "Defense Wards",
        filterText: "defense wards derived tags",
        listLabel: discoveryMode === "matching" ? "Defense Wards | 1 tag" : "Defense Wards | 2 tags",
        detailTitle: "Derived Tag Family",
        detailLines: [
          { text: "Defense Wards", tone: "section" },
          {
            text:
              discoveryMode === "matching"
                ? "Matching shows only nonzero derived tags."
                : "Catalog keeps zero-count derived tags openable.",
          },
        ],
        children,
      },
    ],
  });
}

function createScrollablePageDocument() {
  const references = Array.from({ length: 18 }, (_, index) => ({
    kind: "record" as const,
    label: `Reference ${String(index + 1).padStart(2, "0")}`,
    recordKey: `spell:reference-${index + 1}`,
    action: "open" as const,
  }));

  return {
    recordKey: "spell:test-alarm",
    title: "Alarm Ward",
    identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
    traits: ["Arcane", "Alarm"],
    sections: [
      {
        id: "summary",
        kind: "summary" as const,
        title: "Summary",
        blocks: [{ kind: "text" as const, text: "A focused ward that rings when trespassers approach." }],
        targets: [],
      },
      {
        id: "details",
        kind: "details" as const,
        title: "Details",
        blocks: [
          {
            kind: "factList" as const,
            facts: [
              { label: "Traditions", value: "Arcane" },
              { label: "Cast", value: "2 actions" },
              { label: "Range", value: "30 feet" },
              { label: "Duration", value: "1 minute" },
              { label: "Targets", value: "creature" },
            ],
          },
        ],
        targets: [],
      },
      {
        id: "references",
        kind: "references" as const,
        title: "References",
        blocks: [{ kind: "targetList" as const, targets: references }],
        targets: references,
      },
    ],
  };
}

function createLinksToBrowseRequest(recordKey: string, category: "creature" | "feat"): SearchRequest {
  return {
    mode: "browse",
    filter: {
      kind: "allOf",
      children: [
        {
          kind: "scope",
          category,
          subcategory: { kind: "any" },
        },
        {
          kind: "linksTo",
          target: recordKey,
        },
      ],
    },
    sort: { kind: "alphabetical" },
    limit: 50,
  };
}

function createReferencedByPageDocument(
  backlinkTargets: readonly { label: string; request: SearchRequest }[],
): EntityPageDocument {
  const targets = backlinkTargets.map(({ label, request }) => ({
    kind: "searchPivot" as const,
    label,
    request,
  }));

  return {
    recordKey: "spell:test-alarm",
    title: "Alarm Ward",
    identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
    traits: ["Arcane", "Alarm"],
    sections: [
      {
        id: "backlinks",
        kind: "backlinks",
        title: "Referenced By",
        blocks: [{ kind: "targetList", targets }],
        targets,
      },
    ],
  };
}

function createClassificationPageDocument(request: SearchRequest): EntityPageDocument {
  const target = {
    kind: "searchPivot" as const,
    label: "Category: Spell",
    request,
  };

  return {
    recordKey: "spell:test-alarm",
    title: "Alarm Ward",
    identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
    traits: [],
    sections: [
      {
        id: "classification",
        kind: "classification",
        title: "Classification",
        blocks: [{ kind: "targetList", targets: [target] }],
        targets: [target],
      },
    ],
  };
}

function createAonPageDocument(): EntityPageDocument {
  return {
    recordKey: "spell:test-alarm",
    title: "Alarm Ward",
    identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
    aonLink: {
      kind: "external",
      label: "Open in Archives of Nethys",
      href: "https://2e.aonprd.com/Spells.aspx?ID=1530",
      plainTextFallback: "Open in Archives of Nethys",
    },
    traits: [],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused ward that rings when trespassers approach." }],
        targets: [],
      },
    ],
  };
}

function createOpenTargetPageDocument(): {
  source: ReturnType<typeof createScrollablePageDocument>;
  target: ReturnType<typeof createScrollablePageDocument>;
} {
  return {
    source: {
      recordKey: "spell:test-alarm",
      title: "Alarm Ward",
      identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
      traits: ["Arcane", "Alarm"],
      sections: [
        {
          id: "references",
          kind: "references" as const,
          title: "References",
          blocks: [
            { kind: "text" as const, text: "Follow the linked spell page." },
            {
              kind: "targetList" as const,
              targets: [
                {
                  kind: "record" as const,
                  label: "Fireball",
                  recordKey: "spell:test-fireball",
                  action: "open" as const,
                },
              ],
            },
          ],
          targets: [
            {
              kind: "record" as const,
              label: "Fireball",
              recordKey: "spell:test-fireball",
              action: "open" as const,
            },
          ],
        },
        {
          id: "summary",
          kind: "summary" as const,
          title: "Summary",
          blocks: [{ kind: "text" as const, text: "A focused ward that rings when trespassers approach." }],
          targets: [],
        },
      ],
    },
    target: {
      recordKey: "spell:test-fireball",
      title: "Fireball",
      identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
      traits: ["Arcane", "Fire"],
      sections: [
        {
          id: "summary",
          kind: "summary" as const,
          title: "Summary",
          blocks: [{ kind: "text" as const, text: "A roaring blast of fire detonates at a spot you designate." }],
          targets: [],
        },
      ],
    },
  };
}

function createInlineTraitPageDocument(request: SearchRequest): EntityPageDocument {
  const fireTraitTarget = {
    kind: "searchPivot" as const,
    label: "Trait: Fire",
    request,
  };

  return {
    recordKey: "spell:test-fireball",
    title: "Fireball",
    identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
    traits: ["Fire"],
    traitTargets: [fireTraitTarget],
    sections: [
      {
        id: "summary",
        kind: "summary",
        title: "Summary",
        blocks: [{ kind: "text", text: "A focused fire spell." }],
        targets: [],
      },
    ],
  };
}

function createInlineUuidOpenTargetPageDocuments(): {
  source: EntityPageDocument;
  target: EntityPageDocument;
} {
  const inlineTarget = {
    kind: "record" as const,
    label: "Fireball",
    recordKey: "spell:test-fireball" as const,
    action: "open" as const,
  };

  return {
    source: {
      recordKey: "spell:test-alarm",
      title: "Alarm Ward",
      identityLine: "Spell | Rank 1 | Common | Pathfinder Player Core",
      traits: ["Arcane", "Alarm"],
      sections: [
        {
          id: "description",
          kind: "description",
          title: "Description",
          blocks: [
            {
              kind: "text",
              text: "Follow Fireball from the ward.",
              segments: [
                { text: "Follow " },
                { text: "Fireball", target: inlineTarget },
                { text: " from the ward." },
              ],
            },
          ],
          targets: [],
        },
      ],
    },
    target: {
      recordKey: "spell:test-fireball",
      title: "Fireball",
      identityLine: "Spell | Rank 3 | Common | Pathfinder Player Core",
      traits: ["Arcane", "Fire"],
      sections: [
        {
          id: "summary",
          kind: "summary",
          title: "Summary",
          blocks: [{ kind: "text", text: "A roaring blast of fire detonates at a spot you designate." }],
          targets: [],
        },
      ],
    },
  };
}

function createFakeServices(overrides: Partial<Pf2eTerminalAppServices> = {}): Pf2eTerminalAppServices {
  const record = createRecord();
  const listFilterValues = vi.fn(({ field }) => {
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
  });
  const discovery = createPf2eApplicationSearchDiscoveryService({
    discoverFilterValues: vi.fn(async (query) => listFilterValues(query)),
    getPack: vi.fn(() => undefined),
    listFilterValues,
  });
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
  const lookup = vi.fn(() => ({ match: record, alternatives: [], matchType: "exact" as const }));
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
        loadSearchSemanticsDomain: vi.fn(async () => createSearchSemanticsModel()),
        loadSearchFilterExplorerDomain: vi.fn(async () => createSearchSemanticsModel()),
      },
      pageRelations,
    },
    dev: {
      tagRefinement: {
        createSession: vi.fn(() => Promise.reject(new Error("not implemented"))),
        getQueueItems: vi.fn(() => []),
        getTranslationQueueItems: vi.fn(() => []),
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

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Choose Search Mode");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Browse | Any Category |");
    expect(app.lastFrame()).toContain("No applied query yet |");
    expect(app.lastFrame()).toContain("Query Editor");

    app.stdin.write("q");
    await flushInk();

    expect(app.lastFrame()).toContain("Choose a first-class TUI area");
  });

  it("returns to the area menu if the initial search mode picker is cancelled", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Choose a first-class TUI area");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Choose Search Mode");
    app.stdin.write("q");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Choose a first-class TUI area");
    expect(app.lastFrame()).not.toContain("Choose Search Mode");
  });

  it("opens the ontology browser directly in search semantics", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");

    expect(services.user.ontology.loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).toContain("Spell");
  });

  it("exposes ontology discovery mode switching through the shared action rail", async () => {
    const services = createFakeServices();
    const loadSearchSemanticsDomain = vi.fn(async () => createSearchSemanticsModel());
    services.user.ontology.loadSearchSemanticsDomain = loadSearchSemanticsDomain;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("matching counts");
    expect(loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);

    app.stdin.write(":");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Actions:");
    expect(app.lastFrame()).toContain("Use Catalog Counts");
  });

  it("switches the mounted ontology browser from matching to catalog and keeps zero-count derived-tag leaves openable", async () => {
    const services = createFakeServices();
    const loadSearchSemanticsDomain = vi.fn(
      async ({ discoveryMode }: { discoveryMode: "matching" | "catalog" }) =>
        createDerivedTagModeSearchSemanticsModel(discoveryMode),
    );
    services.user.ontology.loadSearchSemanticsDomain = loadSearchSemanticsDomain;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(loadSearchSemanticsDomain).toHaveBeenNthCalledWith(1, { discoveryMode: "matching" });
    expect(app.lastFrame()).toContain("matching counts");
    expect(app.lastFrame()).toContain("Defense Wards | 1 tag");
    expect(app.lastFrame()).not.toContain("Ghost Ward | 0");

    app.stdin.write(":");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Use Catalog Counts");

    app.stdin.write("\r");
    await flushFrames(4);

    expect(loadSearchSemanticsDomain).toHaveBeenNthCalledWith(2, { discoveryMode: "catalog" });
    expect(app.lastFrame()).toContain("Defense Wards | 2 tags");
    expect(app.lastFrame()).toContain("Use Matching Counts");

    app.stdin.write(":");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("catalog counts");

    app.stdin.write("\r");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Search Semantics > Defense Wards");
    expect(app.lastFrame()).toContain("Ember Ward | 3");
    expect(app.lastFrame()).toContain("Ghost Ward | 0");

    app.stdin.write("j");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Search Semantics > Defense Wards > Ghost Ward");

    app.stdin.write("\r");
    await flushFrames(3);
    expect(app.lastFrame()).toContain("Browse | Spell | Alphabetical | 1/1");
    expect(app.lastFrame()).toContain("[RESULTS]");
  });

  it("renders prepared ontology routes without calling the search-semantics loader", async () => {
    const services = createFakeServices();
    const loadSearchSemanticsDomain = vi.fn(async () => createSearchSemanticsModel());
    services.user.ontology.loadSearchSemanticsDomain = loadSearchSemanticsDomain;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          initialRoute={{ kind: "ontology", model: createSearchSemanticsModel() } as unknown as Pf2eAppRoute}
          services={services}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    await flushInk();

    expect(loadSearchSemanticsDomain).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");
  });

  it("keeps the area menu mounted while Search Semantics prepares", async () => {
    const services = createFakeServices();
    const pendingModel = createDeferred<OntologyDomainModel>();
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => pendingModel.promise);
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

    const pendingFrame = app.lastFrame();
    expect(pendingFrame).toContain("Choose a first-class TUI area");
    expect(pendingFrame).toContain("Ontology Browser");
    expect(pendingFrame).toContain("Loading next view | Opening Search Semantics...");
    expect(pendingFrame).not.toContain("Explorer Entries");

    pendingModel.resolve(createSearchSemanticsModel());

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");
  });

  it("keeps the area menu mounted and clears pending status when Search Semantics preparation fails", async () => {
    const services = createFakeServices();
    const pendingModel = createDeferred<OntologyDomainModel>();
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(() => pendingModel.promise);
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

    expect(app.lastFrame()).toContain("Loading next view | Opening Search Semantics...");

    pendingModel.reject(new Error("index offline"));

    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Could not open Search Semantics.");
    expect(app.lastFrame()).toContain("index offline");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();

    const recoveredFrame = app.lastFrame();
    expect(recoveredFrame).toContain("Choose a first-class TUI area");
    expect(recoveredFrame).toContain("Ontology Browser");
    expect(recoveredFrame).not.toContain("Loading next view |");
  });

  it("passes shared prompt adapters into custom workbench session creation", async () => {
    const promptAndCreateSession = vi.fn<Pf2eTerminalAppServices["dev"]["tagRefinement"]["promptAndCreateSession"]>(
      () => Promise.resolve(undefined),
    );
    const services = createFakeServices();
    services.dev.tagRefinement.promptAndCreateSession = promptAndCreateSession;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(promptAndCreateSession).toHaveBeenCalledWith(process.cwd(), "legacy_seed", expect.any(Object));
    const prompts: WorkbenchPrompts | undefined = promptAndCreateSession.mock.calls[0]?.[2];
    expect(prompts).toBeDefined();
    expect(typeof prompts?.promptOptionalSelectOption).toBe("function");
    expect(typeof prompts?.promptSelectOption).toBe("function");
    expect(typeof prompts?.promptTextInput).toBe("function");
    expect(typeof prompts?.pauseForAnyKey).toBe("function");
    expect("exitApp" in (prompts ?? {})).toBe(false);
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

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Could not create the legacy seed session.");
    expect(app.lastFrame()).toContain("bad scope");

    app.stdin.write(" ");
    await flushInk();
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");
  });

  it("opens the ontology translation queue from tag refinement", async () => {
    const services = createFakeServices();
    services.dev.tagRefinement.getTranslationQueueItems = vi.fn(() => [
      {
        currentCategory: "spell",
        currentBrowseAxis: "support",
        currentFamily: "support",
        currentTag: "quickened_support",
        currentAssignmentMode: "hybrid",
        translationStatus: "provisional",
        canonicalConceptId: "quickened_support",
        canonicalConceptLabel: "quickened_support",
        schemaKind: "descriptive",
        projectionAxis: "support",
        projectionFamily: "support",
        targetProjectionId: "spell:quickened_support",
        notes: "Still needs a final operational vs capability decision.",
      },
    ]);
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("j");
    await flushInk();
    app.stdin.write("j");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(app.lastFrame()).toContain("Pending Review Queue");
    expect(app.lastFrame()).toContain("Review ontology translation queue");

    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Ontology Translation Review");
    expect(app.lastFrame()).toContain("quickened_support");
    expect(app.lastFrame()).toContain("Proposed canonical target");
    expect(app.lastFrame()).toContain("Current category: spell");
  });

  it("renders grouped return wording on the direct ontology explorer entry", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);

    expect(app.lastFrame()).toContain("q back");

    app.stdin.write("?");
    await flushInk();

    expect(app.lastFrame()).toContain("\u2190 or h / Backspace / Escape: return from the explorer");
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
    await flushFrames();
    app.stdin.write("l");
    await flushFrames(3);

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
          query: browseQuery("Browse records with this value", {
            filter: scopeFilter("creature"),
            limit: 20,
          }),
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
          query: browseQuery("Browse records with this value", {
            filter: scopeFilter("creature"),
            limit: 20,
          }),
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Monster Core");

    app.stdin.write("j");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");

    app.stdin.write(":");
    await flushInk();
    app.stdin.write("\u001b[C");
    await flushInk();
    app.stdin.write("\r");
    await flushFrames(2);
    const searchFrame = app.lastFrame();
    expect(searchFrame).toContain("Browse | Creature |");

    pressLeft(app);
    await flushFrames(2);

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
          query: browseQuery("Browse records with this value", {
            filter: scopeFilter("creature"),
            limit: 20,
          }),
        },
        {
          id: "creature:publicationTitle:rage-of-elements",
          kind: "value",
          label: "Pathfinder Rage of Elements",
          filterText: "pathfinder rage of elements rage",
          listLabel: "Pathfinder Rage of Elements | 81",
          detailTitle: "Filter Value",
          detailLines: [{ text: "Pathfinder Rage of Elements", tone: "section" }],
          query: browseQuery("Browse records with this value", {
            filter: scopeFilter("creature"),
            limit: 20,
          }),
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Monster Core");

    app.stdin.write("j");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");

    app.stdin.write("\r");
    await flushFrames(2);
    expect(app.lastFrame()).toContain("Browse | Creature | Alphabetical | 1/1");
    expect(app.lastFrame()).toContain("[RESULTS]");
    expect(app.lastFrame()).not.toContain("[EDITOR] Query");

    pressLeft(app);
    await flushFrames(2);

    expect(app.lastFrame()).toContain("Search Semantics > Pathfinder Rage of Elements");
    expect(app.lastFrame()).toContain("Pathfinder Rage of Elements | 81");
  });

  it("keeps the ontology explorer mounted while direct-result transitions prepare", async () => {
    const services = createFakeServices();
    const pendingSession = createDeferred<Awaited<ReturnType<typeof services.user.search.executeQuery>>>();
    const preparedQuery = browseQuery("Browse records with this value", {
      filter: scopeFilter("creature"),
      limit: 20,
    });
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

    await flushFrames();
    await openOntologyBrowser(app);

    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Pathfinder Monster Core | 320");

    app.stdin.write("\r");
    await flushFrames(2);

    const pendingFrame = app.lastFrame();
    expect(pendingFrame).toContain("Search Semantics");
    expect(pendingFrame).toContain("Explorer Entries");
    expect(pendingFrame).toContain("Loading next view | Loading results for Browse records with this value...");
    expect(pendingFrame).not.toContain("Browse | Creature | Alphabetical | 1/1");
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

    await flushFrames(2);

    expect(app.lastFrame()).toContain("Browse | Creature | Alphabetical | 1/1");
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
          query: browseQuery("Browse records with this value", {
            filter: scopeFilter("creature"),
            limit: 20,
          }),
        },
      ],
    }));
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames();
    await openOntologyBrowser(app);

    app.stdin.write("\r");
    await flushFrames(2);

    expect(app.lastFrame()).toContain("Loading next view | Loading results for Browse records with this value...");

    pendingSession.reject(new Error("index offline"));

    await flushFrames(2);

    expect(app.lastFrame()).toContain("Query execution failed.");
    expect(app.lastFrame()).toContain("index offline");

    app.stdin.write(" ");
    await flushFrames(2);

    const recoveredFrame = app.lastFrame();
    expect(recoveredFrame).toContain("Search Semantics");
    expect(recoveredFrame).toContain("Pathfinder Monster Core | 320");
    expect(recoveredFrame).not.toContain("Loading next view |");
  });

  it("supports Ctrl-Y and Ctrl-E smooth scrolling on dedicated entity-page routes", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(createScrollablePageDocument())}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Alarm Ward");
    expect(app.lastFrame()).toContain("Summary");
    expect(app.lastFrame()).not.toContain("Reference 01");

    for (let step = 0; step < 10; step += 1) {
      pressCtrlE(app);
      await flushFrames();
    }

    expect(app.lastFrame()).toContain("Reference 01");

    for (let step = 0; step < 10; step += 1) {
      pressCtrlY(app);
      await flushFrames();
    }

    expect(app.lastFrame()).toContain("Summary");
    expect(app.lastFrame()).not.toContain("Reference 01");
  });

  it("keeps offscreen selected targets visible on dedicated entity-page routes", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(createScrollablePageDocument())}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).not.toContain("Reference 18");

    app.stdin.write("G");
    await flushFrames();
    expect(app.lastFrame()).toContain("References");

    app.stdin.write("\r");
    await flushFrames();
    app.stdin.write("G");
    await flushFrames(2);

    expect(app.lastFrame()).toContain("Reference 18");
    expect(app.lastFrame()).toContain("↑/↓ target");
  });

  it("opens grouped backlink page targets through the shared app search-navigation seam", async () => {
    const services = createFakeServices();
    const sourceRecord = createRecord();
    const creatureBacklinkRequest = {
      mode: "browse" as const,
      filter: {
        kind: "allOf" as const,
        children: [
          {
            kind: "scope" as const,
            category: "creature" as const,
            subcategory: { kind: "any" as const },
          },
          {
            kind: "linksTo" as const,
            target: sourceRecord.recordKey,
          },
        ],
      },
      sort: { kind: "alphabetical" as const },
      limit: 50,
    };
    const featBacklinkRequest = {
      mode: "browse" as const,
      filter: {
        kind: "allOf" as const,
        children: [
          {
            kind: "scope" as const,
            category: "feat" as const,
            subcategory: { kind: "any" as const },
          },
          {
            kind: "linksTo" as const,
            target: sourceRecord.recordKey,
          },
        ],
      },
      sort: { kind: "alphabetical" as const },
      limit: 50,
    };
    const backlinkResult = createRecord({
      recordKey: "feat:backlink-feat",
      id: "backlink-feat",
      name: "Backlink Feat",
      normalizedName: "backlink feat",
      type: "feat",
      category: "feat",
      subcategory: null,
      packName: "feat",
      packLabel: "Feats",
      descriptionText: "A feat that references Alarm Ward.",
      descriptionSnippet: "A feat that references Alarm Ward.",
    });
    services.user.entityPages = createPf2eApplicationEntityPageService({
      loadPageRelations: vi.fn(() => ({
        recordKey: sourceRecord.recordKey,
        outgoing: { records: [], edges: [] },
        incoming: { records: [], edges: [] },
        edges: [],
        incomingGroups: [
          {
            category: "creature",
            subcategory: null,
            count: 1,
            request: creatureBacklinkRequest,
          },
          {
            category: "feat",
            subcategory: null,
            count: 2,
            request: featBacklinkRequest,
          },
        ],
      })),
    });
    services.user.search.executeQuery = vi.fn(async () => ({
      windowId: "window-2",
      query: featBacklinkRequest,
      results: [backlinkResult],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    })) as typeof services.user.search.executeQuery;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2eSearchResultsRoute({
            initialSession: {
              windowId: "window-1",
              query: services.user.search.createDefaultQuery("browse"),
              results: [sourceRecord],
              windowOffset: 0,
              resultMode: "browse",
              total: 1,
              loadedCount: 1,
              hasMore: false,
              nextOffset: null,
              searchProfile: null,
              sort: "alphabetical",
              sortSeed: null,
            },
          })}
        />
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
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Referenced By");

    app.stdin.write("\r");
    await flushInk();
    pressDown(app);
    await flushInk();
    app.stdin.write("\r");
    await flushFrames(10);

    expect(services.user.search.executeQuery).toHaveBeenCalledWith(featBacklinkRequest);
    expect(app.lastFrame()).toContain("Browse | Feat |");
    expect(app.lastFrame()).toContain("Backlink Feat");
  });

  it("opens referenced-by page targets from dedicated entity-page routes", async () => {
    const services = createFakeServices();
    const sourceRecord = createRecord();
    const request = createLinksToBrowseRequest(sourceRecord.recordKey, "feat");
    const result = createRecord({
      recordKey: "feat:dedicated-page-backlink",
      id: "dedicated-page-backlink",
      name: "Dedicated Page Backlink",
      normalizedName: "dedicated page backlink",
      type: "feat",
      category: "feat",
      subcategory: null,
      packName: "feat",
      packLabel: "Feats",
      descriptionText: "A dedicated page backlink result.",
      descriptionSnippet: "A dedicated page backlink result.",
    });
    services.user.search.executeQuery = vi.fn(async () => ({
      windowId: "window-entity-page-backlink",
      query: request,
      results: [result],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    })) as typeof services.user.search.executeQuery;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(
            createReferencedByPageDocument([
              {
                label: "Feats (2)",
                request,
              },
            ]),
          )}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Referenced By");
    expect(app.lastFrame()).toContain("Feats (2)");

    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(10);

    expect(services.user.search.executeQuery).toHaveBeenCalledWith(request);
    expect(app.lastFrame()).toContain("Browse | Feat |");
    expect(app.lastFrame()).toContain("Dedicated Page Backlink");
  });

  it("opens Classification row pivots from dedicated entity-page routes", async () => {
    const services = createFakeServices();
    const request = browseQuery("Browse spell records", {
      filter: scopeFilter("spell"),
      limit: 50,
    }).request;
    const result = createRecord({
      recordKey: "spell:classification-result",
      id: "classification-result",
      name: "Classification Result",
      normalizedName: "classification result",
    });
    services.user.search.executeQuery = vi.fn(async () => ({
      windowId: "window-classification-pivot",
      query: request,
      results: [result],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    })) as typeof services.user.search.executeQuery;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(createClassificationPageDocument(request))}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Classification");
    expect(app.lastFrame()).toContain("Category: Spell");

    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(10);

    expect(services.user.search.executeQuery).toHaveBeenCalledWith(request);
    expect(app.lastFrame()).toContain("Browse | Spell |");
    expect(app.lastFrame()).toContain("Classification Result");
  });

  it("warns for external Archives of Nethys targets from dedicated entity-page routes", async () => {
    const services = createFakeServices();
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(createAonPageDocument())}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Open in Archives of Nethys");

    app.stdin.write("\r");
    await flushFrames();
    app.stdin.write("\r");
    await flushFrames(2);

    expect(app.lastFrame()).toContain("This target opens outside the current page flow.");
    expect(app.lastFrame()).toContain("Open in Archives of Nethys");
  });

  it("opens referenced-by page targets from ontology inspect routes", async () => {
    const services = createFakeServices();
    const sourceRecord = createRecord({ recordKey: "spell:test-fireball", name: "Fireball", normalizedName: "fireball" });
    const request = createLinksToBrowseRequest(sourceRecord.recordKey, "feat");
    const result = createRecord({
      recordKey: "feat:ontology-backlink",
      id: "ontology-backlink",
      name: "Ontology Backlink",
      normalizedName: "ontology backlink",
      type: "feat",
      category: "feat",
      subcategory: null,
      packName: "feat",
      packLabel: "Feats",
      descriptionText: "An ontology backlink result.",
      descriptionSnippet: "An ontology backlink result.",
    });
    services.user.entityPages = {
      ...services.user.entityPages,
      buildDocumentByRecordKey: vi.fn(() =>
        createReferencedByPageDocument([
          {
            label: "Feats (2)",
            request,
          },
        ]),
      ),
    };
    services.user.search.executeQuery = vi.fn(async () => ({
      windowId: "window-ontology-backlink",
      query: request,
      results: [result],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    })) as typeof services.user.search.executeQuery;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2eOntologyRoute({ model: createRecordOntologyModel() })}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Referenced By");
    expect(app.lastFrame()).toContain("Feats (2)");

    app.stdin.write("\t");
    await flushFrames();
    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(10);

    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball", {
      recordTargetAction: "preview",
    });
    expect(services.user.search.executeQuery).toHaveBeenCalledWith(request);
    expect(app.lastFrame()).toContain("Browse | Feat |");
    expect(app.lastFrame()).toContain("Ontology Backlink");
  });

  it("previews record page targets in place from the shared search preview host", async () => {
    const services = createFakeServices();
    services.user.search.executeQuery = vi.fn(services.user.search.executeQuery) as typeof services.user.search.executeQuery;
    const sourceRecord = createRecord();
    const targetRecord = createRecord({
      recordKey: "spell:test-fireball",
      id: "test-fireball",
      name: "Fireball",
      normalizedName: "fireball",
      descriptionText: "A roaring blast of fire detonates at a spot you designate.",
      descriptionSnippet: "A roaring blast of fire detonates at a spot you designate.",
      rangeText: "500 feet",
      targetText: null,
    });
    services.user.entityPages = createPf2eApplicationEntityPageService({
      loadPageRelations: vi.fn(() => ({
        recordKey: sourceRecord.recordKey,
        outgoing: {
          records: [targetRecord],
          edges: [
            {
              fromRecordKey: sourceRecord.recordKey,
              toRecordKey: targetRecord.recordKey,
              displayText: "Fireball",
              referenceText: "Fireball",
              direction: "outgoing",
              relationshipType: "references",
              sourcePackName: sourceRecord.packName,
              sourceRecordType: sourceRecord.type,
              sourceDocumentType: sourceRecord.documentType,
              sourceCategory: sourceRecord.sourceCategory,
            },
          ],
        },
        incoming: { records: [], edges: [] },
        edges: [
          {
            fromRecordKey: sourceRecord.recordKey,
            toRecordKey: targetRecord.recordKey,
            displayText: "Fireball",
            referenceText: "Fireball",
            direction: "outgoing",
            relationshipType: "references",
            sourcePackName: sourceRecord.packName,
            sourceRecordType: sourceRecord.type,
            sourceDocumentType: sourceRecord.documentType,
            sourceCategory: sourceRecord.sourceCategory,
          },
        ],
        incomingGroups: [],
      })),
      getRecord: vi.fn((recordKey) => {
        if (recordKey === sourceRecord.recordKey) {
          return sourceRecord;
        }
        if (recordKey === targetRecord.recordKey) {
          return targetRecord;
        }
        return undefined;
      }),
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2eSearchResultsRoute({
            initialSession: {
              windowId: "window-1",
              query: services.user.search.createDefaultQuery("browse"),
              results: [sourceRecord],
              windowOffset: 0,
              resultMode: "browse",
              total: 1,
              loadedCount: 1,
              hasMore: false,
              nextOffset: null,
              searchProfile: null,
              sort: "alphabetical",
              sortSeed: null,
            },
          })}
        />
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
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | References");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(services.user.search.executeQuery).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("[PREVIEW] Fireball | Identity");
  });

  it("previews inline UUID record targets in place from the shared search preview host", async () => {
    const services = createFakeServices();
    services.user.search.executeQuery = vi.fn(services.user.search.executeQuery) as typeof services.user.search.executeQuery;
    const referenceText = "@UUID[Compendium.pf2e.spells.Item.fireball]{Fireball}";
    const sourceRecord = createRecord({
      descriptionText: `Trigger ${referenceText} from the ward.`,
      descriptionSnippet: `Trigger ${referenceText} from the ward.`,
    });
    const targetRecord = createRecord({
      recordKey: "spell:test-fireball",
      id: "test-fireball",
      name: "Fireball",
      normalizedName: "fireball",
      descriptionText: "A roaring blast of fire detonates at a spot you designate.",
      descriptionSnippet: "A roaring blast of fire detonates at a spot you designate.",
      rangeText: "500 feet",
      targetText: null,
    });
    const edge = {
      fromRecordKey: sourceRecord.recordKey,
      toRecordKey: targetRecord.recordKey,
      displayText: "Fireball",
      referenceText,
      direction: "outgoing" as const,
      relationshipType: "references" as const,
      sourcePackName: sourceRecord.packName,
      sourceRecordType: sourceRecord.type,
      sourceDocumentType: sourceRecord.documentType,
      sourceCategory: sourceRecord.sourceCategory,
    };
    services.user.entityPages = createPf2eApplicationEntityPageService({
      loadPageRelations: vi.fn(() => ({
        recordKey: sourceRecord.recordKey,
        outgoing: {
          records: [targetRecord],
          edges: [edge],
        },
        incoming: { records: [], edges: [] },
        edges: [edge],
        incomingGroups: [],
      })),
      getRecord: vi.fn((recordKey) => {
        if (recordKey === sourceRecord.recordKey) {
          return sourceRecord;
        }
        if (recordKey === targetRecord.recordKey) {
          return targetRecord;
        }
        return undefined;
      }),
    });
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2eSearchResultsRoute({
            initialSession: {
              windowId: "window-1",
              query: services.user.search.createDefaultQuery("browse"),
              results: [sourceRecord],
              windowOffset: 0,
              resultMode: "browse",
              total: 1,
              loadedCount: 1,
              hasMore: false,
              nextOffset: null,
              searchProfile: null,
              sort: "alphabetical",
              sortSeed: null,
            },
          })}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\t");
    await flushInk();
    pressDown(app);
    await flushInk();
    pressDown(app);
    await flushInk();
    expect(app.lastFrame()).toContain("[PREVIEW] Alarm Ward | Description");
    expect(app.lastFrame()).toContain("Trigger Fireball from the ward.");
    expect(app.lastFrame()).not.toContain("@UUID");

    app.stdin.write("\r");
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(services.user.search.executeQuery).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("[PREVIEW] Fireball | Identity");
  });

  it("opens record page targets through dedicated page routes and returns to the previous page", async () => {
    const services = createFakeServices();
    const documents = createOpenTargetPageDocument();
    services.user.entityPages = {
      ...services.user.entityPages,
      buildDocumentByRecordKey: vi.fn((recordKey: string) => {
        if (recordKey === documents.target.recordKey) {
          return documents.target;
        }
        return null;
      }),
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(documents.source)}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Alarm Ward");
    expect(app.lastFrame()).toContain("Follow the linked spell page.");

    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(3);

    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball", {
      recordTargetAction: "open",
    });
    expect(app.lastFrame()).toContain("Fireball");
    expect(app.lastFrame()).toContain("A roaring blast of fire detonates at a spot you designate.");

    app.stdin.write("q");
    await flushFrames(2);

    expect(app.lastFrame()).toContain("Alarm Ward");
    expect(app.lastFrame()).toContain("Follow the linked spell page.");
  });

  it("opens inline trait page targets from dedicated page routes", async () => {
    const services = createFakeServices();
    const request = browseQuery("Browse fire spells", {
      filter: scopeFilter("spell"),
      limit: 20,
    }).request;
    const result = createRecord({
      recordKey: "spell:fire-result",
      id: "fire-result",
      name: "Fire Result",
      normalizedName: "fire result",
      traits: ["fire"],
    });
    services.user.search.executeQuery = vi.fn(async () => ({
      windowId: "window-inline-trait",
      query: request,
      results: [result],
      windowOffset: 0,
      resultMode: "browse",
      total: 1,
      loadedCount: 1,
      hasMore: false,
      nextOffset: null,
      searchProfile: null,
      sort: "alphabetical",
      sortSeed: null,
    })) as typeof services.user.search.executeQuery;

    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(createInlineTraitPageDocument(request))}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Traits: Fire");

    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(10);

    expect(services.user.search.executeQuery).toHaveBeenCalledWith(request);
    expect(app.lastFrame()).toContain("Browse | Spell |");
    expect(app.lastFrame()).toContain("Fire Result");
  });

  it("opens inline UUID record targets from dedicated page routes", async () => {
    const services = createFakeServices();
    const documents = createInlineUuidOpenTargetPageDocuments();
    services.user.entityPages = {
      ...services.user.entityPages,
      buildDocumentByRecordKey: vi.fn((recordKey: string) => {
        if (recordKey === documents.target.recordKey) {
          return documents.target;
        }
        return null;
      }),
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp
          rootPath={process.cwd()}
          onExit={vi.fn()}
          services={services}
          initialRoute={createPf2ePageRoute(documents.source)}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushFrames(2);
    expect(app.lastFrame()).toContain("Follow Fireball from the ward.");

    app.stdin.write("\r");
    await flushFrames(2);
    app.stdin.write("\r");
    await flushFrames(3);

    expect(services.user.entityPages.buildDocumentByRecordKey).toHaveBeenCalledWith("spell:test-fireball", {
      recordTargetAction: "open",
    });
    expect(app.lastFrame()).toContain("Fireball");
    expect(app.lastFrame()).toContain("A roaring blast of fire detonates at a spot you designate.");
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
