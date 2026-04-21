import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as z from "zod/v4";

import { CATEGORY_SUBCATEGORY_MAP } from "../domain/categories.js";
import { getMetadataFilterSemantics } from "../search/filters/semantics.js";
import { Pf2eDataService } from "../data/service.js";
import {
  CATEGORY_HINT_DESCRIPTION,
  filterValueFieldSchema,
  linksToModeSchema,
  metadataFilterSchema,
  recordKeyArraySchema,
  SCOPES_HINT_DESCRIPTION,
  searchCategorySchema,
  searchProfileSchema,
  searchScopeSchema,
  searchSubcategorySchema,
  SUBCATEGORY_HINT_DESCRIPTION,
} from "./tool-schemas.js";
import { formatSearchResult, summarizePack, summarizeRecord } from "./presenters.js";

export function registerSearchTools(
  server: McpServer,
  dataService: Pf2eDataService,
  stats: { packCount: number; recordCount: number },
  startupWarnings: string[],
): void {
  server.registerTool(
    "pf2e_get_search_semantics",
    {
      description:
        "Describe the explicit category-first search ontology, Pathfinder-native tags, and structured filters available to the calling agent. Use this for the full category/subcategory map and filter vocabulary.",
      inputSchema: {
        traitLimitPerCategory: z
          .number()
          .int()
          .min(3)
          .max(25)
          .optional()
          .describe("Maximum common traits to return per category. Defaults to 12."),
      },
    },
    ({ traitLimitPerCategory }) => {
      const vocabulary = dataService.getSearchVocabulary({ traitLimitPerCategory });
      const metadataSemantics = getMetadataFilterSemantics();
      return {
        content: [
          {
            type: "text",
            text: `Search semantics expose ${vocabulary.categories.length} top-level categories and Pathfinder-native filtering vocabulary.`,
          },
        ],
        structuredContent: {
          supportedFilters: [
            {
              name: "category",
              strength: "strong boundary",
              description:
                "Best first cut for separating creature, hazard, spell, equipment, lore, and other user-facing PF2E families.",
            },
            {
              name: "subcategory",
              strength: "within-category boundary",
              description: "Include one narrower family such as hazard/haunt, equipment/consumable, or lore/deity.",
            },
            {
              name: "scopes",
              strength: "paired multi-family boundary",
              description:
                "Use for multi-category search when each category needs its own optional subcategory list, such as feat/archetype plus rule/action.",
            },
            {
              name: "metadata",
              strength: "typed metadata DSL",
              description:
                "Use grouped boolean predicates for metadata filtering. Prefer top-level category, subcategory, scopes, and numeric bounds before metadata predicates.",
            },
            {
              name: "actorMetric",
              strength: "actor metric predicate",
              description:
                "Use inside metadata for creature and hazard stat filters with symbolic operators such as >= or ==.",
            },
            {
              name: "itemMetric",
              strength: "equipment metric predicate",
              description:
                "Use inside metadata for weapon, armor, and shield stat filters with symbolic operators such as >= or ==.",
            },
            {
              name: "spellKinds",
              strength: "metadata field example",
              description: "Use in metadata predicates for spell refinement such as focus, ritual, or cantrip.",
            },
            {
              name: "saveType",
              strength: "metadata field example",
              description: "Use in metadata predicates for spell defense targeting such as reflex or will.",
            },
            {
              name: "areaType",
              strength: "metadata field example",
              description: "Use in metadata predicates for spell area shapes such as burst, cone, or line.",
            },
            {
              name: "query",
              strength: "literal text",
              description:
                "Prefer one short natural-language phrase or sentence with 1-3 concrete anchor terms. Avoid long comma-separated keyword lists by default.",
            },
            {
              name: "excludeQuery",
              strength: "literal exclusion",
              description:
                "Optional free-text exclusion terms. Remove ranked-search results whose indexed search text mentions these normalized terms.",
            },
            {
              name: "linksTo",
              strength: "exact indexed link filter",
              description:
                "Require records to reference one or more canonical PF2E record keys through indexed UUID-derived links.",
            },
            {
              name: "excludeLinksTo",
              strength: "exact indexed link exclusion",
              description:
                "Remove records that reference one or more canonical PF2E record keys through indexed UUID-derived links.",
            },
          ],
          retrievalPatterns: [
            {
              name: "lexical",
              description:
                "Lexical-first retrieval for exact names, rules terms, and precise Pathfinder vocabulary. Use short exact or near-exact text.",
            },
            {
              name: "balanced",
              description:
                "Default hybrid retrieval for broad themed search inside explicit category and subcategory boundaries. Prefer one concise phrase or sentence with concrete anchors.",
            },
            {
              name: "concept",
              description:
                "Semantic-forward hybrid retrieval for exploratory concept search when exact wording is less important. Prefer one or two natural-language sentences over keyword piles.",
            },
          ],
          categories: vocabulary.categories,
          subcategories: vocabulary.subcategories,
          subcategoriesByCategory: CATEGORY_SUBCATEGORY_MAP,
          rarities: vocabulary.rarities,
          sizes: vocabulary.sizes,
          sourceCategories: vocabulary.sourceCategories,
          searchProfiles: [
            {
              value: "lexical",
              summary: "Lexical-first exact matching with short exact or near-exact text.",
            },
            {
              value: "balanced",
              summary: "Default hybrid search for concise natural-language queries with concrete anchors.",
            },
            {
              value: "concept",
              summary: "Semantic-forward hybrid search for exploratory natural-language concept descriptions.",
            },
          ],
          topLevelGuidance: {
            subcategory: "Structured within-category inclusion. Useful for narrowing to a specific public family.",
            rarity: "Keep rarity top-level for the common common/uncommon/rare/unique boundary.",
            levelMin: "Numeric lower level bound.",
            levelMax: "Numeric upper level bound.",
            priceMin: "Numeric lower price bound in copper pieces.",
            priceMax: "Numeric upper price bound in copper pieces.",
            actionCost: "Numeric action-cost boundary shared by spells and item activations.",
            linksTo:
              "Exact indexed reverse-link filter over canonical record keys. Use linksToMode to require any or all targets.",
            linksToMode: "Controls whether linksTo requires any supplied target or all supplied targets.",
            excludeLinksTo: "Exact indexed reverse-link exclusion over canonical record keys.",
            note: "Use top-level boundaries first; use metadata for typed field predicates and boolean composition.",
          },
          metadataFilters: {
            ...metadataSemantics,
            notes: {
              useCase:
                "Use metadata filters for category-specific facets after top-level category, subcategory, scopes, and numeric bounds.",
              applicability:
                "metadataFieldsByCategory is the primary discovery surface for which metadata fields are meaningful within a category.",
              subcategoryNarrowing:
                "metadataFieldsByCategoryAndSubcategory is intentionally sparse and only lists narrower cases where a field is more specific than its parent category.",
            },
          },
          filterValueDiscovery: {
            nonMetadataFields: ["sources", "categories", "subcategories", "packs", "actorMetrics", "itemMetrics"],
            note: "pf2e_list_filter_values enumerates live values for one chosen field. Learn which metadata fields are meaningful from metadataFilters first.",
          },
          heuristicVocabulary: {
            commonDerivedTagsByCategory: vocabulary.commonDerivedTagsByCategory,
            derivedTagOntologyFamilies: vocabulary.derivedTagOntologyFamilies,
            derivedTagOntologyTags: vocabulary.derivedTagOntologyTags,
            derivedTagCatalog: vocabulary.derivedTagCatalog,
          },
          vocabulary,
          rankingConfig: dataService.getRankingConfigStatus(),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_list_filter_values",
    {
      description:
        "Enumerate live corpus values for a filterable field, optionally constrained by category, subcategory, or scopes. Use this when the caller needs discoverable filter values before searching.",
      inputSchema: {
        field: filterValueFieldSchema.describe("Filter field to enumerate from the current search corpus."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        scopes: z.array(searchScopeSchema).min(1).optional().describe(SCOPES_HINT_DESCRIPTION),
        metricPrefix: z
          .string()
          .optional()
          .describe(
            "Optional metric namespace prefix when field is actorMetrics or itemMetrics, such as ability., save., weapon., armor., or shield.",
          ),
        metric: z
          .string()
          .optional()
          .describe(
            "Optional specific metric key when field is actorMetrics or itemMetrics. Text and boolean metrics return live values; otherwise the metric field lists keys.",
          ),
      },
    },
    (input) => {
      const result = dataService.listFilterValues(input);
      return {
        content: [
          {
            type: "text",
            text: `Found ${result.values.length} ${result.field} value${result.values.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          field: result.field,
          values: result.values,
        },
      };
    },
  );

  server.registerTool(
    "pf2e_list_packs",
    {
      description:
        "List available PF2E packs with labels, document types, and record counts. Use pf2e_get_search_semantics for the category and subcategory ontology.",
    },
    () => {
      const packs = dataService.listPacks().map(summarizePack);
      return {
        content: [
          {
            type: "text",
            text: `Loaded ${stats.packCount} packs with ${stats.recordCount} records.`,
          },
        ],
        structuredContent: {
          packCount: stats.packCount,
          recordCount: stats.recordCount,
          packs,
          warnings: startupWarnings,
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_pack_metadata",
    {
      description: "Get metadata for a specific PF2E pack.",
      inputSchema: {
        pack: z.string().describe("Pack name or label, for example spells or Pathfinder Monster Core."),
      },
    },
    ({ pack }) => {
      const match = dataService.getPack(pack);
      if (!match) {
        throw new Error(`Unknown pack: ${pack}`);
      }

      return {
        content: [
          {
            type: "text",
            text: `${match.label} (${match.name}) contains ${match.recordCount} records.`,
          },
        ],
        structuredContent: {
          pack: summarizePack(match),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_list_records",
    {
      description:
        "List PF2E records using deterministic, non-ranked structured filters and pagination. Use this for browse-and-page flows when stable listing matters more than ranked retrieval.",
      inputSchema: {
        pack: z.string().optional().describe("Optional pack name or label."),
        linksTo: recordKeyArraySchema
          .optional()
          .describe(
            "Canonical target record keys that candidate records must reference through indexed UUID-derived links.",
          ),
        linksToMode: linksToModeSchema
          .optional()
          .describe("How linksTo behaves when multiple target record keys are provided. Defaults to any."),
        excludeLinksTo: recordKeyArraySchema
          .optional()
          .describe(
            "Canonical target record keys that candidate records must not reference through indexed UUID-derived links.",
          ),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        scopes: z.array(searchScopeSchema).min(1).optional().describe(SCOPES_HINT_DESCRIPTION),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter, for example common or uncommon."),
        metadata: metadataFilterSchema
          .optional()
          .describe(
            "Typed metadata filter DSL. Use grouped boolean predicates over metadata fields such as traits, families, sourceCategory, usage, weaponGroup, or damageTypes.",
          ),
        priceMin: z.number().optional().describe("Minimum item price in copper pieces."),
        priceMax: z.number().optional().describe("Maximum item price in copper pieces."),
        actionCost: z.number().int().optional().describe("Action cost filter."),
        offset: z.number().int().optional().describe("Pagination offset."),
        limit: z.number().int().optional().describe("Pagination limit, max 100."),
      },
    },
    (input) => {
      const result = dataService.listRecords(input);
      return {
        content: [
          {
            type: "text",
            text: formatSearchResult("PF2E records:", result.total, result.records),
          },
        ],
        structuredContent: {
          total: result.total,
          offset: result.offset,
          limit: result.limit,
          records: result.records.map((record) => summarizeRecord(record)),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_search",
    {
      description:
        "Search PF2E records using ranked retrieval with natural-language text and/or structured filters. Best for exploratory discovery; use pf2e_list_records for deterministic listing and pf2e_lookup for exact names.",
      inputSchema: {
        searchProfile: searchProfileSchema
          .optional()
          .describe(
            "User-facing retrieval profile. lexical is lexical-first, balanced is the default hybrid profile for broad themed search, and concept is semantic-forward hybrid search.",
          ),
        explain: z
          .boolean()
          .optional()
          .describe("Include score breakdowns and query-analysis details in the response."),
        nameQuery: z.string().optional().describe("Name text to search for."),
        query: z
          .string()
          .optional()
          .describe(
            "General free-text search input. Prefer one short natural-language phrase or sentence with 1-3 concrete anchor terms. Avoid long comma-separated keyword lists by default. If searchProfile is omitted, query defaults search to the balanced profile.",
          ),
        excludeQuery: z
          .string()
          .optional()
          .describe(
            "Optional free-text exclusion terms. Remove results whose indexed search text mentions these normalized terms.",
          ),
        linksTo: recordKeyArraySchema
          .optional()
          .describe(
            "Canonical target record keys that candidate records must reference through indexed UUID-derived links.",
          ),
        linksToMode: linksToModeSchema
          .optional()
          .describe("How linksTo behaves when multiple target record keys are provided. Defaults to any."),
        excludeLinksTo: recordKeyArraySchema
          .optional()
          .describe(
            "Canonical target record keys that candidate records must not reference through indexed UUID-derived links.",
          ),
        pack: z.string().optional().describe("Optional pack name or label."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        scopes: z.array(searchScopeSchema).min(1).optional().describe(SCOPES_HINT_DESCRIPTION),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter."),
        metadata: metadataFilterSchema
          .optional()
          .describe(
            "Typed metadata filter DSL. Use grouped boolean predicates over metadata fields such as traits, families, sourceCategory, usage, weaponGroup, or damageTypes.",
          ),
        priceMin: z.number().optional().describe("Minimum item price in copper pieces."),
        priceMax: z.number().optional().describe("Maximum item price in copper pieces."),
        actionCost: z.number().int().optional().describe("Action cost filter."),
        offset: z.number().int().optional().describe("Pagination offset."),
        limit: z.number().int().optional().describe("Pagination limit, max 100."),
      },
    },
    async (input) => {
      const result = await dataService.search(input);
      return {
        content: [
          {
            type: "text",
            text: formatSearchResult("PF2E search results:", result.total, result.records),
          },
        ],
        structuredContent: {
          searchProfile: result.searchProfile,
          mode: result.mode,
          total: result.total,
          offset: result.offset,
          limit: result.limit,
          explain: result.explain ?? null,
          records: result.records.map((record, index) =>
            summarizeRecord(record, "full", result.explain?.records[index]),
          ),
        },
      };
    },
  );
}
