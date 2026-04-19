import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarPolarity,
  DerivedTagExemplarRecord,
  DerivedTagExemplarReviewCategory,
  DerivedTagExemplarReviewDecision,
  SearchCategory,
} from "../../types.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentDecision,
  DerivedTagAssignmentDecisionSource,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentMemoryDecision,
  DerivedTagAssignmentReviewCategory,
  DerivedTagAssignmentReviewDecision,
} from "../runtime/assignments.js";
import { normalizeDerivedTag } from "../runtime/shared.js";
import { uniqueSorted } from "../../utils.js";
import { getCurrentDerivedTagMigrationAuthoredState, writeDerivedTagMigrationAuthoredState } from "./authored-state.js";
import { lintDerivedTagMigrationSession } from "./linter.js";
import type {
  DerivedTagMigrationAssignmentDecision,
  DerivedTagMigrationAuthoredState,
  DerivedTagManagedCategory,
  DerivedTagMigrationRecordDecision,
  DerivedTagMigrationSession,
} from "./types.js";

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
  recordDecision: DerivedTagMigrationRecordDecision,
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
  decision: DerivedTagMigrationAssignmentDecision,
): DerivedTagAssignmentDecisionSource {
  if (decision.status === "auto_applied") {
    return "llm_auto";
  }
  if (decision.source === "human") {
    return "human";
  }
  if (decision.source === "llm") {
    return "llm_reviewed";
  }
  return "human";
}

function toStoredAssignmentDecision(decision: DerivedTagMigrationAssignmentDecision): DerivedTagAssignmentDecision {
  return {
    tag: normalizeDerivedTag(decision.tag),
    source: mapAssignmentDecisionSource(decision),
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
  };
}

function removeAssignmentTag(
  groupedAssignments: Record<string, DerivedTagAssignmentDecision[]> | undefined,
  family: string,
  tag: string,
): Record<string, DerivedTagAssignmentDecision[]> | undefined {
  if (!groupedAssignments) {
    return undefined;
  }

  const nextAssignments = Object.fromEntries(
    Object.entries(groupedAssignments)
      .map(([currentFamily, decisions]) => {
        if (normalizeDerivedTag(currentFamily) !== family) {
          return [currentFamily, decisions] as const;
        }
        const filtered = decisions.filter((entry) => normalizeDerivedTag(entry.tag) !== tag);
        return [currentFamily, filtered] as const;
      })
      .filter(([, decisions]) => decisions.length > 0),
  );

  return Object.keys(nextAssignments).length > 0 ? nextAssignments : undefined;
}

function upsertAssignmentTag(
  groupedAssignments: Record<string, DerivedTagAssignmentDecision[]> | undefined,
  family: string,
  decision: DerivedTagAssignmentDecision,
): Record<string, DerivedTagAssignmentDecision[]> {
  const current = groupedAssignments?.[family] ?? [];
  const filtered = current.filter((entry) => normalizeDerivedTag(entry.tag) !== normalizeDerivedTag(decision.tag));
  const nextFamilyAssignments = [...filtered, decision].sort((left, right) =>
    normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)),
  );

  return {
    ...(groupedAssignments ?? {}),
    [family]: nextFamilyAssignments,
  };
}

function sortGroupedAssignmentDecisions(
  groupedAssignments: Record<string, DerivedTagAssignmentDecision[]> | undefined,
): Record<string, DerivedTagAssignmentDecision[]> | undefined {
  if (!groupedAssignments) {
    return undefined;
  }

  return Object.fromEntries(
    Object.entries(groupedAssignments)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([family, decisions]) => [
        family,
        decisions
          .map((decision) => ({ ...decision, tag: normalizeDerivedTag(decision.tag) }))
          .sort((left, right) => normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag))),
      ]),
  );
}

function cleanAssignment(assignment: AuthoredDerivedTagAssignment): AuthoredDerivedTagAssignment | null {
  assignment.applied = sortGroupedAssignmentDecisions(assignment.applied);
  assignment.excluded = sortGroupedAssignmentDecisions(assignment.excluded);
  if (!assignment.applied && !assignment.excluded) {
    return null;
  }
  return assignment;
}

function applyLiveAssignmentDecision(
  assignment: AuthoredDerivedTagAssignment,
  decision: DerivedTagMigrationAssignmentDecision,
): void {
  const family = normalizeDerivedTag(decision.family);
  const tag = normalizeDerivedTag(decision.tag);
  const storedDecision = toStoredAssignmentDecision(decision);

  if (decision.mode === "include") {
    assignment.excluded = removeAssignmentTag(assignment.excluded, family, tag);
    assignment.applied = upsertAssignmentTag(assignment.applied, family, storedDecision);
    return;
  }

  assignment.applied = removeAssignmentTag(assignment.applied, family, tag);
  assignment.excluded = upsertAssignmentTag(assignment.excluded, family, storedDecision);
}

function sortAssignments(assignments: AuthoredDerivedTagAssignment[]): AuthoredDerivedTagAssignment[] {
  return assignments
    .map((assignment) => cleanAssignment(assignment))
    .filter((assignment): assignment is AuthoredDerivedTagAssignment => assignment !== null)
    .sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey));
}

export function applyMigrationSessionToAssignments(
  assignments: AuthoredDerivedTagAssignment[],
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): AuthoredDerivedTagAssignment[] {
  const nextAssignments = structuredClone(assignments);

  for (const recordDecision of sessionDecisions) {
    const assignmentDecisions = recordDecision.decisions.filter(
      (decision): decision is DerivedTagMigrationAssignmentDecision =>
        decision.kind === "assignment" && (decision.status === "approved" || decision.status === "auto_applied"),
    );
    if (assignmentDecisions.length === 0) {
      continue;
    }
    const assignment = ensureAssignment(nextAssignments, recordDecision);
    for (const decision of assignmentDecisions) {
      applyLiveAssignmentDecision(assignment, decision);
    }
  }

  return sortAssignments(nextAssignments);
}

function toAssignmentReviewDecision(
  recordDecision: DerivedTagMigrationRecordDecision,
  decision: DerivedTagMigrationAssignmentDecision,
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
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): DerivedTagAssignmentReviewCategory {
  const nextAssignmentReviews = structuredClone(assignmentReviews);
  const decisionsByIdentity = new Map(
    nextAssignmentReviews.decisions.map((decision) => [storedAssignmentIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is DerivedTagMigrationAssignmentDecision => entry.kind === "assignment",
    )) {
      const storedDecision = toAssignmentReviewDecision(recordDecision, decision);
      const identity = storedAssignmentIdentity(storedDecision);
      if (decision.status === "needs_review") {
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
  recordDecision: DerivedTagMigrationRecordDecision,
  decision: DerivedTagMigrationAssignmentDecision,
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
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): DerivedTagAssignmentMemoryCategory {
  const nextAssignmentMemory = structuredClone(assignmentMemory);
  const decisionsByIdentity = new Map(
    nextAssignmentMemory.decisions.map((decision) => [memoryIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is DerivedTagMigrationAssignmentDecision => entry.kind === "assignment",
    )) {
      const storedDecision = toAssignmentMemoryDecision(recordDecision, decision);
      const identity = memoryIdentity(storedDecision);
      if (decision.status === "rejected") {
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
  recordDecision: DerivedTagMigrationRecordDecision,
  decision: Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "exemplar" }>,
): void {
  const exemplarSet = ensureExemplarSet(exemplars, decision.tag);
  const record = { recordKey: recordDecision.recordKey, name: recordDecision.name };

  if (decision.action === "drop") {
    exemplarSet.positives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.positives, recordDecision.recordKey));
    exemplarSet.negatives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.negatives, recordDecision.recordKey));
    return;
  }

  if (decision.polarity === "positive") {
    exemplarSet.positives = sortExemplarRecords(upsertExemplarRecord(exemplarSet.positives, record));
    exemplarSet.negatives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.negatives, recordDecision.recordKey));
  } else {
    exemplarSet.negatives = sortExemplarRecords(upsertExemplarRecord(exemplarSet.negatives, record));
    exemplarSet.positives = sortExemplarRecords(withoutExemplarRecord(exemplarSet.positives, recordDecision.recordKey));
  }
}

export function applyMigrationSessionToExemplars(
  exemplars: DerivedTagExemplarCategory,
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): DerivedTagExemplarCategory {
  const nextExemplars = structuredClone(exemplars);

  for (const recordDecision of sessionDecisions) {
    const exemplarDecisions = recordDecision.decisions.filter(
      (decision): decision is Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "exemplar" }> =>
        decision.kind === "exemplar",
    );
    for (const decision of exemplarDecisions) {
      if (decision.status !== "approved" && decision.status !== "auto_applied") {
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
  decision: Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "exemplar" }>,
): DerivedTagExemplarPolarity | "drop" {
  return decision.action === "drop" ? "drop" : decision.polarity;
}

function toExemplarReviewDecision(
  recordDecision: DerivedTagMigrationRecordDecision,
  decision: Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "exemplar" }>,
): DerivedTagExemplarReviewDecision {
  return {
    name: recordDecision.name,
    recordKey: recordDecision.recordKey,
    tag: normalizeDerivedTag(decision.tag),
    proposedPolarity: toProposedPolarity(decision),
    status: decision.status === "auto_applied" ? "approved" : decision.status,
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
    ...(decision.source ? { source: decision.source } : {}),
    ...(decision.currentPolarity ? { currentPolarity: decision.currentPolarity } : {}),
  };
}

export function applyMigrationSessionToExemplarReviews(
  exemplarReviews: DerivedTagExemplarReviewCategory,
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): DerivedTagExemplarReviewCategory {
  const nextExemplarReviews = structuredClone(exemplarReviews);
  const decisionsByIdentity = new Map(
    nextExemplarReviews.decisions.map((decision) => [exemplarReviewIdentity(decision), decision] as const),
  );

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "exemplar" }> =>
        entry.kind === "exemplar",
    )) {
      const reviewDecision = toExemplarReviewDecision(recordDecision, decision);
      const identity = exemplarReviewIdentity(reviewDecision);
      if (decision.status === "needs_review") {
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
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): AuthoredDerivedTagRule[] {
  const nextRules = [...rules];
  const seenRules = new Set(nextRules.map(ruleIdentity));

  for (const recordDecision of sessionDecisions) {
    for (const decision of recordDecision.decisions.filter(
      (entry): entry is Extract<DerivedTagMigrationRecordDecision["decisions"][number], { kind: "rule" }> =>
        entry.kind === "rule",
    )) {
      if (decision.status !== "approved" && decision.status !== "auto_applied") {
        continue;
      }
      if (decision.decision !== "recreate_authored") {
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
  if (
    category !== "affliction" &&
    category !== "creature" &&
    category !== "equipment" &&
    category !== "hazard" &&
    category !== "spell"
  ) {
    throw new Error(`Derived-tag migration importer does not manage category "${category}".`);
  }
  return category;
}

function categoriesTouchedBySession(session: DerivedTagMigrationSession): DerivedTagManagedCategory[] {
  return uniqueSorted(session.decisions.map((decision) => toManagedCategory(decision.category)));
}

function applySessionToState(
  state: DerivedTagMigrationAuthoredState,
  session: DerivedTagMigrationSession,
): DerivedTagMigrationAuthoredState {
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

export async function importDerivedTagMigrationSession(
  rootPath: string,
  session: DerivedTagMigrationSession,
): Promise<void> {
  lintDerivedTagMigrationSession(session);
  const currentState = getCurrentDerivedTagMigrationAuthoredState();
  const nextState = applySessionToState(currentState, session);
  await writeDerivedTagMigrationAuthoredState(rootPath, nextState, categoriesTouchedBySession(session));
}
