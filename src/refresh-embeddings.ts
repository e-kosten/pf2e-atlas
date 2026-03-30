#!/usr/bin/env node

import { loadConfig } from "./config.js";
import { prepareEmbeddingAssets } from "./embeddings.js";

async function main(): Promise<void> {
  const config = await loadConfig();
  const assets = await prepareEmbeddingAssets(config.embeddings);

  if (assets.provider.provider === "hash") {
    console.error("Hash embeddings are configured; no remote embedding assets need to be prepared.");
    return;
  }

  console.error(
    `Prepared local embedding assets for ${assets.provider.model} (${assets.provider.dimensions} dimensions) in ${assets.cachePath}.`,
  );
}

main().catch((error) => {
  console.error(`Embedding refresh failed: ${(error as Error).message}`);
  process.exit(1);
});
