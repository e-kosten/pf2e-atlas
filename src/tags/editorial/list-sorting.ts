import { DERIVED_TAG_MANAGED_CATEGORIES } from "../manifest.js";
import type { SearchCategory } from "../../domain/derived-tag-types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import type { DerivedTagReviewQueueSummaryItem } from "./types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "./review-vocabulary.js";

export { DERIVED_TAG_MANAGED_CATEGORIES } from "../manifest.js";

const DISPLAY_TEXT_COLLATOR = new Intl.Collator("en", {
  sensitivity: "base",
  numeric: true,
});

const managedCategoryRank = new Map(
  DERIVED_TAG_MANAGED_CATEGORIES.map((category, index) => [category, index] as const),
);

const reviewQueueKindPriority = new Map<DerivedTagReviewQueueSummaryItem["kind"], number>([
  [DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT, 0],
  [DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR, 1],
]);

const reviewQueueConfidencePriority = new Map<DerivedTagReviewQueueSummaryItem["confidence"], number>([
  ["mixed", 0],
  [DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.LOW, 1],
  [DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM, 2],
  [DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.HIGH, 3],
  ["unspecified", 4],
]);

export function normalizeSortLabel(value: string): string {
  return value.toLowerCase().replace(/[_-]+/g, " ").replace(/\s+/g, " ").trim();
}

export function compareDisplayText(left: string, right: string): number {
  return DISPLAY_TEXT_COLLATOR.compare(normalizeSortLabel(left), normalizeSortLabel(right));
}

export function compareOptionalDisplayText(left: string | null | undefined, right: string | null | undefined): number {
  if (left && right) {
    return compareDisplayText(left, right) || DISPLAY_TEXT_COLLATOR.compare(left, right);
  }
  if (left) {
    return -1;
  }
  if (right) {
    return 1;
  }
  return 0;
}

export function compareManagedCategory(
  left: SearchCategory | DerivedTagManagedCategory,
  right: SearchCategory | DerivedTagManagedCategory,
): number {
  const leftRank = managedCategoryRank.get(left as DerivedTagManagedCategory);
  const rightRank = managedCategoryRank.get(right as DerivedTagManagedCategory);

  if (leftRank !== undefined || rightRank !== undefined) {
    return (leftRank ?? Number.MAX_SAFE_INTEGER) - (rightRank ?? Number.MAX_SAFE_INTEGER);
  }

  return compareDisplayText(left, right) || DISPLAY_TEXT_COLLATOR.compare(left, right);
}

export function compareReviewQueueItems(
  left: DerivedTagReviewQueueSummaryItem,
  right: DerivedTagReviewQueueSummaryItem,
): number {
  return (
    (reviewQueueKindPriority.get(left.kind) ?? Number.MAX_SAFE_INTEGER) -
      (reviewQueueKindPriority.get(right.kind) ?? Number.MAX_SAFE_INTEGER) ||
    (reviewQueueConfidencePriority.get(left.confidence) ?? Number.MAX_SAFE_INTEGER) -
      (reviewQueueConfidencePriority.get(right.confidence) ?? Number.MAX_SAFE_INTEGER) ||
    right.count - left.count ||
    compareManagedCategory(left.category, right.category) ||
    compareOptionalDisplayText(left.family, right.family) ||
    compareDisplayText(left.tag, right.tag) ||
    left.tag.localeCompare(right.tag)
  );
}
