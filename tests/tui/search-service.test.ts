import { describe, expect, it, vi } from "vitest";

import type { MetadataFilterNode } from "../../src/search/filters/types.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import type { SearchFilters } from "../../src/domain/search-types.js";
import {
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
} from "../../src/tui/filter-explorer/search-draft-model.js";
import { cloneFilterExplorerComposeDraft } from "../../src/tui/filter-explorer/compose-state.js";
import { createPf2eTerminalSearchService } from "../../src/tui/search/service.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
} from "../../src/tui/search/query-state.js";

type SearchServiceDependencies = Parameters<typeof createPf2eTerminalSearchService>[0];

function createDependencies(overrides: Partial<SearchServiceDependencies> = {}): SearchServiceDependencies {
  return {
    closeSearchWindow: vi.fn(),
    countRecords: vi.fn(() =>
      Promise.resolve({
        searchProfile: null,
        mode: "structured" as const,
        total: 0,
      }),
    ),
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
        return { values: [{ value: "rare", count: 1 }] };
      }
      if (field === "traits") {
        return { values: [{ value: "illusion", count: 1 }] };
      }
      return { values: [] };
    }),
    lookup: vi.fn(() => ({ match: null, alternatives: [] })),
    listRecords: vi.fn((filters: SearchFilters) => ({
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
    search: vi.fn((filters: SearchFilters) =>
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

function createSearchSemanticsDomain(rootNodes: OntologyNode[]): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Test search semantics domain",
    rootNodes,
  };
}

describe("createPf2eTerminalSearchService", () => {
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
        label: "Creature",
        description: "12 indexed canonical records.",
      },
    ]);
    expect(getSearchVocabulary).not.toHaveBeenCalled();
  });

  it("normalizes legacy filter state into structured parts and trims unavailable action cost", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const defaultQuery = service.createDefaultQuery();
    const normalized = service.normalizeQuery({
      ...defaultQuery,
      queryText: "  Alarm Ward  ",
      sourceLabel: "  picked result  ",
      filters: {
        ...defaultQuery.filters,
        category: "spell",
        rarity: {
          any: ["rare"],
          all: [],
          exclude: [],
        },
        actionCost: {
          any: [2],
          all: [],
          exclude: [],
        },
      },
    });

    expect(normalized.queryText).toBe("Alarm Ward");
    expect(normalized.sourceLabel).toBe("picked result");
    expect(normalized.filters.parts).toEqual([
      {
        kind: "rarityPolicy",
        policy: {
          any: ["rare"],
          all: [],
          exclude: [],
        },
      },
    ]);
    expect(getSearchQueryActionCostPolicy(normalized)).toEqual({
      any: [],
      all: [],
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
    const query = service.normalizeQuery({
      ...defaultQuery,
      filters: {
        ...defaultQuery.filters,
        category: "spell",
        metadata: {
          and: [
            {
              field: "traits",
              op: "includesAny",
              values: ["illusion"],
            },
            {
              field: "sourceCategory",
              op: "eq",
              value: "core",
            },
          ],
        },
      },
    });

    const selections = service.buildDiscoverableQueryFieldSelections(query, ["traits"]);
    expect(selections).toEqual({
      traits: {
        any: ["illusion"],
        all: [],
        exclude: [],
      },
    });

    const updated = service.applyDiscoverableQueryFieldSelections(
      query,
      {
        traits: {
          any: ["evocation"],
          all: [],
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
          op: "includesAny",
          values: ["evocation"],
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

    expect(draft.selection).toEqual({});
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
    const query = service.normalizeQuery({
      ...defaultQuery,
      filters: {
        ...defaultQuery.filters,
        category: "spell",
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
        metadata: {
          field: "traits",
          op: "includesAny",
          values: ["illusion"],
        },
      },
    });

    const preparedDraft = service.prepareFilterExplorerDraft(query, ["rarity", "actionCost"]);
    const draft = preparedDraft.draft;

    expect(preparedDraft.scopedFields).toEqual(["rarity", "actionCost"]);
    expect(preparedDraft.preservedMetadata).toEqual({
      field: "traits",
      op: "includesAny",
      values: ["illusion"],
    } satisfies MetadataFilterNode);
    expect(draft.selection).toEqual({
      rarity: {
        any: ["common"],
        all: [],
        exclude: ["rare"],
      },
      actionCost: {
        any: ["2"],
        all: [],
        exclude: ["1"],
      },
    });

    const updated = service.applyFilterExplorerDraft(
      query,
      {
        ...draft,
        selection: {
          rarity: {
            any: ["uncommon"],
            all: [],
            exclude: [],
          },
          actionCost: {
            any: ["1"],
            all: [],
            exclude: [],
          },
        },
      },
      {
        preservedMetadata: preparedDraft.preservedMetadata,
        scopedFields: preparedDraft.scopedFields,
      },
    );

    expect(getSearchQueryRarityPolicy(updated)).toEqual({
      any: ["uncommon"],
      all: [],
      exclude: [],
    });
    expect(getSearchQueryActionCostPolicy(updated)).toEqual({
      any: [1],
      all: [],
      exclude: [],
    });
    expect(getSearchQueryMetadataTree(updated)).toEqual({
      field: "traits",
      op: "includesAny",
      values: ["illusion"],
    } satisfies MetadataFilterNode);
  });

  it("rebuilds search drafts from the shared compose draft without carrying session metadata in the draft", () => {
    const service = createPf2eTerminalSearchService(createDependencies());
    const preservedMetadata = {
      field: "traits",
      op: "includesAny",
      values: ["illusion"],
    } satisfies MetadataFilterNode;
    const composeDraft = cloneFilterExplorerComposeDraft({
      selection: {},
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
          op: "includesAny",
          values: ["illusion"],
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

  it("roots the shared explorer model at the scoped field nodes instead of reviving a picker snapshot bridge", () => {
    const model = buildSearchFilterExplorerModel(
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
                  label: "derivedTags",
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

  it("keeps the derived-tag field node and axis grouping intact when opened directly", () => {
    const model = buildSearchFilterExplorerModel(
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
    expect(rootNode?.children?.map((node) => node.id)).toEqual(["spell:field:derivedTags:family:coast"]);
  });

  it("locates metric explorer roots without traversing unrelated ontology branches", () => {
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

    const model = buildSearchFilterExplorerModel(
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
      query: {
        kind: "listRecords",
        label: "Perception Modifier",
        filters: {
          category: "creature",
          metadata: {
            field: "actorMetricCompare",
            op: ">=",
            leftMetric: "perception.mod",
            rightMetric: "perception.mod",
          },
        },
      },
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
      id: "item:actorMetrics:weapon.range_increment",
      kind: "metric",
      label: "Range Increment",
      filterText: "range increment",
      detailTitle: "Metric",
      detailLines: [{ text: "Range Increment" }],
      query: {
        kind: "listRecords",
        label: "Range Increment",
        filters: {
          category: "equipment",
          metadata: {
            field: "itemMetricCompare",
            op: ">=",
            leftMetric: "weapon.range_increment",
            rightMetric: "weapon.range_increment",
          },
        },
      },
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
