import { describe, expect, it, vi } from "vitest";

import { buildSearchSemanticsDomain } from "../../src/app/ontology/search-semantics-domain.js";
import type { OntologyNode } from "../../src/domain/ontology-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { FilterValueField, SearchFilters } from "../../src/domain/search-types.js";
import type { SearchSemanticsBootstrapSummaryResult, SearchVocabularyResult } from "../../src/data/vocabulary.js";
import type { Pf2eDataService } from "../../src/data/service.js";

function createTestConfig(indexPath = ".cache/pf2e-index.sqlite"): AppConfig {
  return {
    dataPath: "vendor/pf2e",
    rootPath: "vendor/pf2e",
    manifestPath: "vendor/pf2e/system.pf2e.json",
    indexPath,
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

function findNodeById(nodes: readonly OntologyNode[], id: string): OntologyNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    const children = node.children ?? [];
    const match = findNodeById(children, id);
    if (match) {
      return match;
    }
  }
  return undefined;
}

function createSummary(): SearchSemanticsBootstrapSummaryResult {
  return {
    categories: [{ value: "hazard", count: 3 }],
    subcategoryCountsByCategory: [
      {
        category: "hazard",
        subcategories: [
          { value: "trap", count: 2 },
          { value: "haunt", count: 1 },
        ],
      },
    ],
    commonTraitsByCategory: [{ category: "hazard", traits: [{ value: "magical", count: 3 }] }],
    commonDerivedTagsByCategory: [{ category: "hazard", tags: [{ value: "fogbound", count: 2 }] }],
    derivedTagCatalog: [
      {
        category: "hazard",
        family: "mist",
        axis: "setting",
        description: "Mist-shaped hazards.",
        tags: [{ value: "fogbound" }],
      },
      {
        category: "hazard",
        family: "tripwire",
        axis: "encounter",
        subcategories: ["trap"],
        description: "Trap-specific ambush hazards.",
        tags: [{ value: "snag_line" }],
      },
      {
        category: "hazard",
        family: "spirits",
        axis: "haunt",
        subcategories: ["haunt"],
        description: "Haunt-specific restless effects.",
        tags: [{ value: "lingering_whisper" }],
      },
    ],
  };
}

function createVocabulary(summary: SearchSemanticsBootstrapSummaryResult): SearchVocabularyResult {
  return {
    categories: summary.categories,
    subcategories: [
      { value: "trap", count: 2 },
      { value: "haunt", count: 1 },
    ],
    rarities: [],
    sizes: [],
    traditions: [],
    spellKinds: [],
    sourceCategories: [],
    commonTraitsByCategory: summary.commonTraitsByCategory,
    commonDerivedTagsByCategory: summary.commonDerivedTagsByCategory,
    derivedTagOntologyFamilies: [],
    derivedTagOntologyTags: [],
    derivedTagCatalog: summary.derivedTagCatalog,
  };
}

function createDataService(options: {
  includeSummary?: boolean;
  includeVocabulary?: boolean;
} = {}): Pick<Pf2eDataService, "listFilterValues" | "listRecords"> & {
  getSearchSemanticsBootstrapSummary?: ReturnType<typeof vi.fn<() => SearchSemanticsBootstrapSummaryResult>>;
  getSearchVocabulary?: ReturnType<typeof vi.fn<() => SearchVocabularyResult>>;
} {
  const summary = createSummary();
  const vocabulary = createVocabulary(summary);

  const service: Pick<Pf2eDataService, "listFilterValues" | "listRecords"> & {
    getSearchSemanticsBootstrapSummary?: ReturnType<typeof vi.fn<() => SearchSemanticsBootstrapSummaryResult>>;
    getSearchVocabulary?: ReturnType<typeof vi.fn<() => SearchVocabularyResult>>;
  } = {
    listFilterValues: vi.fn(
      ({
        field,
        category,
        subcategory,
      }: {
        field: FilterValueField;
        category?: string;
        subcategory?: string;
      }) => ({
      field,
      values:
        field === "subcategories" && category === "hazard"
          ? [
              { value: "trap", count: 2 },
              { value: "haunt", count: 1 },
            ]
          : field === "derivedTags" && category === "hazard" && subcategory === "trap"
            ? [
                { value: "fogbound", count: 2 },
                { value: "snag_line", count: 1 },
              ]
          : [],
      }),
    ),
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
  };

  if (options.includeSummary ?? true) {
    service.getSearchSemanticsBootstrapSummary = vi.fn(() => summary);
  }
  if (options.includeVocabulary ?? true) {
    service.getSearchVocabulary = vi.fn(() => vocabulary);
  }

  return service;
}

describe("buildSearchSemanticsDomain", () => {
  it("scopes derived-tag families and tag queries to the active subcategory", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(createTestConfig(), dataService);
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");

    expect(dataService.getSearchSemanticsBootstrapSummary).toHaveBeenCalledTimes(1);
    expect(dataService.getSearchVocabulary).not.toHaveBeenCalled();
    expect(derivedTagsField?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    });
    expect(derivedTagsField?.children?.map((node) => node.label)).toEqual(["mist", "tripwire"]);

    const mistFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:mist"));
    const tripwireFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:tripwire"));
    const fogboundTag = findNodeById(mistFamilyNode?.loadChildren?.() ?? [], "hazard:trap:field:derivedTags:family:mist:tag:fogbound");
    const trapTag = findNodeById(
      tripwireFamilyNode?.loadChildren?.() ?? [],
      "hazard:trap:field:derivedTags:family:tripwire:tag:snag_line",
    );

    expect(fogboundTag?.query?.filters.subcategory).toBe("trap");
    expect(trapTag?.query?.filters.subcategory).toBe("trap");
    expect(fogboundTag?.children).toBeUndefined();
    expect(fogboundTag?.loadChildren).toBeUndefined();
  });

  it("loads derived-tag family children with scoped live counts", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(createTestConfig(), dataService);
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");
    const mistFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:mist"));
    const tripwireFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:tripwire"));

    const mistTags = mistFamilyNode?.loadChildren?.() ?? [];
    const tripwireTags = tripwireFamilyNode?.loadChildren?.() ?? [];

    expect(mistTags[0]?.listLabel).toBe("fogbound | 2");
    expect(tripwireTags[0]?.listLabel).toBe("snag_line | 1");
    expect(dataService.listFilterValues).toHaveBeenCalledWith({
      field: "derivedTags",
      category: "hazard",
      subcategory: "trap",
    });
  });

  it("builds common-trait shortcuts from summary data without eagerly loading the trait field value space", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(createTestConfig(), dataService);

    const commonTraitsNode = findNodeById(domain.rootNodes, "hazard:commonTraits");
    const magicalTraitNode = commonTraitsNode?.children?.[0];

    expect(dataService.getSearchSemanticsBootstrapSummary).toHaveBeenCalledTimes(1);
    expect(dataService.getSearchVocabulary).not.toHaveBeenCalled();
    expect(dataService.listFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ category: "hazard", field: "traits" }),
    );
    expect(magicalTraitNode?.label).toBe("magical");
    expect(magicalTraitNode?.listLabel).toBe("magical | 3");
    expect(magicalTraitNode?.query?.filters.metadata).toEqual({
      field: "traits",
      op: "includesAny",
      values: ["magical"],
    });
    expect(magicalTraitNode?.loadChildren).toBeTypeOf("function");
  });

  it("falls back to the full vocabulary loader when a summary loader is unavailable", () => {
    const dataService = createDataService({ includeSummary: false, includeVocabulary: true });

    buildSearchSemanticsDomain(createTestConfig(), dataService);

    expect(dataService.getSearchSemanticsBootstrapSummary).toBeUndefined();
    expect(dataService.getSearchVocabulary).toHaveBeenCalledTimes(1);
  });
});
