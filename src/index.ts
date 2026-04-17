#!/usr/bin/env node

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";

import { loadPf2eApplicationRuntime } from "./app/runtime.js";
import { registerLookupTools } from "./server/register-lookup-tools.js";
import { registerRuleTools } from "./server/register-rule-tools.js";
import { registerSearchTools } from "./server/register-search-tools.js";

async function main(): Promise<void> {
  const runtime = await loadPf2eApplicationRuntime();
  const { config, dataService, startupWarnings, stats } = runtime;

  const server = new McpServer({
    name: "pathfinder-2e-foundry-mcp",
    version: "0.1.0",
  });

  try {
    registerSearchTools(server, dataService, stats, startupWarnings);
    registerLookupTools(server, dataService);
    registerRuleTools(server, dataService);

    const transport = new StdioServerTransport();
    await server.connect(transport);
    console.error(
      `Pathfinder 2E MCP server running on stdio with ${stats.packCount} packs and ${stats.recordCount} records from ${config.rootPath}`,
    );
  } catch (error) {
    runtime.close();
    throw error;
  }
}

main().catch((error) => {
  console.error(`Server error: ${(error as Error).message}`);
  process.exit(1);
});
