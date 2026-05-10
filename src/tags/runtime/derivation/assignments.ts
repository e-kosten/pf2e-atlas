import type { DerivedTagCategoryProjection, SearchCategory } from "../../../domain/derived-tag-types.js";
import { uniqueSorted } from "../../../shared/utils.js";
import { DERIVED_TAG_ASSIGNMENTS } from "../../assignments/index.js";
import { normalizeDerivedTag } from "../matcher/engine.js";
import type { PublishedDerivedTagOntology } from "../publication/catalog.js";

export type DerivedTagReviewStatus = "auto_applied" | "needs_review" | "approved" | "rejected";

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
  applied?: DerivedTagAssignmentDecision[];
  excluded?: DerivedTagAssignmentDecision[];
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

type DerivedTagAssignmentReviewGroup = DerivedTagAssignmentReviewCategory;
type DerivedTagAssignmentMemoryGroup = DerivedTagAssignmentMemoryCategory;

export type DerivedTagExplicitAssignment = {
  category?: SearchCategory;
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

const RAW_DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = DERIVED_TAG_ASSIGNMENTS;

type NormalizedProjectionDecisionMap = Map<string, DerivedTagAssignmentDecision>;

function createEmptyProjectionDecisionMap(): NormalizedProjectionDecisionMap {
  return new Map<string, DerivedTagAssignmentDecision>();
}

function normalizeAssignmentDecision(decision: DerivedTagAssignmentDecision): DerivedTagAssignmentDecision {
  return {
    ...decision,
    tag: normalizeDerivedTag(decision.tag),
  };
}

function renderProjectionTag(projection: DerivedTagCategoryProjection): string {
  return `${normalizeDerivedTag(projection.family)}.${normalizeDerivedTag(projection.currentTag)}`;
}

function validateProjectionTagReference(
  fieldName: string,
  category: SearchCategory,
  recordKey: string,
  tag: string,
  ontology: PublishedDerivedTagOntology,
): DerivedTagCategoryProjection {
  const normalizedTag = normalizeDerivedTag(tag);
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizedTag}`);
  if (!ontologyTag) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" does not exist in category "${category}".`,
    );
  }
  const projection = ontology.conceptModel.projectionsByTagKey.get(`${category}:${normalizedTag}`);
  if (!projection) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" is missing a canonical projection in category "${category}".`,
    );
  }
  if (projection.isComposite) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" cannot target composite tag "${projection.currentTag}"; assign one of its child tags instead.`,
    );
  }
  return projection;
}

function normalizeProjectionAssignments(
  decisions: DerivedTagAssignmentDecision[] | undefined,
  category: SearchCategory,
  fieldName: "applied" | "excluded",
  recordKey: string,
  ontology: PublishedDerivedTagOntology,
): NormalizedProjectionDecisionMap {
  const normalizedAssignments = createEmptyProjectionDecisionMap();
  if (!decisions) {
    return normalizedAssignments;
  }

  for (const rawDecision of decisions) {
    const normalizedDecision = normalizeAssignmentDecision(rawDecision);
    const projection = validateProjectionTagReference(
      `assignment ${fieldName}`,
      category,
      recordKey,
      normalizedDecision.tag,
      ontology,
    );
    if (normalizedAssignments.has(projection.id)) {
      throw new Error(
        `Derived tag assignment ${fieldName} for "${recordKey}" repeats "${renderProjectionTag(projection)}".`,
      );
    }
    normalizedAssignments.set(projection.id, normalizedDecision);
  }

  return normalizedAssignments;
}

function flattenNormalizedAssignments(
  assignments: NormalizedProjectionDecisionMap,
  projectionsById: PublishedDerivedTagOntology["conceptModel"]["projectionsById"],
): string[] {
  const flattenedTags = new Set<string>();
  for (const projectionId of assignments.keys()) {
    const projection = projectionsById.get(projectionId);
    if (!projection) {
      throw new Error(`Missing projection "${projectionId}" while flattening explicit derived tag assignments.`);
    }
    flattenedTags.add(normalizeDerivedTag(projection.currentTag));
  }

  return uniqueSorted([...flattenedTags]);
}

function normalizeAssignment(
  assignment: AuthoredDerivedTagAssignment,
  category: SearchCategory,
  ontology: PublishedDerivedTagOntology,
): {
  includeDecisions: NormalizedProjectionDecisionMap;
  excludeDecisions: NormalizedProjectionDecisionMap;
} {
  const includeDecisions = normalizeProjectionAssignments(
    assignment.applied,
    category,
    "applied",
    assignment.recordKey,
    ontology,
  );
  const excludeDecisions = normalizeProjectionAssignments(
    assignment.excluded,
    category,
    "excluded",
    assignment.recordKey,
    ontology,
  );

  for (const projectionId of uniqueSorted([...includeDecisions.keys()])) {
    if (excludeDecisions.has(projectionId)) {
      const projection = ontology.conceptModel.projectionsById.get(projectionId);
      const renderedTarget = projection ? renderProjectionTag(projection) : projectionId;
      throw new Error(
        `Derived tag assignment for "${assignment.recordKey}" places "${renderedTarget}" in both applied and excluded.`,
      );
    }
  }

  if (includeDecisions.size === 0 && excludeDecisions.size === 0) {
    throw new Error(
      `Derived tag assignment for "${assignment.recordKey}" must include at least one applied or excluded tag.`,
    );
  }

  return {
    includeDecisions,
    excludeDecisions,
  };
}

function validateReviewTagReference(
  fieldName: string,
  recordKey: string,
  category: SearchCategory,
  family: string,
  tag: string,
  ontology: PublishedDerivedTagOntology,
): { family: string; tag: string; projection: DerivedTagCategoryProjection } {
  const normalizedFamily = normalizeDerivedTag(family);
  const normalizedTag = normalizeDerivedTag(tag);
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizedTag}`);
  if (!ontologyTag) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" does not exist in category "${category}".`,
    );
  }
  if (normalizeDerivedTag(ontologyTag.family) !== normalizedFamily) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" does not belong to family "${family}" in category "${category}".`,
    );
  }
  if (ontologyTag.isComposite) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" cannot target composite tag "${ontologyTag.tag}"; assign one of its child tags instead.`,
    );
  }
  const projection = ontology.conceptModel.projectionsByTagKey.get(`${category}:${normalizedTag}`);
  if (!projection) {
    throw new Error(
      `Derived tag ${fieldName} tag "${tag}" for "${recordKey}" is missing a canonical projection in category "${category}".`,
    );
  }
  return { family: normalizedFamily, tag: normalizedTag, projection };
}

function reviewIdentity(
  decision: Pick<DerivedTagAssignmentReviewDecision, "recordKey" | "family" | "tag" | "mode">,
): string {
  return [
    decision.recordKey,
    normalizeDerivedTag(decision.family),
    normalizeDerivedTag(decision.tag),
    decision.mode,
  ].join("|");
}

function memoryIdentity(
  decision: Pick<DerivedTagAssignmentMemoryDecision, "recordKey" | "family" | "tag" | "mode">,
): string {
  return [
    decision.recordKey,
    normalizeDerivedTag(decision.family),
    normalizeDerivedTag(decision.tag),
    decision.mode,
  ].join("|");
}

export function buildDerivedTagPendingAssignmentViews(
  ontology: PublishedDerivedTagOntology,
  groups: DerivedTagAssignmentReviewGroup[] = [],
): DerivedTagPendingAssignmentView[] {
  const seenDecisionKeys = new Set<string>();
  const pendingByRecord = new Map<string, DerivedTagPendingAssignmentView>();

  for (const group of groups) {
    for (const decision of group.decisions) {
      const validated = validateReviewTagReference(
        "assignment review",
        decision.recordKey,
        group.category,
        decision.family,
        decision.tag,
        ontology,
      );
      const key = reviewIdentity(decision);
      if (seenDecisionKeys.has(key)) {
        throw new Error(
          `Derived tag assignment review repeats "${validated.family}.${validated.tag}" (${decision.mode}) for "${decision.recordKey}".`,
        );
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
        throw new Error(`Derived tag assignment reviews disagree on canonical name for "${decision.recordKey}".`);
      }
      const current = pendingView.pending[validated.family] ?? [];
      current.push(validated.tag);
      pendingView.pending[validated.family] = uniqueSorted(current);
      pendingByRecord.set(decision.recordKey, pendingView);
    }
  }

  return [...pendingByRecord.values()].sort(
    (left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey),
  );
}

export function validateDerivedTagAssignmentMemory(
  ontology: PublishedDerivedTagOntology,
  groups: DerivedTagAssignmentMemoryGroup[] = [],
): void {
  const seenDecisionKeys = new Set<string>();

  for (const group of groups) {
    for (const decision of group.decisions) {
      validateReviewTagReference(
        "assignment memory",
        decision.recordKey,
        group.category,
        decision.family,
        decision.tag,
        ontology,
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
  assignments: AuthoredDerivedTagAssignment[] = RAW_DERIVED_TAG_ASSIGNMENTS,
  records: Iterable<DerivedTagAssignmentRecordSummary> = [],
  options: { requireCompleteRecordCoverage?: boolean } = {},
): DerivedTagExplicitAssignmentIndex {
  const projectionsById = ontology.conceptModel.projectionsById;
  const recordsByKey = new Map<string, DerivedTagAssignmentRecordSummary>();
  for (const record of records) {
    recordsByKey.set(record.recordKey, record);
  }
  const assignmentsByRecordKey = new Map<string, DerivedTagExplicitAssignment>();

  for (const assignment of assignments) {
    if (assignmentsByRecordKey.has(assignment.recordKey)) {
      throw new Error(`Duplicate explicit derived tag assignment for "${assignment.recordKey}".`);
    }

    const record = recordsByKey.get(assignment.recordKey);
    if (recordsByKey.size > 0 && !record && options.requireCompleteRecordCoverage) {
      throw new Error(`Cannot resolve explicit derived tag assignment category for "${assignment.recordKey}".`);
    }
    if (record && record.name !== assignment.name) {
      throw new Error(
        `Explicit derived tag assignment for "${assignment.recordKey}" expected name "${assignment.name}" but resolved canonical name "${record.name}".`,
      );
    }

    if (!record) {
      const includeTags = uniqueSorted((assignment.applied ?? []).map((decision) => normalizeDerivedTag(decision.tag)));
      const excludeTags = uniqueSorted((assignment.excluded ?? []).map((decision) => normalizeDerivedTag(decision.tag)));
      if (includeTags.length === 0 && excludeTags.length === 0) {
        throw new Error(
          `Derived tag assignment for "${assignment.recordKey}" must include at least one applied or excluded tag.`,
        );
      }
      assignmentsByRecordKey.set(assignment.recordKey, {
        name: assignment.name,
        includeTags,
        excludeTags,
      });
      continue;
    }

    const normalizedAssignment = normalizeAssignment(assignment, record.category, ontology);

    assignmentsByRecordKey.set(assignment.recordKey, {
      category: record.category,
      name: assignment.name,
      includeTags: flattenNormalizedAssignments(normalizedAssignment.includeDecisions, projectionsById),
      excludeTags: flattenNormalizedAssignments(normalizedAssignment.excludeDecisions, projectionsById),
    });
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
    if (!assignment.category) {
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
  records: Iterable<DerivedTagAssignmentRecordSummary> = [],
): DerivedTagExplicitAssignmentIndex {
  return buildDerivedTagExplicitAssignmentIndex(ontology, RAW_DERIVED_TAG_ASSIGNMENTS, records);
}
