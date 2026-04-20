import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import type { RankingConfigStore } from "../../search/ranking-config.js";
import type { EmbeddingConfig, PackInfo } from "../../types.js";

export type Pf2eDataServiceLoadOptions = {
  indexPath?: string;
  embedding?: EmbeddingConfig;
  embeddingProviderFactory?: (
    config: EmbeddingConfig,
  ) => Promise<{ provider: EmbeddingProvider; warnings: string[] }>;
  rankingConfigStore?: RankingConfigStore;
  progressLogger?: (message: string) => void;
  progressStatusLogger?: (message: string) => void;
  reuseEmbeddings?: boolean;
  vectorExtensionLoader?: (db: DatabaseSync) => void;
};

export type Pf2eLoadedDataRuntime = {
  db: DatabaseSync;
  packs: PackInfo[];
  warnings: string[];
  recordCount: number;
  embeddingProvider: EmbeddingProvider;
  rankingConfigStore: RankingConfigStore | null;
};
