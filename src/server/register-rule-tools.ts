import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as z from "zod/v4";

import { Pf2eDataService } from "../data/service.js";
import { summarizeEdge, summarizeRecord } from "./presenters.js";

export function registerRuleTools(server: McpServer, dataService: Pf2eDataService): void {
  server.registerTool(
    "pf2e_collect_rule_question_context",
    {
      description:
        "Resolve one or more named PF2E rules from a narrow rules question and collect linked support records and optional curated backlinks. Use this first for interaction questions. Returns context without a synthesized answer.",
      inputSchema: {
        rules: z
          .array(z.string())
          .optional()
          .describe("Explicit rule names to resolve. Preferred over free-text questions."),
        question: z
          .string()
          .optional()
          .describe("Optional free-text question used only for shallow name extraction when rules are not provided."),
        coreOnly: z
          .boolean()
          .optional()
          .describe("Restrict primary and related matches to core content where supported."),
        includeBacklinks: z
          .boolean()
          .optional()
          .describe("Include curated backlinks from actions, feats, and class features. Defaults to false."),
        maxOutgoingPerPrimary: z.coerce
          .number()
          .int()
          .min(1)
          .max(25)
          .optional()
          .describe("Maximum outgoing support records per primary. Defaults to 4."),
        maxBacklinksPerPrimary: z.coerce
          .number()
          .int()
          .min(1)
          .max(25)
          .optional()
          .describe("Maximum curated backlinks per primary. Defaults to 4."),
        detail: z
          .enum(["minimal", "standard", "full"])
          .optional()
          .describe("Response detail level. Defaults to minimal."),
        includeAlternatives: z.boolean().optional().describe("Include alternative primary matches. Defaults to false."),
      },
    },
    ({
      rules,
      question,
      coreOnly,
      includeBacklinks,
      maxOutgoingPerPrimary,
      maxBacklinksPerPrimary,
      detail = "minimal",
      includeAlternatives = false,
    }) => {
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
            alternatives: includeAlternatives
              ? entry.alternatives.map((record) => summarizeRecord(record, detail))
              : [],
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
    "pf2e_get_rule_graph",
    {
      description:
        "Fetch low-level rule-graph records and edges for known canonical PF2E record keys. Use this when you already know the primary nodes or need explicit graph traversal.",
      inputSchema: {
        recordKeys: z.array(z.string()).min(1).max(25).describe("Primary canonical record keys."),
        coreOnly: z.boolean().optional().describe("Restrict linked support records to core content."),
        includeOutgoing: z
          .boolean()
          .optional()
          .describe(
            "Include direct outgoing linked-rule support records. Defaults to true when both direction flags are omitted.",
          ),
        includeBacklinks: z
          .boolean()
          .optional()
          .describe(
            "Include curated backlinks from actions, feats, and class features. Defaults to false when both direction flags are omitted.",
          ),
        maxOutgoingPerPrimary: z.coerce
          .number()
          .int()
          .min(1)
          .max(25)
          .optional()
          .describe("Maximum outgoing linked records to keep per primary. Defaults to 4."),
        maxBacklinksPerPrimary: z.coerce
          .number()
          .int()
          .min(1)
          .max(25)
          .optional()
          .describe("Maximum curated backlinks to keep per primary. Defaults to 4."),
        detail: z
          .enum(["minimal", "standard", "full"])
          .optional()
          .describe("Response detail level. Defaults to minimal."),
      },
    },
    ({
      recordKeys,
      coreOnly,
      includeOutgoing,
      includeBacklinks,
      maxOutgoingPerPrimary,
      maxBacklinksPerPrimary,
      detail = "minimal",
    }) => {
      const result = dataService.getRuleGraph(recordKeys, {
        coreOnly,
        includeOutgoing,
        includeBacklinks,
        maxOutgoingPerPrimary,
        maxBacklinksPerPrimary,
      });
      return {
        content: [
          {
            type: "text",
            text: `Collected ${result.outgoing.records.length} outgoing support record${result.outgoing.records.length === 1 ? "" : "s"} and ${result.backlinks.records.length} backlink record${result.backlinks.records.length === 1 ? "" : "s"} across ${recordKeys.length} primar${recordKeys.length === 1 ? "y" : "ies"}.`,
          },
        ],
        structuredContent: {
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
}
