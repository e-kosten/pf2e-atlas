import { describe, expect, it, vi } from "vitest";

import type { MetadataFilterNode, SearchFilters } from "../../src/types.js";
import {
  createPf2eTerminalSearchService,
  getSearchQueryMetadataTree,
} from "../../src/tui/search-service.js";

type SearchServiceDependencies = Parameters<typeof createPf2eTerminalSearchService>[0];

function createDependencies(
  overrides: Partial<SearchServiceDependencies> = {},
): SearchServiceDependencies {
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
            return { values: [{ value: "illusion", count: 1 }, { value: "evocation", count: 1 }] };
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
});
