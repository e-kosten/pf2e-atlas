import { SearchCategory, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import { normalizeDerivedTag } from "./shared.js";

export const REVIEWED_DISCOVERY_REASONS = [
  "not_family_salient",
  "insufficient_evidence",
  "mixed_family_cues",
  "manual_lore_only",
] as const;

export type ReviewedDiscoveryReason = typeof REVIEWED_DISCOVERY_REASONS[number];

export type ReviewedDiscoveryRegistryRecord = {
  recordKey: string;
  subcategory?: SearchSubcategory | null;
  note?: string;
};

export type ReviewedDiscoveryEntry = ReviewedDiscoveryRegistryRecord & {
  category: SearchCategory;
  family: string;
  reason: ReviewedDiscoveryReason;
};

export type ReviewedDiscoveryReasonCount = {
  reason: ReviewedDiscoveryReason;
  count: number;
};

export type ReviewedDiscoverySelectionMode = "excluded" | "included" | "filtered";

export type ReviewedDiscoverySelection = {
  mode: ReviewedDiscoverySelectionMode;
  reviewReason: ReviewedDiscoveryReason | null;
  entries: ReviewedDiscoveryEntry[];
  recordKeys: string[];
  reasonCounts: ReviewedDiscoveryReasonCount[];
};

export type ReviewedDiscoveryApplicationSummary = {
  mode: ReviewedDiscoverySelectionMode;
  reviewReason: ReviewedDiscoveryReason | null;
  scopedCount: number;
  appliedCount: number;
  reasonCounts: ReviewedDiscoveryReasonCount[];
};

export type ReviewedDiscoveryScope = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  reason?: ReviewedDiscoveryReason;
};

export type ReviewedDiscoverySelectionOptions = ReviewedDiscoveryScope & {
  includeReviewed?: boolean;
  reviewReason?: ReviewedDiscoveryReason;
};

export type ReviewedDiscoveryRegistry = Partial<Record<
  SearchCategory,
  Partial<Record<string, Partial<Record<ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[]>>>>
>>;

// Curated during family-gap passes to suppress repeatedly reviewed negatives from the default queue.
export const REVIEWED_DISCOVERY_RECORDS: ReviewedDiscoveryRegistry = {
  creature: {
    setting: {
      not_family_salient: [],
      insufficient_evidence: [],
      mixed_family_cues: [],
      manual_lore_only: [],
    },
  },
};

export function isReviewedDiscoveryReason(value: string): value is ReviewedDiscoveryReason {
  return REVIEWED_DISCOVERY_REASONS.includes(value as ReviewedDiscoveryReason);
}

export function getReviewedDiscoveryEntries(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoveryEntry[] {
  const requestedFamily = scope.family ? normalizeDerivedTag(scope.family) : undefined;
  const entries: ReviewedDiscoveryEntry[] = [];

  for (const [category, families] of Object.entries(registry) as Array<[SearchCategory, ReviewedDiscoveryRegistry[SearchCategory]]>) {
    if (!families) {
      continue;
    }
    if (scope.category && category !== scope.category) {
      continue;
    }

    for (const [family, reasonBuckets] of Object.entries(families)) {
      const normalizedFamily = normalizeDerivedTag(family);
      if (requestedFamily && normalizedFamily !== requestedFamily) {
        continue;
      }

      for (const [reason, records] of Object.entries(reasonBuckets ?? {}) as Array<[ReviewedDiscoveryReason, ReviewedDiscoveryRegistryRecord[] | undefined]>) {
        if (!records || records.length === 0) {
          continue;
        }
        if (scope.reason && reason !== scope.reason) {
          continue;
        }

        for (const record of records) {
          if (scope.subcategory !== undefined && (record.subcategory ?? null) !== scope.subcategory) {
            continue;
          }
          entries.push({
            category,
            family: normalizedFamily,
            reason,
            recordKey: record.recordKey,
            subcategory: record.subcategory ?? null,
            note: record.note,
          });
        }
      }
    }
  }

  return entries
    .slice()
    .sort((left, right) =>
      left.category.localeCompare(right.category) ||
      left.family.localeCompare(right.family) ||
      left.reason.localeCompare(right.reason) ||
      left.recordKey.localeCompare(right.recordKey));
}

export function getReviewedDiscoveryRecordKeys(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): string[] {
  return uniqueSorted(getReviewedDiscoveryEntries(scope, registry).map((entry) => entry.recordKey));
}

export function getReviewedDiscoveryReasonCounts(
  scope: ReviewedDiscoveryScope = {},
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoveryReasonCount[] {
  const entries = getReviewedDiscoveryEntries(scope, registry);
  const counts = new Map<ReviewedDiscoveryReason, Set<string>>();
  for (const entry of entries) {
    const bucket = counts.get(entry.reason) ?? new Set<string>();
    bucket.add(entry.recordKey);
    counts.set(entry.reason, bucket);
  }

  return [...counts.entries()]
    .map(([reason, recordKeys]) => ({
      reason,
      count: recordKeys.size,
    }))
    .sort((left, right) => right.count - left.count || left.reason.localeCompare(right.reason));
}

export function getReviewedDiscoverySelection(
  options: ReviewedDiscoverySelectionOptions,
  registry: ReviewedDiscoveryRegistry = REVIEWED_DISCOVERY_RECORDS,
): ReviewedDiscoverySelection | undefined {
  if (!options.family) {
    return undefined;
  }

  const reviewReason = options.reviewReason ?? null;
  const entries = getReviewedDiscoveryEntries({
    category: options.category,
    subcategory: options.subcategory,
    family: options.family,
    reason: reviewReason ?? undefined,
  }, registry);

  return {
    mode: reviewReason
      ? "filtered"
      : options.includeReviewed
        ? "included"
        : "excluded",
    reviewReason,
    entries,
    recordKeys: uniqueSorted(entries.map((entry) => entry.recordKey)),
    reasonCounts: getReviewedDiscoveryReasonCounts({
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      reason: reviewReason ?? undefined,
    }, registry),
  };
}

export function summarizeReviewedDiscoverySelection(
  selection: ReviewedDiscoverySelection,
  appliedCount = selection.recordKeys.length,
): ReviewedDiscoveryApplicationSummary {
  return {
    mode: selection.mode,
    reviewReason: selection.reviewReason,
    scopedCount: selection.recordKeys.length,
    appliedCount,
    reasonCounts: selection.reasonCounts,
  };
}
