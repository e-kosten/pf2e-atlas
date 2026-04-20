import { describe, expect, it, vi } from "vitest";

import { buildSearchSemanticsDomain } from "../../src/app/ontology/search-semantics-domain.js";
import type { OntologyDomainModel, OntologyNode } from "../../src/domain/ontology-types.js";
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

function createDerivedTagsDomain(): OntologyDomainModel {
  const trapRecord: OntologyNode = {
    id: "hazard:fog-lattice",
    kind: "record",
    label: "Fog Lattice",
    filterText: "fog lattice trap",
    detailTitle: "Record Details",
    detailLines: [{ text: "Fog Lattice", tone: "section" }],
    query: {
      kind: "lookup",
      label: "Open exact record lookup",
      filters: {
        nameQuery: "Fog Lattice",
        category: "hazard",
        subcategory: "trap",
        limit: 5,
      },
    },
  };
  const hauntRecord: OntologyNode = {
    id: "hazard:mourning-fog",
    kind: "record",
    label: "Mourning Fog",
    filterText: "mourning fog haunt",
    detailTitle: "Record Details",
    detailLines: [{ text: "Mourning Fog", tone: "section" }],
    query: {
      kind: "lookup",
      label: "Open exact record lookup",
      filters: {
        nameQuery: "Mourning Fog",
        category: "hazard",
        subcategory: "haunt",
        limit: 5,
      },
    },
  };

  return {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Scoped derived-tag test domain",
    rootNodes: [
      {
        id: "hazard",
        kind: "category",
        label: "Hazard",
        shortLabel: "hazard",
        filterText: "hazard",
        detailTitle: "Category Details",
        detailLines: [{ text: "Hazard", tone: "section" }],
        childPresentation: {
          mode: "grouped",
          groupBy: "axis",
          render: "inline",
        },
        children: [
          {
            id: "hazard:mist",
            kind: "family",
            label: "mist",
            filterText: "mist",
            detailTitle: "Family Details",
            detailLines: [{ text: "mist", tone: "section" }],
            children: [
              {
                id: "hazard:fogbound",
                kind: "tag",
                label: "fogbound",
                filterText: "fogbound",
                detailTitle: "Tag Details",
                detailLines: [{ text: "fogbound", tone: "section" }],
                query: {
                  kind: "listRecords",
                  label: "List records with this derived tag",
                  filters: {
                    category: "hazard",
                    metadata: {
                      field: "derivedTags",
                      op: "includesAny",
                      values: ["fogbound"],
                    },
                    limit: 20,
                  },
                },
                children: [trapRecord, hauntRecord],
              },
            ],
          },
          {
            id: "hazard:tripwire",
            kind: "family",
            label: "tripwire",
            filterText: "tripwire",
            detailTitle: "Family Details",
            detailLines: [{ text: "tripwire", tone: "section" }],
            children: [
              {
                id: "hazard:snag_line",
                kind: "tag",
                label: "snag_line",
                filterText: "snag line",
                detailTitle: "Tag Details",
                detailLines: [{ text: "snag_line", tone: "section" }],
                query: {
                  kind: "listRecords",
                  label: "List records with this derived tag",
                  filters: {
                    category: "hazard",
                    subcategory: "trap",
                    metadata: {
                      field: "derivedTags",
                      op: "includesAny",
                      values: ["snag_line"],
                    },
                    limit: 20,
                  },
                },
                children: [trapRecord],
              },
            ],
          },
          {
            id: "hazard:spirits",
            kind: "family",
            label: "spirits",
            filterText: "spirits",
            detailTitle: "Family Details",
            detailLines: [{ text: "spirits", tone: "section" }],
            children: [
              {
                id: "hazard:lingering_whisper",
                kind: "tag",
                label: "lingering_whisper",
                filterText: "lingering whisper",
                detailTitle: "Tag Details",
                detailLines: [{ text: "lingering_whisper", tone: "section" }],
                query: {
                  kind: "listRecords",
                  label: "List records with this derived tag",
                  filters: {
                    category: "hazard",
                    subcategory: "haunt",
                    metadata: {
                      field: "derivedTags",
                      op: "includesAny",
                      values: ["lingering_whisper"],
                    },
                    limit: 20,
                  },
                },
                children: [hauntRecord],
              },
            ],
          },
        ],
      },
    ],
  };
}

describe("buildSearchSemanticsDomain", () => {
  it("scopes derived-tag families and tag queries to the active subcategory without cloning sample records", () => {
    const domain = buildSearchSemanticsDomain(createTestConfig(), createDataService(), createDerivedTagsDomain);
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");

    expect(derivedTagsField?.children?.map((node) => node.label)).toEqual(["mist", "tripwire"]);

    const fogboundTag = findNodeById(derivedTagsField?.children ?? [], "hazard:trap:field:derivedTags:hazard:fogbound");
    const trapTag = findNodeById(derivedTagsField?.children ?? [], "hazard:trap:field:derivedTags:hazard:snag_line");
    const hauntTag = findNodeById(
      derivedTagsField?.children ?? [],
      "hazard:trap:field:derivedTags:hazard:lingering_whisper",
    );

    expect(hauntTag).toBeUndefined();
    expect(fogboundTag?.query?.filters.subcategory).toBe("trap");
    expect(trapTag?.query?.filters.subcategory).toBe("trap");
    expect(fogboundTag?.children).toBeUndefined();
  });
});
