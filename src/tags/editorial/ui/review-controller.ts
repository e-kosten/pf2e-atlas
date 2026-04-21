import { writeDerivedTagReviewSummary } from "../writeback/review-summary.js";
import { importDerivedTagReviewSession as importReviewedSession } from "../writeback/importer.js";
import { lintDerivedTagReviewSession } from "../writeback/linter.js";
import { summarizeDerivedTagReviewProgress } from "../sessions/review-session.js";
import { writeDerivedTagReviewSession } from "../sessions/session-store.js";
import type { DerivedTagReviewSession } from "../types.js";

export type DerivedTagReviewServices = {
  importSession: typeof importReviewedSession;
  lintSession: typeof lintDerivedTagReviewSession;
  writeSession: typeof writeDerivedTagReviewSession;
  writeSummary: typeof writeDerivedTagReviewSummary;
};

export const DEFAULT_DERIVED_TAG_REVIEW_SERVICES: DerivedTagReviewServices = {
  importSession: importReviewedSession,
  lintSession: lintDerivedTagReviewSession,
  writeSession: writeDerivedTagReviewSession,
  writeSummary: writeDerivedTagReviewSummary,
};

export function renderDerivedTagReviewSummary(session: DerivedTagReviewSession): string {
  const progress = summarizeDerivedTagReviewProgress(session);
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

export async function persistDerivedTagReviewSession(
  rootPath: string,
  session: DerivedTagReviewSession,
  services: DerivedTagReviewServices = DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
): Promise<void> {
  await Promise.all([
    services.writeSession(rootPath, session),
    services.writeSummary(rootPath, session.manifest.id, renderDerivedTagReviewSummary(session)),
  ]);
}

export async function importDerivedTagReviewSession(
  rootPath: string,
  session: DerivedTagReviewSession,
  services: DerivedTagReviewServices = DEFAULT_DERIVED_TAG_REVIEW_SERVICES,
): Promise<void> {
  services.lintSession(session);
  await services.importSession(rootPath, session);
  await persistDerivedTagReviewSession(rootPath, session, services);
}
