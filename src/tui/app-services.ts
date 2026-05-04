import { createPf2eApplicationPageRelationsService, type Pf2eApplicationPageRelationsService } from "../app/page-relations-service.js";
import { createPf2eApplicationEntityPageService, type Pf2eApplicationEntityPageService } from "../app/ontology/entity-page-service.js";
import { createPf2eApplicationOntologyService, type Pf2eApplicationOntologyService } from "../app/ontology-service.js";
import { createPf2eApplicationSearchDiscoveryService } from "../app/search-discovery-service.js";
import { createPf2eApplicationStorageService, type Pf2eApplicationStorageService } from "../app/storage-service.js";
import { loadPf2eApplicationRuntime, type Pf2eApplicationRuntime } from "../app/runtime.js";
import type { AppConfig } from "../domain/config-types.js";
import {
  buildDerivedTagReviewSession,
  createDerivedTagWorkbenchSession,
  getDerivedTagWorkbenchQueueItems,
  summarizeCurrentDerivedTagReviewQueue,
  writeDerivedTagReviewSession,
  writeDerivedTagReviewSummary,
  type DerivedTagWorkbenchMode,
  type DerivedTagReviewDecisionKind,
  type DerivedTagReviewSession,
  type DerivedTagWorkbenchServices,
  type DerivedTagWorkbenchSessionCreationOptions,
  type DerivedTagReviewQueueSummaryItem,
} from "../tags/editorial.js";
import {
  promptAndCreateDerivedTagWorkbenchSession,
  type DerivedTagWorkbenchSessionPrompts,
} from "../tags/editorial-ui.js";
import { createPf2eTerminalSearchService, type Pf2eTerminalSearchService } from "./search/service.js";

type SessionOptions = Omit<DerivedTagWorkbenchSessionCreationOptions, "decisionKind"> & {
  decisionKind?: DerivedTagReviewDecisionKind;
};

export type Pf2eTerminalTagWorkbenchService = {
  createSession: (
    rootPath: string,
    mode: DerivedTagWorkbenchMode,
    options: SessionOptions,
  ) => Promise<DerivedTagReviewSession>;
  getQueueItems: () => DerivedTagReviewQueueSummaryItem[];
  promptAndCreateSession: (
    rootPath: string,
    mode: DerivedTagWorkbenchMode,
    prompts: DerivedTagWorkbenchSessionPrompts,
  ) => Promise<DerivedTagReviewSession | undefined>;
};

export type Pf2eTerminalUserServices = {
  entityPages: Pf2eApplicationEntityPageService;
  ontology: Pf2eApplicationOntologyService;
  pageRelations: Pf2eApplicationPageRelationsService;
  search: Pf2eTerminalSearchService;
};

export type Pf2eTerminalDevelopmentServices = {
  tagRefinement: Pf2eTerminalTagWorkbenchService;
};

export type Pf2eTerminalAppServices = {
  config: AppConfig;
  user: Pf2eTerminalUserServices;
  dev: Pf2eTerminalDevelopmentServices;
  close: () => void;
};

function createConfiguredWorkbenchServices(
  storage: Pick<Pf2eApplicationStorageService, "openIndex">,
): DerivedTagWorkbenchServices {
  return {
    buildSession: buildDerivedTagReviewSession,
    openIndex: () => storage.openIndex(),
    summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
    writeSession: writeDerivedTagReviewSession,
    writeSummary: writeDerivedTagReviewSummary,
  };
}

function createTagWorkbenchService(
  storage: Pick<Pf2eApplicationStorageService, "openIndex">,
): Pf2eTerminalTagWorkbenchService {
  const services = createConfiguredWorkbenchServices(storage);

  return {
    createSession: (rootPath, mode, options) =>
      createDerivedTagWorkbenchSession(rootPath, [], mode, options, services),
    getQueueItems: () => getDerivedTagWorkbenchQueueItems(services),
    promptAndCreateSession: (rootPath, mode, prompts) =>
      promptAndCreateDerivedTagWorkbenchSession(rootPath, [], mode, prompts, services),
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
  const discovery = createPf2eApplicationSearchDiscoveryService(dataService);
  const pageRelations = createPf2eApplicationPageRelationsService(dataService);
  return {
    config,
    user: {
      entityPages: createPf2eApplicationEntityPageService(pageRelations),
      ontology: createPf2eApplicationOntologyService(config, dataService, discovery),
      pageRelations,
      search: createPf2eTerminalSearchService({
        closeSearchWindow: (windowId) => dataService.closeSearchWindow(windowId),
        countRecords: (request, options) => dataService.countRecords(request, options),
        discovery,
        getPack: (packValue) => dataService.getPack(packValue),
        getSearchCategorySummary: () => dataService.getSearchCategorySummary(),
        getSearchVocabulary: () => dataService.getSearchVocabulary(),
        lookup: (name, options) => dataService.lookup(name, options),
        listRecords: (request) => dataService.listRecords(request),
        openSearchWindow: (request) => dataService.openSearchWindow(request),
        readSearchWindowPage: (windowId, offset, limit) => dataService.readSearchWindowPage(windowId, offset, limit),
        search: (request) => dataService.search(request),
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
