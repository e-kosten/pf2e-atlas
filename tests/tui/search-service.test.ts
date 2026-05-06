import { describe, expect, it, vi } from "vitest";

import { resolveOntologyNodeChildren } from "../../src/app/ontology/node-helpers.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import type { MetadataFilterNode } from "../../src/tui/search/metadata-filter-draft.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import type { SearchRequest } from "../../src/domain/search-request-types.js";
import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../../src/tui/filter-explorer/search-draft-model.js";
import { cloneFilterExplorerComposeDraft } from "../../src/tui/filter-explorer/compose-state.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import {
  getSearchQueryActionCostSelection,
  getSearchQueryPackSelection,
  getSearchQueryText,
  getSearchQueryMetadataTree,
  getSearchQueryRaritySelection,
  replaceSearchQueryRootScope,
  setSearchQueryActionCostSelection,
  setSearchQueryPackSelection,
  setSearchQueryMetadataTree,
  setSearchQueryRaritySelection,
  setSearchQuerySearchProfile,
  setSearchQueryText,
} from "../../src/tui/search/query-state.js";
import {
  actionCostFilter,
  allOfFilter,
  metadataPredicateFilter,
  metricCompareFilter,
  metricFilter,
  scopeFilter,
} from "../helpers/search-request-fixture.js";

type SearchServiceDependencies = Parameters<typeof createPf2eTerminalSearchService>[0];

function createDependencies(
  overrides: Partial<SearchServiceDependencies> & {
    getPack?: (packValue: string) => { name: string; label?: string } | undefined;
    listFilterValues?: (query: { field?: string; target?: { field: string } }) => {
      values: Array<{ value: string; count: number }>;
    };
  } = {},
): SearchServiceDependencies {
  const listFilterValues =
    overrides.listFilterValues ??
    vi.fn((query: { field?: string; target?: { field: string } }) => {
      const field = query.field ?? query.target?.field;
      if (field === "rarity") {
        return { values: [{ value: "rare", count: 1 }] };
      }
      if (field === "traits") {
        return { values: [{ value: "illusion", count: 1 }] };
      }
      return { values: [] };
    });

  return {
    closeSearchWindow: vi.fn(),
    countRecords: vi.fn(() =>
      Promise.resolve({
        searchProfile: null,
        mode: "structured" as const,
        total: 0,
      }),
    ),
    discovery:
      overrides.discovery ??
      createPf2eApplicationSearchDiscoveryService({
        discoverFilterValues: vi.fn(async (query) => listFilterValues(query)),
        getPack: overrides.getPack ?? vi.fn(() => undefined),
        listFilterValues,
      }),
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
    lookup: vi.fn(() => ({ match: null, alternatives: [], matchType: "none" as const })),
    listRecords: vi.fn((filters: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: filters.sort ?? "alphabetical",
      total: 0,
      offset: filters.offset ?? 0,
      limit: filters.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [],
    })),
    openSearchWindow: vi.fn(() =>
      Promise.resolve({
        id: "window-1",
        searchProfile: null,
        mode: "structured" as const,
        sort: "alphabetical" as const,
        sortSeed: null,
        total: 0,
        offset: 0,
        limit: 20,
        hasMore: false,
        nextOffset: null,
        records: [],
      }),
    ),
    readSearchWindowPage: vi.fn(() => ({
      id: "window-1",
      searchProfile: null,
      mode: "structured" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 0,
      offset: 0,
      limit: 20,
      hasMore: false,
      nextOffset: null,
      records: [],
    })),
    search: vi.fn((filters: SearchRequest) =>
      Promise.resolve({
        searchProfile: filters.searchProfile ?? "balanced",
        mode: "hybrid" as const,
        sort: filters.sort ?? "ranked",
        total: 0,
        offset: filters.offset ?? 0,
        limit: filters.limit ?? 20,
        hasMore: false,
        nextOffset: null,
        records: [],
      }),
    ),
    ...overrides,
  };
}

function normalizeLegacyNode(
  node: OntologyNode & { children?: OntologyNode[]; loadChildren?: () => OntologyNode[] },
): OntologyNode {
  const { children, loadChildren, ...rest } = node;
  return {
    ...rest,
    childSource:
      node.childSource ??
      (children
        ? { kind: "static", children: children.map((child) => normalizeLegacyNode(child)) }
        : loadChildren
          ? { kind: "sync", load: () => loadChildren().map((child) => normalizeLegacyNode(child)) }
          : undefined),
  };
}

function createSearchSemanticsDomain(rootNodes: OntologyNode[]): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Test search semantics domain",
    rootNodes: rootNodes.map((node) => normalizeLegacyNode(node)),
  };
}

describe("createPf2eTerminalSearchService", () => {
  it("keeps mode-specific result-sort choices explicit", () => {
    const service = createPf2eTerminalSearchService(createDependencies());

    expect(service.getDefaultSort("browse")).toBe("alphabetical");
    expect(service.getDefaultSort("search")).toBe("ranked");
    expect(service.getDefaultSort("lookup")).toBe("alphabeticalTiered");
    expect(service.getResultSortOptions("browse").map((option) => option.value)).toEqual([
      "alphabetical",
      "levelAsc",
      "levelDesc",
      "random",
    ]);
    expect(service.getResultSortOptions("search").map((option) => option.value)).toEqual([]);
    expect(service.getResultSortOptions("lookup").map((option) => option.value)).toEqual([
      "alphabeticalTiered",
      "alphabeticalGlobal",
      "levelAscTiered",
      "levelAscGlobal",
      "levelDescTiered",
      "levelDescGlobal",
    ]);
  });

  it("builds lookup search-window requests with explicit tiered or global sort policy", async () => {
    const openSearchWindow = vi.fn(async () => ({
      id: "window-1",
      searchProfile: null,
      mode: "lexical" as const,
      sort: "alphabetical" as const,
      sortSeed: null,
      total: 0,
      offset: 0,
      limit: 20,
      hasMore: false,
      nextOffset: null,
      records: [],
    }));
    const service = createPf2eTerminalSearchService(createDependencies({ openSearchWindow }));

    await service.executeQuery(service.createDefaultQuery("lookup"));
    await service.executeQuery(service.createDefaultQuery("lookup"), { sort: "levelDescGlobal" });

    expect(openSearchWindow).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({
        mode: "lookup",
        sort: { kind: "alphabetical", policy: "tiered" },
      }),
    );
    expect(openSearchWindow).toHaveBeenNthCalledWith(
      2,
      expect.objectContaining({
        mode: "lookup",
        sort: { kind: "levelDesc", policy: "global" },
      }),
    );
  });

  it("preserves explicit lookup match type metadata from search-window pages", async () => {
    const record = {
      id: "spell-fire-ball",
      recordKey: "spell:fire-ball",
      name: "Fire Ball",
      normalizedName: "fire ball",
      type: "spell",
      category: "spell",
      subcategory: null,
      packName: "spell",
      packLabel: "Spells",
      documentType: "Item",
      level: 3,
      rarity: "common",
      traits: [],
      derivedTags: [],
      publicationTitle: "Player Core",
      publicationRemaster: true,
      descriptionText: null,
      blurbText: null,
      hasDescription: false,
      descriptionSnippet: null,
      sourceCategory: "core",
      folderId: null,
      families: [],
      variantFamilyKey: null,
      variantBaseName: null,
      variantLabel: null,
      variantAxes: [],
      variantConfidence: null,
      variantSource: "none",
      sourcePath: "packs/spells/fire-ball.json",
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
      rangeText: "500 feet",
      saveType: "reflex",
      areaType: "burst",
      durationText: null,
      durationUnit: null,
      targetText: null,
      areaValue: 20,
      sustained: false,
      basicSave: true,
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
      rangeValue: 500,
      aliases: [],
      legacyRecordLinks: [],
      raw: {},
    } as const;
    const service = createPf2eTerminalSearchService(
      createDependencies({
        openSearchWindow: vi.fn(async () => ({
          id: "window-1",
          searchProfile: null,
          mode: "lexical" as const,
          sort: "alphabetical" as const,
          sortSeed: null,
          total: 1,
          offset: 0,
          limit: 20,
          hasMore: false,
          nextOffset: null,
          records: [{ ...record, matchType: "exact" as const }],
        })),
      }),
    );

    const session = await service.executeQuery({
      mode: "lookup",
      limit: 20,
      search: {
        query: "Fire Ball",
      },
    });

    expect(session.results[0]?.matchType).toBe("exact");
  });

  it("prefers the cached category summary when building category options", () => {
    const getSearchVocabulary = vi.fn(() => ({
      categories: [{ value: "spell", count: 99 }],
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
    }));
    const service = createPf2eTerminalSearchService(
      createDependencies({
        getSearchCategorySummary: () => ({
          categories: [{ value: "creature", count: 12 }],
        }),
        getSearchVocabulary,
      }),
    );

    expect(service.getCategoryOptions()).toEqual([
      {
        value: null,
        label: "Any Category",
        description: "Search or browse across the full indexed PF2E corpus.",
      },
      {
        value: "creature",
        label: "Creature | 12",
        description: "12 indexed canonical records.",
      },
    ]);
    expect(getSearchVocabulary).not.toHaveBeenCalled();
  });

  it("shows live counts on subcategory scope options", () => {
    const service = createPf2eTerminalSearchService(
      createDependencies({
        getSearchSemanticsBootstrapSummary: () => ({
          categories: [{ value: "creature", count: 9 }],
          subcategoryCountsByCategory: [
            {
              category: "creature",
              subcategories: [{ value: "character", count: 7 }],
            },
          ],
          commonTraitsByCategory: [],
          commonDerivedTagsByCategory: [],
          derivedTagCatalog: [],
        }),
      }),
    );

    expect(service.getSubcategoryOptions("creature")).toEqual([
      {
        value: null,
        label: "Any Subcategory",
        description: "Browse every Creature record in the current category.",
      },
      {
        value: "character",
        label: "Character | 7",
        description: "Restrict the workspace to Character records.",
      },
      {
        value: "familiar",
        label: "Familiar | 0",
        description: "Restrict the workspace to Familiar records.",
      },
    ]);
  });

  it("loads scope option counts from the query-aware discovery service", async () => {
    const discoverFilterValues = vi.fn(
      async (request: { mode: string; context: { request: SearchRequest }; target: { field: string } }) => {
        if (request.target.field === "categories") {
          return {
            mode: request.mode as "matching" | "catalog",
            target: request.target,
            options: [
              { id: "creature", value: "creature", count: 2 },
              { id: "spell", value: "spell", count: 5 },
            ],
          };
        }
        if (request.target.field === "subcategories") {
          return {
            mode: request.mode as "matching" | "catalog",
            target: request.target,
            options: [{ id: "familiar", value: "familiar", count: 1 }],
          };
        }
        return { mode: request.mode as "matching" | "catalog", target: request.target, options: [] };
      },
    );
    const discovery = createDependencies().discovery;
    discovery.discoverFilterValues = discoverFilterValues;
    const service = createPf2eTerminalSearchService(createDependencies({ discovery }));
    const query = setSearchQueryPackSelection(service.createDefaultQuery("browse"), {
      include: ["Bestiary"],
      exclude: [],
    });

    await expect(service.loadCategoryOptions(query, "matching")).resolves.toEqual(
      expect.arrayContaining([
        {
          value: "creature",
          label: "Creature | 2",
          description: "2 matching canonical records.",
        },
        {
          value: "spell",
          label: "Spell | 5",
          description: "5 matching canonical records.",
        },
      ]),
    );
    await expect(service.loadSubcategoryOptions(query, "creature", "catalog")).resolves.toEqual([
      {
        value: null,
        label: "Any Subcategory",
        description: "Browse every Creature record in the current category.",
      },
      {
        value: "character",
        label: "Character | 0",
        description: "0 applicable canonical records.",
      },
      {
        value: "familiar",
        label: "Familiar | 1",
        description: "1 applicable canonical record.",
      },
    ]);
    expect(discoverFilterValues).toHaveBeenNthCalledWith(
      1,
      expect.objectContaining({
        mode: "matching",
        target: { field: "categories" },
      }),
    );
    expect(discoverFilterValues).toHaveBeenNthCalledWith(
      2,
      expect.objectContaining({
        mode: "catalog",
        target: { field: "subcategories" },
        context: expect.objectContaining({
          request: expect.objectContaining({
            filter: expect.objectContaining({
              kind: "allOf",
            }),
          }),
        }),
      }),
    );
  });

  it("loads metric-key options from the query-aware shared discovery service and can restrict them to numeric keys", async () => {
    const discoverFilterValues = vi.fn(
      async (request: { mode: string; context: unknown; target: { field: string } }) => {
        if (request.target.field === "actorMetrics") {
          return {
            mode: "matching" as const,
            target: request.target,
            options: [
              { id: "hp.value", value: "hp.value", count: 2 },
              { id: "save.best", value: "save.best", count: 1 },
            ],
          };
        }
        return { mode: "matching" as const, target: request.target, options: [] };
      },
    );
    const discovery = createDependencies().discovery;
    discovery.discoverFilterValues = discoverFilterValues;
    const service = createPf2eTerminalSearchService(
      createDependencies({
        discovery,
      }),
    );
    const query = {
      ...service.createDefaultQuery("browse"),
      filter: {
        kind: "scope" as const,
        category: "creature" as const,
        subcategory: { kind: "any" as const },
      },
    };

    expect(await service.loadMetricKeyOptions(query, "actorMetric", "matching")).toEqual([
      {
        value: "hp.value",
        label: "hp.value",
        description: "2 matching canonical records.",
        count: 2,
      },
      {
        value: "save.best",
        label: "save.best",
        description: "1 matching canonical record.",
        count: 1,
      },
    ]);
    expect(discoverFilterValues).toHaveBeenCalledWith({
      mode: "matching",
      context: expect.objectContaining({
        request: expect.objectContaining({
          mode: "browse",
          filter: expect.objectContaining({
            kind: "scope",
            category: "creature",
          }),
        }),
      }),
      target: { field: "actorMetrics" },
    });
    expect(await service.loadMetricKeyOptions(query, "actorMetric", "matching", { numericOnly: true })).toEqual([
      {
        value: "hp.value",
        label: "hp.value",
        description: "2 matching canonical records.",
        count: 2,
      },
    ]);
  });

  it("loads pack options from canonical pack values and human-facing labels through query-aware catalog discovery", async () => {
    const discoverFilterValues = vi.fn(
      async (request: { mode: string; context: unknown; target: { field: string } }) => {
        if (request.target.field === "packs") {
          return {
            mode: "catalog" as const,
            target: request.target,
            options: [
              { id: "pathfinder-npc-core", value: "pathfinder-npc-core", count: 4 },
              { id: "bestiary", value: "bestiary", count: 2 },
            ],
          };
        }
        return { mode: "catalog" as const, target: request.target, options: [] };
      },
    );
    const discovery = createDependencies({
      getPack: (packValue) =>
        packValue === "pathfinder-npc-core"
          ? { name: "pathfinder-npc-core", label: "Pathfinder NPC Core" }
          : packValue === "bestiary"
            ? { name: "bestiary", label: "Bestiary" }
            : undefined,
    }).discovery;
    discovery.discoverFilterValues = discoverFilterValues;
    const service = createPf2eTerminalSearchService(
      createDependencies({
        discovery,
        getPack: (packValue) =>
          packValue === "pathfinder-npc-core"
            ? { name: "pathfinder-npc-core", label: "Pathfinder NPC Core" }
            : packValue === "bestiary"
              ? { name: "bestiary", label: "Bestiary" }
              : undefined,
      }),
    );
    const query = {
      ...service.createDefaultQuery("browse"),
      filter: {
        kind: "scope" as const,
        category: "creature" as const,
        subcategory: { kind: "any" as const },
      },
    };

    expect(await service.loadPackOptions(query, "catalog")).toEqual([
      {
        value: "bestiary",
        label: "Bestiary",
        description: "2 applicable canonical records.",
        count: 2,
      },
      {
        value: "pathfinder-npc-core",
        label: "Pathfinder NPC Core",
        description: "4 applicable canonical records.",
        count: 4,
      },
    ]);
    expect(discoverFilterValues).toHaveBeenCalledWith({
      mode: "catalog",
      context: expect.objectContaining({
        request: expect.objectContaining({
          mode: "browse",
          filter: expect.objectContaining({
            kind: "scope",
            category: "creature",
          }),
        }),
      }),
      target: { field: "packs" },
    });
  });

  it("normalizes canonical structured parts and trims unavailable action cost", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    let query = service.createDefaultQuery("search");
    query = setSearchQueryText(query, "  Alarm Ward  ");
    query = setSearchQuerySearchProfile(query, "balanced");
    query = setSearchQueryRaritySelection(
      {
        ...query,
        filter: {
          kind: "scope",
          category: "spell",
          subcategory: { kind: "any" },
        },
      },
      {
        include: ["rare"],
        exclude: [],
      },
    );
    query = setSearchQueryActionCostSelection(query, {
      include: [2],
      exclude: [],
    });
    const normalized = service.normalizeQuery(query);

    expect(getSearchQueryText(normalized)).toBe("Alarm Ward");
    expect(getSearchQueryRaritySelection(normalized)).toEqual({
      include: ["rare"],
      exclude: [],
    });
    expect(getSearchQueryActionCostSelection(normalized)).toEqual({
      include: [],
      exclude: [],
    });
  });

  it("does not rebuild structured parts from legacy filter-shaped extras", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const normalized = service.normalizeQuery({
      ...defaultQuery,
      filter: {
        kind: "scope",
        category: "spell",
        subcategory: { kind: "any" },
      },
      filters: {
        category: "spell",
        parts: [],
      } as unknown as never,
    });

    expect(getSearchQueryRaritySelection(normalized)).toEqual({
      include: [],
      exclude: [],
    });
    expect(getSearchQueryActionCostSelection(normalized)).toEqual({
      include: [],
      exclude: [],
    });
  });

  it("extracts scoped discoverable selections and preserves unrelated metadata when reapplied", () => {
    const service = createPf2eTerminalSearchService(
      createDependencies({
        listFilterValues: vi.fn(({ field }) => {
          if (field === "actionCost") {
            return { values: [{ value: "1", count: 1 }] };
          }
          if (field === "traits") {
            return {
              values: [
                { value: "illusion", count: 1 },
                { value: "evocation", count: 1 },
              ],
            };
          }
          return { values: [] };
        }),
      }),
    );
    const defaultQuery = service.createDefaultQuery();
    const query = setSearchQueryMetadataTree(
      {
        ...defaultQuery,
        filter: {
          kind: "scope",
          category: "spell",
          subcategory: { kind: "any" },
        },
      },
      {
        and: [
          {
            field: "traits",
            op: "includes",
            value: "illusion",
          },
          {
            field: "sourceCategory",
            op: "eq",
            value: "core",
          },
        ],
      },
    );

    const selections = service.buildDiscoverableQueryFieldSelections(query, ["traits"]);
    expect(selections).toEqual({
      traits: {
        include: ["illusion"],
        exclude: [],
      },
    });

    const updated = service.applyDiscoverableQueryFieldSelections(
      query,
      {
        traits: {
          include: ["evocation"],
          exclude: [],
        },
      },
      ["traits"],
    );

    expect(getSearchQueryMetadataTree(updated)).toEqual({
      and: [
        {
          field: "sourceCategory",
          op: "eq",
          value: "core",
        },
        {
          field: "traits",
          op: "includes",
          value: "evocation",
        },
      ],
    } satisfies MetadataFilterNode);
  });

  it("preserves grouped same-field set conjunctions as metadata when query-field selections reopen", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const groupedTraits = {
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
      ],
    } satisfies MetadataFilterNode;
    const query = setSearchQueryMetadataTree(
      {
        ...defaultQuery,
        filter: {
          kind: "scope",
          category: "spell",
          subcategory: { kind: "any" },
        },
      },
      groupedTraits,
    );

    expect(service.buildDiscoverableQueryFieldSelections(query, ["traits"])).toEqual({
      traits: {
        include: [],
        exclude: [],
      },
    });

    const updated = service.applyDiscoverableQueryFieldSelections(
      query,
      {
        traits: {
          include: ["evocation"],
          exclude: [],
        },
      },
      ["traits"],
    );

    expect(getSearchQueryMetadataTree(updated)).toEqual({
      and: [
        groupedTraits,
        {
          field: "traits",
          op: "includes",
          value: "evocation",
        },
      ],
    } satisfies MetadataFilterNode);
  });

  it("extracts numeric metric clauses into the filter explorer draft and rebuilds them as metadata", () => {
    const service = createPf2eTerminalSearchService(createDependencies());

    const preparedDraft = service.prepareFilterExplorerDraftFromMetadataNode(
      {
        field: "actorMetric",
        metric: "perception.mod",
        op: ">=",
        value: 12,
      },
      ["actorMetric"],
    );
    const { draft } = preparedDraft;

    expect(draft.discreteClauses).toEqual([]);
    expect(draft.scalarClauses).toEqual({
      "actorMetric:perception.mod": {
        operator: "gte",
        value: 12,
      },
    });
    expect(service.buildFilterExplorerMetadataNode(draft)).toEqual({
      field: "actorMetric",
      metric: "perception.mod",
      op: ">=",
      value: 12,
    } satisfies MetadataFilterNode);
  });

  it("emits peer metadata leaves when explorer-backed categorical drafts are inserted into the current group", () => {
    const service = createPf2eTerminalSearchService(createDependencies());

    expect(
      service.buildFilterExplorerInsertionResult({
        discreteClauses: [
          { field: "traits", value: "evocation", operator: "include" },
          { field: "traits", value: "illusion", operator: "include" },
        ],
        scalarClauses: {},
      }),
    ).toEqual({
      kind: "insert",
      nodes: [
        {
          field: "traits",
          op: "includes",
          value: "evocation",
        },
        {
          field: "traits",
          op: "includes",
          value: "illusion",
        },
      ],
    });
  });

  it("round-trips scoped rarity and action-cost explorer drafts through top-level query parts", () => {
    const service = createPf2eTerminalSearchService(
      createDependencies({
        listFilterValues: vi.fn(({ field }) => {
          if (field === "rarity") {
            return {
              values: [
                { value: "common", count: 1 },
                { value: "rare", count: 1 },
                { value: "uncommon", count: 1 },
              ],
            };
          }
          if (field === "actionCost") {
            return {
              values: [
                { value: "1", count: 1 },
                { value: "2", count: 1 },
              ],
            };
          }
          return { values: [] };
        }),
      }),
    );
    const defaultQuery = service.createDefaultQuery();
    const query = setSearchQueryMetadataTree(
      setSearchQueryActionCostSelection(
        setSearchQueryRaritySelection(
          {
            ...defaultQuery,
            filter: {
              kind: "scope",
              category: "spell",
              subcategory: { kind: "any" },
            },
          },
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
        field: "traits",
        op: "includes",
        value: "illusion",
      },
    );

    const preparedDraft = service.prepareFilterExplorerDraft(query, ["rarity", "actionCost"]);
    const draft = preparedDraft.draft;

    expect(preparedDraft.scopedFields).toEqual(["rarity", "actionCost"]);
    expect(preparedDraft.preservedMetadata).toEqual({
      field: "traits",
      op: "includes",
      value: "illusion",
    } satisfies MetadataFilterNode);
    expect(draft.discreteClauses).toEqual([
      { field: "actionCost", value: "1", operator: "exclude" },
      { field: "actionCost", value: "2", operator: "include" },
      { field: "rarity", value: "common", operator: "include" },
      { field: "rarity", value: "rare", operator: "exclude" },
    ]);

    const updated = service.applyFilterExplorerDraft(
      query,
      {
        ...draft,
        discreteClauses: [
          { field: "actionCost", value: "1", operator: "include" },
          { field: "rarity", value: "uncommon", operator: "include" },
        ],
      },
      {
        preservedMetadata: preparedDraft.preservedMetadata,
        scopedFields: preparedDraft.scopedFields,
      },
    );

    expect(getSearchQueryRaritySelection(updated)).toEqual({
      include: ["uncommon"],
      exclude: [],
    });
    expect(getSearchQueryActionCostSelection(updated)).toEqual({
      include: [1],
      exclude: [],
    });
    expect(getSearchQueryMetadataTree(updated)).toEqual({
      field: "traits",
      op: "includes",
      value: "illusion",
    } satisfies MetadataFilterNode);
  });

  it("preserves grouped same-field set clauses as preserved metadata when reopening the filter explorer", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const groupedTraits = {
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
      ],
    } satisfies MetadataFilterNode;

    const preparedDraft = service.prepareFilterExplorerDraftFromMetadataNode(groupedTraits, ["traits"]);

    expect(preparedDraft.draft.discreteClauses).toEqual([]);
    expect(preparedDraft.preservedMetadata).toEqual(groupedTraits);

    expect(
      service.buildFilterExplorerMetadataNode(
        {
          ...preparedDraft.draft,
          discreteClauses: [{ field: "traits", value: "evocation", operator: "include" }],
        },
        { preservedMetadata: preparedDraft.preservedMetadata },
      ),
    ).toEqual({
      and: [
        groupedTraits,
        {
          field: "traits",
          op: "includes",
          value: "evocation",
        },
      ],
    } satisfies MetadataFilterNode);
  });

  it("round-trips canonical pack clauses through the shared explorer draft without emitting impossible conjunctions", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const query = setSearchQueryMetadataTree(
      setSearchQueryPackSelection(
        {
          ...defaultQuery,
          filter: {
            kind: "scope",
            category: "creature",
            subcategory: { kind: "any" },
          },
        },
        {
          include: ["pathfinder-npc-core", "monster-core"],
          exclude: ["abomination-vaults"],
        },
      ),
      {
        field: "traits",
        op: "includes",
        value: "undead",
      },
    );

    const preparedDraft = service.prepareFilterExplorerDraft(query, ["pack"]);
    expect(preparedDraft.scopedFields).toEqual(["pack"]);
    expect(preparedDraft.preservedMetadata).toEqual({
      field: "traits",
      op: "includes",
      value: "undead",
    } satisfies MetadataFilterNode);
    expect(preparedDraft.draft.discreteClauses).toEqual([
      { field: "pack", value: "abomination-vaults", operator: "exclude" },
      { field: "pack", value: "monster-core", operator: "include" },
      { field: "pack", value: "pathfinder-npc-core", operator: "include" },
    ]);

    const updated = service.applyFilterExplorerDraft(
      query,
      {
        ...preparedDraft.draft,
        discreteClauses: [
          { field: "pack", value: "monster-core", operator: "exclude" },
          { field: "pack", value: "pathfinder-npc-core", operator: "include" },
        ],
      },
      {
        preservedMetadata: preparedDraft.preservedMetadata,
        scopedFields: preparedDraft.scopedFields,
      },
    );

    expect(getSearchQueryPackSelection(updated)).toEqual({
      include: ["pathfinder-npc-core"],
      exclude: ["monster-core"],
    });
    expect(getSearchQueryMetadataTree(updated)).toEqual({
      field: "traits",
      op: "includes",
      value: "undead",
    } satisfies MetadataFilterNode);
  });

  it("deduplicates top-level pack clauses and writes excluded packs back as flat peers", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const query = {
      ...defaultQuery,
      filter: {
        kind: "allOf",
        children: [
          {
            kind: "scope",
            category: "equipment",
            subcategory: { kind: "any" },
          },
          { kind: "pack", value: "equipment" },
          { kind: "not", child: { kind: "pack", value: "vehicles" } },
          { kind: "not", child: { kind: "pack", value: "vehicles" } },
          { kind: "pack", value: "equipment" },
          {
            kind: "not",
            child: {
              kind: "anyOf",
              children: [
                { kind: "pack", value: "feats" },
                { kind: "pack", value: "vehicles" },
              ],
            },
          },
        ],
      },
    } satisfies SearchRequest;

    expect(getSearchQueryPackSelection(query)).toEqual({
      include: ["equipment"],
      exclude: ["feats", "vehicles"],
    });

    const updated = setSearchQueryPackSelection(query, {
      include: ["equipment", "equipment"],
      exclude: ["vehicles", "feats", "vehicles"],
    });

    expect(updated.filter).toEqual({
      kind: "allOf",
      children: [
        {
          kind: "scope",
          category: "equipment",
          subcategory: { kind: "any" },
        },
        { kind: "pack", value: "equipment" },
        { kind: "not", child: { kind: "pack", value: "feats" } },
        { kind: "not", child: { kind: "pack", value: "vehicles" } },
      ],
    });
  });

  it("prunes scope-dependent metadata, metric, metric comparison, and action-cost clauses when replacing root scope category", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const query = {
      ...defaultQuery,
      filter: allOfFilter([
        scopeFilter("creature"),
        metadataPredicateFilter({ field: "traits", op: "includes", value: "undead" }),
        metricFilter("hp.value", "gte", 30),
        metricCompareFilter("ac.value", "gt", "ability.cha.mod"),
        actionCostFilter({ kind: "eq", value: 2 }),
        { kind: "level", match: { kind: "gt", value: 5 } },
      ]),
    } satisfies SearchRequest;

    const updated = replaceSearchQueryRootScope(query, scopeFilter("equipment"));

    expect(updated.filter).toEqual(
      allOfFilter([scopeFilter("equipment"), { kind: "level", match: { kind: "gt", value: 5 } }]),
    );
  });

  it("rebuilds search drafts from the shared compose draft without carrying session metadata in the draft", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const preservedMetadata = {
      field: "traits",
      op: "includes",
      value: "illusion",
    } satisfies MetadataFilterNode;
    const composeDraft = cloneFilterExplorerComposeDraft({
      discreteClauses: [],
      scalarClauses: {},
    });

    const nextDraft = cloneFilterExplorerComposeDraft({
      ...composeDraft,
      scalarClauses: {
        "itemMetric:weapon.range_increment": {
          operator: "between",
          min: 60,
          max: 120,
        },
      },
    });

    expect(nextDraft.scalarClauses).toEqual({
      "itemMetric:weapon.range_increment": {
        operator: "between",
        min: 60,
        max: 120,
      },
    });
    expect(service.buildFilterExplorerMetadataNode(nextDraft, { preservedMetadata })).toEqual({
      and: [
        {
          field: "traits",
          op: "includes",
          value: "illusion",
        },
        {
          and: [
            {
              field: "itemMetric",
              metric: "weapon.range_increment",
              op: ">=",
              value: 60,
            },
            {
              field: "itemMetric",
              metric: "weapon.range_increment",
              op: "<=",
              value: 120,
            },
          ],
        },
      ],
    } satisfies MetadataFilterNode);
  });

  it("roots the shared explorer model at the scoped field nodes instead of reviving a picker snapshot bridge", async () => {
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailTitle: "Spell",
          detailLines: [{ text: "Spell" }],
          children: [
            {
              id: "spell:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
              children: [
                {
                  id: "spell:field:traits",
                  kind: "field",
                  label: "traits",
                  filterText: "traits",
                  detailTitle: "Traits",
                  detailLines: [{ text: "Traits" }],
                  children: [],
                },
                {
                  id: "spell:field:derivedTags",
                  kind: "field",
                  label: "Derived Tags",
                  filterText: "derived tags",
                  detailTitle: "Derived Tags",
                  detailLines: [{ text: "Derived Tags" }],
                  children: [],
                },
              ],
            },
          ],
        },
      ]),
      {
        category: "spell",
        subcategory: null,
        fieldOptions: [
          {
            value: "traits",
            label: "Traits",
            description: "Trait query field",
            fieldType: "set",
            editor: "sharedExplorer",
          },
          {
            value: "derivedTags",
            label: "Derived Tags",
            description: "Derived-tag query field",
            fieldType: "set",
            editor: "sharedExplorer",
          },
        ],
        singleFieldBehavior: "list",
      },
    );

    expect(model.label).toBe("Filter Explorer");
    expect(model.rootNodes.map((node) => node.id)).toEqual(["spell:field:traits", "spell:field:derivedTags"]);
  });

  it("loads scoped pack direct values from the requested lazy branch", async () => {
    const loadPackValues = vi.fn(async () => [
      {
        id: "creature:pack:monster-core",
        kind: "value" as const,
        label: "Monster Core",
        filterText: "monster core",
        listLabel: "Monster Core | 8",
        detailTitle: "Pack",
        detailLines: [{ text: "Monster Core" }],
      },
    ]);
    const loadCreatureChildren = vi.fn(async () => [
      {
        id: "creature:pack",
        kind: "field" as const,
        label: "Pack",
        filterText: "pack",
        detailTitle: "Pack",
        detailLines: [{ text: "Pack" }],
        childSource: { kind: "lazy" as const, load: loadPackValues },
      },
    ]);
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:creature",
          kind: "category",
          label: "Creature",
          filterText: "creature",
          detailTitle: "Creature",
          detailLines: [{ text: "Creature" }],
          childSource: { kind: "lazy", load: loadCreatureChildren },
        },
      ]),
      {
        category: "creature",
        subcategory: null,
        fieldOptions: [
          {
            value: "pack",
            label: "Pack",
            description: "Pack query field",
            fieldType: "enumString",
            editor: "sharedExplorer",
          },
        ],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes.map((node) => node.listLabel)).toEqual(["Monster Core | 8"]);
    expect(loadCreatureChildren).toHaveBeenCalledTimes(1);
    expect(loadPackValues).toHaveBeenCalledTimes(1);
  });

  it("keeps the derived-tag field node and axis grouping intact when opened directly", async () => {
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailTitle: "Spell",
          detailLines: [{ text: "Spell" }],
          children: [
            {
              id: "spell:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
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
                      id: "spell:field:derivedTags:family:coast",
                      kind: "family",
                      label: "coast",
                      filterText: "coastal setting",
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
                          detailLines: [{ text: "coastal_setting", tone: "section" }],
                        },
                      ],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ]),
      {
        category: "spell",
        subcategory: null,
        fieldOptions: [
          {
            value: "derivedTags",
            label: "Derived Tags",
            description: "Derived-tag query field",
            fieldType: "set",
            editor: "sharedExplorer",
          },
        ],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes).toHaveLength(1);
    const [rootNode] = model.rootNodes;
    expect(rootNode?.id).toBe("spell:field:derivedTags");
    expect(rootNode?.detailTitle).toBe("Metadata Field Details");
    expect(rootNode?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    });
    expect((await resolveOntologyNodeChildren(rootNode)).map((node) => node.id)).toEqual([
      "spell:field:derivedTags:family:coast",
    ]);
  });

  it("preserves zero-count derived-tag catalog leaves on the shared explorer path", async () => {
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailTitle: "Spell",
          detailLines: [{ text: "Spell" }],
          children: [
            {
              id: "spell:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
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
                      id: "spell:field:derivedTags:family:coast",
                      kind: "family",
                      label: "Coast",
                      filterText: "coastal setting",
                      listLabel: "Coast | 1 tags",
                      detailTitle: "Family Details",
                      detailLines: [{ text: "Coast", tone: "section" }],
                      groupValues: {
                        axis: "environment",
                      },
                      children: [
                        {
                          id: "spell:field:derivedTags:family:coast:tag:coastal_setting",
                          kind: "tag",
                          label: "Coastal Setting",
                          filterText: "coastal setting",
                          listLabel: "Coastal Setting | 0",
                          detailTitle: "Tag Details",
                          detailLines: [{ text: "Coastal Setting", tone: "section" }],
                          query: {
                            label: "Browse records with the Coastal Setting derived tag",
                            request: {
                              mode: "browse",
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
        },
      ]),
      {
        category: "spell",
        subcategory: null,
        fieldOptions: [
          {
            value: "derivedTags",
            label: "Derived Tags",
            description: "Derived-tag query field",
            fieldType: "set",
            editor: "sharedExplorer",
          },
        ],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes).toHaveLength(1);
    const [rootNode] = model.rootNodes;
    const zeroCountLeaf = (await resolveOntologyNodeChildren((await resolveOntologyNodeChildren(rootNode))[0]))[0];

    expect(rootNode?.id).toBe("spell:field:derivedTags");
    expect(zeroCountLeaf?.listLabel).toBe("Coastal Setting | 0");
    expect(zeroCountLeaf?.query?.label).toBe("Browse records with the Coastal Setting derived tag");
  });

  it("aggregates unscoped rarity direct-value counts across search categories", async () => {
    const rarityFieldOption = {
      value: "rarity",
      label: "Rarity",
      description: "Rarity query field",
      fieldType: "enumString",
      editor: "sharedExplorer",
    } as const;
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailTitle: "Spell",
          detailLines: [{ text: "Spell" }],
          children: [
            {
              id: "spell:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
              children: [
                {
                  id: "spell:field:rarity",
                  kind: "field",
                  label: "Rarity",
                  filterText: "rarity",
                  detailTitle: "Rarity",
                  detailLines: [{ text: "Rarity" }],
                  children: [
                    {
                      id: "spell:field:rarity:value:common",
                      kind: "value",
                      label: "common",
                      filterText: "common",
                      listLabel: "common | 598",
                      detailTitle: "Rarity",
                      detailLines: [{ text: "common" }],
                    },
                    {
                      id: "spell:field:rarity:value:rare",
                      kind: "value",
                      label: "rare",
                      filterText: "rare",
                      listLabel: "rare | 10",
                      detailTitle: "Rarity",
                      detailLines: [{ text: "rare" }],
                    },
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "searchSemantics:creature",
          kind: "category",
          label: "Creature",
          filterText: "creature",
          detailTitle: "Creature",
          detailLines: [{ text: "Creature" }],
          children: [
            {
              id: "creature:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
              children: [
                {
                  id: "creature:field:rarity",
                  kind: "field",
                  label: "Rarity",
                  filterText: "rarity",
                  detailTitle: "Rarity",
                  detailLines: [{ text: "Rarity" }],
                  children: [
                    {
                      id: "creature:field:rarity:value:common",
                      kind: "value",
                      label: "common",
                      filterText: "common",
                      listLabel: "common | 14,000",
                      detailTitle: "Rarity",
                      detailLines: [{ text: "common" }],
                    },
                  ],
                },
              ],
            },
          ],
        },
      ]),
      {
        category: null,
        subcategory: null,
        fieldOptions: [rarityFieldOption],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes.map((node) => node.listLabel)).toEqual(["common | 14598", "rare | 10"]);

    const resolver = buildSearchFilterExplorerTargetResolver([rarityFieldOption]);
    expect(resolver(model.rootNodes[0])).toEqual({
      kind: "discrete",
      field: "rarity",
      fieldLabel: "Rarity",
      value: "common",
      valueLabel: "common",
      allowedOperators: ["include", "exclude"],
    });
  });

  it("aggregates unscoped pack direct-value counts across search categories", async () => {
    const packFieldOption = {
      value: "pack",
      label: "Pack",
      description: "Pack query field",
      fieldType: "enumString",
      editor: "sharedExplorer",
    } as const;
    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailTitle: "Spell",
          detailLines: [{ text: "Spell" }],
          children: [
            {
              id: "spell:pack",
              kind: "field",
              label: "Pack",
              filterText: "pack",
              detailTitle: "Pack",
              detailLines: [{ text: "Pack" }],
              children: [
                {
                  id: "spell:pack:pathfinder-npc-core",
                  kind: "value",
                  label: "Pathfinder NPC Core",
                  filterText: "pathfinder npc core",
                  listLabel: "Pathfinder NPC Core | 2",
                  detailTitle: "Pack",
                  detailLines: [{ text: "Pathfinder NPC Core" }],
                },
              ],
            },
          ],
        },
        {
          id: "searchSemantics:creature",
          kind: "category",
          label: "Creature",
          filterText: "creature",
          detailTitle: "Creature",
          detailLines: [{ text: "Creature" }],
          children: [
            {
              id: "creature:pack",
              kind: "field",
              label: "Pack",
              filterText: "pack",
              detailTitle: "Pack",
              detailLines: [{ text: "Pack" }],
              children: [
                {
                  id: "creature:pack:pathfinder-npc-core",
                  kind: "value",
                  label: "Pathfinder NPC Core",
                  filterText: "pathfinder npc core",
                  listLabel: "Pathfinder NPC Core | 4",
                  detailTitle: "Pack",
                  detailLines: [{ text: "Pathfinder NPC Core" }],
                },
                {
                  id: "creature:pack:monster-core",
                  kind: "value",
                  label: "Monster Core",
                  filterText: "monster core",
                  listLabel: "Monster Core | 8",
                  detailTitle: "Pack",
                  detailLines: [{ text: "Monster Core" }],
                },
              ],
            },
          ],
        },
      ]),
      {
        category: null,
        subcategory: null,
        fieldOptions: [packFieldOption],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes.map((node) => node.listLabel)).toEqual(["Pathfinder NPC Core | 6", "Monster Core | 8"]);

    const resolver = buildSearchFilterExplorerTargetResolver([packFieldOption]);
    expect(resolver(model.rootNodes[0])).toEqual({
      kind: "discrete",
      field: "pack",
      fieldLabel: "Pack",
      value: "pathfinder-npc-core",
      valueLabel: "Pathfinder NPC Core",
      allowedOperators: ["include", "exclude"],
    });
  });

  it("locates metric explorer roots without traversing unrelated ontology branches", async () => {
    const unrelatedRootLoadChildren = vi.fn(() => [
      {
        id: "equipment:field:traits:value:bulky",
        kind: "value",
        label: "bulky",
        filterText: "bulky",
        detailTitle: "Value",
        detailLines: [{ text: "bulky" }],
      },
    ]);
    const unrelatedCreatureLoadChildren = vi.fn(() => [
      {
        id: "creature:npc:field:traits:value:undead",
        kind: "value",
        label: "undead",
        filterText: "undead",
        detailTitle: "Value",
        detailLines: [{ text: "undead" }],
      },
    ]);

    const model = await buildSearchFilterExplorerModel(
      createSearchSemanticsDomain([
        {
          id: "searchSemantics:equipment",
          kind: "category",
          label: "Equipment",
          filterText: "equipment",
          detailTitle: "Equipment",
          detailLines: [{ text: "Equipment" }],
          children: [
            {
              id: "equipment:metadataFields",
              kind: "group",
              label: "Metadata Fields",
              filterText: "metadata fields",
              detailTitle: "Metadata Fields",
              detailLines: [{ text: "Metadata Fields" }],
              children: [
                {
                  id: "equipment:field:traits",
                  kind: "field",
                  label: "traits",
                  filterText: "traits",
                  detailTitle: "Traits",
                  detailLines: [{ text: "Traits" }],
                  loadChildren: unrelatedRootLoadChildren,
                },
              ],
            },
          ],
        },
        {
          id: "searchSemantics:creature",
          kind: "category",
          label: "Creature",
          filterText: "creature",
          detailTitle: "Creature",
          detailLines: [{ text: "Creature" }],
          children: [
            {
              id: "creature:subcategories",
              kind: "group",
              label: "Subcategories",
              filterText: "subcategories",
              detailTitle: "Subcategories",
              detailLines: [{ text: "Subcategories" }],
              children: [
                {
                  id: "creature:subcategory:npc",
                  kind: "subcategory",
                  label: "npc",
                  filterText: "npc",
                  detailTitle: "NPC",
                  detailLines: [{ text: "NPC" }],
                  children: [
                    {
                      id: "creature:npc:metadataFields",
                      kind: "group",
                      label: "Metadata Fields",
                      filterText: "metadata fields",
                      detailTitle: "Metadata Fields",
                      detailLines: [{ text: "Metadata Fields" }],
                      children: [
                        {
                          id: "creature:npc:field:traits",
                          kind: "field",
                          label: "traits",
                          filterText: "traits",
                          detailTitle: "Traits",
                          detailLines: [{ text: "Traits" }],
                          loadChildren: unrelatedCreatureLoadChildren,
                        },
                      ],
                    },
                  ],
                },
              ],
            },
            {
              id: "creature:actorMetrics:discovery",
              kind: "group",
              label: "Creature Statistics",
              filterText: "creature statistics",
              detailTitle: "Creature Statistics",
              detailLines: [{ text: "Creature Statistics" }],
              children: [
                {
                  id: "creature:actorMetrics:namespace:perception.",
                  kind: "metricNamespace",
                  label: "perception.",
                  filterText: "perception",
                  detailTitle: "Metric Namespace",
                  detailLines: [{ text: "perception." }],
                },
              ],
            },
          ],
        },
      ]),
      {
        category: "creature",
        subcategory: null,
        fieldOptions: [
          {
            value: "actorMetric",
            label: "Creature Statistics",
            description: "Browse live statistic keys.",
            fieldType: "enumString",
            editor: "sharedExplorer",
          },
        ],
        singleFieldBehavior: "directValues",
      },
    );

    expect(model.rootNodes.map((node) => node.id)).toEqual(["creature:actorMetrics:namespace:perception."]);
    expect(unrelatedRootLoadChildren).not.toHaveBeenCalled();
    expect(unrelatedCreatureLoadChildren).not.toHaveBeenCalled();
  });

  it("uses friendly metric labels when resolving search-side metric compose targets", () => {
    const resolver = buildSearchFilterExplorerTargetResolver([
      {
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live statistic keys.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);

    const target = resolver({
      id: "creature:actorMetrics:perception.mod",
      kind: "metric",
      label: "Perception Modifier",
      filterText: "perception modifier",
      detailTitle: "Metric",
      detailLines: [{ text: "Perception Modifier" }],
    } as OntologyNode);

    expect(target).toEqual({
      kind: "scalar",
      key: "actorMetric:perception.mod",
      fieldLabel: "Creature Statistics",
      subjectLabel: "Perception Modifier",
      valueType: "number",
      editorLabel: "Creature Statistics / Perception Modifier",
    });
  });

  it("keeps compare-style numeric metric targets actionable in compose mode", () => {
    const resolver = buildSearchFilterExplorerTargetResolver([
      {
        value: "itemMetric",
        label: "Item Properties",
        description: "Browse live item metric keys.",
        fieldType: "enumString",
        editor: "sharedExplorer",
      },
    ]);

    const target = resolver({
      id: "item:itemMetrics:weapon.range_increment",
      kind: "metric",
      label: "Range Increment",
      filterText: "range increment",
      detailTitle: "Metric",
      detailLines: [{ text: "Range Increment" }],
    } as OntologyNode);

    expect(target).toEqual({
      kind: "scalar",
      key: "itemMetric:weapon.range_increment",
      fieldLabel: "Item Properties",
      subjectLabel: "Range Increment",
      valueType: "number",
      editorLabel: "Item Properties / Range Increment",
    });
  });
});
