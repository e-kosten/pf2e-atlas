import type { DerivedTagExemplarReviewDecision, SearchCategory } from "../../domain/index.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../manifest.js";
import { DERIVED_TAG_ONTOLOGY_FAMILIES, DERIVED_TAG_ONTOLOGY_TAGS, deriveRecordTagDerivation } from "../index.js";
import {
  buildDerivedTagPendingAssignmentViews,
  type DerivedTagAssignmentReviewCategory,
} from "../runtime/assignments.js";
import { publishDerivedTagOntology, type PublishedDerivedTagOntology } from "../runtime/catalog-utils.js";
import type { DerivedTagSource } from "../runtime/catalog-utils.js";
import { compareReviewQueueItems } from "./list-sorting.js";
import type { DerivedTagMigrationDecision, DerivedTagReviewQueueSummaryItem } from "./types.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";

let publishedOntologyCache: PublishedDerivedTagOntology | null = null;

export function getPublishedDerivedTagMigrationOntology(): PublishedDerivedTagOntology {
  if (!publishedOntologyCache) {
    publishedOntologyCache = publishDerivedTagOntology(DERIVED_TAG_ONTOLOGY_FAMILIES, DERIVED_TAG_ONTOLOGY_TAGS);
  }
  return publishedOntologyCache;
}

export function deriveCurrentTagSources(
  input: Parameters<typeof deriveRecordTagDerivation>[0],
): Record<string, DerivedTagSource> {
  const derivation = deriveRecordTagDerivation(input);
  return Object.fromEntries([...derivation.sources.entries()]);
}

export function getCurrentDerivedTagPendingAssignmentViews() {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  return buildDerivedTagPendingAssignmentViews(
    getPublishedDerivedTagMigrationOntology(),
    DERIVED_TAG_MANAGED_CATEGORIES.map((category) => state.assignmentReviews[category]),
  );
}

function flattenAssignmentReviewDecisions(
  assignmentReviews: DerivedTagAssignmentReviewCategory[],
): Array<{ category: SearchCategory; decision: DerivedTagMigrationDecision }> {
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
): DerivedTagMigrationDecision[] {
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

export function summarizeCurrentDerivedTagReviewQueue(): DerivedTagReviewQueueSummaryItem[] {
  const state = getCurrentDerivedTagMigrationAuthoredState();
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
