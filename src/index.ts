#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import * as z from "zod/v4";

import { loadConfig } from "./config.js";
import { Pf2eDataService } from "./pf2e-data.js";
import { refreshPf2eCheckout } from "./pf2e-refresh.js";
import { NormalizedRecord, PackInfo } from "./types.js";

function summarizeRecord(record: NormalizedRecord): Record<string, unknown> {
  return {
    recordKey: record.recordKey,
    id: record.id,
    name: record.name,
    type: record.type,
    packName: record.packName,
    packLabel: record.packLabel,
    documentType: record.documentType,
    level: record.level,
    rarity: record.rarity,
    traits: record.traits,
    publicationTitle: record.publicationTitle,
    descriptionText: record.descriptionText,
    sourcePath: record.sourcePath,
    isUnique: record.isUnique,
    size: record.size,
    itemCategory: record.itemCategory,
    priceCp: record.priceCp,
    bulkValue: record.bulkValue,
    actionCost: record.actionCost,
    traditions: record.traditions,
  };
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
    return `- ${record.name} (${record.packLabel}, ${record.type}, ${level})`;
  });

  return [prefix, `Total matches: ${total}`, ...lines].join("\n");
}

async function main(): Promise<void> {
  const config = await loadConfig();
  const refreshResult = await refreshPf2eCheckout(config.rootPath);
  if (refreshResult.warning) {
    console.error(refreshResult.warning);
  } else {
    console.error(refreshResult.summary);
  }

  const dataService = await Pf2eDataService.load(config.rootPath, config.manifestPath, { indexPath: config.indexPath });
  const stats = dataService.getStats();
  const startupWarnings = refreshResult.warning
    ? [refreshResult.warning, ...dataService.warnings]
    : dataService.warnings;

  const server = new McpServer({
    name: "pathfinder-2e-foundry-mcp",
    version: "0.1.0",
  });

  server.registerTool(
    "pf2e_list_categories",
    {
      description: "List available PF2E packs/categories with labels, document types, and record counts.",
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
      description: "List records inside a specific PF2E pack/category with optional filters.",
      inputSchema: {
        mode: z.enum(["structured", "lexical", "hybrid"]).optional().describe("Retrieval mode. Defaults to structured."),
        pack: z.string().describe("Pack name or label."),
        recordType: z.string().optional().describe("Optional Foundry record type, for example spell, feat, npc, or hazard."),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter, for example common or uncommon."),
        traitsAll: z.array(z.string()).optional().describe("All listed traits must be present."),
        traitsAny: z.array(z.string()).optional().describe("At least one listed trait must be present."),
        tradition: z.string().optional().describe("Explicit spell tradition filter."),
        publicationTitle: z.string().optional().describe("Publication title contains this text."),
        excludeUnique: z.boolean().optional().describe("Exclude unique records."),
        size: z.string().optional().describe("Actor size filter."),
        itemCategory: z.string().optional().describe("Item category filter, for example weapon, spell, equipment, or consumable."),
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
          records: result.records.map(summarizeRecord),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_search",
    {
      description: "Search PF2E records across packs using name lookup and structured filters.",
      inputSchema: {
        mode: z.enum(["structured", "lexical", "hybrid"]).optional().describe("Retrieval mode. Defaults to structured."),
        nameQuery: z.string().optional().describe("Name text to search for."),
        themeQuery: z.string().optional().describe("Theme or semantic query text for lexical or hybrid search."),
        pack: z.string().optional().describe("Optional pack name or label."),
        documentType: z.string().optional().describe("Optional Foundry document type, for example Actor or Item."),
        recordType: z.string().optional().describe("Optional record type, for example spell, action, npc, or hazard."),
        levelMin: z.number().int().optional().describe("Minimum level inclusive."),
        levelMax: z.number().int().optional().describe("Maximum level inclusive."),
        rarity: z.string().optional().describe("Rarity filter."),
        traitsAll: z.array(z.string()).optional().describe("All listed traits must be present."),
        traitsAny: z.array(z.string()).optional().describe("At least one listed trait must be present."),
        tradition: z.string().optional().describe("Explicit spell tradition filter."),
        publicationTitle: z.string().optional().describe("Publication title contains this text."),
        excludeUnique: z.boolean().optional().describe("Exclude unique records."),
        size: z.string().optional().describe("Actor size filter."),
        itemCategory: z.string().optional().describe("Item category filter, for example weapon, spell, equipment, or consumable."),
        priceMin: z.number().optional().describe("Minimum item price in copper pieces."),
        priceMax: z.number().optional().describe("Maximum item price in copper pieces."),
        actionCost: z.number().int().optional().describe("Action cost filter."),
        offset: z.number().int().optional().describe("Pagination offset."),
        limit: z.number().int().optional().describe("Pagination limit, max 100."),
      },
    },
    async (input) => {
      const result = dataService.search(input);
      return {
        content: [
          {
            type: "text",
            text: formatSearchResult("PF2E search results:", result.total, result.records),
          },
        ],
        structuredContent: {
          total: result.total,
          offset: result.offset,
          limit: result.limit,
          records: result.records.map(summarizeRecord),
        },
      };
    },
  );

  server.registerTool(
    "pf2e_lookup",
    {
      description: "Find the best-matching PF2E record by name, with optional pack or type hints.",
      inputSchema: {
        name: z.string().describe("Record name to look up."),
        pack: z.string().optional().describe("Optional pack name or label."),
        documentType: z.string().optional().describe("Optional document type, for example Actor or Item."),
        recordType: z.string().optional().describe("Optional record type, for example spell, action, npc, or hazard."),
      },
    },
    async ({ name, ...options }) => {
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
          match: summarizeRecord(lookup.match),
          alternatives: lookup.alternatives.map(summarizeRecord),
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
