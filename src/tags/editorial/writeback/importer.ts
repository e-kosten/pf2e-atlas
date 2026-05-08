import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarPolarity,
  DerivedTagExemplarRecord,
  DerivedTagExemplarReviewCategory,
  DerivedTagExemplarReviewDecision,
  SearchCategory,
} from "../../../domain/derived-tag-types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentDecision,
  DerivedTagAssignmentDecisionSource,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentMemoryDecision,
  DerivedTagAssignmentReviewCategory,
  DerivedTagAssignmentReviewDecision,
} from "../../runtime/derivation/assignments.js";
import { expectDerivedTagManagedCategory } from "../../manifest.js";
import { normalizeDerivedTag } from "../../runtime/matcher/shared.js";
import { uniqueSorted } from "../../../shared/utils.js";
import { getCurrentDerivedTagAuthoredState } from "../state/authored-state.js";
import { getPublishedDerivedTagOntology } from "../state/runtime-state.js";
import { lintDerivedTagReviewSession } from "./linter.js";
import { writeDerivedTagAuthoredState } from "./authored-state-writer.js";
import type {
  DerivedTagReviewAssignmentDecision,
  DerivedTagAuthoredState,
  DerivedTagManagedCategory,
  DerivedTagReviewRecordDecision,
  DerivedTagReviewSession,
} from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";

function storedAssignmentIdentity(
  decision: Pick<DerivedTagAssignmentReviewDecision, "recordKey" | "family" | "tag" | "mode">,
): string {
  return [
    decision.recordKey,
    normalizeDerivedTag(decision.family),
    normalizeDerivedTag(decision.tag),
    decision.mode,
  ].join("|");
}

function ensureAssignment(
  assignments: AuthoredDerivedTagAssignment[],
  recordDecision: DerivedTagReviewRecordDecision,
): AuthoredDerivedTagAssignment {
  const existing = assignments.find((assignment) => assignment.recordKey === recordDecision.recordKey);
  if (existing) {
    existing.name = recordDecision.name;
    return existing;
  }

  const created: AuthoredDerivedTagAssignment = {
    name: recordDecision.name,
    recordKey: recordDecision.recordKey,
  };
  assignments.push(created);
  return created;
}

function mapAssignmentDecisionSource(
  decision: DerivedTagReviewAssignmentDecision,
): DerivedTagAssignmentDecisionSource {
  if (decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.AUTO_APPLIED) {
    return "llm_auto";
  }
  if (decision.source === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.HUMAN) {
    return DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.HUMAN;
  }
  if (decision.source === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM) {
    return "llm_reviewed";
  }
  return DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.HUMAN;
}

function resolveProjectionIdForAssignmentDecision(
  category: SearchCategory,
  recordKey: string,
  decision: Pick<DerivedTagReviewAssignmentDecision, "family" | "tag">,
): string {
  const ontology = getPublishedDerivedTagOntology();
  const normalizedTag = normalizeDerivedTag(decision.tag);
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizedTag}`);
  if (!ontologyTag) {
    throw new Error(
      `Could not resolve assignment projection for "${recordKey}" because "${decision.tag}" is not in the published ontology.`,
    );
  }
  if (normalizeDerivedTag(ontologyTag.family) !== normalizeDerivedTag(decision.family)) {
    throw new Error(
      `Could not resolve assignment projection for "${recordKey}" because "${decision.family}.${decision.tag}" uses the wrong family.`,
    );
  }
  const projection = ontology.conceptModel.projectionsByTagKey.get(`${category}:${normalizedTag}`);
  if (!projection) {
    throw new Error(
      `Could not resolve assignment projection for "${recordKey}" because "${category}:${normalizedTag}" has no canonical projection.`,
    );
  }
  return projection.id;
}

function toStoredAssignmentDecision(
  category: SearchCategory,
  recordKey: string,
  decision: DerivedTagReviewAssignmentDecision,
): DerivedTagAssignmentDecision {
  return {
    projectionId: resolveProjectionIdForAssignmentDecision(category, recordKey, decision),
    source: mapAssignmentDecisionSource(decision),
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
  };
}

function removeAssignmentDecision(
  decisions: DerivedTagAssignmentDecision[] | undefined,
  projectionId: string,
): DerivedTagAssignmentDecision[] | undefined {
  if (!decisions) {
    return undefined;
  }

  const filtered = decisions.filter((entry) => entry.projectionId.trim() !== projectionId);
  return filtered.length > 0 ? filtered : undefined;
}

function upsertAssignmentDecision(
  decisions: DerivedTagAssignmentDecision[] | undefined,
  decision: DerivedTagAssignmentDecision,
): DerivedTagAssignmentDecision[] {
  const filtered = (decisions ?? []).filter((entry) => entry.projectionId.trim() !== decision.projectionId.trim());
  return [...filtered, decision].sort((left, right) => left.projectionId.localeCompare(right.projectionId));
}

function sortAssignmentDecisions(
  decisions: DerivedTagAssignmentDecision[] | undefined,
): DerivedTagAssignmentDecision[] | undefined {
  if (!decisions) {
    return undefined;
  }

  return decisions
    .map((decision) => ({ ...decision, projectionId: decision.projectionId.trim() }))
    .sort((left, right) => left.projectionId.localeCompare(right.projectionId));
}

function cleanAssignment(assignment: AuthoredDerivedTagAssignment): AuthoredDerivedTagAssignment | null {
  assignment.applied = sortAssignmentDecisions(assignment.applied);
  assignment.excluded = sortAssignmentDecisions(assignment.excluded);
  if (!assignment.applied && !assignment.excluded) {
    return null;
  }
  return assignment;
}

function applyLiveAssignmentDecision(
  assignment: AuthoredDerivedTagAssignment,
  category: SearchCategory,
  recordKey: string,
  decision: DerivedTagReviewAssignmentDecision,
): void {
  const storedDecision = toStoredAssignmentDecision(category, recordKey, decision);
  const projectionId = storedDecision.projectionId;

  if (decision.mode === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.INCLUDE) {
    assignment.excluded = removeAssignmentDecision(assignment.excluded, projectionId);
    assignment.applied = upsertAssignmentDecision(assignment.applied, storedDecision);
    return;
  }

  assignment.applied = removeAssignmentDecision(assignment.applied, projectionId);
  assignment.excluded = upsertAssignmentDecision(assignment.excluded, storedDecision);
}

function sortAssignments(assignments: AuthoredDerivedTagAssignment[]): AuthoredDerivedTagAssignment[] {
  return assignments
    .map((assignment) => cleanAssignment(assignment))
    .filter((assignment): assignment is AuthoredDerivedTagAssignment => assignment !== null)
    .sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey));
}

export function applyMigrationSessionToAssignments(
  assignments: AuthoredDerivedTagAssignment[],
  sessionDecisions: DerivedTagReviewRecordDecision[],
): AuthoredDerivedTagAssignment[] {
  const nextAssignments = structuredClone(assignments);

  for (const recordDecision of sessionDecisions) {
    const assignmentDecisions = recordDecision.decisions.filter(
      (decision): decision is DerivedTagReviewAssignmentDecision =>
        decision.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT &&
        (decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.APPROVED || decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.AUTO_APPLIED),
    );
    if (assignmentDecisions.length === 0) {
      continue;
    }
    const assignment = ensureAssignment(nextAssignments, recordDecision);
    for (const decision of assignmentDecisions) {
      applyLiveAssignmentDecision(assignment, recordDecision.category, recordDecision.recordKey, decision);
    }
  }

  return sortAssignments(nextAssignments);
}

function toAssignmentReviewDecision(
  recordDecision: DerivedTagReviewRecordDecision,
  decision: DerivedTagReviewAssignmentDecision,
): DerivedTagAssignmentReviewDecision {
  return {
    name: recordDecision.name,
    recordKey: recordDecision.recordKey,
    family: normalizeDerivedTag(decision.family),
    tag: normalizeDerivedTag(decision.tag),
    mode: decision.mode,
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
    ...(decision.source ? { source: decision.source } : {}),
  };
}

function sortAssignmentReviewCategory(
  assignmentReviews: DerivedTagAssignmentReviewCategory,
): DerivedTagAssignmentReviewCategory {
  assignmentReviews.decisions = [...assignmentReviews.decisions].sort(
    (left, right) =>
      normalizeDerivedTag(left.family).localeCompare(normalizeDerivedTag(right.family)) ||
      normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)) ||
      left.name.localeCompare(right.name) ||
      left.recordKey.localeCompare(right.recordKey) ||
      left.mode.localeCompare(right.mode),
  );
  return assignmentReviews;
}

export function applyMigrationSessionToAssignmentReviews(
  assignmentReviews: DerivedTagAssignmentReviewCategory,
  sessionDecisions: DerivedTagReviewRecordDecision[],
): DerivedTagAssignmentReviewCategory {
  const nextAssignmentReviews = structuredClone(assignmentReviews);
  const decisionsByIdentity = new Map(
    nextAssignmentReviews.decisions.map((decision) => [storedAssignmentIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is DerivedTagReviewAssignmentDecision =>
        entry.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
    )) {
      const storedDecision = toAssignmentReviewDecision(recordDecision, decision);
      const identity = storedAssignmentIdentity(storedDecision);
      if (decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW) {
        decisionsByIdentity.set(identity, storedDecision);
        continue;
      }
      decisionsByIdentity.delete(identity);
    }
  }

  nextAssignmentReviews.decisions = [...decisionsByIdentity.values()];
  return sortAssignmentReviewCategory(nextAssignmentReviews);
}

function toAssignmentMemoryDecision(
  recordDecision: DerivedTagReviewRecordDecision,
  decision: DerivedTagReviewAssignmentDecision,
): DerivedTagAssignmentMemoryDecision {
  return {
    name: recordDecision.name,
    recordKey: recordDecision.recordKey,
    family: normalizeDerivedTag(decision.family),
    tag: normalizeDerivedTag(decision.tag),
    mode: decision.mode,
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
    ...(decision.source ? { source: decision.source } : {}),
  };
}

function sortAssignmentMemoryCategory(
  assignmentMemory: DerivedTagAssignmentMemoryCategory,
): DerivedTagAssignmentMemoryCategory {
  assignmentMemory.decisions = [...assignmentMemory.decisions].sort(
    (left, right) =>
      normalizeDerivedTag(left.family).localeCompare(normalizeDerivedTag(right.family)) ||
      normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)) ||
      left.name.localeCompare(right.name) ||
      left.recordKey.localeCompare(right.recordKey) ||
      left.mode.localeCompare(right.mode),
  );
  return assignmentMemory;
}

export function applyMigrationSessionToAssignmentMemory(
  assignmentMemory: DerivedTagAssignmentMemoryCategory,
  sessionDecisions: DerivedTagReviewRecordDecision[],
): DerivedTagAssignmentMemoryCategory {
  const nextAssignmentMemory = structuredClone(assignmentMemory);
  const decisionsByIdentity = new Map(
    nextAssignmentMemory.decisions.map((decision) => [memoryIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is DerivedTagReviewAssignmentDecision =>
        entry.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
    )) {
      const storedDecision = toAssignmentMemoryDecision(recordDecision, decision);
      const identity = memoryIdentity(storedDecision);
      if (decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.REJECTED) {
        decisionsByIdentity.set(identity, storedDecision);
        continue;
      }
      decisionsByIdentity.delete(identity);
    }
  }

  nextAssignmentMemory.decisions = [...decisionsByIdentity.values()];
  return sortAssignmentMemoryCategory(nextAssignmentMemory);
}

function ensureExemplarSet(
  exemplars: DerivedTagExemplarCategory,
  tag: string,
): { tag: string; positives?: DerivedTagExemplarRecord[]; negatives?: DerivedTagExemplarRecord[]; notes?: string } {
  let exemplarSet = exemplars.exemplars.find((entry) => normalizeDerivedTag(entry.tag) === normalizeDerivedTag(tag));
  if (!exemplarSet) {
    exemplarSet = { tag: normalizeDerivedTag(tag), positives: [], negatives: [] };
    exemplars.exemplars.push(exemplarSet);
  }
  exemplarSet.positives = exemplarSet.positives ?? [];
  exemplarSet.negatives = exemplarSet.negatives ?? [];
  return exemplarSet;
}

function sortExemplarRecords(records: DerivedTagExemplarRecord[]): DerivedTagExemplarRecord[] {
  return [...records].sort(
    (left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey),
  );
}

function withoutExemplarRecord(
  records: DerivedTagExemplarRecord[] | undefined,
  recordKey: string,
): DerivedTagExemplarRecord[] {
  return (records ?? []).filter((record) => record.recordKey !== recordKey);
}

function upsertExemplarRecord(
  records: DerivedTagExemplarRecord[] | undefined,
  record: DerivedTagExemplarRecord,
): DerivedTagExemplarRecord[] {
  return [...withoutExemplarRecord(records, record.recordKey), record];
}

function applyApprovedExemplarDecision(
  exemplars: DerivedTagExemplarCategory,
  recordDecision: DerivedTagReviewRecordDecision,
  decision: Extract<
    DerivedTagReviewRecordDecision["decisions"][number],
    { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR }
  >,
): void {
  const exemplarSet = ensureExemplarSet(exemplars, decision.tag);
  const record = { recordKey: recordDecision.recordKey, name: recordDecision.name };

  if (decision.action === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP) {
    exemplarSet.positives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.positives, recordDecision.recordKey));
    exemplarSet.negatives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.negatives, recordDecision.recordKey));
    return;
  }

  if (decision.polarity === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE) {
    exemplarSet.positives = sortExemplarRecords(upsertExemplarRecord(exemplarSet.positives, record));
    exemplarSet.negatives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.negatives, recordDecision.recordKey));
  } else {
    exemplarSet.negatives = sortExemplarRecords(upsertExemplarRecord(exemplarSet.negatives, record));
    exemplarSet.positives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.positives, recordDecision.recordKey));
  }
}

export function applyMigrationSessionToExemplars(
  exemplars: DerivedTagExemplarCategory,
  sessionDecisions: DerivedTagReviewRecordDecision[],
): DerivedTagExemplarCategory {
  const nextExemplars = structuredClone(exemplars);

  for (const recordDecision of sessionDecisions) {
    const exemplarDecisions = recordDecision.decisions.filter(
      (
        decision,
      ): decision is Extract<
        DerivedTagReviewRecordDecision["decisions"][number],
        { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR }
      > => decision.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
    );
    for (const decision of exemplarDecisions) {
      if (
        decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.APPROVED &&
        decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.AUTO_APPLIED
      ) {
        continue;
      }
      applyApprovedExemplarDecision(nextExemplars, recordDecision, decision);
    }
  }

  nextExemplars.exemplars = nextExemplars.exemplars
    .map((entry) => ({
      ...entry,
      positives: sortExemplarRecords(entry.positives ?? []),
      negatives: sortExemplarRecords(entry.negatives ?? []),
    }))
    .filter((entry) => entry.positives.length > 0 || entry.negatives.length > 0 || entry.notes)
    .sort((left, right) => normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)));

  return nextExemplars;
}

function exemplarReviewIdentity(decision: Pick<DerivedTagExemplarReviewDecision, "recordKey" | "tag">): string {
  return `${decision.recordKey}:${normalizeDerivedTag(decision.tag)}`;
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

function toProposedPolarity(
  decision: Extract<
    DerivedTagReviewRecordDecision["decisions"][number],
    { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR }
  >,
): DerivedTagExemplarPolarity | typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP {
  return decision.action === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP
    ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.DROP
    : decision.polarity;
}

function toExemplarReviewDecision(
  recordDecision: DerivedTagReviewRecordDecision,
  decision: Extract<
    DerivedTagReviewRecordDecision["decisions"][number],
    { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR }
  >,
): DerivedTagExemplarReviewDecision {
  return {
    name: recordDecision.name,
    recordKey: recordDecision.recordKey,
    tag: normalizeDerivedTag(decision.tag),
    proposedPolarity: toProposedPolarity(decision),
    status:
      decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.AUTO_APPLIED ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.APPROVED : decision.status,
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
    ...(decision.source ? { source: decision.source } : {}),
    ...(decision.currentPolarity ? { currentPolarity: decision.currentPolarity } : {}),
  };
}

export function applyMigrationSessionToExemplarReviews(
  exemplarReviews: DerivedTagExemplarReviewCategory,
  sessionDecisions: DerivedTagReviewRecordDecision[],
): DerivedTagExemplarReviewCategory {
  const nextExemplarReviews = structuredClone(exemplarReviews);
  const decisionsByIdentity = new Map(
    nextExemplarReviews.decisions.map((decision) => [exemplarReviewIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (
        entry,
      ): entry is Extract<
        DerivedTagReviewRecordDecision["decisions"][number],
        { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR }
      > => entry.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
    )) {
      const reviewDecision = toExemplarReviewDecision(recordDecision, decision);
      const identity = exemplarReviewIdentity(reviewDecision);
      if (decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW) {
        decisionsByIdentity.set(identity, reviewDecision);
        continue;
      }
      decisionsByIdentity.delete(identity);
    }
  }

  nextExemplarReviews.decisions = [...decisionsByIdentity.values()].sort(
    (left, right) =>
      normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)) ||
      left.name.localeCompare(right.name) ||
      left.recordKey.localeCompare(right.recordKey),
  );

  return nextExemplarReviews;
}

function ruleIdentity(rule: AuthoredDerivedTagRule): string {
  return JSON.stringify(rule);
}

export function applyMigrationSessionToAuthoredRules(
  rules: AuthoredDerivedTagRule[],
  sessionDecisions: DerivedTagReviewRecordDecision[],
): AuthoredDerivedTagRule[] {
  const nextRules = [...rules];
  const seenRules = new Set(nextRules.map(ruleIdentity));

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (
        entry,
      ): entry is Extract<
        DerivedTagReviewRecordDecision["decisions"][number],
        { kind: typeof DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.RULE }
      > => entry.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.RULE,
    )) {
      if (
        decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.APPROVED &&
        decision.status !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.AUTO_APPLIED
      ) {
        continue;
      }
      if (decision.decision !== DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RULE_DECISION.RECREATE_AUTHORED) {
        continue;
      }
      for (const rule of decision.authoredRules ?? []) {
        const identity = ruleIdentity(rule);
        if (seenRules.has(identity)) {
          continue;
        }
        seenRules.add(identity);
        nextRules.push(rule);
      }
    }
  }

  return nextRules.sort(
    (left, right) =>
      normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)) ||
      left.kind.localeCompare(right.kind) ||
      JSON.stringify(left).localeCompare(JSON.stringify(right)),
  );
}

function toManagedCategory(category: SearchCategory): DerivedTagManagedCategory {
  return expectDerivedTagManagedCategory(category, "Derived-tag migration importer");
}

function categoriesTouchedBySession(session: DerivedTagReviewSession): DerivedTagManagedCategory[] {
  return uniqueSorted(session.decisions.map((decision) => toManagedCategory(decision.category)));
}

function applySessionToState(
  state: DerivedTagAuthoredState,
  session: DerivedTagReviewSession,
): DerivedTagAuthoredState {
  const nextState = structuredClone(state);

  for (const category of categoriesTouchedBySession(session)) {
    const relevantDecisions = session.decisions.filter((decision) => decision.category === category);
    nextState.assignments[category] = applyMigrationSessionToAssignments(
      nextState.assignments[category],
      relevantDecisions,
    );
    nextState.assignmentReviews[category] = applyMigrationSessionToAssignmentReviews(
      nextState.assignmentReviews[category],
      relevantDecisions,
    );
    nextState.assignmentMemory[category] = applyMigrationSessionToAssignmentMemory(
      nextState.assignmentMemory[category],
      relevantDecisions,
    );
    nextState.exemplars[category] = applyMigrationSessionToExemplars(nextState.exemplars[category], relevantDecisions);
    nextState.exemplarReviews[category] = applyMigrationSessionToExemplarReviews(
      nextState.exemplarReviews[category],
      relevantDecisions,
    );
    nextState.authoredRules[category] = applyMigrationSessionToAuthoredRules(
      nextState.authoredRules[category],
      relevantDecisions,
    );
  }

  return nextState;
}

export async function importDerivedTagReviewSession(
  rootPath: string,
  session: DerivedTagReviewSession,
): Promise<void> {
  lintDerivedTagReviewSession(session);
  const currentState = getCurrentDerivedTagAuthoredState();
  const nextState = applySessionToState(currentState, session);
  await writeDerivedTagAuthoredState(rootPath, nextState, categoriesTouchedBySession(session));
}
