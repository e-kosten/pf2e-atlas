#!/usr/bin/env node

import { loadConfig } from "./app/config.js";
import { prepareEmbeddingAssets } from "./embeddings.js";

function formatDuration(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes === 0) {
    return `${seconds}s`;
  }

  return `${minutes}m ${seconds}s`;
}

async function main(): Promise<void> {
  const config = await loadConfig();
  const startTime = Date.now();

  if (config.embeddings.provider === "hash") {
    console.error("Preparing embedding assets for the configured hash provider.");
  } else {
    console.error(
      `Preparing embedding assets for ${config.embeddings.modelId}${config.embeddings.modelRevision ? `@${config.embeddings.modelRevision}` : ""}.`,
    );
    console.error(`Model cache: ${config.embeddings.cachePath}`);
    console.error("The first run may download model files and can take several minutes.");
  }

  const assets = await prepareEmbeddingAssets(config.embeddings, {
    progressLogger: (message) => console.error(message),
  });

  if (assets.provider.provider === "hash") {
    console.error("Hash embeddings are configured; no remote embedding assets need to be prepared.");
    return;
  }

  console.error(
    `Prepared local embedding assets for ${assets.provider.model} (${assets.provider.dimensions} dimensions) in ${assets.cachePath} in ${formatDuration(Date.now() - startTime)}.`,
  );
}

main().catch((error) => {
  console.error(`Embedding refresh failed: ${(error as Error).message}`);
  process.exit(1);
});
