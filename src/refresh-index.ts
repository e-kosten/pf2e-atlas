#!/usr/bin/env node

import { loadConfig } from "./config.js";
import { Pf2eDataService } from "./pf2e-data.js";

async function main(): Promise<void> {
  const config = await loadConfig();
  const service = await Pf2eDataService.rebuildIndex(config.rootPath, config.manifestPath, {
    indexPath: config.indexPath,
    embedding: config.embeddings,
  });

  const stats = service.getStats();
  for (const warning of service.warnings) {
    console.error(warning);
  }
  console.error(`Rebuilt PF2E index at ${config.indexPath} with ${stats.packCount} packs and ${stats.recordCount} records.`);
  service.close();
}

main().catch((error) => {
  console.error(`Index refresh failed: ${(error as Error).message}`);
  process.exit(1);
});
