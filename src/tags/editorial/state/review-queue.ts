import type {
  DerivedTagExemplarReviewDecision,
  SearchCategory,
} from "../../../domain/derived-tag-types.js";
import {
  DerivedTagAssignmentReviewDecision,
  DerivedTagAssignmentReviewCategory,
} from "../../runtime/derivation/assignments.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../../manifest.js";
import { compareReviewQueueItems } from "../list-sorting.js";
import { getCurrentDerivedTagAuthoredState } from "./authored-state.js";
import type { DerivedTagReviewDecision, DerivedTagReviewQueueSummaryItem } from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";

export type PendingAssignmentReviewEntry = {
  category: SearchCategory;
  decision: DerivedTagAssignmentReviewDecision;
};

export type PendingExemplarReviewEntry = {
  category: SearchCategory;
  decision: DerivedTagExemplarReviewDecision;
};

function flattenAssignmentReviewDecisions(
  assignmentReviews: DerivedTagAssignmentReviewCategory[],
): Array<{ category: SearchCategory; decision: DerivedTagReviewDecision }> {
  return assignmentReviews.flatMap((categoryReview) =>
    categoryReview.decisions.map((reviewDecision) => ({
      category: categoryReview.category,
      decision: {
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
        family: reviewDecision.family,
        tag: reviewDecision.tag,
        mode: reviewDecision.mode,
        status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        confidence: reviewDecision.confidence,
        rationale: reviewDecision.rationale,
        source: reviewDecision.source,
      },
    })),
  );
}

function flattenExemplarReviewDecisions(
  exemplarReviews: DerivedTagExemplarReviewDecision[],
): DerivedTagReviewDecision[] {
  return exemplarReviews.map((reviewDecision) => ({
    kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
    tag: reviewDecision.tag,
    polarity:
      reviewDecision.proposedPolarity === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
        ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
        : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
    action:
      reviewDecision.proposedPolarity === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP
        ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP
        : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
    status: reviewDecision.status,
    confidence: reviewDecision.confidence,
    rationale: reviewDecision.rationale,
    source: reviewDecision.source,
    currentPolarity: reviewDecision.currentPolarity,
  }));
}

export function listCurrentPendingAssignmentReviews(): PendingAssignmentReviewEntry[] {
  const state = getCurrentDerivedTagAuthoredState();
  const pending: PendingAssignmentReviewEntry[] = [];

  for (const [category, assignmentReviewCategory] of Object.entries(state.assignmentReviews) as Array<
    [SearchCategory, { decisions: DerivedTagAssignmentReviewDecision[] }]
  >) {
    for (const decision of assignmentReviewCategory.decisions) {
      pending.push({ category, decision });
    }
  }

  return pending;
}

export function listCurrentPendingLlmAssignmentReviews(): PendingAssignmentReviewEntry[] {
  return listCurrentPendingAssignmentReviews().filter((entry) => entry.decision.source === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM);
}

export function listCurrentPendingExemplarReviews(): PendingExemplarReviewEntry[] {
  const state = getCurrentDerivedTagAuthoredState();
  const pending: PendingExemplarReviewEntry[] = [];

  for (const [category, exemplarReviewCategory] of Object.entries(state.exemplarReviews) as Array<
    [SearchCategory, { decisions: DerivedTagExemplarReviewDecision[] }]
  >) {
    for (const decision of exemplarReviewCategory.decisions) {
      if (decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW) {
        continue;
      }
      pending.push({ category, decision });
    }
  }

  return pending;
}

export function listCurrentPendingLlmExemplarReviews(): PendingExemplarReviewEntry[] {
  return listCurrentPendingExemplarReviews().filter((entry) => entry.decision.source === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM);
}

export function summarizeCurrentDerivedTagReviewQueue(): DerivedTagReviewQueueSummaryItem[] {
  const state = getCurrentDerivedTagAuthoredState();
  const counts = new Map<string, DerivedTagReviewQueueSummaryItem>();
  const confidencesByKey = new Map<string, Set<DerivedTagReviewQueueSummaryItem["confidence"]>>();

  for (const { category, decision } of flattenAssignmentReviewDecisions(
    DERIVED_TAG_MANAGED_CATEGORIES.map((managedCategory) => state.assignmentReviews[managedCategory]),
  )) {
    if (decision.kind !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT) {
      continue;
    }
    const confidence = decision.confidence ?? "unspecified";
    const key = [
      DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
      category,
      decision.family,
      decision.tag,
    ].join("|");
    const current = counts.get(key) ?? {
      kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
      category,
      family: decision.family,
      tag: decision.tag,
      count: 0,
      confidence,
    };
    current.count += 1;
    counts.set(key, current);
    const confidenceBucket = confidencesByKey.get(key) ?? new Set<DerivedTagReviewQueueSummaryItem["confidence"]>();
    confidenceBucket.add(confidence);
    confidencesByKey.set(key, confidenceBucket);
  }

  for (const [category, exemplarReviewCategory] of Object.entries(state.exemplarReviews) as Array<
    [SearchCategory, { decisions: DerivedTagExemplarReviewDecision[] }]
  >) {
    for (const decision of flattenExemplarReviewDecisions(exemplarReviewCategory.decisions)) {
      if (decision.kind !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR || decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW) {
        continue;
      }
      const confidence = decision.confidence ?? "unspecified";
      const key = [
        DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
        category,
        decision.tag,
      ].join("|");
      const current = counts.get(key) ?? {
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
        category,
        tag: decision.tag,
        count: 0,
        confidence,
      };
      current.count += 1;
      counts.set(key, current);
      const confidenceBucket = confidencesByKey.get(key) ?? new Set<DerivedTagReviewQueueSummaryItem["confidence"]>();
      confidenceBucket.add(confidence);
      confidencesByKey.set(key, confidenceBucket);
    }
  }

  for (const [key, item] of counts.entries()) {
    const confidences = confidencesByKey.get(key) ?? new Set<DerivedTagReviewQueueSummaryItem["confidence"]>();
    item.confidence = confidences.size <= 1 ? ([...confidences][0] ?? "unspecified") : "mixed";
  }

  return [...counts.values()].sort(compareReviewQueueItems);
}
