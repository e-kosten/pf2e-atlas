import type { AuthoredDerivedTagRule, DerivedTagExemplarCategory, DerivedTagExemplarRecord, SearchCategory } from "../../types.js";
import type { AuthoredDerivedTagAssignment, DerivedTagReviewEntry } from "../runtime/assignments.js";
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

function setReviewEntry(
  assignment: AuthoredDerivedTagAssignment,
  decision: DerivedTagMigrationAssignmentDecision,
): void {
  const family = normalizeDerivedTag(decision.family);
  const tag = normalizeDerivedTag(decision.tag);
  assignment.review = assignment.review ?? {};
  assignment.review[family] = assignment.review[family] ?? {};
  const entry: DerivedTagReviewEntry = {
    mode: decision.mode,
    status: decision.status,
    rationale: decision.rationale,
    ...(decision.confidence ? { confidence: decision.confidence } : {}),
    ...(decision.source ? { source: decision.source } : {}),
  };
  assignment.review[family]![tag] = entry;
}

function syncLiveAssignmentBuckets(assignment: AuthoredDerivedTagAssignment): void {
  const applied: Record<string, string[]> = {};
  const excluded: Record<string, string[]> = {};

  for (const [family, familyReview] of Object.entries(assignment.review ?? {})) {
    for (const [tag, reviewEntry] of Object.entries(familyReview)) {
      if (reviewEntry.status !== "approved" && reviewEntry.status !== "auto_applied") {
        continue;
      }
      const bucket = reviewEntry.mode === "include" ? applied : excluded;
      const current = bucket[family] ?? [];
      current.push(normalizeDerivedTag(tag));
      bucket[family] = uniqueSorted(current);
    }
  }

  assignment.applied = Object.keys(applied).length > 0 ? applied : undefined;
  assignment.excluded = Object.keys(excluded).length > 0 ? excluded : undefined;
}

function sortAssignments(assignments: AuthoredDerivedTagAssignment[]): AuthoredDerivedTagAssignment[] {
  const normalized = assignments.map((assignment) => {
    if (assignment.applied) {
      assignment.applied = Object.fromEntries(
        Object.entries(assignment.applied)
          .sort(([left], [right]) => left.localeCompare(right))
          .map(([family, tags]) => [family, uniqueSorted(tags.map((tag) => normalizeDerivedTag(tag)))]),
      );
    }
    if (assignment.excluded) {
      assignment.excluded = Object.fromEntries(
        Object.entries(assignment.excluded)
          .sort(([left], [right]) => left.localeCompare(right))
          .map(([family, tags]) => [family, uniqueSorted(tags.map((tag) => normalizeDerivedTag(tag)))]),
      );
    }
    if (assignment.review) {
      assignment.review = Object.fromEntries(
        Object.entries(assignment.review)
          .sort(([left], [right]) => left.localeCompare(right))
          .map(([family, taggedReview]) => [
            family,
            Object.fromEntries(
              Object.entries(taggedReview)
                .sort(([left], [right]) => left.localeCompare(right)),
            ),
          ]),
      );
    }
    return assignment;
  });

  return normalized.sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey));
}

export function applyMigrationSessionToAssignments(
  assignments: AuthoredDerivedTagAssignment[],
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): AuthoredDerivedTagAssignment[] {
  const nextAssignments = structuredClone(assignments);

  for (const recordDecision of sessionDecisions) {
    const assignmentDecisions = recordDecision.decisions.filter((decision) => decision.kind === "assignment");
    if (assignmentDecisions.length === 0) {
      continue;
    }
    const assignment = ensureAssignment(nextAssignments, recordDecision);
    for (const decision of assignmentDecisions) {
      setReviewEntry(assignment, decision);
    }
    syncLiveAssignmentBuckets(assignment);
  }

  return sortAssignments(nextAssignments);
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
  return [...records].sort((left, right) => left.name.localeCompare(right.name) || left.recordKey.localeCompare(right.recordKey));
}

export function applyMigrationSessionToExemplars(
  exemplars: DerivedTagExemplarCategory,
  sessionDecisions: DerivedTagMigrationRecordDecision[],
): DerivedTagExemplarCategory {
  const nextExemplars = structuredClone(exemplars);

  for (const recordDecision of sessionDecisions) {
    const exemplarDecisions = recordDecision.decisions.filter((decision) => decision.kind === "exemplar");
    for (const decision of exemplarDecisions) {
      if (decision.status !== "approved" && decision.status !== "auto_applied") {
        continue;
      }
      const exemplarSet = ensureExemplarSet(nextExemplars, decision.tag);
      const bucket = decision.polarity === "positive" ? (exemplarSet.positives ?? []) : (exemplarSet.negatives ?? []);
      const withoutRecord = bucket.filter((record) => record.recordKey !== recordDecision.recordKey);
      const updated = decision.action === "keep"
        ? [...withoutRecord, { recordKey: recordDecision.recordKey, name: recordDecision.name }]
        : withoutRecord;
      if (decision.polarity === "positive") {
        exemplarSet.positives = sortExemplarRecords(updated);
      } else {
        exemplarSet.negatives = sortExemplarRecords(updated);
      }
    }
  }

  nextExemplars.exemplars = nextExemplars.exemplars
    .map((entry) => ({
      ...entry,
      positives: sortExemplarRecords(entry.positives ?? []),
      negatives: sortExemplarRecords(entry.negatives ?? []),
    }))
    .filter((entry) => (entry.positives?.length ?? 0) > 0 || (entry.negatives?.length ?? 0) > 0 || entry.notes)
    .sort((left, right) => normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag)));

  return nextExemplars;
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
    for (const decision of recordDecision.decisions.filter((entry) => entry.kind === "rule")) {
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

  return nextRules.sort((left, right) =>
    normalizeDerivedTag(left.tag).localeCompare(normalizeDerivedTag(right.tag))
    || left.kind.localeCompare(right.kind)
    || JSON.stringify(left).localeCompare(JSON.stringify(right)));
}

function toManagedCategory(category: SearchCategory): DerivedTagManagedCategory {
  if (
    category !== "affliction"
    && category !== "creature"
    && category !== "equipment"
    && category !== "hazard"
    && category !== "spell"
  ) {
    throw new Error(`Derived-tag migration importer does not manage category "${category}".`);
  }
  return category;
}

function categoriesTouchedBySession(session: DerivedTagMigrationSession): DerivedTagManagedCategory[] {
  return uniqueSorted(session.decisions.map((decision) => toManagedCategory(decision.category))) as DerivedTagManagedCategory[];
}

function applySessionToState(
  state: DerivedTagMigrationAuthoredState,
  session: DerivedTagMigrationSession,
): DerivedTagMigrationAuthoredState {
  const nextState = structuredClone(state);

  for (const category of categoriesTouchedBySession(session)) {
    const relevantDecisions = session.decisions.filter((decision) => decision.category === category);
    nextState.assignments[category] = applyMigrationSessionToAssignments(nextState.assignments[category], relevantDecisions);
    nextState.exemplars[category] = applyMigrationSessionToExemplars(nextState.exemplars[category], relevantDecisions);
    nextState.authoredRules[category] = applyMigrationSessionToAuthoredRules(nextState.authoredRules[category], relevantDecisions);
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
