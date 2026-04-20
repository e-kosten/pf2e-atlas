import { describe, expect, it, vi } from "vitest";

import type { MetadataFilterNode } from "../../src/domain/metadata-types.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
import type { SearchFilters } from "../../src/domain/search-types.js";
import {
  applyFilterExplorerComposeDraft,
  buildSearchFilterExplorerModel,
  buildSearchFilterExplorerTargetResolver,
  createFilterExplorerComposeDraft,
  createPf2eTerminalSearchService,
  getSearchQueryMetadataTree,
} from "../../src/tui/search/service.js";

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
    expect(normalized.filters.actionCost).toEqual({
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

    const draft = service.createFilterExplorerDraftFromMetadataNode(
      {
        field: "actorMetric",
        metric: "perception.mod",
        op: ">=",
        value: 12,
      },
      ["actorMetric"],
    );

    expect(draft.fieldSelections).toEqual({});
    expect(draft.structuredMetadata).toBeNull();
    expect(draft.scalarClauses).toEqual({
      "actorMetric:perception.mod": {
        field: "actorMetric",
        metric: "perception.mod",
        valueType: "number",
        clause: {
          operator: "gte",
          value: 12,
        },
      },
    });
    expect(service.buildFilterExplorerMetadataNode(draft)).toEqual({
      field: "actorMetric",
      metric: "perception.mod",
      op: ">=",
      value: 12,
    } satisfies MetadataFilterNode);
  });

  it("rebuilds search drafts from the shared compose draft while inferring metric identity from the key", () => {
    const structuredMetadata = {
      field: "traits",
      op: "includesAny",
      values: ["illusion"],
    } satisfies MetadataFilterNode;
    const composeDraft = createFilterExplorerComposeDraft({
      fieldSelections: {},
      scalarClauses: {},
      structuredMetadata,
    });

    const nextDraft = applyFilterExplorerComposeDraft(
      {
        fieldSelections: {},
        scalarClauses: {},
        structuredMetadata,
      },
      {
        ...composeDraft,
        scalarClauses: {
          "itemMetric:weapon.range_increment": {
            operator: "between",
            min: 60,
            max: 120,
          },
        },
      },
    );

    expect(nextDraft.structuredMetadata).toEqual(structuredMetadata);
    expect(nextDraft.scalarClauses).toEqual({
      "itemMetric:weapon.range_increment": {
        field: "itemMetric",
        metric: "weapon.range_increment",
        valueType: "number",
        clause: {
          operator: "between",
          min: 60,
          max: 120,
        },
      },
    });
  });

  it("roots the shared explorer model at the scoped field nodes instead of a hosted picker snapshot", () => {
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
            editor: "ontologyPicker",
          },
          {
            value: "derivedTags",
            label: "Derived Tags",
            description: "Derived-tag query field",
            fieldType: "set",
            editor: "ontologyPicker",
          },
        ],
        singleFieldBehavior: "list",
      },
    );

    expect(model.label).toBe("Filter Explorer");
    expect(model.rootNodes.map((node) => node.id)).toEqual(["spell:field:traits", "spell:field:derivedTags"]);
  });

  it("uses friendly metric labels when resolving search-side metric compose targets", () => {
    const resolver = buildSearchFilterExplorerTargetResolver([
      {
        value: "actorMetric",
        label: "Creature Statistics",
        description: "Browse live statistic keys.",
        fieldType: "enumString",
        editor: "ontologyPicker",
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
            field: "actorMetric",
            metric: "perception.mod",
            op: ">=",
            value: 12,
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
});
