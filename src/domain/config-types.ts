export interface AppConfig {
  dataPath: string;
  rootPath: string;
  manifestPath: string;
  indexPath: string;
  embeddings: EmbeddingConfig;
  ranking: RankingRuntimeConfig;
}

export interface RankingRuntimeConfig {
  configPath: string;
  watch: boolean;
}

export type EmbeddingProviderKind = "hash" | "hf-local";

export interface EmbeddingConfig {
  provider: EmbeddingProviderKind;
  modelId: string;
  modelRevision: string | null;
  cachePath: string;
  localModelPath: string | null;
}
