import type { SearchCategory } from "../../types.js";
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

export function summarizeCurrentDerivedTagReviewQueue(): DerivedTagReviewQueueSummaryItem[] {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const counts = new Map<string, DerivedTagReviewQueueSummaryItem>();

  for (const [category, assignments] of Object.entries(state.assignments) as Array<[SearchCategory, AuthoredDerivedTagAssignment[]]>) {
    for (const decision of flattenAssignmentDecisions(assignments)) {
      if (decision.kind !== "assignment" || decision.status !== "needs_review") {
        continue;
      }
      const confidence = decision.confidence ?? "unspecified";
      const key = [category, decision.family, decision.tag, confidence].join("|");
      const current = counts.get(key) ?? {
        category,
        family: decision.family,
        tag: decision.tag,
        count: 0,
        confidence,
      };
      current.count += 1;
      counts.set(key, current);
    }
  }

  return [...counts.values()]
    .sort((left, right) =>
      left.category.localeCompare(right.category)
      || left.family.localeCompare(right.family)
      || left.tag.localeCompare(right.tag)
      || left.confidence.localeCompare(right.confidence));
}
