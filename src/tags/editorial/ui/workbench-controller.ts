import type { DatabaseSync } from "node:sqlite";

import {
  openConfiguredPf2eApplicationIndex,
  type Pf2eApplicationIndexHandle,
} from "../../../app/storage-service.js";
import { renderDerivedTagReviewSessionSummary } from "./render.js";
import { summarizeCurrentDerivedTagReviewQueue } from "../state/runtime-state.js";
import { buildDerivedTagReviewSession } from "../sessions/session-builder.js";
import { writeDerivedTagReviewSession } from "../sessions/session-store.js";
import { writeDerivedTagReviewSummary } from "../writeback/review-summary.js";
import type {
  DerivedTagWorkbenchMode,
  DerivedTagReviewDecisionKind,
  DerivedTagReviewSession,
  DerivedTagReviewQueueSummaryItem,
} from "../types.js";
import {
  promptDerivedTagWorkbenchSessionOptions,
  type DerivedTagWorkbenchSessionOptions,
  type DerivedTagWorkbenchSessionPrompts,
} from "./workbench-session-prompts.js";

export type DerivedTagWorkbenchServices = {
  buildSession: typeof buildDerivedTagReviewSession;
  openIndex: () => Promise<Pf2eApplicationIndexHandle>;
  summarizeQueue: typeof summarizeCurrentDerivedTagReviewQueue;
  writeSession: typeof writeDerivedTagReviewSession;
  writeSummary: typeof writeDerivedTagReviewSummary;
};

export type DerivedTagWorkbenchSessionCreationOptions = DerivedTagWorkbenchSessionOptions & {
  decisionKind?: DerivedTagReviewDecisionKind;
};

export type DerivedTagWorkbenchOntologyHandle = {
  cacheKey?: string;
  close: () => void;
  db: DatabaseSync;
};

async function writeDerivedTagReviewSessionArtifacts(
  rootPath: string,
  session: DerivedTagReviewSession,
  services: DerivedTagWorkbenchServices,
): Promise<void> {
  await services.writeSession(rootPath, session);
  await services.writeSummary(rootPath, session.manifest.id, renderDerivedTagReviewSessionSummary(session));
}

export function getDerivedTagWorkbenchQueueItems(
  services: Pick<DerivedTagWorkbenchServices, "summarizeQueue"> = {
    summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
  },
): DerivedTagReviewQueueSummaryItem[] {
  return services.summarizeQueue();
}

function createDefaultDerivedTagWorkbenchServices(argv: string[]): DerivedTagWorkbenchServices {
  return {
    buildSession: buildDerivedTagReviewSession,
    openIndex: () => openConfiguredPf2eApplicationIndex(argv),
    summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
    writeSession: writeDerivedTagReviewSession,
    writeSummary: writeDerivedTagReviewSummary,
  };
}

export async function createDerivedTagWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagWorkbenchMode,
  options: DerivedTagWorkbenchSessionCreationOptions,
  services: DerivedTagWorkbenchServices = createDefaultDerivedTagWorkbenchServices(argv),
): Promise<DerivedTagReviewSession> {
  const handle = await services.openIndex();

  try {
    const session = services.buildSession(handle.db, {
      mode,
      ...options,
    });
    await writeDerivedTagReviewSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    handle.close();
  }
}

export async function promptAndCreateDerivedTagWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagWorkbenchMode,
  prompts: DerivedTagWorkbenchSessionPrompts,
  services: DerivedTagWorkbenchServices = createDefaultDerivedTagWorkbenchServices(argv),
): Promise<DerivedTagReviewSession | undefined> {
  const handle = await services.openIndex();

  try {
    const options = await promptDerivedTagWorkbenchSessionOptions(prompts, handle.db, mode);
    if (!options) {
      return undefined;
    }

    const session = services.buildSession(handle.db, {
      mode,
      ...options,
    });
    await writeDerivedTagReviewSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    handle.close();
  }
}

export async function openDerivedTagWorkbenchOntology(
  argv: string[],
  services: DerivedTagWorkbenchServices = createDefaultDerivedTagWorkbenchServices(argv),
): Promise<DerivedTagWorkbenchOntologyHandle> {
  const handle = await services.openIndex();
  return {
    cacheKey: handle.config.indexPath,
    close: handle.close,
    db: handle.db,
  };
}
