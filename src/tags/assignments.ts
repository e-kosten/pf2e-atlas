import type { DerivedTagCatalogEntry, SearchCategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import { normalizeDerivedTag } from "./shared.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENTS } from "./assignments/creature.js";

export type DerivedTagReviewStatus =
  | "auto_applied"
  | "needs_review"
  | "approved"
  | "rejected";

export type DerivedTagReviewConfidence = "high" | "medium" | "low";

export type DerivedTagReviewSource = "human" | "llm";

export type DerivedTagReviewEntry = {
  mode: "include" | "exclude";
  status: DerivedTagReviewStatus;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type AuthoredDerivedTagAssignment = {
  name: string;
  recordKey: string;
  applied?: Record<string, string[]>;
  excluded?: Record<string, string[]>;
  review?: Record<string, Record<string, DerivedTagReviewEntry>>;
};

type DerivedTagAssignmentGroup = {
  category: SearchCategory;
  assignments: AuthoredDerivedTagAssignment[];
};

export type DerivedTagExplicitAssignment = {
  category: SearchCategory;
  name: string;
  includeTags: string[];
  excludeTags: string[];
};

export type DerivedTagExplicitAssignmentIndex = {
  assignmentsByRecordKey: Map<string, DerivedTagExplicitAssignment>;
};

export type DerivedTagPendingAssignmentView = {
  name: string;
  recordKey: string;
  pending: Record<string, string[]>;
};

type DerivedTagAssignmentRecordSummary = {
  recordKey: string;
  name: string;
  category: SearchCategory;
};

const RAW_DERIVED_TAG_ASSIGNMENTS: DerivedTagAssignmentGroup[] = [
  { category: "creature", assignments: CREATURE_DERIVED_TAG_ASSIGNMENTS },
];

function buildFamilyTagMap(
  catalog: DerivedTagCatalogEntry[],
): Map<SearchCategory, Map<string, Set<string>>> {
  const familiesByCategory = new Map<SearchCategory, Map<string, Set<string>>>();

  for (const entry of catalog) {
    const categoryFamilies = familiesByCategory.get(entry.category) ?? new Map<string, Set<string>>();
    const normalizedFamily = normalizeDerivedTag(entry.family);
    const familyTags = categoryFamilies.get(normalizedFamily) ?? new Set<string>();
    for (const tag of entry.tags) {
      familyTags.add(normalizeDerivedTag(tag.value));
    }
    categoryFamilies.set(normalizedFamily, familyTags);
    familiesByCategory.set(entry.category, categoryFamilies);
  }

  return familiesByCategory;
}

type NormalizedFamilyTagMap = Map<string, Set<string>>;

type NormalizedReviewMap = Map<string, Map<string, DerivedTagReviewEntry>>;

type NormalizedAuthoredDerivedTagAssignment = {
  includeByFamily: NormalizedFamilyTagMap;
  excludeByFamily: NormalizedFamilyTagMap;
  reviewByFamily: NormalizedReviewMap;
};

function createEmptyFamilyTagMap(): NormalizedFamilyTagMap {
  return new Map<string, Set<string>>();
}

function addNormalizedTag(
  bucket: NormalizedFamilyTagMap,
  family: string,
  tag: string,
): void {
  const familyTags = bucket.get(family) ?? new Set<string>();
  familyTags.add(tag);
  bucket.set(family, familyTags);
}

function normalizeFamilyTagAssignments(
  groupedTags: Record<string, string[]> | undefined,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
  fieldName: "applied" | "excluded",
  recordKey: string,
): NormalizedFamilyTagMap {
  const normalizedAssignments = createEmptyFamilyTagMap();
  if (!groupedTags) {
    return normalizedAssignments;
  }

  const categoryFamilies = familyTagMap.get(category) ?? new Map<string, Set<string>>();

  for (const [rawFamily, rawTags] of Object.entries(groupedTags)) {
    const normalizedFamily = normalizeDerivedTag(rawFamily);
    const familyTags = categoryFamilies.get(normalizedFamily);
    if (!familyTags) {
      throw new Error(
        `Derived tag assignment ${fieldName} family "${rawFamily}" for "${recordKey}" does not exist in category "${category}".`,
      );
    }

    for (const rawTag of rawTags) {
      const normalizedTag = normalizeDerivedTag(rawTag);
      if (!familyTags.has(normalizedTag)) {
        throw new Error(
          `Derived tag assignment ${fieldName} tag "${rawTag}" for "${recordKey}" does not belong to family "${rawFamily}" in category "${category}".`,
        );
      }
      addNormalizedTag(normalizedAssignments, normalizedFamily, normalizedTag);
    }
  }

  return normalizedAssignments;
}

function normalizeReviewAssignments(
  review: AuthoredDerivedTagAssignment["review"],
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
  recordKey: string,
): NormalizedReviewMap {
  const reviewByFamily = new Map<string, Map<string, DerivedTagReviewEntry>>();
  if (!review) {
    return reviewByFamily;
  }

  const categoryFamilies = familyTagMap.get(category) ?? new Map<string, Set<string>>();

  for (const [rawFamily, taggedReview] of Object.entries(review)) {
    const normalizedFamily = normalizeDerivedTag(rawFamily);
    const familyTags = categoryFamilies.get(normalizedFamily);
    if (!familyTags) {
      throw new Error(
        `Derived tag assignment review family "${rawFamily}" for "${recordKey}" does not exist in category "${category}".`,
      );
    }

    const familyReview = reviewByFamily.get(normalizedFamily) ?? new Map<string, DerivedTagReviewEntry>();
    for (const [rawTag, reviewEntry] of Object.entries(taggedReview)) {
      const normalizedTag = normalizeDerivedTag(rawTag);
      if (!familyTags.has(normalizedTag)) {
        throw new Error(
          `Derived tag assignment review tag "${rawTag}" for "${recordKey}" does not belong to family "${rawFamily}" in category "${category}".`,
        );
      }
      familyReview.set(normalizedTag, reviewEntry);
    }
    reviewByFamily.set(normalizedFamily, familyReview);
  }

  return reviewByFamily;
}

function flattenNormalizedAssignments(assignments: NormalizedFamilyTagMap): string[] {
  const flattenedTags = new Set<string>();
  for (const familyTags of assignments.values()) {
    for (const tag of familyTags) {
      flattenedTags.add(tag);
    }
  }

  return uniqueSorted([...flattenedTags]);
}

function hasNormalizedTag(
  assignments: NormalizedFamilyTagMap,
  family: string,
  tag: string,
): boolean {
  return assignments.get(family)?.has(tag) ?? false;
}

function getReviewEntry(
  reviewByFamily: NormalizedReviewMap,
  family: string,
  tag: string,
): DerivedTagReviewEntry | undefined {
  return reviewByFamily.get(family)?.get(tag);
}

function renderQualifiedTag(family: string, tag: string): string {
  return `${family}.${tag}`;
}

function buildPendingView(
  assignment: AuthoredDerivedTagAssignment,
  reviewByFamily: NormalizedReviewMap,
): DerivedTagPendingAssignmentView | null {
  const pendingByFamily: Record<string, string[]> = {};

  for (const family of uniqueSorted([...reviewByFamily.keys()])) {
    const familyReview = reviewByFamily.get(family);
    if (!familyReview) {
      continue;
    }

    const pendingTags = uniqueSorted(
      [...familyReview.entries()]
        .filter(([, reviewEntry]) => reviewEntry.status === "needs_review")
        .map(([tag]) => tag),
    );
    if (pendingTags.length > 0) {
      pendingByFamily[family] = pendingTags;
    }
  }

  return Object.keys(pendingByFamily).length > 0
    ? {
        name: assignment.name,
        recordKey: assignment.recordKey,
        pending: pendingByFamily,
      }
    : null;
}

function normalizeAssignment(
  assignment: AuthoredDerivedTagAssignment,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
): NormalizedAuthoredDerivedTagAssignment {
  const includeByFamily = normalizeFamilyTagAssignments(
    assignment.applied,
    category,
    familyTagMap,
    "applied",
    assignment.recordKey,
  );
  const excludeByFamily = normalizeFamilyTagAssignments(
    assignment.excluded,
    category,
    familyTagMap,
    "excluded",
    assignment.recordKey,
  );
  const reviewByFamily = normalizeReviewAssignments(
    assignment.review,
    category,
    familyTagMap,
    assignment.recordKey,
  );

  for (const family of uniqueSorted([...includeByFamily.keys()])) {
    const includedTags = includeByFamily.get(family);
    if (!includedTags) {
      continue;
    }

    for (const tag of includedTags) {
      if (hasNormalizedTag(excludeByFamily, family, tag)) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" places "${renderQualifiedTag(family, tag)}" in both applied and excluded.`,
        );
      }

      const reviewEntry = getReviewEntry(reviewByFamily, family, tag);
      if (!reviewEntry) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" is missing review metadata for applied tag "${renderQualifiedTag(family, tag)}".`,
        );
      }
      if (reviewEntry.mode !== "include") {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" marks applied tag "${renderQualifiedTag(family, tag)}" with review mode "${reviewEntry.mode}".`,
        );
      }
      if (reviewEntry.status !== "approved" && reviewEntry.status !== "auto_applied") {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" places "${renderQualifiedTag(family, tag)}" in applied but review status is "${reviewEntry.status}".`,
        );
      }
    }
  }

  for (const family of uniqueSorted([...excludeByFamily.keys()])) {
    const excludedTags = excludeByFamily.get(family);
    if (!excludedTags) {
      continue;
    }

    for (const tag of excludedTags) {
      const reviewEntry = getReviewEntry(reviewByFamily, family, tag);
      if (!reviewEntry) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" is missing review metadata for excluded tag "${renderQualifiedTag(family, tag)}".`,
        );
      }
      if (reviewEntry.mode !== "exclude") {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" marks excluded tag "${renderQualifiedTag(family, tag)}" with review mode "${reviewEntry.mode}".`,
        );
      }
      if (reviewEntry.status !== "approved" && reviewEntry.status !== "auto_applied") {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" places "${renderQualifiedTag(family, tag)}" in excluded but review status is "${reviewEntry.status}".`,
        );
      }
    }
  }

  for (const family of uniqueSorted([...reviewByFamily.keys()])) {
    const familyReview = reviewByFamily.get(family);
    if (!familyReview) {
      continue;
    }

    for (const tag of uniqueSorted([...familyReview.keys()])) {
      const reviewEntry = familyReview.get(tag)!;
      const isApplied = hasNormalizedTag(includeByFamily, family, tag);
      const isExcluded = hasNormalizedTag(excludeByFamily, family, tag);
      const qualifiedTag = renderQualifiedTag(family, tag);

      if (reviewEntry.status === "needs_review" || reviewEntry.status === "rejected") {
        if (isApplied || isExcluded) {
          throw new Error(
            `Derived tag assignment for "${assignment.recordKey}" keeps "${qualifiedTag}" live even though review status is "${reviewEntry.status}".`,
          );
        }
        continue;
      }

      if (reviewEntry.mode === "include") {
        if (isExcluded) {
          throw new Error(
            `Derived tag assignment for "${assignment.recordKey}" marks "${qualifiedTag}" as include in review but places it in excluded.`,
          );
        }
        if (!isApplied) {
          throw new Error(
            `Derived tag assignment for "${assignment.recordKey}" review marks "${qualifiedTag}" as "${reviewEntry.status}" include but it is missing from applied.`,
          );
        }
        continue;
      }

      if (isApplied) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" marks "${qualifiedTag}" as exclude in review but places it in applied.`,
        );
      }
      if (!isExcluded) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" review marks "${qualifiedTag}" as "${reviewEntry.status}" exclude but it is missing from excluded.`,
        );
      }
    }
  }

  return {
    includeByFamily,
    excludeByFamily,
    reviewByFamily,
  };
}

export function buildDerivedTagExplicitAssignmentIndex(
  catalog: DerivedTagCatalogEntry[],
  groups: DerivedTagAssignmentGroup[] = RAW_DERIVED_TAG_ASSIGNMENTS,
): DerivedTagExplicitAssignmentIndex {
  const familyTagMap = buildFamilyTagMap(catalog);
  const assignmentsByRecordKey = new Map<string, DerivedTagExplicitAssignment>();

  for (const group of groups) {
    for (const assignment of group.assignments) {
      if (assignmentsByRecordKey.has(assignment.recordKey)) {
        throw new Error(`Duplicate explicit derived tag assignment for "${assignment.recordKey}".`);
      }

      const normalizedAssignment = normalizeAssignment(
        assignment,
        group.category,
        familyTagMap,
      );

      assignmentsByRecordKey.set(assignment.recordKey, {
        category: group.category,
        name: assignment.name,
        includeTags: flattenNormalizedAssignments(normalizedAssignment.includeByFamily),
        excludeTags: flattenNormalizedAssignments(normalizedAssignment.excludeByFamily),
      });
    }
  }

  return { assignmentsByRecordKey };
}

export function buildDerivedTagPendingAssignmentViews(
  catalog: DerivedTagCatalogEntry[],
  groups: DerivedTagAssignmentGroup[] = RAW_DERIVED_TAG_ASSIGNMENTS,
): DerivedTagPendingAssignmentView[] {
  const familyTagMap = buildFamilyTagMap(catalog);
  const pendingViews: DerivedTagPendingAssignmentView[] = [];

  for (const group of groups) {
    for (const assignment of group.assignments) {
      const normalizedAssignment = normalizeAssignment(
        assignment,
        group.category,
        familyTagMap,
      );
      const pendingView = buildPendingView(assignment, normalizedAssignment.reviewByFamily);
      if (pendingView) {
        pendingViews.push(pendingView);
      }
    }
  }

  return pendingViews;
}

export function validateDerivedTagExplicitAssignmentsAgainstRecords(
  records: Iterable<DerivedTagAssignmentRecordSummary>,
  assignmentIndex: DerivedTagExplicitAssignmentIndex,
): void {
  const recordMap = new Map<string, DerivedTagAssignmentRecordSummary>();
  for (const record of records) {
    recordMap.set(record.recordKey, record);
  }

  for (const [recordKey, assignment] of assignmentIndex.assignmentsByRecordKey) {
    const record = recordMap.get(recordKey);
    if (!record) {
      continue;
    }
    if (record.category !== assignment.category) {
      throw new Error(
        `Explicit derived tag assignment for "${recordKey}" expected category "${assignment.category}" but resolved record category "${record.category}".`,
      );
    }
    if (record.name !== assignment.name) {
      throw new Error(
        `Explicit derived tag assignment for "${recordKey}" expected name "${assignment.name}" but resolved canonical name "${record.name}".`,
      );
    }
  }
}

export function createDerivedTagExplicitAssignmentIndex(
  catalog: DerivedTagCatalogEntry[],
): DerivedTagExplicitAssignmentIndex {
  return buildDerivedTagExplicitAssignmentIndex(catalog, RAW_DERIVED_TAG_ASSIGNMENTS);
}
