import { mkdir } from "node:fs/promises";

import { normalizeText } from "./utils.js";
import { EmbeddingConfig, EmbeddingProviderKind } from "./types.js";

const DEFAULT_HASH_DIMENSIONS = 192;
export const DEFAULT_EMBEDDING_MODEL_ID = "Xenova/all-MiniLM-L12-v2";
export const DEFAULT_EMBEDDING_REVISION = "main";

type FeatureExtractionOutput = {
  data: Float32Array | number[];
  dims?: number[];
};

type FeatureExtractor = (
  input: string | string[],
  options?: Record<string, unknown>,
) => Promise<FeatureExtractionOutput>;

type TransformersModule = {
  env: {
    allowRemoteModels?: boolean;
    allowLocalModels?: boolean;
    cacheDir?: string;
    localModelPath?: string;
  };
  pipeline: (
    task: "feature-extraction",
    model: string,
    options?: Record<string, unknown>,
  ) => Promise<FeatureExtractor>;
};

export type EmbeddingProviderIdentity = {
  provider: EmbeddingProviderKind;
  model: string;
  revision: string | null;
  dimensions: number;
};

export interface EmbeddingProvider {
  readonly identity: EmbeddingProviderIdentity;
  embed(text: string): Promise<Float32Array>;
}

export type PreparedEmbeddingAssets = {
  provider: EmbeddingProviderIdentity;
  cachePath: string;
  localModelPath: string | null;
};

type EmbeddingPreparationOptions = {
  progressLogger?: (message: string) => void;
};

export class HashEmbeddingProvider implements EmbeddingProvider {
  readonly identity: EmbeddingProviderIdentity;

  constructor(dimensions = DEFAULT_HASH_DIMENSIONS) {
    this.identity = {
      provider: "hash",
      model: `feature-hash-${dimensions}`,
      revision: null,
      dimensions,
    };
  }

  async embed(text: string): Promise<Float32Array> {
    const normalized = normalizeText(text);
    const vector = new Float32Array(this.identity.dimensions);
    if (!normalized) {
      return vector;
    }

    for (const token of normalized.split(" ").filter(Boolean)) {
      const bucket = hashText(token) % this.identity.dimensions;
      const sign = hashText(`${token}:sign`) % 2 === 0 ? 1 : -1;
      vector[bucket] = (vector[bucket] ?? 0) + sign;
    }

    return normalizeVector(vector);
  }
}

export class HuggingFaceEmbeddingProvider implements EmbeddingProvider {
  readonly identity: EmbeddingProviderIdentity;

  private constructor(
    private readonly extractor: FeatureExtractor,
    identity: EmbeddingProviderIdentity,
  ) {
    this.identity = identity;
  }

  static async create(
    config: EmbeddingConfig,
    options: { allowRemoteModels: boolean },
  ): Promise<HuggingFaceEmbeddingProvider> {
    const { extractor, dimensions } = await initializeHuggingFaceExtractor(config, options);
    return new HuggingFaceEmbeddingProvider(extractor, buildProviderIdentity(config, dimensions));
  }

  async embed(text: string): Promise<Float32Array> {
    const normalized = normalizeText(text);
    if (!normalized) {
      return new Float32Array(this.identity.dimensions);
    }

    const output = await this.extractor(normalized, {
      pooling: "mean",
      normalize: true,
    });
    return output.data instanceof Float32Array
      ? new Float32Array(output.data)
      : Float32Array.from(output.data);
  }
}

export async function createEmbeddingProvider(config: EmbeddingConfig): Promise<{ provider: EmbeddingProvider; warnings: string[] }> {
  if (config.provider === "hash") {
    return {
      provider: new HashEmbeddingProvider(),
      warnings: [],
    };
  }

  try {
    return {
      provider: await HuggingFaceEmbeddingProvider.create(config, { allowRemoteModels: false }),
      warnings: [],
    };
  } catch (error) {
    throw new Error(buildRuntimeEmbeddingErrorMessage(config, error));
  }
}

export async function prepareEmbeddingAssets(
  config: EmbeddingConfig,
  options: EmbeddingPreparationOptions = {},
): Promise<PreparedEmbeddingAssets> {
  if (config.provider === "hash") {
    const provider = new HashEmbeddingProvider();
    options.progressLogger?.("Hash embeddings are configured; skipping remote model preparation.");
    return {
      provider: provider.identity,
      cachePath: config.cachePath,
      localModelPath: config.localModelPath,
    };
  }

  const { dimensions } = await initializeHuggingFaceExtractor(config, {
    allowRemoteModels: true,
    progressLogger: options.progressLogger,
  });
  options.progressLogger?.(
    `Embedding model ${config.modelId} is ready with ${dimensions} dimensions.`,
  );
  return {
    provider: buildProviderIdentity(config, dimensions),
    cachePath: config.cachePath,
    localModelPath: config.localModelPath,
  };
}

async function initializeHuggingFaceExtractor(
  config: EmbeddingConfig,
  options: { allowRemoteModels: boolean; progressLogger?: (message: string) => void },
): Promise<{ extractor: FeatureExtractor; dimensions: number }> {
  options.progressLogger?.(`Ensuring embedding cache directories exist under ${config.cachePath}.`);
  await mkdir(config.cachePath, { recursive: true });
  if (config.localModelPath) {
    await mkdir(config.localModelPath, { recursive: true });
  }

  options.progressLogger?.("Loading @huggingface/transformers.");
  const transformers = await import("@huggingface/transformers") as unknown as TransformersModule;
  transformers.env.allowLocalModels = true;
  transformers.env.allowRemoteModels = options.allowRemoteModels;
  transformers.env.cacheDir = config.cachePath;
  if (config.localModelPath) {
    transformers.env.localModelPath = config.localModelPath;
  }

  options.progressLogger?.(
    `Loading embedding model ${config.modelId}${config.modelRevision ? `@${config.modelRevision}` : ""}.`,
  );
  const extractor = await transformers.pipeline("feature-extraction", config.modelId, {
    revision: config.modelRevision ?? undefined,
  });
  options.progressLogger?.("Running embedding warmup query.");
  const warmup = await extractor("pf2e semantic search warmup", {
    pooling: "mean",
    normalize: true,
  });

  return {
    extractor,
    dimensions: Number(warmup.dims?.at(-1) ?? warmup.data.length),
  };
}

function buildProviderIdentity(config: EmbeddingConfig, dimensions: number): EmbeddingProviderIdentity {
  return {
    provider: "hf-local",
    model: config.modelId,
    revision: config.modelRevision,
    dimensions,
  };
}

function buildRuntimeEmbeddingErrorMessage(config: EmbeddingConfig, error: unknown): string {
  const reason = error instanceof Error ? error.message : String(error);
  return [
    `Failed to initialize local embedding model ${config.modelId}.`,
    "Normal MCP startup is offline-only and will not download model assets.",
    `Run 'npm run refresh-embeddings' or 'npm run refresh-external' first, then retry startup.`,
    `Underlying error: ${reason}`,
  ].join(" ");
}

function hashText(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }

  return hash >>> 0;
}

function normalizeVector(vector: Float32Array): Float32Array {
  let magnitude = 0;
  for (const value of vector) {
    magnitude += value * value;
  }

  if (magnitude === 0) {
    return vector;
  }

  const scale = 1 / Math.sqrt(magnitude);
  for (let index = 0; index < vector.length; index += 1) {
    vector[index] = (vector[index] ?? 0) * scale;
  }

  return vector;
}
