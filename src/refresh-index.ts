#!/usr/bin/env node

import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "./app/config.js";
import { writeMetadataGlossaryArtifact } from "./data/metadata-glossary.js";
import { Pf2eDataService } from "./data/service.js";
import { ConsoleProgressReporter } from "./progress.js";
import { writeDerivedTagOntologyExplorerDbCache } from "./tui/ontology-explorer/data.js";

function formatDuration(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes === 0) {
    return `${seconds}s`;
  }

  return `${minutes}m ${seconds}s`;
}

function parseReuseEmbeddingsFlag(argv: string[]): { reuseEmbeddings: boolean; remainingArgv: string[] } {
  let reuseEmbeddings = false;
  const remainingArgv: string[] = [];

  for (const arg of argv) {
    if (arg === "--reuse-embeddings") {
      reuseEmbeddings = true;
      continue;
    }
    if (arg === "--reuse-embeddings=true" || arg === "--reuse-embeddings=1") {
      reuseEmbeddings = true;
      continue;
    }
    if (arg === "--reuse-embeddings=false" || arg === "--reuse-embeddings=0") {
      reuseEmbeddings = false;
      continue;
    }
    remainingArgv.push(arg);
  }

  return { reuseEmbeddings, remainingArgv };
}

async function main(): Promise<void> {
  const { reuseEmbeddings, remainingArgv } = parseReuseEmbeddingsFlag(process.argv.slice(2));
  const config = await loadConfig(remainingArgv);
  const startTime = Date.now();
  const progress = new ConsoleProgressReporter(process.stderr);
  progress.log(`Rebuilding the PF2E index at ${config.indexPath}.`);

  try {
    const service = await Pf2eDataService.rebuildIndex(config.rootPath, config.manifestPath, {
      indexPath: config.indexPath,
      embedding: config.embeddings,
      progressLogger: (message) => progress.log(message),
      progressStatusLogger: (message) => progress.status(message),
      reuseEmbeddings,
    });

    const stats = service.getStats();
    for (const warning of service.warnings) {
      progress.log(warning);
    }
    service.close();

    progress.log("Materializing the ontology explorer cache.");
    const cacheDb = new DatabaseSync(config.indexPath);
    try {
      writeDerivedTagOntologyExplorerDbCache(cacheDb);
    } finally {
      cacheDb.close();
    }

    progress.log("Writing the metadata glossary artifact.");
    await writeMetadataGlossaryArtifact(config.rootPath, config.indexPath);

    progress.log(
      `Rebuilt PF2E index at ${config.indexPath} with ${stats.packCount} packs and ${stats.recordCount} records in ${formatDuration(Date.now() - startTime)}.`,
    );
  } catch (error) {
    progress.clear();
    throw error;
  }
}

main().catch((error) => {
  console.error(`Index refresh failed: ${(error as Error).message}`);
  process.exit(1);
});
