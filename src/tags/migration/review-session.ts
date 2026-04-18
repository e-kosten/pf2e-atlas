import type { DerivedTagMigrationDecision, DerivedTagMigrationSession } from "./types.js";

export type DerivedTagMigrationReviewItem = {
  recordIndex: number;
  decisionIndex: number;
};

export type DerivedTagMigrationReviewProgress = {
  candidateRecordCount: number;
  actionableRecordCount: number;
  resolvedActionableRecordCount: number;
  visibleItemCount: number;
};

export function getDerivedTagMigrationReviewItems(
  session: DerivedTagMigrationSession,
): DerivedTagMigrationReviewItem[] {
  const items: DerivedTagMigrationReviewItem[] = [];
  session.decisions.forEach((recordDecision, recordIndex) => {
    recordDecision.decisions.forEach((decision, decisionIndex) => {
      if (session.reviewState.unresolvedOnly && decision.status !== "needs_review") {
        return;
      }
      items.push({ recordIndex, decisionIndex });
    });
  });
  return items;
}

export function summarizeDerivedTagMigrationReviewProgress(
  session: DerivedTagMigrationSession,
): DerivedTagMigrationReviewProgress {
  const actionableDecisions = session.decisions.filter((recordDecision) => recordDecision.decisions.length > 0);

  return {
    candidateRecordCount: session.manifest.recordCount,
    actionableRecordCount: actionableDecisions.length,
    resolvedActionableRecordCount: actionableDecisions.filter(
      (recordDecision) => recordDecision.resolutionStatus === "complete",
    ).length,
    visibleItemCount: getDerivedTagMigrationReviewItems(session).length,
  };
}

export function updateDerivedTagMigrationDecisionStatus(
  session: DerivedTagMigrationSession,
  item: DerivedTagMigrationReviewItem,
  status: DerivedTagMigrationDecision["status"],
): DerivedTagMigrationSession {
  const next = structuredClone(session);
  const recordDecision = next.decisions[item.recordIndex];
  if (!recordDecision) {
    return next;
  }
  const decision = recordDecision.decisions[item.decisionIndex];
  if (!decision) {
    return next;
  }
  decision.status = status;
  recordDecision.resolutionStatus = recordDecision.decisions.some((entry) => entry.status === "needs_review")
    ? "needs_review"
    : "complete";
  next.reviewState.updatedAt = new Date().toISOString();
  return next;
}

export function toggleDerivedTagMigrationUnresolvedOnly(
  session: DerivedTagMigrationSession,
): DerivedTagMigrationSession {
  const next = structuredClone(session);
  next.reviewState.unresolvedOnly = !next.reviewState.unresolvedOnly;
  next.reviewState.updatedAt = new Date().toISOString();
  next.reviewState.currentIndex = 0;
  return next;
}

export function clampDerivedTagMigrationReviewIndex(session: DerivedTagMigrationSession): DerivedTagMigrationSession {
  const next = structuredClone(session);
  const items = getDerivedTagMigrationReviewItems(next);
  if (items.length === 0) {
    next.reviewState.currentIndex = 0;
    return next;
  }
  next.reviewState.currentIndex = Math.max(0, Math.min(next.reviewState.currentIndex, items.length - 1));
  return next;
}
