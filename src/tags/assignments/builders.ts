import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentDecision,
} from "../runtime/derivation/assignments.js";

export type AuthoredDerivedTagAssignmentRecord = Omit<AuthoredDerivedTagAssignment, "recordKey">;

export type AuthoredDerivedTagAssignmentRecordMap = Record<string, AuthoredDerivedTagAssignmentRecord>;

export function tag(
  currentTag: string,
  decision: Omit<DerivedTagAssignmentDecision, "tag">,
): DerivedTagAssignmentDecision {
  return {
    ...decision,
    tag: currentTag,
  };
}

export function defineAssignments(
  assignmentsByRecordKey: AuthoredDerivedTagAssignmentRecordMap,
): AuthoredDerivedTagAssignment[] {
  return Object.entries(assignmentsByRecordKey)
    .map(([recordKey, assignment]) => ({
      ...assignment,
      recordKey,
    }))
    .sort((left, right) => left.recordKey.localeCompare(right.recordKey));
}
