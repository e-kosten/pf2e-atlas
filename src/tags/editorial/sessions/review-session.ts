import type { DerivedTagReviewDecision, DerivedTagReviewSession } from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";

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
      if (session.reviewState.unresolvedOnly && decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW) {
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
      (recordDecision) => recordDecision.resolutionStatus === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUS.COMPLETE,
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
  recordDecision.resolutionStatus = recordDecision.decisions.some((entry) => entry.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW)
    ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW
    : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUS.COMPLETE;
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
