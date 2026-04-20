import { Pf2eDataService } from "../data/service.js";
import { RankingConfigStore } from "../search/ranking-config.js";
import type { AppConfig } from "../domain/index.js";
import { loadConfig } from "./config.js";

export type Pf2eApplicationRuntime = {
  config: AppConfig;
  dataService: Pf2eDataService;
  startupWarnings: string[];
  stats: { packCount: number; recordCount: number };
  close: () => void;
};

export async function loadPf2eApplicationRuntime(
  argv: string[] = process.argv.slice(2),
): Promise<Pf2eApplicationRuntime> {
  const config = await loadConfig(argv);
  const rankingConfigStore = await RankingConfigStore.create(config.ranking.configPath);
  let dataService: Pf2eDataService | null = null;

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

  return {
    config,
    dataService,
    startupWarnings: dataService.warnings,
    stats: dataService.getStats(),
    close: () => {
      dataService.close();
    },
  };
}
