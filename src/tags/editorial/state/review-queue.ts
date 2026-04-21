import type {
  DerivedTagExemplarReviewDecision,
  SearchCategory,
} from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../../manifest.js";
import type {
  DerivedTagAssignmentReviewCategory,
  DerivedTagAssignmentReviewDecision,
} from "../../runtime/derivation/assignments.js";
import { compareReviewQueueItems } from "../list-sorting.js";
import type { DerivedTagReviewDecision, DerivedTagReviewQueueSummaryItem } from "../types.js";
import { getCurrentDerivedTagAuthoredState } from "./authored-state.js";

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
        kind: "assignment" as const,
        family: reviewDecision.family,
        tag: reviewDecision.tag,
        mode: reviewDecision.mode,
        status: "needs_review" as const,
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
    kind: "exemplar" as const,
    tag: reviewDecision.tag,
    polarity: reviewDecision.proposedPolarity === "negative" ? "negative" : "positive",
    action: reviewDecision.proposedPolarity === "drop" ? "drop" : "keep",
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
  return listCurrentPendingAssignmentReviews().filter((entry) => entry.decision.source === "llm");
}

export function listCurrentPendingExemplarReviews(): PendingExemplarReviewEntry[] {
  const state = getCurrentDerivedTagAuthoredState();
  const pending: PendingExemplarReviewEntry[] = [];

  for (const [category, exemplarReviewCategory] of Object.entries(state.exemplarReviews) as Array<
    [SearchCategory, { decisions: DerivedTagExemplarReviewDecision[] }]
  >) {
    for (const decision of exemplarReviewCategory.decisions) {
      if (decision.status !== "needs_review") {
        continue;
      }
      pending.push({ category, decision });
    }
  }

  return pending;
}

export function listCurrentPendingLlmExemplarReviews(): PendingExemplarReviewEntry[] {
  return listCurrentPendingExemplarReviews().filter((entry) => entry.decision.source === "llm");
}

export function summarizeCurrentDerivedTagReviewQueue(): DerivedTagReviewQueueSummaryItem[] {
  const state = getCurrentDerivedTagAuthoredState();
  const counts = new Map<string, DerivedTagReviewQueueSummaryItem>();
  const confidencesByKey = new Map<string, Set<DerivedTagReviewQueueSummaryItem["confidence"]>>();

  for (const { category, decision } of flattenAssignmentReviewDecisions(
    DERIVED_TAG_MANAGED_CATEGORIES.map((managedCategory) => state.assignmentReviews[managedCategory]),
  )) {
    if (decision.kind !== "assignment") {
      continue;
    }
    const confidence = decision.confidence ?? "unspecified";
    const key = ["assignment", category, decision.family, decision.tag].join("|");
    const current = counts.get(key) ?? {
      kind: "assignment",
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
      if (decision.kind !== "exemplar" || decision.status !== "needs_review") {
        continue;
      }
      const confidence = decision.confidence ?? "unspecified";
      const key = ["exemplar", category, decision.tag].join("|");
      const current = counts.get(key) ?? {
        kind: "exemplar",
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
