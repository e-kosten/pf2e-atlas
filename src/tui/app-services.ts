import { DatabaseSync } from "node:sqlite";

import {
  createPf2eApplicationOntologyService,
  type Pf2eApplicationOntologyService,
} from "../app/ontology-service.js";
import { loadPf2eApplicationRuntime, type Pf2eApplicationRuntime } from "../app/runtime.js";
import { Pf2eDataService } from "../data/service.js";
import type { AppConfig } from "../types.js";
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

export type Pf2eTerminalUserServices = {
  ontology: Pf2eApplicationOntologyService;
  search: Pf2eTerminalSearchService;
};

export type Pf2eTerminalDevelopmentServices = {
  tagRefinement: Pf2eTerminalTagWorkbenchService;
};

export type Pf2eTerminalAppServices = {
  config: AppConfig;
  catalog: Pf2eTerminalCatalogService;
  user: Pf2eTerminalUserServices;
  dev: Pf2eTerminalDevelopmentServices;
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
    getQueueItems: () => getDerivedTagMigrationWorkbenchQueueItems(services),
    promptAndCreateSession: (rootPath, mode, terminal) =>
      promptAndCreateDerivedTagMigrationWorkbenchSession(rootPath, [], mode, terminal, services),
  };
}

export async function loadPf2eTerminalAppServices(
  argv: string[],
): Promise<Pf2eTerminalAppServices> {
  const runtime = await loadPf2eApplicationRuntime(argv);
  return createPf2eTerminalAppServices(runtime);
}

export function createPf2eTerminalAppServices(
  runtime: Pick<Pf2eApplicationRuntime, "config" | "dataService" | "close">,
): Pf2eTerminalAppServices {
  const { config, dataService } = runtime;
  return {
    config,
    catalog: dataService,
    user: {
      ontology: createPf2eApplicationOntologyService(config, dataService),
      search: createPf2eTerminalSearchService({
        getSearchVocabulary: () => dataService.getSearchVocabulary(),
        lookup: (name, options) => dataService.lookup(name, options),
        search: (filters) => dataService.search(filters),
      }),
    },
    dev: {
      tagRefinement: createTagWorkbenchService(config),
    },
    close: () => {
      runtime.close();
    },
  };
}
