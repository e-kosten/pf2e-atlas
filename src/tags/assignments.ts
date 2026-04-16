import type { DerivedTagCatalogEntry, SearchCategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import { normalizeDerivedTag } from "./shared.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENTS } from "./assignments/creature.js";

export type AuthoredDerivedTagAssignment = {
  recordKey: string;
  name: string;
  byFamily: Record<string, string[]>;
  excludeByFamily?: Record<string, string[]>;
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

function flattenFamilyAssignments(
  groupedTags: Record<string, string[]> | undefined,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
  fieldName: "byFamily" | "excludeByFamily",
  recordKey: string,
): string[] {
  if (!groupedTags) {
    return [];
  }

  const categoryFamilies = familyTagMap.get(category) ?? new Map<string, Set<string>>();
  const flattenedTags = new Set<string>();

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
      flattenedTags.add(normalizedTag);
    }
  }

  return uniqueSorted([...flattenedTags]);
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

      const includeTags = flattenFamilyAssignments(
        assignment.byFamily,
        group.category,
        familyTagMap,
        "byFamily",
        assignment.recordKey,
      );
      const excludeTags = flattenFamilyAssignments(
        assignment.excludeByFamily,
        group.category,
        familyTagMap,
        "excludeByFamily",
        assignment.recordKey,
      );
      const overlap = includeTags.filter((tag) => excludeTags.includes(tag));
      if (overlap.length > 0) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" includes and excludes the same tags: ${overlap.join(", ")}.`,
        );
      }

      assignmentsByRecordKey.set(assignment.recordKey, {
        category: group.category,
        name: assignment.name,
        includeTags,
        excludeTags,
      });
    }
  }

  return { assignmentsByRecordKey };
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
