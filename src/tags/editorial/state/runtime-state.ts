import { DERIVED_TAG_MANAGED_CATEGORIES } from "../../manifest.js";
import { buildDerivedTagPendingAssignmentViews } from "../../runtime/derivation/assignments.js";
import type {
  PublishedDerivedTagConceptModel,
  DerivedTagTranslationRecord,
} from "../../../domain/derived-tag-types.js";
import type { PublishedDerivedTagOntology } from "../../runtime/publication/catalog.js";
import type { DerivedTagSource } from "../../runtime/publication/catalog.js";
import { getCurrentDerivedTagAuthoredState } from "./authored-state.js";
import { deriveCurrentRecordTagDerivation, getCurrentDerivedTagWorkingRuntime } from "./working-runtime.js";

export function getPublishedDerivedTagOntology(): PublishedDerivedTagOntology {
  return getCurrentDerivedTagWorkingRuntime().ontology;
}

export function getPublishedDerivedTagConceptModel(): PublishedDerivedTagConceptModel {
  return getCurrentDerivedTagWorkingRuntime().ontology.conceptModel;
}

export function listPublishedDerivedTagTranslations(): DerivedTagTranslationRecord[] {
  return getPublishedDerivedTagConceptModel().translations;
}

export function summarizeCurrentDerivedTagTranslationQueue(): Array<{
  category: string;
  translationStatus: DerivedTagTranslationRecord["translationStatus"];
  count: number;
}> {
  const counts = new Map<string, number>();
  for (const translation of listPublishedDerivedTagTranslations()) {
    if (translation.translationStatus === "mapped") {
      continue;
    }
    const key = `${translation.currentCategory}:${translation.translationStatus}`;
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }

  return [...counts.entries()]
    .map(([key, count]) => {
      const [category, translationStatus] = key.split(":");
      return {
        category: category!,
        translationStatus: translationStatus as DerivedTagTranslationRecord["translationStatus"],
        count,
      };
    })
    .sort(
      (left, right) =>
        left.category.localeCompare(right.category) ||
        left.translationStatus.localeCompare(right.translationStatus),
    );
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
