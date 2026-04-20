import { normalizeDerivedTag } from "../../runtime/shared.js";
import { getPublishedDerivedTagMigrationOntology } from "../state/runtime-state.js";
import type { DerivedTagMigrationDecision, DerivedTagMigrationSession } from "../types.js";

function qualifiedKey(family: string, tag: string): string {
  return `${normalizeDerivedTag(family)}.${normalizeDerivedTag(tag)}`;
}

function normalizeRecordResolution(decisions: DerivedTagMigrationDecision[]): "complete" | "needs_review" {
  if (decisions.length === 0) {
    return "needs_review";
  }
  return decisions.some((decision) => decision.status === "needs_review") ? "needs_review" : "complete";
}

export function lintDerivedTagMigrationSession(session: DerivedTagMigrationSession): void {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const recordKeys = new Set(session.records.map((record) => record.entityRecord.recordKey));
  const seenDecisionRecords = new Set<string>();

  for (const decisionRecord of session.decisions) {
    if (!recordKeys.has(decisionRecord.recordKey)) {
      throw new Error(`Migration session decision references unknown record "${decisionRecord.recordKey}".`);
    }
    if (seenDecisionRecords.has(decisionRecord.recordKey)) {
      throw new Error(`Migration session contains duplicate decision rows for "${decisionRecord.recordKey}".`);
    }
    seenDecisionRecords.add(decisionRecord.recordKey);

    const seenAssignments = new Set<string>();
    const seenExemplars = new Set<string>();
    const computedResolution = normalizeRecordResolution(decisionRecord.decisions);
    if (decisionRecord.resolutionStatus !== computedResolution) {
      throw new Error(
        `Migration session record "${decisionRecord.recordKey}" declares resolutionStatus "${decisionRecord.resolutionStatus}" but computed "${computedResolution}".`,
      );
    }

    for (const decision of decisionRecord.decisions) {
      if (decision.kind === "assignment") {
        const key = qualifiedKey(decision.family, decision.tag);
        if (seenAssignments.has(`${key}:${decision.mode}`)) {
          throw new Error(
            `Migration session repeats assignment decision "${key}" (${decision.mode}) for "${decisionRecord.recordKey}".`,
          );
        }
        seenAssignments.add(`${key}:${decision.mode}`);

        const ontologyTag = ontology.tagByKey.get(`${decisionRecord.category}:${normalizeDerivedTag(decision.tag)}`);
        if (!ontologyTag) {
          throw new Error(
            `Migration session assignment "${key}" for "${decisionRecord.recordKey}" does not exist in the ontology.`,
          );
        }
        if (normalizeDerivedTag(ontologyTag.family) !== normalizeDerivedTag(decision.family)) {
          throw new Error(
            `Migration session assignment "${key}" for "${decisionRecord.recordKey}" uses the wrong family.`,
          );
        }
      } else if (decision.kind === "exemplar") {
        const key = `${normalizeDerivedTag(decision.tag)}:${decision.polarity}`;
        if (seenExemplars.has(key)) {
          throw new Error(`Migration session repeats exemplar decision "${key}" for "${decisionRecord.recordKey}".`);
        }
        seenExemplars.add(key);

        const ontologyTag = ontology.tagByKey.get(`${decisionRecord.category}:${normalizeDerivedTag(decision.tag)}`);
        if (!ontologyTag) {
          throw new Error(
            `Migration session exemplar tag "${decision.tag}" for "${decisionRecord.recordKey}" does not exist in the ontology.`,
          );
        }
      } else {
        const ontologyTag = ontology.tagByKey.get(`${decisionRecord.category}:${normalizeDerivedTag(decision.tag)}`);
        if (!ontologyTag) {
          throw new Error(
            `Migration session rule tag "${decision.tag}" for "${decisionRecord.recordKey}" does not exist in the ontology.`,
          );
        }
        if (
          decision.decision === "recreate_authored" &&
          (!decision.authoredRules || decision.authoredRules.length === 0)
        ) {
          throw new Error(
            `Migration session rule decision for "${decisionRecord.recordKey}" must include authoredRules when recreating an authored rule.`,
          );
        }
      }
    }

    for (const assignment of seenAssignments) {
      const [qualified, mode] = assignment.split(":");
      const opposite = `${qualified}:${mode === "include" ? "exclude" : "include"}`;
      if (seenAssignments.has(opposite)) {
        throw new Error(
          `Migration session places "${qualified}" in both include and exclude for "${decisionRecord.recordKey}".`,
        );
      }
    }
  }
}
