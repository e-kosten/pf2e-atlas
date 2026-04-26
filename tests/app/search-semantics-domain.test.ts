import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it, vi } from "vitest";

import { buildSearchSemanticsDomain } from "../../src/app/ontology/search-semantics-domain.js";
import { createPf2eApplicationSearchDiscoveryService } from "../../src/app/search-discovery-service.js";
import { getMetadataGlossaryArtifactPath } from "../../src/data/metadata-glossary.js";
import type { OntologyNode } from "../../src/domain/ontology-types.js";
import type { AppConfig } from "../../src/domain/config-types.js";
import { buildAllOfFilter, buildScopeFilter, findSearchScopeFilter } from "../../src/domain/search-request-types.js";
import type { SearchFilterNode, SearchRequest } from "../../src/domain/search-request-types.js";
import type { FilterValueField, SearchResult } from "../../src/domain/search-types.js";
import type { SearchSemanticsBootstrapSummaryResult } from "../../src/data/vocabulary.js";
import type { Pf2eDataService } from "../../src/data/service.js";
import type { MetadataGlossaryArtifact } from "../../src/domain/metadata-glossary-types.js";

const tempDirs: string[] = [];

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

function createTempIndexPath(prefix: string): string {
  const directory = mkdtempSync(path.join(tmpdir(), prefix));
  tempDirs.push(directory);
  return path.join(directory, "pf2e-index.sqlite");
}

function writeTraitGlossary(indexPath: string, entries: MetadataGlossaryArtifact["fields"]["traits"]): void {
  const artifactPath = getMetadataGlossaryArtifactPath(indexPath);
  writeFileSync(
    artifactPath,
    `${JSON.stringify({
      generatedAt: "2026-04-25T00:00:00.000Z",
      fields: {
        traits: entries,
      },
    } satisfies MetadataGlossaryArtifact)}\n`,
    "utf8",
  );
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

function getQueryFilter(node: OntologyNode | undefined): SearchFilterNode | undefined {
  return node?.query?.request.filter;
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
        tags: [{ value: "fogbound" }, { value: "still_air" }],
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
        family: "silence",
        axis: "encounter",
        subcategories: ["trap"],
        description: "Trap-specific quiet hazards.",
        tags: [{ value: "soundless" }],
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

function createDataService(options: {
  includeSummary?: boolean;
} = {}): Pick<Pf2eDataService, "discoverFilterValues" | "listFilterValues" | "listRecords"> & {
  getPack: ReturnType<typeof vi.fn>;
  getSearchSemanticsBootstrapSummary?: ReturnType<typeof vi.fn<() => SearchSemanticsBootstrapSummaryResult>>;
} {
  const summary = createSummary();

  const service: Pick<Pf2eDataService, "discoverFilterValues" | "listFilterValues" | "listRecords"> & {
    getPack: ReturnType<typeof vi.fn>;
    getSearchSemanticsBootstrapSummary?: ReturnType<typeof vi.fn<() => SearchSemanticsBootstrapSummaryResult>>;
  } = {
    getPack: vi.fn(() => undefined),
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
    discoverFilterValues: vi.fn(async (query) => service.listFilterValues(query)),
    listRecords: vi.fn((request: SearchRequest) => ({
      searchProfile: null,
      mode: "structured" as const,
      sort: request.sort ?? "alphabetical",
      total: 0,
      offset: request.offset ?? 0,
      limit: request.limit ?? 20,
      hasMore: false,
      nextOffset: null,
      records: [],
    }) satisfies SearchResult),
  };

  if (options.includeSummary ?? true) {
    service.getSearchSemanticsBootstrapSummary = vi.fn(() => summary);
  }

  return service;
}

afterEach(() => {
  while (tempDirs.length > 0) {
    rmSync(tempDirs.pop()!, { recursive: true, force: true });
  }
});

describe("buildSearchSemanticsDomain", () => {
  it("scopes derived-tag families and tag queries to the active subcategory", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(
      createTestConfig(),
      dataService,
      createPf2eApplicationSearchDiscoveryService(dataService),
    );
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");

    expect(dataService.getSearchSemanticsBootstrapSummary).toHaveBeenCalledTimes(1);
    expect(derivedTagsField?.childPresentation).toEqual({
      mode: "grouped",
      groupBy: "axis",
      render: "inline",
    });
    expect(derivedTagsField?.children?.map((node) => node.label)).toEqual(["Mist", "Tripwire"]);

    const mistFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:mist"));
    const tripwireFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:tripwire"));
    const fogboundTag = findNodeById(mistFamilyNode?.children ?? [], "hazard:trap:field:derivedTags:family:mist:tag:fogbound");
    const trapTag = findNodeById(
      tripwireFamilyNode?.children ?? [],
      "hazard:trap:field:derivedTags:family:tripwire:tag:snag_line",
    );
    const stillAirTag = findNodeById(mistFamilyNode?.children ?? [], "hazard:trap:field:derivedTags:family:mist:tag:still_air");

    expect(findSearchScopeFilter(fogboundTag?.query?.request.filter)?.subcategory).toEqual({ kind: "eq", value: "trap" });
    expect(findSearchScopeFilter(trapTag?.query?.request.filter)?.subcategory).toEqual({ kind: "eq", value: "trap" });
    expect(mistFamilyNode?.listLabel).toBe("Mist | 1 tags");
    expect(stillAirTag).toBeUndefined();
    expect(fogboundTag?.children).toBeUndefined();
    expect(fogboundTag?.loadChildren).toBeUndefined();
  });

  it("loads matching-mode derived-tag family children with scoped live counts", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(
      createTestConfig(),
      dataService,
      createPf2eApplicationSearchDiscoveryService(dataService),
    );
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");
    const mistFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:mist"));
    const tripwireFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:tripwire"));

    const mistTags = mistFamilyNode?.children ?? [];
    const tripwireTags = tripwireFamilyNode?.children ?? [];

    expect(mistTags[0]?.listLabel).toBe("Fogbound | 2");
    expect(tripwireTags[0]?.listLabel).toBe("Snag Line | 1");
    expect(dataService.listFilterValues).toHaveBeenCalledWith({
      field: "derivedTags",
      category: "hazard",
      subcategory: "trap",
    });
  });

  it("keeps zero-count catalog-only derived-tag families and leaves visible and queryable", () => {
    const dataService = createDataService();
    const domain = buildSearchSemanticsDomain(
      createTestConfig(),
      dataService,
      createPf2eApplicationSearchDiscoveryService(dataService),
      { discoveryMode: "catalog" },
    );
    const derivedTagsField = findNodeById(domain.rootNodes, "hazard:trap:field:derivedTags");

    expect(derivedTagsField?.children?.map((node) => node.label)).toEqual(["Mist", "Tripwire", "Silence"]);

    const mistFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:mist"));
    const silenceFamilyNode = derivedTagsField?.children?.find((node) => node.id.endsWith(":family:silence"));
    const stillAirTag = findNodeById(mistFamilyNode?.children ?? [], "hazard:trap:field:derivedTags:family:mist:tag:still_air");
    const soundlessTag = findNodeById(
      silenceFamilyNode?.children ?? [],
      "hazard:trap:field:derivedTags:family:silence:tag:soundless",
    );

    expect(mistFamilyNode?.listLabel).toBe("Mist | 2 tags");
    expect(silenceFamilyNode?.listLabel).toBe("Silence | 1 tags");
    expect(stillAirTag?.listLabel).toBe("Still Air | 0");
    expect(soundlessTag?.listLabel).toBe("Soundless | 0");
    expect(findSearchScopeFilter(stillAirTag?.query?.request.filter)?.subcategory).toEqual({ kind: "eq", value: "trap" });
    expect(findSearchScopeFilter(soundlessTag?.query?.request.filter)?.subcategory).toEqual({
      kind: "eq",
      value: "trap",
    });
  });

  it("builds common-trait shortcuts from summary data without eagerly loading the trait field value space", () => {
    const dataService = createDataService();
    const indexPath = createTempIndexPath("search-semantics-domain-");
    writeTraitGlossary(indexPath, {
      magical: {
        value: "magical",
        label: "Magical",
        description: "Magic-infused hazards and effects.",
      },
    });
    const domain = buildSearchSemanticsDomain(
      createTestConfig(indexPath),
      dataService,
      createPf2eApplicationSearchDiscoveryService(dataService),
    );

    const commonTraitsNode = findNodeById(domain.rootNodes, "hazard:commonTraits");
    const magicalTraitNode = commonTraitsNode?.children?.[0];

    expect(dataService.getSearchSemanticsBootstrapSummary).toHaveBeenCalledTimes(1);
    expect(dataService.listFilterValues).not.toHaveBeenCalledWith(
      expect.objectContaining({ category: "hazard", field: "traits" }),
    );
    expect(magicalTraitNode?.label).toBe("Magical");
    expect(magicalTraitNode?.listLabel).toBe("Magical | 3");
    expect(getQueryFilter(magicalTraitNode)).toEqual(
      buildAllOfFilter([
        buildScopeFilter("hazard"),
        {
          kind: "metadataPredicate",
          predicate: {
            field: "traits",
            op: "includes",
            value: "magical",
          },
        },
      ]),
    );
    expect(magicalTraitNode?.loadChildren).toBeTypeOf("function");
  });

  it("requires the dedicated search semantics bootstrap summary loader", () => {
    const dataService = createDataService({ includeSummary: false });

    expect(() =>
      buildSearchSemanticsDomain(
        createTestConfig(),
        dataService as unknown as Parameters<typeof buildSearchSemanticsDomain>[1],
        createPf2eApplicationSearchDiscoveryService(dataService),
      ),
    ).toThrow(/getSearchSemanticsBootstrapSummary/);
  });
});
