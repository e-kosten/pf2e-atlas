import type { DerivedTagExemplarReviewDecision, SearchCategory } from "../../types.js";
import {
  DERIVED_TAG_ONTOLOGY_FAMILIES,
  DERIVED_TAG_ONTOLOGY_TAGS,
  deriveRecordTagDerivation,
} from "../index.js";
import {
  buildDerivedTagPendingAssignmentViews,
  type AuthoredDerivedTagAssignment,
} from "../runtime/assignments.js";
import { publishDerivedTagOntology, type PublishedDerivedTagOntology } from "../runtime/catalog-utils.js";
import type { DerivedTagSource } from "../runtime/catalog-utils.js";
import type { DerivedTagMigrationDecision, DerivedTagReviewQueueSummaryItem } from "./types.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";

let publishedOntologyCache: PublishedDerivedTagOntology | null = null;

export function getPublishedDerivedTagMigrationOntology(): PublishedDerivedTagOntology {
  if (!publishedOntologyCache) {
    publishedOntologyCache = publishDerivedTagOntology(DERIVED_TAG_ONTOLOGY_FAMILIES, DERIVED_TAG_ONTOLOGY_TAGS);
  }
  return publishedOntologyCache;
}

export function deriveCurrentTagSources(input: Parameters<typeof deriveRecordTagDerivation>[0]): Record<string, DerivedTagSource> {
  const derivation = deriveRecordTagDerivation(input);
  return Object.fromEntries([...derivation.sources.entries()]) as Record<string, DerivedTagSource>;
}

export function getCurrentDerivedTagPendingAssignmentViews() {
  return buildDerivedTagPendingAssignmentViews(getPublishedDerivedTagMigrationOntology());
}

function flattenAssignmentDecisions(assignments: AuthoredDerivedTagAssignment[]): DerivedTagMigrationDecision[] {
  const decisions: DerivedTagMigrationDecision[] = [];
  for (const assignment of assignments) {
    for (const [family, familyReview] of Object.entries(assignment.review ?? {})) {
      for (const [tag, reviewEntry] of Object.entries(familyReview)) {
        decisions.push({
          kind: "assignment",
          family,
          tag,
          mode: reviewEntry.mode,
          status: reviewEntry.status,
          confidence: reviewEntry.confidence,
          rationale: reviewEntry.rationale,
          source: reviewEntry.source,
        });
      }
    }
  }
  return decisions;
}

function flattenExemplarReviewDecisions(exemplarReviews: DerivedTagExemplarReviewDecision[]): DerivedTagMigrationDecision[] {
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

  for (const [category, assignments] of Object.entries(state.assignments) as Array<[SearchCategory, AuthoredDerivedTagAssignment[]]>) {
    for (const decision of flattenAssignmentDecisions(assignments)) {
      if (decision.kind !== "assignment" || decision.status !== "needs_review") {
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
  }

  for (const [category, exemplarReviewCategory] of Object.entries(state.exemplarReviews) as Array<[SearchCategory, { decisions: DerivedTagExemplarReviewDecision[] }]>) {
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
    item.confidence = confidences.size <= 1 ? [...confidences][0] ?? "unspecified" : "mixed";
  }

  return [...counts.values()]
    .sort((left, right) =>
      left.kind.localeCompare(right.kind)
      || left.category.localeCompare(right.category)
      || (left.family ?? "").localeCompare(right.family ?? "")
      || left.tag.localeCompare(right.tag)
      || left.confidence.localeCompare(right.confidence));
}
