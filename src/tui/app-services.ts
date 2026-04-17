import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../app/config.js";
import { Pf2eDataService } from "../data/service.js";
import { RankingConfigStore } from "../search/ranking-config.js";
import type { AppConfig } from "../types.js";
import { buildDerivedTagOntologyExplorerModel, type DerivedTagOntologyExplorerModel } from "./ontology-explorer/data.js";
import { createPf2eTerminalSearchService, type Pf2eTerminalSearchService } from "./search-service.js";
import {
  createDerivedTagMigrationWorkbenchSession,
  getDerivedTagMigrationWorkbenchQueueItems,
  promptAndCreateDerivedTagMigrationWorkbenchSession,
  type DerivedTagMigrationWorkbenchServices,
  type DerivedTagMigrationWorkbenchSessionCreationOptions,
} from "../tags/migration/workbench-controller.js";
import {
  buildDerivedTagMigrationSession,
} from "../tags/migration/session-builder.js";
import {
  summarizeCurrentDerivedTagReviewQueue,
} from "../tags/migration/runtime-state.js";
import { writeDerivedTagMigrationSession } from "../tags/migration/session-store.js";
import { writeDerivedTagMigrationSummary } from "../tags/migration/cli-utils.js";
import type { DerivedTagTerminalApp } from "./terminal-ui.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
  DerivedTagReviewQueueSummaryItem,
} from "../tags/migration/types.js";

type SessionOptions = Omit<DerivedTagMigrationWorkbenchSessionCreationOptions, "decisionKind"> & {
  decisionKind?: DerivedTagMigrationReviewDecisionKind;
};

export type Pf2eTerminalTagWorkbenchService = {
  createSession: (
    rootPath: string,
    mode: DerivedTagMigrationMode,
    options: SessionOptions,
  ) => Promise<DerivedTagMigrationSession>;
  getOntologyModel: () => DerivedTagOntologyExplorerModel;
  getQueueItems: () => DerivedTagReviewQueueSummaryItem[];
  promptAndCreateSession: (
    rootPath: string,
    mode: DerivedTagMigrationMode,
    terminal: DerivedTagTerminalApp,
  ) => Promise<DerivedTagMigrationSession | undefined>;
};

export type Pf2eTerminalCatalogService = Pick<
  Pf2eDataService,
  "getRecord" | "getSearchVocabulary" | "listFilterValues" | "lookup" | "search"
>;

export type Pf2eTerminalAppServices = {
  config: AppConfig;
  dataService: Pf2eTerminalCatalogService;
  search: Pf2eTerminalSearchService;
  tagWorkbench: Pf2eTerminalTagWorkbenchService;
  close: () => void;
};

function createConfiguredWorkbenchServices(config: AppConfig): DerivedTagMigrationWorkbenchServices {
  const openIndex = async (_argv: string[]): Promise<{ db: DatabaseSync; config: AppConfig }> => ({
    config,
    db: new DatabaseSync(config.indexPath),
  });

  return {
    buildSession: buildDerivedTagMigrationSession,
    openIndex,
    summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
    writeSession: writeDerivedTagMigrationSession,
    writeSummary: writeDerivedTagMigrationSummary,
  };
}

function createTagWorkbenchService(config: AppConfig): Pf2eTerminalTagWorkbenchService {
  const services = createConfiguredWorkbenchServices(config);

  return {
    createSession: (rootPath, mode, options) =>
      createDerivedTagMigrationWorkbenchSession(rootPath, [], mode, options, services),
    getOntologyModel: () => {
      const db = new DatabaseSync(config.indexPath);
      try {
        return buildDerivedTagOntologyExplorerModel(db, { cacheKey: config.indexPath });
      } finally {
        db.close();
      }
    },
    getQueueItems: () => getDerivedTagMigrationWorkbenchQueueItems(services),
    promptAndCreateSession: (rootPath, mode, terminal) =>
      promptAndCreateDerivedTagMigrationWorkbenchSession(rootPath, [], mode, terminal, services),
  };
}

export async function loadPf2eTerminalAppServices(
  argv: string[],
): Promise<Pf2eTerminalAppServices> {
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
    search: createPf2eTerminalSearchService({
      getSearchVocabulary: () => dataService.getSearchVocabulary(),
      lookup: (name, options) => dataService.lookup(name, options),
      search: (filters) => dataService.search(filters),
    }),
    tagWorkbench: createTagWorkbenchService(config),
    close: () => {
      dataService?.close();
    },
  };
}
