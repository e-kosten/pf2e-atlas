import { DERIVED_TAG_MANAGED_CATEGORIES } from "../../manifest.js";
import { buildDerivedTagPendingAssignmentViews } from "../../runtime/derivation/assignments.js";
import type { PublishedDerivedTagOntology } from "../../runtime/publication/catalog.js";
import type { DerivedTagSource } from "../../runtime/publication/catalog.js";
import { getCurrentDerivedTagAuthoredState } from "./authored-state.js";
import { deriveCurrentRecordTagDerivation, getCurrentDerivedTagWorkingRuntime } from "./working-runtime.js";

export function getPublishedDerivedTagOntology(): PublishedDerivedTagOntology {
  return getCurrentDerivedTagWorkingRuntime().ontology;
}

export function deriveCurrentTagSources(
  input: Parameters<typeof deriveCurrentRecordTagDerivation>[0],
): Record<string, DerivedTagSource> {
  const derivation = deriveCurrentRecordTagDerivation(input);
  return Object.fromEntries([...derivation.sources.entries()]);
}

export function getCurrentDerivedTagPendingAssignmentViews() {
  const state = getCurrentDerivedTagAuthoredState();
  return buildDerivedTagPendingAssignmentViews(
    getPublishedDerivedTagOntology(),
    DERIVED_TAG_MANAGED_CATEGORIES.map((category) => state.assignmentReviews[category]),
  );
}

export { summarizeCurrentDerivedTagReviewQueue } from "./review-queue.js";
