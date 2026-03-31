import { access } from "node:fs/promises";
import path from "node:path";
import { constants } from "node:fs";

import { DEFAULT_EMBEDDING_MODEL_ID, DEFAULT_EMBEDDING_REVISION } from "./embeddings.js";
import { AppConfig } from "./types.js";
import { expandHome } from "./utils.js";

function parseCliArgs(argv: string[]): Record<string, string> {
  const parsed: Record<string, string> = {};

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (!current || !current.startsWith("--")) {
      continue;
    }

    const [rawKey, inlineValue] = current.slice(2).split("=", 2);
    if (!rawKey) {
      continue;
    }

    const nextValue = inlineValue ?? argv[index + 1];
    if (!nextValue || nextValue.startsWith("--")) {
      continue;
    }

    parsed[rawKey] = nextValue;
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  return parsed;
}

async function canRead(targetPath: string): Promise<boolean> {
  try {
    await access(targetPath, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

export async function loadConfig(argv = process.argv.slice(2), env = process.env): Promise<AppConfig> {
  const args = parseCliArgs(argv);
  const configuredPath = args["data-path"] ?? env.PF2E_DATA_PATH ?? path.join(process.cwd(), "vendor", "pf2e");
  const configuredIndexPath = args["index-path"] ?? env.PF2E_INDEX_PATH ?? path.join(process.cwd(), ".cache", "pf2e-index.sqlite");
  const configuredEmbeddingProvider = args["embedding-provider"] ?? env.PF2E_EMBEDDING_PROVIDER ?? "hf-local";
  const configuredEmbeddingModel = args["embedding-model"] ?? env.PF2E_EMBEDDING_MODEL ?? DEFAULT_EMBEDDING_MODEL_ID;
  const configuredEmbeddingRevision = args["embedding-revision"] ?? env.PF2E_EMBEDDING_REVISION ?? DEFAULT_EMBEDDING_REVISION;
  const configuredEmbeddingCachePath = args["embedding-cache-path"] ?? env.PF2E_EMBEDDING_CACHE_PATH ?? path.join(process.cwd(), ".cache", "hf-models");
  const configuredEmbeddingLocalModelPath = args["embedding-local-model-path"] ?? env.PF2E_EMBEDDING_LOCAL_MODEL_PATH;
  const configuredRankingConfigPath = args["ranking-config-path"] ?? env.PF2E_RANKING_CONFIG_PATH ?? path.join(process.cwd(), "pf2e-ranking.json");

  const rootPath = path.resolve(expandHome(configuredPath));
  const indexPath = path.resolve(expandHome(configuredIndexPath));
  const embeddingCachePath = path.resolve(expandHome(configuredEmbeddingCachePath));
  const embeddingLocalModelPath = configuredEmbeddingLocalModelPath
    ? path.resolve(expandHome(configuredEmbeddingLocalModelPath))
    : null;
  const rankingConfigPath = path.resolve(expandHome(configuredRankingConfigPath));
  const manifestCandidates = [
    path.join(rootPath, "system.pf2e.json"),
    path.join(rootPath, "static", "system.json"),
  ];

  for (const candidate of manifestCandidates) {
    if (await canRead(candidate)) {
      return {
        dataPath: configuredPath,
        rootPath,
        manifestPath: candidate,
        indexPath,
        embeddings: {
          provider: configuredEmbeddingProvider === "hash" ? "hash" : "hf-local",
          modelId: configuredEmbeddingModel,
          modelRevision: configuredEmbeddingRevision,
          cachePath: embeddingCachePath,
          localModelPath: embeddingLocalModelPath,
        },
        ranking: {
          configPath: rankingConfigPath,
        },
      };
    }
  }

  throw new Error(
    `Could not find a readable PF2E system manifest under ${rootPath}. Clone the PF2E repo into vendor/pf2e or pass --data-path /path/to/pf2e.`,
  );
}
