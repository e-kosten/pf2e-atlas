import { createPf2eApplicationOntologyService, type Pf2eApplicationOntologyService } from "../app/ontology-service.js";
import { loadPf2eApplicationRuntime, type Pf2eApplicationRuntime } from "../app/runtime.js";
import { createPf2eApplicationStorageService, type Pf2eApplicationStorageService } from "../app/storage-service.js";
import { Pf2eDataService } from "../data/service.js";
import type { AppConfig } from "../domain/config-types.js";
import { createPf2eTerminalSearchService, type Pf2eTerminalSearchService } from "./search/service.js";
import {
  createDerivedTagMigrationWorkbenchSession,
  getDerivedTagMigrationWorkbenchQueueItems,
  promptAndCreateDerivedTagMigrationWorkbenchSession,
  type DerivedTagMigrationWorkbenchServices,
  type DerivedTagMigrationWorkbenchSessionCreationOptions,
} from "../tags/editorial/workbench-controller.js";
import type { DerivedTagMigrationWorkbenchSessionPrompts } from "../tags/editorial/workbench-session-prompts.js";
import { buildDerivedTagMigrationSession } from "../tags/editorial/session-builder.js";
import { summarizeCurrentDerivedTagReviewQueue } from "../tags/editorial/runtime-state.js";
import { writeDerivedTagMigrationSession } from "../tags/editorial/session-store.js";
import { writeDerivedTagMigrationSummary } from "../tags/editorial/cli-utils.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
  DerivedTagReviewQueueSummaryItem,
} from "../tags/editorial/types.js";

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
    prompts: DerivedTagMigrationWorkbenchSessionPrompts,
  ) => Promise<DerivedTagMigrationSession | undefined>;
};

export type Pf2eTerminalCatalogService = Pick<
  Pf2eDataService,
  | "closeSearchWindow"
  | "countRecords"
  | "getRecord"
  | "getSearchCategorySummary"
  | "getSearchVocabulary"
  | "listFilterValues"
  | "listRecords"
  | "lookup"
  | "openSearchWindow"
  | "readSearchWindowPage"
  | "search"
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

function createConfiguredWorkbenchServices(
  storage: Pick<Pf2eApplicationStorageService, "openIndex">,
): DerivedTagMigrationWorkbenchServices {
  return {
    buildSession: buildDerivedTagMigrationSession,
    openIndex: storage.openIndex,
    summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
    writeSession: writeDerivedTagMigrationSession,
    writeSummary: writeDerivedTagMigrationSummary,
  };
}

function createTagWorkbenchService(
  storage: Pick<Pf2eApplicationStorageService, "openIndex">,
): Pf2eTerminalTagWorkbenchService {
  const services = createConfiguredWorkbenchServices(storage);

  return {
    createSession: (rootPath, mode, options) =>
      createDerivedTagMigrationWorkbenchSession(rootPath, [], mode, options, services),
    getQueueItems: () => getDerivedTagMigrationWorkbenchQueueItems(services),
    promptAndCreateSession: (rootPath, mode, prompts) =>
      promptAndCreateDerivedTagMigrationWorkbenchSession(rootPath, [], mode, prompts, services),
  };
}

export async function loadPf2eTerminalAppServices(argv: string[]): Promise<Pf2eTerminalAppServices> {
  const runtime = await loadPf2eApplicationRuntime(argv);
  return createPf2eTerminalAppServices(runtime);
}

export function createPf2eTerminalAppServices(
  runtime: Pick<Pf2eApplicationRuntime, "config" | "dataService" | "close">,
): Pf2eTerminalAppServices {
  const { config, dataService } = runtime;
  const storage = createPf2eApplicationStorageService(config);
  return {
    config,
    catalog: dataService,
    user: {
      ontology: createPf2eApplicationOntologyService(config, dataService, storage),
      search: createPf2eTerminalSearchService({
        closeSearchWindow: (windowId) => dataService.closeSearchWindow(windowId),
        countRecords: (filters, options) => dataService.countRecords(filters, options),
        getSearchCategorySummary: () => dataService.getSearchCategorySummary(),
        getSearchVocabulary: () => dataService.getSearchVocabulary(),
        listFilterValues: (query) => dataService.listFilterValues(query),
        lookup: (name, options) => dataService.lookup(name, options),
        listRecords: (filters) => dataService.listRecords(filters),
        openSearchWindow: (filters, options) => dataService.openSearchWindow(filters, options),
        readSearchWindowPage: (windowId, offset, limit) => dataService.readSearchWindowPage(windowId, offset, limit),
        search: (filters) => dataService.search(filters),
      }),
    },
    dev: {
      tagRefinement: createTagWorkbenchService(storage),
    },
    close: () => {
      runtime.close();
    },
  };
}
