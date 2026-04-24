import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { AppConfig } from "../../src/domain/config-types.js";
import type { OntologyDomainModel } from "../../src/domain/ontology-types.js";
import type { NormalizedRecord } from "../../src/domain/record-types.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import { Pf2eTerminalApp, Pf2eTerminalBootstrap } from "../../src/tui/pf2e-app.js";
import type { Pf2eAppRoute } from "../../src/tui/pf2e-app-state.js";
import type { Pf2eTerminalAppServices } from "../../src/tui/app-services.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery } from "../helpers/search-request-fixture.js";

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

function pressLeft(app: ReturnType<typeof render>): void {
  app.stdin.write("\u001b[D");
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
  return {
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
                      category: "spell",
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
        loadSearchSemanticsDomain: vi.fn(() => createSearchSemanticsModel()),
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

    expect(app.lastFrame()).toContain("Choose Search Mode");
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("Browse | Any Category | Counting matches...");

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

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("Search Semantics");
    expect(app.lastFrame()).toContain("Explorer Entries");

    expect(services.user.ontology.loadSearchSemanticsDomain).toHaveBeenCalledTimes(1);
    expect(app.lastFrame()).toContain("Spell");
  });

  it("exposes ontology discovery mode switching through the shared explorer commands", async () => {
    const services = createFakeServices();
    const createModeSpecificModel = (valueLabel: string): OntologyDomainModel => {
      const baseModel = createSearchSemanticsModel();
      const baseRootNode = baseModel.rootNodes[0]!;
      const baseMetadataGroup = baseRootNode.children![0]!;
      const baseFieldNode = baseMetadataGroup.children![0]!;

      return {
        ...baseModel,
        rootNodes: [
          {
            ...baseRootNode,
            detailLines: [
              ...baseRootNode.detailLines,
              { text: `Discovery fixture: ${valueLabel}` },
            ],
            children: [
              {
                ...baseMetadataGroup,
                children: [
                  {
                    ...baseFieldNode,
                    children: [
                      {
                        id: `spell:publicationTitle:${valueLabel}`,
                        kind: "value",
                        label: valueLabel,
                        filterText: valueLabel,
                        listLabel: `${valueLabel} | 1`,
                        detailTitle: "Filter Value",
                        detailLines: [{ text: valueLabel, tone: "section" }],
                        query: browseQuery("Browse records with this value", {
                          category: "spell",
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
      };
    };
    const loadSearchSemanticsDomain = vi.fn((mode?: "matching" | "catalog") =>
      mode === "catalog" ? createModeSpecificModel("catalog-only") : createModeSpecificModel("matching-only"),
    );
    services.user.ontology.loadSearchSemanticsDomain = loadSearchSemanticsDomain;
    const app = render(
      <DerivedTagTerminalProvider>
        <Pf2eTerminalApp rootPath={process.cwd()} onExit={vi.fn()} services={services} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    await openOntologyBrowser(app);
    expect(app.lastFrame()).toContain("matching counts");
    expect(app.lastFrame()).toContain("Discovery fixture: matching-only");
    expect(app.lastFrame()).not.toContain("Discovery fixture: catalog-only");

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Search Semantics Commands");
    expect(app.lastFrame()).toContain("Use Catalog Counts");

    for (const character of "catalog") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();
    await flushInk();

    expect(app.lastFrame()).toContain("catalog counts");
    expect(app.lastFrame()).toContain("Discovery fixture: catalog-only");
    expect(app.lastFrame()).not.toContain("Discovery fixture: matching-only");
    expect(loadSearchSemanticsDomain).toHaveBeenNthCalledWith(1);
    expect(loadSearchSemanticsDomain).toHaveBeenNthCalledWith(2, "catalog");
  });

  it("renders prepared ontology routes without calling the search-semantics loader", async () => {
    const services = createFakeServices();
    const loadSearchSemanticsDomain = vi.fn(() => createSearchSemanticsModel());
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
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(
      () => pendingModel.promise as unknown as OntologyDomainModel,
    );
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
    services.user.ontology.loadSearchSemanticsDomain = vi.fn(
      () => pendingModel.promise as unknown as OntologyDomainModel,
    );
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

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");

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

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("Pending Review Queue");

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
            category: "creature",
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
            category: "creature",
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
    for (const character of "open query") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushFrames(2);
    const searchFrame = app.lastFrame();
    expect(searchFrame).toContain("Browse | Creature |");
    expect(searchFrame).toContain("Category | Creature");
    expect(searchFrame).toContain("Filters > | 1 active");

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
            category: "creature",
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
            category: "creature",
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
      category: "creature",
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
            category: "creature",
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
