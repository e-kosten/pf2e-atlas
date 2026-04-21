import { DatabaseSync } from "node:sqlite";

import { openEditorialConfiguredIndex } from "../configured-index.js";
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
  openIndex: typeof openEditorialConfiguredIndex;
  summarizeQueue: typeof summarizeCurrentDerivedTagReviewQueue;
  writeSession: typeof writeDerivedTagReviewSession;
  writeSummary: typeof writeDerivedTagReviewSummary;
};

export const DEFAULT_DERIVED_TAG_WORKBENCH_SERVICES: DerivedTagWorkbenchServices = {
  buildSession: buildDerivedTagReviewSession,
  openIndex: openEditorialConfiguredIndex,
  summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
  writeSession: writeDerivedTagReviewSession,
  writeSummary: writeDerivedTagReviewSummary,
};

export type DerivedTagWorkbenchSessionCreationOptions = DerivedTagWorkbenchSessionOptions & {
  decisionKind?: DerivedTagReviewDecisionKind;
};

export type DerivedTagWorkbenchOntologyHandle = {
  cacheKey?: string;
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
  services: DerivedTagWorkbenchServices = DEFAULT_DERIVED_TAG_WORKBENCH_SERVICES,
): DerivedTagReviewQueueSummaryItem[] {
  return services.summarizeQueue();
}

export async function createDerivedTagWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagWorkbenchMode,
  options: DerivedTagWorkbenchSessionCreationOptions,
  services: DerivedTagWorkbenchServices = DEFAULT_DERIVED_TAG_WORKBENCH_SERVICES,
): Promise<DerivedTagReviewSession> {
  const { db } = await services.openIndex(argv);

  try {
    const session = services.buildSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagReviewSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    db.close();
  }
}

export async function promptAndCreateDerivedTagWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagWorkbenchMode,
  prompts: DerivedTagWorkbenchSessionPrompts,
  services: DerivedTagWorkbenchServices = DEFAULT_DERIVED_TAG_WORKBENCH_SERVICES,
): Promise<DerivedTagReviewSession | undefined> {
  const { db } = await services.openIndex(argv);

  try {
    const options = await promptDerivedTagWorkbenchSessionOptions(prompts, db, mode);
    if (!options) {
      return undefined;
    }

    const session = services.buildSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagReviewSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    db.close();
  }
}

export async function openDerivedTagWorkbenchOntology(
  argv: string[],
  services: DerivedTagWorkbenchServices = DEFAULT_DERIVED_TAG_WORKBENCH_SERVICES,
): Promise<DerivedTagWorkbenchOntologyHandle> {
  const { db, config } = await services.openIndex(argv);
  return {
    cacheKey: config.indexPath,
    db,
  };
}
