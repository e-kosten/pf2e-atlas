import { describe, expect, it } from "vitest";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";

import { registerSearchTools } from "../../src/server/register-search-tools.js";

type RegisteredToolHandler = (input: Record<string, unknown>) => Promise<unknown>;

function createServerHarness() {
  const handlers = new Map<string, RegisteredToolHandler>();
  const server = {
    registerTool(name: string, _config: unknown, handler: RegisteredToolHandler) {
      handlers.set(name, handler);
    },
  } as unknown as McpServer;

  return {
    server,
    getHandler(name: string): RegisteredToolHandler {
      const handler = handlers.get(name);
      if (!handler) {
        throw new Error(`Missing registered handler for ${name}.`);
      }
      return handler;
    },
  };
}

describe("registerSearchTools", () => {
  it("does not expose retired grouped metadata-filter DSL in public search semantics output", async () => {
    const harness = createServerHarness();
    registerSearchTools(
      harness.server,
      {
        getSearchVocabulary: () => ({
          categories: [{ value: "creature", count: 10 }],
          subcategories: [{ value: "undead", category: "creature", count: 2 }],
          rarities: ["common", "rare"],
          sizes: ["medium"],
          sourceCategories: ["core"],
          commonDerivedTagsByCategory: {},
          derivedTagOntologyFamilies: [],
          derivedTagOntologyTags: [],
          derivedTagCatalog: [],
        }),
        getRankingConfigStatus: () => ({ source: "defaults", warnings: [] }),
      } as never,
      { packCount: 3, recordCount: 10 },
      [],
    );

    const result = await harness.getHandler("pf2e_get_search_semantics")({});
    const structuredContent = (result as { structuredContent: Record<string, unknown> }).structuredContent;
    const metadataFilters = structuredContent.metadataFilters as Record<string, unknown>;

    expect(metadataFilters.booleanGroups).toBeUndefined();
    expect(metadataFilters.examplesByCategory).toBeUndefined();
    expect(metadataFilters.metadataFieldsByCategory).toBeDefined();
    expect(metadataFilters.advancedPredicates).toBeUndefined();
    expect(metadataFilters.actorMetricDiscovery).toBeDefined();
    expect(metadataFilters.itemMetricDiscovery).toBeDefined();
    expect(Array.isArray(metadataFilters.fieldTypes)).toBe(true);
    expect(Array.isArray(metadataFilters.metadataFields)).toBe(true);
    expect((metadataFilters.fieldTypes as Array<Record<string, unknown>>)[0]?.operators).toBeUndefined();
    expect(
      (metadataFilters.metadataFields as Array<Record<string, unknown>>).find((field) => field.field === "traits")
        ?.operators,
    ).toBeUndefined();
  });
});
