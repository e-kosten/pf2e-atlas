import { normalizeDerivedTag } from "../../runtime/matcher/shared.js";
import { getPublishedDerivedTagOntology } from "../state/runtime-state.js";
import type { DerivedTagReviewDecision, DerivedTagReviewSession, DerivedTagReviewResolutionStatus } from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";

function qualifiedKey(family: string, tag: string): string {
  return `${normalizeDerivedTag(family)}.${normalizeDerivedTag(tag)}`;
}

function normalizeRecordResolution(decisions: DerivedTagReviewDecision[]): DerivedTagReviewResolutionStatus {
  if (decisions.length === 0) {
    return DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUS.NEEDS_REVIEW;
  }
  return decisions.some((decision) => decision.status === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW)
    ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUS.NEEDS_REVIEW
    : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RESOLUTION_STATUS.COMPLETE;
}

export function lintDerivedTagReviewSession(session: DerivedTagReviewSession): void {
  const ontology = getPublishedDerivedTagOntology();
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
      if (decision.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT) {
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
      } else if (decision.kind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR) {
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
          decision.decision === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.RULE_DECISION.RECREATE_AUTHORED &&
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
    const opposite = `${qualified}:${mode === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.INCLUDE ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.EXCLUDE : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.INCLUDE}`;
    if (seenAssignments.has(opposite)) {
        throw new Error(
          `Migration session places "${qualified}" in both include and exclude for "${decisionRecord.recordKey}".`,
        );
      }
    }
  }
}
