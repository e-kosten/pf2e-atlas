#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

import { loadConfig } from "./app/config.js";
import { Pf2eDataService } from "./data/service.js";
import { RankingConfigStore } from "./search/ranking-config.js";
import { registerLookupTools } from "./server/register-lookup-tools.js";
import { registerRuleTools } from "./server/register-rule-tools.js";
import { registerSearchTools } from "./server/register-search-tools.js";

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

  registerSearchTools(server, dataService, stats, startupWarnings);
  registerLookupTools(server, dataService);
  registerRuleTools(server, dataService);

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
