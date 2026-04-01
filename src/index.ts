#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import * as z from "zod/v4";

import { CATEGORY_SUBCATEGORY_MAP } from "./categories.js";
import { loadConfig } from "./config.js";
import { Pf2eDataService } from "./pf2e-data.js";
import { RankingConfigStore } from "./ranking-config.js";
import {
  CATEGORY_HINT_DESCRIPTION,
  SCOPES_HINT_DESCRIPTION,
  SUBCATEGORY_HINT_DESCRIPTION,
  searchCategorySchema,
  searchProfileSchema,
  searchScopeSchema,
  searchSubcategorySchema,
  sourceCategorySchema,
  spellKindSchema,
} from "./tool-schemas.js";
import { NormalizedRecord, PackInfo, RecordDetail, RuleReferenceEdge, SearchRecordExplanation } from "./types.js";

function summarizeRecord(
  record: NormalizedRecord,
  detail: RecordDetail = "full",
  explanation?: SearchRecordExplanation,
): Record<string, unknown> {
  const summary: Record<string, unknown> = {
    recordKey: record.recordKey,
    id: record.id,
    name: record.name,
    aliases: record.aliases,
    category: record.category,
    subcategory: record.subcategory,
    rawRecordType: record.type,
    packName: record.packName,
    packLabel: record.packLabel,
    level: record.level,
    rarity: record.rarity,
    traits: record.traits,
    traditions: record.traditions,
    spellKinds: record.spellKinds,
    publicationTitle: record.publicationTitle,
    hasDescription: record.hasDescription,
    descriptionSnippet: record.descriptionSnippet,
    sourceCategory: record.sourceCategory,
  };

  if (explanation) {
    summary.searchExplain = explanation;
  }

  if (detail === "minimal") {
    return summary;
  }

  Object.assign(summary, {
    descriptionText: record.descriptionText,
    isUnique: record.isUnique,
    size: record.size,
    priceCp: record.priceCp,
    bulkValue: record.bulkValue,
    actionCost: record.actionCost,
  });

  if (detail === "full") {
    summary.sourcePath = record.sourcePath;
  }

  return summary;
}

function summarizePack(pack: PackInfo): Record<string, unknown> {
  return {
    name: pack.name,
    label: pack.label,
    documentType: pack.documentType,
    declaredPath: pack.declaredPath,
    resolvedPath: pack.resolvedPath,
    recordCount: pack.recordCount,
  };
}

function formatSearchResult(prefix: string, total: number, records: NormalizedRecord[]): string {
  const lines = records.map((record) => {
    const level = record.level !== null ? `level ${record.level}` : "level n/a";
    const subtype = record.subcategory ? `/${record.subcategory}` : "";
    const aliasText = record.aliases.length > 0 ? `; aliases: ${record.aliases.join(", ")}` : "";
    return `- ${record.name} (${record.packLabel}, ${record.category}${subtype}, ${level}${aliasText})`;
  });

  return [prefix, `Total matches: ${total}`, ...lines].join("\n");
}

function summarizeEdge(edge: RuleReferenceEdge): Record<string, unknown> {
  return {
    fromRecordKey: edge.fromRecordKey,
    toRecordKey: edge.toRecordKey,
    displayText: edge.displayText,
    referenceText: edge.referenceText,
    direction: edge.direction,
    relationshipType: edge.relationshipType,
    sourcePackName: edge.sourcePackName,
    sourceRecordType: edge.sourceRecordType,
    sourceDocumentType: edge.sourceDocumentType,
    sourceCategory: edge.sourceCategory,
  };
}

async function main(): Promise<void> {
  const config = await loadConfig();
  const rankingConfigStore = await RankingConfigStore.create(config.ranking.configPath);
  let dataService: Pf2eDataService;
  try {
    dataService = await Pf2eDataService.load(config.rootPath, config.manifestPath, {
      indexPath: config.indexPath,
      embedding: config.embeddings,
      rankingConfigStore,
    });
  } catch (error) {
    rankingConfigStore.close();
    throw error;
  }
  const stats = dataService.getStats();
  const startupWarnings = dataService.warnings;

  const server = new McpServer({
    name: "pathfinder-2e-foundry-mcp",
    version: "0.1.0",
  });

  server.registerTool(
    "pf2e_get_search_semantics",
    {
      description: "Describe the explicit category-first search ontology, Pathfinder-native tags, and structured filters available to the calling agent. Use this for the full category/subcategory map and filter vocabulary.",
      inputSchema: {
        traitLimitPerCategory: z.number().int().min(3).max(25).optional().describe("Maximum common traits to return per category. Defaults to 12."),
      },
    },
    async ({ traitLimitPerCategory }) => {
      const vocabulary = dataService.getSearchVocabulary({ traitLimitPerCategory });
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
              description: "Best first cut for separating creature, hazard, spell, equipment, lore, and other user-facing PF2E families.",
            },
            {
              name: "subcategory",
              strength: "within-category boundary",
              description: "Include one narrower family such as hazard/haunt, equipment/consumable, or lore/deity.",
            },
            {
              name: "scopes",
              strength: "paired multi-family boundary",
              description: "Use for multi-category search when each category needs its own optional subcategory list, such as feat/archetype plus rule/action.",
            },
            {
              name: "traitsAny",
              strength: "strong taxonomy hint",
              description: "Preferred structured bridge from broad themes into PF2E ontology terms.",
            },
            {
              name: "traitsAll",
              strength: "strict backstop",
              description: "Best for deterministic narrowing when multiple taxonomy terms are essential.",
            },
            {
              name: "excludeTraits",
              strength: "hard exclusion",
              description: "Exclude Pathfinder-native tags when certain traits are disallowed.",
            },
            {
              name: "sources",
              strength: "hard boundary",
              description: "Include one or more source families such as core, rules, adventure, or unknown.",
            },
            {
              name: "excludeSources",
              strength: "hard exclusion",
              description: "Exclude one or more source families such as omitting adventure records from a broader search.",
            },
            {
              name: "traditions",
              strength: "spell refinement",
              description: "Useful for spell searches when the theme implies one or more magical traditions such as divine or occult.",
            },
            {
              name: "spellKinds",
              strength: "spell refinement",
              description: "Useful for spell searches when the theme implies spell kinds such as focus, ritual, or cantrip.",
            },
            {
              name: "query",
              strength: "literal text",
              description: "Prefer one short natural-language phrase or sentence with 1-3 concrete anchor terms. Avoid long comma-separated keyword lists by default.",
            },
          ],
          retrievalPatterns: [
            {
              name: "lexical",
              description: "Lexical-first retrieval for exact names, rules terms, and precise Pathfinder vocabulary. Use short exact or near-exact text.",
            },
            {
              name: "balanced",
              description: "Default hybrid retrieval for broad themed search inside explicit category and subcategory boundaries. Prefer one concise phrase or sentence with concrete anchors.",
            },
            {
              name: "concept",
              description: "Semantic-forward hybrid retrieval for exploratory concept search when exact wording is less important. Prefer one or two natural-language sentences over keyword piles.",
            },
          ],
          categories: vocabulary.categories,
          subcategories: vocabulary.subcategories,
          subcategoriesByCategory: CATEGORY_SUBCATEGORY_MAP,
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
          tagGuidance: {
            traitsAny: "Pathfinder-native tags. Use for soft narrowing when any listed tag is acceptable.",
            traitsAll: "Pathfinder-native tags. Use for strict narrowing when every listed tag is essential.",
            excludeTraits: "Pathfinder-native tags. Use for explicit exclusions when listed tags must not be present.",
            subcategory: "Structured within-category inclusion. Useful for narrowing to a specific public family.",
            sources: "Structured source-family inclusions. Useful for pinning a search to core, rules, adventure, or unknown records.",
            excludeSources: "Structured source-family exclusions. Useful for omitting one or more source families from a broader search.",
            traditions: "Spell-only structured inclusions. Use for spell tradition refinement such as arcane, divine, occult, or primal.",
            spellKinds: "Spell-only structured inclusions. Use for spell kind refinement such as focus, ritual, or cantrip.",
            note: "The server no longer infers hidden semantic tags or taxonomy terms from theme text.",
          },
          vocabulary,
          rankingConfig: dataService.getRankingConfigStatus(),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_list_packs",
    {
      description: "List available PF2E packs with labels, document types, and record counts. This lists packs, not the search category ontology; use pf2e_get_search_semantics for categories and subcategories.",
    },
    async () => {
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
      description: "Get metadata for a specific PF2E pack/category.",
      inputSchema: {
        pack: z.string().describe("Pack name or label, for example spells or Pathfinder Monster Core."),
      },
    },
    async ({ pack }) => {
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
      description: "List records inside a specific PF2E pack with optional structured filters. Use this when pack scope matters more than ranked search.",
      inputSchema: {
        pack: z.string().describe("Pack name or label."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        scopes: z.array(searchScopeSchema).min(1).optional().describe(SCOPES_HINT_DESCRIPTION),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter, for example common or uncommon."),
        traitsAll: z.array(z.string()).optional().describe("All listed traits must be present."),
        traitsAny: z.array(z.string()).optional().describe("At least one listed trait must be present."),
        excludeTraits: z.array(z.string()).optional().describe("Listed traits must not be present."),
        sources: z.array(sourceCategorySchema).optional().describe("Restrict results to the listed source families."),
        excludeSources: z.array(sourceCategorySchema).optional().describe("Exclude results from the listed source families."),
        traditions: z.array(z.string()).optional().describe("Include spells from any of the listed traditions."),
        spellKinds: z.array(spellKindSchema).optional().describe("Include spells with any of the listed spell kinds."),
        publicationTitle: z.string().optional().describe("Publication title contains this text."),
        excludeUnique: z.boolean().optional().describe("Exclude unique records."),
        excludeMissingDescription: z.boolean().optional().describe("Exclude records without description or lore text."),
        size: z.string().optional().describe("Actor size filter."),
        priceMin: z.number().optional().describe("Minimum item price in copper pieces."),
        priceMax: z.number().optional().describe("Maximum item price in copper pieces."),
        actionCost: z.number().int().optional().describe("Action cost filter."),
        offset: z.number().int().optional().describe("Pagination offset."),
        limit: z.number().int().optional().describe("Pagination limit, max 100."),
      },
    },
    async (input) => {
      const result = dataService.listRecords(input);
      return {
        content: [
          {
            type: "text",
            text: formatSearchResult(`Records in ${input.pack}:`, result.total, result.records),
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
      description: "Search PF2E records using natural-language text and structured filters. Best for exploratory discovery; use pf2e_lookup for exact names.",
      inputSchema: {
        searchProfile: searchProfileSchema.optional().describe("User-facing retrieval profile. lexical is lexical-first, balanced is the default hybrid profile for broad themed search, and concept is semantic-forward hybrid search."),
        explain: z.boolean().optional().describe("Include score breakdowns and query-analysis details in the response."),
        nameQuery: z.string().optional().describe("Name text to search for."),
        query: z.string().optional().describe("General free-text search input. Prefer one short natural-language phrase or sentence with 1-3 concrete anchor terms. Avoid long comma-separated keyword lists by default. If searchProfile is omitted, query defaults search to the balanced profile."),
        pack: z.string().optional().describe("Optional pack name or label."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        scopes: z.array(searchScopeSchema).min(1).optional().describe(SCOPES_HINT_DESCRIPTION),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter."),
        traitsAll: z.array(z.string()).optional().describe("All listed traits must be present."),
        traitsAny: z.array(z.string()).optional().describe("At least one listed trait must be present."),
        excludeTraits: z.array(z.string()).optional().describe("Listed traits must not be present."),
        sources: z.array(sourceCategorySchema).optional().describe("Restrict results to the listed source families."),
        excludeSources: z.array(sourceCategorySchema).optional().describe("Exclude results from the listed source families."),
        traditions: z.array(z.string()).optional().describe("Include spells from any of the listed traditions."),
        spellKinds: z.array(spellKindSchema).optional().describe("Include spells with any of the listed spell kinds."),
        publicationTitle: z.string().optional().describe("Publication title contains this text."),
        excludeUnique: z.boolean().optional().describe("Exclude unique records."),
        excludeMissingDescription: z.boolean().optional().describe("Exclude records without description or lore text."),
        size: z.string().optional().describe("Actor size filter."),
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
          records: result.records.map((record, index) => summarizeRecord(record, "full", result.explain?.records[index])),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_lookup",
    {
      description: "Find the best-matching PF2E record by name. Use this first for exact named lookups such as feats, spells, items, creatures, actions, and conditions. For the full search ontology, use pf2e_get_search_semantics.",
      inputSchema: {
        name: z.string().describe("Record name to look up."),
        pack: z.string().optional().describe("Optional pack name or label."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to full for backward compatibility."),
        includeAlternatives: z.boolean().optional().describe("Include alternative matches. Defaults to true."),
      },
    },
    async ({ name, detail = "full", includeAlternatives = true, ...options }) => {
      const lookup = dataService.lookup(name, options);
      if (!lookup.match) {
        return {
          content: [
            {
              type: "text",
              text: `No PF2E record matched "${name}".`,
            },
          ],
          structuredContent: {
            match: null,
            alternatives: [],
          },
        };
      }

      return {
        content: [
          {
            type: "text",
            text: `Best match: ${lookup.match.name} (${lookup.match.packLabel}, ${lookup.match.type})`,
          },
        ],
        structuredContent: {
          match: summarizeRecord(lookup.match, detail),
          alternatives: includeAlternatives ? lookup.alternatives.map((record) => summarizeRecord(record, detail)) : [],
        },
      };
    },
  );

  server.registerTool(
    "pf2e_lookup_many",
    {
      description: "Resolve multiple PF2E names in one call. Use this for batches of exact named lookups when you want compact match metadata.",
      inputSchema: {
        queries: z.array(
          z.object({
            name: z.string().describe("Record name to look up."),
            pack: z.string().optional().describe("Optional pack name or label."),
            category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
            subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
          }),
        ).min(1).max(25),
        coreOnly: z.boolean().optional().describe("Restrict primary matches to core content."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to minimal."),
        includeAlternatives: z.boolean().optional().describe("Include alternative matches. Defaults to false."),
      },
    },
    async ({ queries, coreOnly, detail = "minimal", includeAlternatives = false }) => {
      const results = dataService.lookupMany(queries, { coreOnly });
      return {
        content: [
          {
            type: "text",
            text: `Resolved ${results.filter((result) => result.match).length} of ${results.length} PF2E lookup${results.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          results: results.map((result) => ({
            query: result.query,
            matchType: result.matchType,
            match: result.match ? summarizeRecord(result.match, detail) : null,
            alternatives: includeAlternatives ? result.alternatives.map((record) => summarizeRecord(record, detail)) : [],
          })),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_rules_context",
    {
      description: "Resolve a named PF2E rule record and follow linked compendium references from its rules text. Use this for interaction questions after you know the rule name. Example: {\"name\":\"Blinded\",\"category\":\"rule\",\"subcategory\":\"condition\",\"referenceDepth\":1}",
      inputSchema: {
        name: z.string().describe("Record name to look up."),
        pack: z.string().optional().describe("Optional pack name or label."),
        category: searchCategorySchema.optional().describe(CATEGORY_HINT_DESCRIPTION),
        subcategory: searchSubcategorySchema.optional().describe(SUBCATEGORY_HINT_DESCRIPTION),
        referenceDepth: z.coerce.number().int().min(1).max(2).optional().describe("How many reference hops to follow. Must be 1 or 2. Defaults to 1."),
        maxReferences: z.coerce.number().int().min(1).max(25).optional().describe("Maximum number of linked records to return. Defaults to 8."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to full for backward compatibility."),
      },
    },
    async ({ name, detail = "full", ...options }) => {
      const result = dataService.getRulesContext(name, options);
      if (!result) {
        return {
          content: [
            {
              type: "text",
              text: `No PF2E record matched "${name}".`,
            },
          ],
          structuredContent: {
            record: null,
            references: [],
            edges: [],
          },
        };
      }

      return {
        content: [
          {
            type: "text",
            text: `${result.record.name} with ${result.references.length} linked rules reference${result.references.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          record: summarizeRecord(result.record, detail),
          references: result.references.map((record) => summarizeRecord(record, detail)),
          edges: result.edges,
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_records",
    {
      description: "Fetch multiple PF2E records by canonical recordKey.",
      inputSchema: {
        recordKeys: z.array(z.string()).min(1).max(100).describe("Canonical keys in the form packName:recordId."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to standard."),
      },
    },
    async ({ recordKeys, detail = "standard" }) => {
      const records = dataService.getRecordsByKeys(recordKeys);
      return {
        content: [
          {
            type: "text",
            text: `Fetched ${records.length} PF2E record${records.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          records: records.map((record) => summarizeRecord(record, detail)),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_linked_rules",
    {
      description: "Fetch direct linked-rule support records for one or more primary PF2E records.",
      inputSchema: {
        recordKeys: z.array(z.string()).min(1).max(25).describe("Primary canonical record keys."),
        coreOnly: z.boolean().optional().describe("Restrict linked support records to core content."),
        maxPerPrimary: z.coerce.number().int().min(1).max(25).optional().describe("Maximum linked records to keep per primary. Defaults to 4."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to minimal."),
      },
    },
    async ({ recordKeys, coreOnly, maxPerPrimary, detail = "minimal" }) => {
      const result = dataService.getLinkedRules(recordKeys, { coreOnly, maxPerPrimary });
      return {
        content: [
          {
            type: "text",
            text: `Collected ${result.records.length} linked support record${result.records.length === 1 ? "" : "s"} across ${recordKeys.length} primar${recordKeys.length === 1 ? "y" : "ies"}.`,
          },
        ],
        structuredContent: {
          records: result.records.map((record) => summarizeRecord(record, detail)),
          edges: result.edges.map(summarizeEdge),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_backlinks",
    {
      description: "Fetch curated reusable-rule backlinks that reference one or more PF2E records. Backlinks are limited to actions, feats, and class features.",
      inputSchema: {
        recordKeys: z.array(z.string()).min(1).max(25).describe("Target canonical record keys."),
        coreOnly: z.boolean().optional().describe("Restrict backlink source records to core content."),
        maxPerPrimary: z.coerce.number().int().min(1).max(25).optional().describe("Maximum backlinks to keep per primary. Defaults to 4."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to minimal."),
      },
    },
    async ({ recordKeys, coreOnly, maxPerPrimary, detail = "minimal" }) => {
      const result = dataService.getBacklinks(recordKeys, { coreOnly, maxPerPrimary });
      return {
        content: [
          {
            type: "text",
            text: `Collected ${result.records.length} curated backlink record${result.records.length === 1 ? "" : "s"} across ${recordKeys.length} target${recordKeys.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          records: result.records.map((record) => summarizeRecord(record, detail)),
          edges: result.edges.map(summarizeEdge),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_collect_rule_question_context",
    {
      description: "Collect retrieval context for a narrow PF2E rules question. Prefer this when the user is asking how two or more named rules interact. Returns matches and linked support records without a synthesized answer.",
      inputSchema: {
        rules: z.array(z.string()).optional().describe("Explicit rule names to resolve. Preferred over free-text questions."),
        question: z.string().optional().describe("Optional free-text question used only for shallow name extraction when rules are not provided."),
        coreOnly: z.boolean().optional().describe("Restrict primary and related matches to core content where supported."),
        includeBacklinks: z.boolean().optional().describe("Include curated backlinks from actions, feats, and class features. Defaults to false."),
        maxOutgoingPerPrimary: z.coerce.number().int().min(1).max(25).optional().describe("Maximum outgoing support records per primary. Defaults to 4."),
        maxBacklinksPerPrimary: z.coerce.number().int().min(1).max(25).optional().describe("Maximum curated backlinks per primary. Defaults to 4."),
        detail: z.enum(["minimal", "standard", "full"]).optional().describe("Response detail level. Defaults to minimal."),
        includeAlternatives: z.boolean().optional().describe("Include alternative primary matches. Defaults to false."),
      },
    },
    async ({ rules, question, coreOnly, includeBacklinks, maxOutgoingPerPrimary, maxBacklinksPerPrimary, detail = "minimal", includeAlternatives = false }) => {
      if ((!rules || rules.length === 0) && !question) {
        throw new Error("Provide rules or question.");
      }

      const result = dataService.collectRuleQuestionContext({
        rules,
        question,
        coreOnly,
        includeBacklinks,
        maxOutgoingPerPrimary,
        maxBacklinksPerPrimary,
      });

      return {
        content: [
          {
            type: "text",
            text: `Collected context for ${result.primary.length} primary lookup${result.primary.length === 1 ? "" : "s"} with ${result.outgoing.records.length} outgoing support record${result.outgoing.records.length === 1 ? "" : "s"} and ${result.backlinks.records.length} backlink record${result.backlinks.records.length === 1 ? "" : "s"}.`,
          },
        ],
        structuredContent: {
          primary: result.primary.map((entry) => ({
            query: entry.query,
            matchType: entry.matchType,
            match: entry.match ? summarizeRecord(entry.match, detail) : null,
            alternatives: includeAlternatives ? entry.alternatives.map((record) => summarizeRecord(record, detail)) : [],
          })),
          outgoing: {
            records: result.outgoing.records.map((record) => summarizeRecord(record, detail)),
            edges: result.outgoing.edges.map(summarizeEdge),
          },
          backlinks: {
            records: result.backlinks.records.map((record) => summarizeRecord(record, detail)),
            edges: result.backlinks.edges.map(summarizeEdge),
          },
          edges: result.edges.map(summarizeEdge),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_get_record",
    {
      description: "Get a full PF2E record by canonical recordKey or by pack and record id.",
      inputSchema: {
        recordKey: z.string().optional().describe("Canonical key in the form packName:recordId."),
        pack: z.string().optional().describe("Pack name or label."),
        id: z.string().optional().describe("Foundry record _id."),
      },
    },
    async ({ recordKey, pack, id }) => {
      const record = recordKey
        ? dataService.getRecord(recordKey)
        : pack && id
          ? dataService.getRecord(dataService.getPack(pack)?.name ?? pack, id)
          : undefined;

      if (!record) {
        throw new Error("Record not found. Provide recordKey or both pack and id.");
      }

      return {
        content: [
          {
            type: "text",
            text: `${record.name} (${record.packLabel}, ${record.type})`,
          },
        ],
        structuredContent: {
          record: summarizeRecord(record),
          raw: record.raw,
        },
      };
    },
  );

  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error(
    `Pathfinder 2E MCP server running on stdio with ${stats.packCount} packs and ${stats.recordCount} records from ${config.rootPath}`,
  );
}

main().catch((error) => {
  console.error(`Server error: ${(error as Error).message}`);
  process.exit(1);
});
