import { describe, expect, it, vi } from "vitest";

import { buildSearchSemanticsDomain } from "../../src/app/ontology/search-semantics-domain.js";
import type { OntologyNode } from "../../src/domain/ontology-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import type { FilterValueField, SearchFilters } from "../../src/domain/search-types.js";
import type { SearchVocabularyResult } from "../../src/data/vocabulary.js";
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

function createDataService(): Pick<Pf2eDataService, "getSearchVocabulary" | "listFilterValues" | "listRecords"> {
  const vocabulary: SearchVocabularyResult = {
    categories: [{ value: "hazard", count: 3 }],
    subcategories: [
      { value: "trap", count: 2 },
      { value: "haunt", count: 1 },
    ],
    rarities: [],
    sizes: [],
    traditions: [],
    spellKinds: [],
    sourceCategories: [],
    commonTraitsByCategory: [],
    commonDerivedTagsByCategory: [],
    derivedTagOntologyFamilies: [],
    derivedTagOntologyTags: [],
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

  return {
    getSearchVocabulary: vi.fn(() => vocabulary),
    listFilterValues: vi.fn(({ field, category }: { field: FilterValueField; category?: string }) => ({
      field,
      values:
        field === "subcategories" && category === "hazard"
          ? [
              { value: "trap", count: 2 },
              { value: "haunt", count: 1 },
            ]
          : [],
    })),
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
}

describe("buildSearchSemanticsDomain", () => {
  it("scopes derived-tag families and tag queries to the active subcategory without loading the derived-tags domain", () => {
    const loadDerivedTagsDomain = vi.fn(() => {
      throw new Error("search semantics should not load the derived-tags domain");
    });
    const domain = buildSearchSemanticsDomain(createTestConfig(), createDataService(), loadDerivedTagsDomain);
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");

    expect(loadDerivedTagsDomain).not.toHaveBeenCalled();
    expect(derivedTagsField?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    });
    expect(derivedTagsField?.children?.map((node) => node.label)).toEqual(["mist", "tripwire"]);

    const fogboundTag = findNodeById(derivedTagsField?.children ?? [], "hazard:trap:field:derivedTags:family:mist:tag:fogbound");
    const trapTag = findNodeById(derivedTagsField?.children ?? [], "hazard:trap:field:derivedTags:family:tripwire:tag:snag_line");
    const hauntTag = findNodeById(
      derivedTagsField?.children ?? [],
      "hazard:trap:field:derivedTags:family:spirits:tag:lingering_whisper",
    );

    expect(hauntTag).toBeUndefined();
    expect(fogboundTag?.query?.filters.subcategory).toBe("trap");
    expect(trapTag?.query?.filters.subcategory).toBe("trap");
    expect(fogboundTag?.children).toBeUndefined();
    expect(fogboundTag?.loadChildren).toBeUndefined();
  });
});
