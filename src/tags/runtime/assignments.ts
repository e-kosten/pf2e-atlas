import type { DerivedTagOntologyTag, SearchCategory } from "../../types.js";
import type { PublishedDerivedTagOntology } from "./catalog-utils.js";
import { uniqueSorted } from "../../utils.js";
import { normalizeDerivedTag } from "./shared.js";
import { AFFLICTION_DERIVED_TAG_ASSIGNMENTS } from "../assignments/affliction.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENTS } from "../assignments/creature.js";
import { EQUIPMENT_DERIVED_TAG_ASSIGNMENTS } from "../assignments/equipment.js";
import { HAZARD_DERIVED_TAG_ASSIGNMENTS } from "../assignments/hazard.js";
import { SPELL_DERIVED_TAG_ASSIGNMENTS } from "../assignments/spell.js";
import { AFFLICTION_DERIVED_TAG_ASSIGNMENT_REVIEWS } from "../assignment-reviews/affliction.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENT_REVIEWS } from "../assignment-reviews/creature.js";
import { EQUIPMENT_DERIVED_TAG_ASSIGNMENT_REVIEWS } from "../assignment-reviews/equipment.js";
import { HAZARD_DERIVED_TAG_ASSIGNMENT_REVIEWS } from "../assignment-reviews/hazard.js";
import { SPELL_DERIVED_TAG_ASSIGNMENT_REVIEWS } from "../assignment-reviews/spell.js";
import { AFFLICTION_DERIVED_TAG_ASSIGNMENT_MEMORY } from "../assignment-memory/affliction.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENT_MEMORY } from "../assignment-memory/creature.js";
import { EQUIPMENT_DERIVED_TAG_ASSIGNMENT_MEMORY } from "../assignment-memory/equipment.js";
import { HAZARD_DERIVED_TAG_ASSIGNMENT_MEMORY } from "../assignment-memory/hazard.js";
import { SPELL_DERIVED_TAG_ASSIGNMENT_MEMORY } from "../assignment-memory/spell.js";
import { listLegacyDerivedTagFamilyAliases } from "./family-compatibility.js";

export type DerivedTagReviewStatus =
  | "auto_applied"
  | "needs_review"
  | "approved"
  | "rejected";

export type DerivedTagReviewConfidence = "high" | "medium" | "low";

export type DerivedTagReviewSource = "human" | "llm";

export type DerivedTagAssignmentDecisionSource = "human" | "llm_auto" | "llm_reviewed";

export type DerivedTagAssignmentDecision = {
  tag: string;
  source: DerivedTagAssignmentDecisionSource;
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
};

export type AuthoredDerivedTagAssignment = {
  name: string;
  recordKey: string;
  applied?: Record<string, DerivedTagAssignmentDecision[]>;
  excluded?: Record<string, DerivedTagAssignmentDecision[]>;
};

export type DerivedTagAssignmentReviewDecision = {
  name: string;
  recordKey: string;
  family: string;
  tag: string;
  mode: "include" | "exclude";
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type DerivedTagAssignmentReviewCategory = {
  category: SearchCategory;
  decisions: DerivedTagAssignmentReviewDecision[];
};

export type DerivedTagAssignmentMemoryDecision = {
  name: string;
  recordKey: string;
  family: string;
  tag: string;
  mode: "include" | "exclude";
  confidence?: DerivedTagReviewConfidence;
  rationale: string;
  source?: DerivedTagReviewSource;
};

export type DerivedTagAssignmentMemoryCategory = {
  category: SearchCategory;
  decisions: DerivedTagAssignmentMemoryDecision[];
};

type DerivedTagAssignmentGroup = {
  category: SearchCategory;
  assignments: AuthoredDerivedTagAssignment[];
};

type DerivedTagAssignmentReviewGroup = DerivedTagAssignmentReviewCategory;
type DerivedTagAssignmentMemoryGroup = DerivedTagAssignmentMemoryCategory;

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
  { category: "affliction", assignments: AFFLICTION_DERIVED_TAG_ASSIGNMENTS },
  { category: "creature", assignments: CREATURE_DERIVED_TAG_ASSIGNMENTS },
  { category: "equipment", assignments: EQUIPMENT_DERIVED_TAG_ASSIGNMENTS },
  { category: "hazard", assignments: HAZARD_DERIVED_TAG_ASSIGNMENTS },
  { category: "spell", assignments: SPELL_DERIVED_TAG_ASSIGNMENTS },
];

const RAW_DERIVED_TAG_ASSIGNMENT_REVIEWS: DerivedTagAssignmentReviewGroup[] = [
  AFFLICTION_DERIVED_TAG_ASSIGNMENT_REVIEWS,
  CREATURE_DERIVED_TAG_ASSIGNMENT_REVIEWS,
  EQUIPMENT_DERIVED_TAG_ASSIGNMENT_REVIEWS,
  HAZARD_DERIVED_TAG_ASSIGNMENT_REVIEWS,
  SPELL_DERIVED_TAG_ASSIGNMENT_REVIEWS,
];

const RAW_DERIVED_TAG_ASSIGNMENT_MEMORY: DerivedTagAssignmentMemoryGroup[] = [
  AFFLICTION_DERIVED_TAG_ASSIGNMENT_MEMORY,
  CREATURE_DERIVED_TAG_ASSIGNMENT_MEMORY,
  EQUIPMENT_DERIVED_TAG_ASSIGNMENT_MEMORY,
  HAZARD_DERIVED_TAG_ASSIGNMENT_MEMORY,
  SPELL_DERIVED_TAG_ASSIGNMENT_MEMORY,
];

function buildFamilyTagMap(
  tags: DerivedTagOntologyTag[],
): Map<SearchCategory, Map<string, Set<string>>> {
  const familiesByCategory = new Map<SearchCategory, Map<string, Set<string>>>();

  for (const tag of tags) {
    const categoryFamilies = familiesByCategory.get(tag.category) ?? new Map<string, Set<string>>();
    const normalizedFamily = normalizeDerivedTag(tag.family);
    const familyTags = categoryFamilies.get(normalizedFamily) ?? new Set<string>();
    familyTags.add(normalizeDerivedTag(tag.tag));
    categoryFamilies.set(normalizedFamily, familyTags);
    familiesByCategory.set(tag.category, categoryFamilies);
  }

  for (const [category, categoryFamilies] of familiesByCategory.entries()) {
    for (const { legacyFamily, targetFamilies } of listLegacyDerivedTagFamilyAliases(category)) {
      const legacyTags = categoryFamilies.get(legacyFamily) ?? new Set<string>();
      for (const targetFamily of targetFamilies) {
        for (const tag of categoryFamilies.get(targetFamily) ?? []) {
          legacyTags.add(tag);
        }
      }
      categoryFamilies.set(legacyFamily, legacyTags);
    }
  }

  return familiesByCategory;
}

type NormalizedFamilyDecisionMap = Map<string, Map<string, DerivedTagAssignmentDecision>>;

function createEmptyFamilyDecisionMap(): NormalizedFamilyDecisionMap {
  return new Map<string, Map<string, DerivedTagAssignmentDecision>>();
}

function normalizeAssignmentDecision(
  decision: DerivedTagAssignmentDecision,
): DerivedTagAssignmentDecision {
  return {
    ...decision,
    tag: normalizeDerivedTag(decision.tag),
  };
}

function normalizeFamilyTagAssignments(
  groupedDecisions: Record<string, DerivedTagAssignmentDecision[]> | undefined,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
  fieldName: "applied" | "excluded",
  recordKey: string,
): NormalizedFamilyDecisionMap {
  const normalizedAssignments = createEmptyFamilyDecisionMap();
  if (!groupedDecisions) {
    return normalizedAssignments;
  }

  const categoryFamilies = familyTagMap.get(category) ?? new Map<string, Set<string>>();

  for (const [rawFamily, rawDecisions] of Object.entries(groupedDecisions)) {
    const normalizedFamily = normalizeDerivedTag(rawFamily);
    const familyTags = categoryFamilies.get(normalizedFamily);
    if (!familyTags) {
      throw new Error(
        `Derived tag assignment ${fieldName} family "${rawFamily}" for "${recordKey}" does not exist in category "${category}".`,
      );
    }

    const familyDecisions = normalizedAssignments.get(normalizedFamily) ?? new Map<string, DerivedTagAssignmentDecision>();
    for (const rawDecision of rawDecisions) {
      const normalizedDecision = normalizeAssignmentDecision(rawDecision);
      if (!familyTags.has(normalizedDecision.tag)) {
        throw new Error(
          `Derived tag assignment ${fieldName} tag "${rawDecision.tag}" for "${recordKey}" does not belong to family "${rawFamily}" in category "${category}".`,
        );
      }
      if (familyDecisions.has(normalizedDecision.tag)) {
        throw new Error(
          `Derived tag assignment ${fieldName} for "${recordKey}" repeats "${rawFamily}.${normalizedDecision.tag}".`,
        );
      }
      familyDecisions.set(normalizedDecision.tag, normalizedDecision);
    }
    if (familyDecisions.size > 0) {
      normalizedAssignments.set(normalizedFamily, familyDecisions);
    }
  }

  return normalizedAssignments;
}

function flattenNormalizedAssignments(assignments: NormalizedFamilyDecisionMap): string[] {
  const flattenedTags = new Set<string>();
  for (const familyDecisions of assignments.values()) {
    for (const tag of familyDecisions.keys()) {
      flattenedTags.add(tag);
    }
  }

  return uniqueSorted([...flattenedTags]);
}

function hasNormalizedDecision(
  assignments: NormalizedFamilyDecisionMap,
  family: string,
  tag: string,
): boolean {
  return assignments.get(family)?.has(tag) ?? false;
}

function renderQualifiedTag(family: string, tag: string): string {
  return `${family}.${tag}`;
}

function normalizeAssignment(
  assignment: AuthoredDerivedTagAssignment,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
): {
  includeByFamily: NormalizedFamilyDecisionMap;
  excludeByFamily: NormalizedFamilyDecisionMap;
} {
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

  for (const family of uniqueSorted([...includeByFamily.keys()])) {
    const includedTags = includeByFamily.get(family);
    if (!includedTags) {
      continue;
    }

    for (const tag of uniqueSorted([...includedTags.keys()])) {
      if (hasNormalizedDecision(excludeByFamily, family, tag)) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" places "${renderQualifiedTag(family, tag)}" in both applied and excluded.`,
        );
      }
    }
  }

  if (includeByFamily.size === 0 && excludeByFamily.size === 0) {
    throw new Error(
      `Derived tag assignment for "${assignment.recordKey}" must include at least one applied or excluded tag.`,
    );
  }

  return {
    includeByFamily,
    excludeByFamily,
  };
}

function validateFamilyTagReference(
  fieldName: string,
  category: SearchCategory,
  familyTagMap: Map<SearchCategory, Map<string, Set<string>>>,
  recordKey: string,
  family: string,
  tag: string,
): { family: string; tag: string } {
  const normalizedFamily = normalizeDerivedTag(family);
  const normalizedTag = normalizeDerivedTag(tag);
  const categoryFamilies = familyTagMap.get(category) ?? new Map<string, Set<string>>();
  const familyTags = categoryFamilies.get(normalizedFamily);

  if (!familyTags) {
    throw new Error(
      `Derived tag ${fieldName} family "${family}" for "${recordKey}" does not exist in category "${category}".`,
    );
  }
  if (!familyTags.has(normalizedTag)) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" does not belong to family "${family}" in category "${category}".`,
    );
  }

  return { family: normalizedFamily, tag: normalizedTag };
}

function reviewIdentity(decision: Pick<DerivedTagAssignmentReviewDecision, "recordKey" | "family" | "tag" | "mode">): string {
  return [
    decision.recordKey,
    normalizeDerivedTag(decision.family),
    normalizeDerivedTag(decision.tag),
    decision.mode,
  ].join("|");
}

function memoryIdentity(decision: Pick<DerivedTagAssignmentMemoryDecision, "recordKey" | "family" | "tag" | "mode">): string {
  return [
    decision.recordKey,
    normalizeDerivedTag(decision.family),
    normalizeDerivedTag(decision.tag),
    decision.mode,
  ].join("|");
}

export function buildDerivedTagPendingAssignmentViews(
  ontology: PublishedDerivedTagOntology,
  groups: DerivedTagAssignmentReviewGroup[] = RAW_DERIVED_TAG_ASSIGNMENT_REVIEWS,
): DerivedTagPendingAssignmentView[] {
  const familyTagMap = buildFamilyTagMap(ontology.tags);
  const seenDecisionKeys = new Set<string>();
  const pendingByRecord = new Map<string, DerivedTagPendingAssignmentView>();

  for (const group of groups) {
    for (const decision of group.decisions) {
      const validated = validateFamilyTagReference(
        "assignment review",
        group.category,
        familyTagMap,
        decision.recordKey,
        decision.family,
        decision.tag,
      );
      const key = reviewIdentity(decision);
      if (seenDecisionKeys.has(key)) {
        throw new Error(`Derived tag assignment review repeats "${validated.family}.${validated.tag}" (${decision.mode}) for "${decision.recordKey}".`);
      }
      const oppositeKey = reviewIdentity({
        recordKey: decision.recordKey,
        family: validated.family,
        tag: validated.tag,
        mode: decision.mode === "include" ? "exclude" : "include",
      });
      if (seenDecisionKeys.has(oppositeKey)) {
        throw new Error(
          `Derived tag assignment review places "${validated.family}.${validated.tag}" in both include and exclude for "${decision.recordKey}".`,
        );
      }
      seenDecisionKeys.add(key);

      const pendingView = pendingByRecord.get(decision.recordKey) ?? {
        name: decision.name,
        recordKey: decision.recordKey,
        pending: {},
      };
      if (pendingView.name !== decision.name) {
        throw new Error(
          `Derived tag assignment reviews disagree on canonical name for "${decision.recordKey}".`,
        );
      }
      const current = pendingView.pending[validated.family] ?? [];
      current.push(validated.tag);
      pendingView.pending[validated.family] = uniqueSorted(current);
      pendingByRecord.set(decision.recordKey, pendingView);
    }
  }

  return [...pendingByRecord.values()]
    .sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey));
}

export function validateDerivedTagAssignmentMemory(
  ontology: PublishedDerivedTagOntology,
  groups: DerivedTagAssignmentMemoryGroup[] = RAW_DERIVED_TAG_ASSIGNMENT_MEMORY,
): void {
  const familyTagMap = buildFamilyTagMap(ontology.tags);
  const seenDecisionKeys = new Set<string>();

  for (const group of groups) {
    for (const decision of group.decisions) {
      validateFamilyTagReference(
        "assignment memory",
        group.category,
        familyTagMap,
        decision.recordKey,
        decision.family,
        decision.tag,
      );
      const key = memoryIdentity(decision);
      if (seenDecisionKeys.has(key)) {
        throw new Error(
          `Derived tag assignment memory repeats "${normalizeDerivedTag(decision.family)}.${normalizeDerivedTag(decision.tag)}" (${decision.mode}) for "${decision.recordKey}".`,
        );
      }
      seenDecisionKeys.add(key);
    }
  }
}

export function buildDerivedTagExplicitAssignmentIndex(
  ontology: PublishedDerivedTagOntology,
  groups: DerivedTagAssignmentGroup[] = RAW_DERIVED_TAG_ASSIGNMENTS,
): DerivedTagExplicitAssignmentIndex {
  const familyTagMap = buildFamilyTagMap(ontology.tags);
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
  ontology: PublishedDerivedTagOntology,
): DerivedTagExplicitAssignmentIndex {
  return buildDerivedTagExplicitAssignmentIndex(ontology, RAW_DERIVED_TAG_ASSIGNMENTS);
}
