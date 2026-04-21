import type { DerivedTagReviewDecision, DerivedTagReviewSession } from "../types.js";

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

export function getDerivedTagReviewItems(
  session: DerivedTagReviewSession,
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

export function summarizeDerivedTagReviewProgress(
  session: DerivedTagReviewSession,
): DerivedTagMigrationReviewProgress {
  const actionableDecisions = session.decisions.filter((recordDecision) => recordDecision.decisions.length > 0);

  return {
    candidateRecordCount: session.manifest.recordCount,
    actionableRecordCount: actionableDecisions.length,
    resolvedActionableRecordCount: actionableDecisions.filter(
      (recordDecision) => recordDecision.resolutionStatus === "complete",
    ).length,
    visibleItemCount: getDerivedTagReviewItems(session).length,
  };
}

export function updateDerivedTagReviewDecisionStatus(
  session: DerivedTagReviewSession,
  item: DerivedTagMigrationReviewItem,
  status: DerivedTagReviewDecision["status"],
): DerivedTagReviewSession {
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

export function toggleDerivedTagReviewUnresolvedOnly(
  session: DerivedTagReviewSession,
): DerivedTagReviewSession {
  const next = structuredClone(session);
  next.reviewState.unresolvedOnly = !next.reviewState.unresolvedOnly;
  next.reviewState.updatedAt = new Date().toISOString();
  next.reviewState.currentIndex = 0;
  return next;
}

export function clampDerivedTagReviewIndex(session: DerivedTagReviewSession): DerivedTagReviewSession {
  const next = structuredClone(session);
  const items = getDerivedTagReviewItems(next);
  if (items.length === 0) {
    next.reviewState.currentIndex = 0;
    return next;
  }
  next.reviewState.currentIndex = Math.max(0, Math.min(next.reviewState.currentIndex, items.length - 1));
  return next;
}
