import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as z from "zod/v4";

import { Pf2eDataService } from "../data/service.js";
import {
  CATEGORY_HINT_DESCRIPTION,
  searchCategorySchema,
  searchSubcategorySchema,
  SUBCATEGORY_HINT_DESCRIPTION,
} from "./tool-schemas.js";
import { summarizeRecord } from "./presenters.js";

export function registerLookupTools(server: McpServer, dataService: Pf2eDataService): void {
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
    "pf2e_get_record_by_key",
    {
      description: "Get one exact PF2E record by canonical recordKey.",
      inputSchema: {
        recordKey: z.string().describe("Canonical key in the form packName:recordId."),
      },
    },
    async ({ recordKey }) => {
      const record = dataService.getRecord(recordKey);

      if (!record) {
        throw new Error("Record not found. Provide a valid recordKey.");
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

  server.registerTool(
    "pf2e_get_records_by_key",
    {
      description: "Fetch multiple exact PF2E records by canonical recordKey.",
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
}
