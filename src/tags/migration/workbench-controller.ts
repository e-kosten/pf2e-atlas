import { DatabaseSync } from "node:sqlite";

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { openConfiguredIndex, writeDerivedTagMigrationSummary } from "./cli-utils.js";
import { renderDerivedTagMigrationSessionSummary } from "./render.js";
import {
  summarizeCurrentDerivedTagReviewQueue,
} from "./runtime-state.js";
import { buildDerivedTagMigrationSession } from "./session-builder.js";
import { writeDerivedTagMigrationSession } from "./session-store.js";
import type { DerivedTagTerminalApp } from "../../tui/terminal-ui.js";
import type {
  DerivedTagMigrationMode,
  DerivedTagMigrationReviewDecisionKind,
  DerivedTagMigrationSession,
  DerivedTagReviewQueueSummaryItem,
} from "./types.js";
import { promptDerivedTagMigrationWorkbenchSessionOptions, type DerivedTagMigrationWorkbenchSessionOptions } from "./workbench-session-prompts.js";

export type DerivedTagMigrationWorkbenchServices = {
  buildSession: typeof buildDerivedTagMigrationSession;
  openIndex: typeof openConfiguredIndex;
  summarizeQueue: typeof summarizeCurrentDerivedTagReviewQueue;
  writeSession: typeof writeDerivedTagMigrationSession;
  writeSummary: typeof writeDerivedTagMigrationSummary;
};

export const DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES: DerivedTagMigrationWorkbenchServices = {
  buildSession: buildDerivedTagMigrationSession,
  openIndex: openConfiguredIndex,
  summarizeQueue: summarizeCurrentDerivedTagReviewQueue,
  writeSession: writeDerivedTagMigrationSession,
  writeSummary: writeDerivedTagMigrationSummary,
};

export type DerivedTagMigrationWorkbenchSessionCreationOptions = DerivedTagMigrationWorkbenchSessionOptions & {
  decisionKind?: DerivedTagMigrationReviewDecisionKind;
};

export type DerivedTagMigrationWorkbenchOntologyHandle = {
  cacheKey?: string;
  db: DatabaseSync;
};

async function writeDerivedTagMigrationSessionArtifacts(
  rootPath: string,
  session: DerivedTagMigrationSession,
  services: DerivedTagMigrationWorkbenchServices,
): Promise<void> {
  await services.writeSession(rootPath, session);
  await services.writeSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
}

export function getDerivedTagMigrationWorkbenchQueueItems(
  services: DerivedTagMigrationWorkbenchServices = DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
): DerivedTagReviewQueueSummaryItem[] {
  return services.summarizeQueue();
}

export async function createDerivedTagMigrationWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagMigrationMode,
  options: DerivedTagMigrationWorkbenchSessionCreationOptions,
  services: DerivedTagMigrationWorkbenchServices = DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
): Promise<DerivedTagMigrationSession> {
  const { db } = await services.openIndex(argv);

  try {
    const session = services.buildSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagMigrationSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    db.close();
  }
}

export async function promptAndCreateDerivedTagMigrationWorkbenchSession(
  rootPath: string,
  argv: string[],
  mode: DerivedTagMigrationMode,
  terminal: DerivedTagTerminalApp,
  services: DerivedTagMigrationWorkbenchServices = DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
): Promise<DerivedTagMigrationSession | undefined> {
  const { db } = await services.openIndex(argv);

  try {
    const options = await promptDerivedTagMigrationWorkbenchSessionOptions(terminal, db, mode);
    if (!options) {
      return undefined;
    }

    const session = services.buildSession(db, {
      mode,
      ...options,
    });
    await writeDerivedTagMigrationSessionArtifacts(rootPath, session, services);
    return session;
  } finally {
    db.close();
  }
}

export async function openDerivedTagMigrationWorkbenchOntology(
  argv: string[],
  services: DerivedTagMigrationWorkbenchServices = DEFAULT_DERIVED_TAG_MIGRATION_WORKBENCH_SERVICES,
): Promise<DerivedTagMigrationWorkbenchOntologyHandle> {
  const { db, config } = await services.openIndex(argv);
  return {
    cacheKey: config.indexPath,
    db,
  };
}
