import { writeDerivedTagMigrationSummary } from "../cli-utils.js";
import { importDerivedTagMigrationSession } from "../writeback/importer.js";
import { lintDerivedTagMigrationSession } from "../writeback/linter.js";
import { summarizeDerivedTagMigrationReviewProgress } from "../sessions/review-session.js";
import { writeDerivedTagMigrationSession } from "../sessions/session-store.js";
import type { DerivedTagMigrationSession } from "../types.js";

export type DerivedTagMigrationReviewServices = {
  importSession: typeof importDerivedTagMigrationSession;
  lintSession: typeof lintDerivedTagMigrationSession;
  writeSession: typeof writeDerivedTagMigrationSession;
  writeSummary: typeof writeDerivedTagMigrationSummary;
};

export const DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES: DerivedTagMigrationReviewServices = {
  importSession: importDerivedTagMigrationSession,
  lintSession: lintDerivedTagMigrationSession,
  writeSession: writeDerivedTagMigrationSession,
  writeSummary: writeDerivedTagMigrationSummary,
};

export function renderDerivedTagMigrationReviewSummary(session: DerivedTagMigrationSession): string {
  const progress = summarizeDerivedTagMigrationReviewProgress(session);
  const actionableSummary =
    progress.actionableRecordCount > 0
      ? `Actionable records resolved: ${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount}`
      : "Actionable review items: 0";

  return [
    `Session: ${session.manifest.id}`,
    `Mode: ${session.manifest.mode}`,
    `Candidate records: ${progress.candidateRecordCount}`,
    actionableSummary,
    `Visible review items: ${progress.visibleItemCount}`,
    `Updated at: ${session.reviewState.updatedAt}`,
  ].join("\n");
}

export async function persistDerivedTagMigrationReviewSession(
  rootPath: string,
  session: DerivedTagMigrationSession,
  services: DerivedTagMigrationReviewServices = DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
): Promise<void> {
  await Promise.all([
    services.writeSession(rootPath, session),
    services.writeSummary(rootPath, session.manifest.id, renderDerivedTagMigrationReviewSummary(session)),
  ]);
}

export async function importDerivedTagMigrationReviewSession(
  rootPath: string,
  session: DerivedTagMigrationSession,
  services: DerivedTagMigrationReviewServices = DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
): Promise<void> {
  services.lintSession(session);
  await services.importSession(rootPath, session);
  await persistDerivedTagMigrationReviewSession(rootPath, session, services);
}
