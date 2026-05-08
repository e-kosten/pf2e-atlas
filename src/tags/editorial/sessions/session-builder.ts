import { DatabaseSync } from "node:sqlite";

import type { SearchCategory } from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_MANAGED_CATEGORIES, expectDerivedTagManagedCategory } from "../../manifest.js";
import { listDerivedTagLegacySeedMigrations } from "../../runtime/derivation/api.js";
import { normalizeDerivedTag } from "../../runtime/matcher/shared.js";
import { matchesDerivedTagFamilyFilter } from "./actionable-session-scope.js";
import { getCurrentDerivedTagAuthoredState } from "../state/authored-state.js";
import {
  listCurrentPendingAssignmentReviews,
  listCurrentPendingExemplarReviews,
  listCurrentPendingLlmAssignmentReviews,
  listCurrentPendingLlmExemplarReviews,
} from "../state/review-queue.js";
import { loadDerivedTagMigrationRecords } from "./record-loader.js";
import { deriveCurrentTagSources, getPublishedDerivedTagOntology } from "../state/runtime-state.js";
import type {
  DerivedTagReviewDecision,
  DerivedTagWorkbenchMode,
  DerivedTagManagedCategory,
  DerivedTagReviewRecordDecision,
  DerivedTagReviewSelectionReason,
  DerivedTagReviewSession,
  DerivedTagReviewSessionCreateOptions,
  DerivedTagReviewSessionRecord,
} from "../types.js";
import { DERIVED_TAG_REVIEW_VOCABULARY } from "../review-vocabulary.js";

function createSessionId(options: DerivedTagReviewSessionCreateOptions): string {
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  const scope = [options.mode, options.category ?? "all", options.tag ?? options.family ?? "batch"]
    .map((part) => normalizeDerivedTag(part))
    .join("-");
  return `${stamp}-${scope}`;
}

function createRecordMap(records: DerivedTagReviewSessionRecord[]): Map<string, DerivedTagReviewSessionRecord> {
  return new Map(records.map((record) => [record.entityRecord.recordKey, record]));
}

function appendSelectionReason(
  record: DerivedTagReviewSessionRecord,
  reason: DerivedTagReviewSelectionReason,
): void {
  record.selectionReasons.push(reason);
}

function toSessionRecord(
  record: ReturnType<typeof loadDerivedTagMigrationRecords>[number],
): DerivedTagReviewSessionRecord {
  const entityRecord = record.entityRecord;
  return {
    entityRecord,
    currentSources: deriveCurrentTagSources({
      recordKey: entityRecord.recordKey,
      name: entityRecord.name,
      category: entityRecord.category,
      subcategory: entityRecord.subcategory,
      descriptionText: entityRecord.descriptionText,
      blurbText: entityRecord.blurbText,
      traits: entityRecord.traits,
      families: entityRecord.families,
      references: record.references,
    }),
    selectionReasons: [],
  };
}

function createDecisionIndex(
  records: DerivedTagReviewSessionRecord[],
): Map<string, DerivedTagReviewRecordDecision> {
  return new Map(
    records.map((record) => [
      record.entityRecord.recordKey,
      {
        recordKey: record.entityRecord.recordKey,
        name: record.entityRecord.name,
        category: record.entityRecord.category,
        resolutionStatus: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        decisions: [],
      },
    ]),
  );
}

function resolveTagFamily(category: SearchCategory, tag: string): string {
  const ontology = getPublishedDerivedTagOntology();
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizeDerivedTag(tag)}`);
  if (!ontologyTag) {
    throw new Error(`Could not resolve ontology family for tag "${tag}" in category "${category}".`);
  }
  return normalizeDerivedTag(ontologyTag.family);
}

function buildLegacySeedWorkset(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  const scope = { category: options.category, subcategory: options.subcategory };
  const definitions = listDerivedTagLegacySeedMigrations(scope).filter(
    (definition) => !options.tag || normalizeDerivedTag(definition.tag) === normalizeDerivedTag(options.tag),
  );

  const recordKeys = [...new Set(definitions.flatMap((definition) => definition.recordKeys))].slice(
    0,
    options.limit ?? Number.MAX_SAFE_INTEGER,
  );
  const records = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys,
  }).map(toSessionRecord);
  const recordMap = createRecordMap(records);
  const decisionIndex = createDecisionIndex(records);

  for (const definition of definitions) {
    const family = resolveTagFamily(definition.category, definition.tag);
    for (const recordKey of definition.recordKeys) {
      const record = recordMap.get(recordKey);
      if (!record) {
        continue;
      }
      appendSelectionReason(record, {
        source: "legacy_seed",
        family,
        tag: definition.tag,
        note: "Legacy seed migration currently keeps this tag live.",
      });
      const currentSource = record.currentSources[normalizeDerivedTag(definition.tag)] ?? "";
      if (currentSource.includes("seed_migration")) {
        decisionIndex.get(recordKey)?.decisions.push({
          kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
          family,
          tag: normalizeDerivedTag(definition.tag),
          mode: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.INCLUDE,
          status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
          confidence: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM,
          rationale:
            "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
          source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
        });
      }
      decisionIndex.get(recordKey)?.decisions.push({
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
        tag: normalizeDerivedTag(definition.tag),
        polarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
        action: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
        status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        confidence: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM,
        rationale: "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
        source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
        currentPolarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NONE,
      });
    }
  }

  const decisions = [...decisionIndex.values()];
  return {
    manifest: {
      id: createSessionId(options),
      mode: options.mode,
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      tag: options.tag,
      createdAt: new Date().toISOString(),
      recordCount: records.length,
    },
    records,
    decisions,
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

function buildReviewQueueWorkset(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  const pendingAssignments = listCurrentPendingAssignmentReviews()
    .filter((entry) => !options.category || entry.category === options.category)
        .filter(() => !options.decisionKind || options.decisionKind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT)
    .filter(
      (entry) => !options.family || normalizeDerivedTag(entry.decision.family) === normalizeDerivedTag(options.family),
    )
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const pendingExemplarReviews = listCurrentPendingExemplarReviews()
    .filter((entry) => !options.category || entry.category === options.category)
    .filter(() => !options.decisionKind || options.decisionKind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR)
    .filter((entry) => matchesDerivedTagFamilyFilter(entry.category, entry.decision.tag, options.family))
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const uniqueRecordKeys = [
    ...new Set([
      ...pendingAssignments.map((entry) => entry.decision.recordKey),
      ...pendingExemplarReviews.map((entry) => entry.decision.recordKey),
    ]),
  ].slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
  const records = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: uniqueRecordKeys,
  }).map(toSessionRecord);
  const recordMap = createRecordMap(records);
  const decisionIndex = createDecisionIndex(records);

  for (const entry of pendingAssignments) {
    const record = recordMap.get(entry.decision.recordKey);
    if (!record) {
      continue;
    }
    appendSelectionReason(record, {
      source: "authored_review_queue",
      family: entry.decision.family,
      tag: entry.decision.tag,
      note: "Existing authored assignment review entry still needs manual confirmation.",
    });
    decisionIndex.get(entry.decision.recordKey)?.decisions.push({
      kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
      family: normalizeDerivedTag(entry.decision.family),
      tag: normalizeDerivedTag(entry.decision.tag),
      mode: entry.decision.mode,
      status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
      confidence: entry.decision.confidence,
      rationale: entry.decision.rationale,
      source: entry.decision.source,
    });
  }

  for (const entry of pendingExemplarReviews) {
    const record = recordMap.get(entry.decision.recordKey);
    if (!record) {
      continue;
    }
    appendSelectionReason(record, {
      source: "authored_exemplar_review_queue",
      tag: entry.decision.tag,
      note: "Existing authored exemplar review entry still needs manual confirmation.",
    });
    decisionIndex.get(entry.decision.recordKey)?.decisions.push({
      kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
      tag: normalizeDerivedTag(entry.decision.tag),
      polarity:
        entry.decision.proposedPolarity === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
          ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
          : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
      action: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
      status: entry.decision.status,
      confidence: entry.decision.confidence,
      rationale: entry.decision.rationale,
      source: entry.decision.source,
      currentPolarity: entry.decision.currentPolarity,
    });
  }

  const decisions = [...decisionIndex.values()];
  return {
    manifest: {
      id: createSessionId(options),
      mode: options.mode,
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      tag: options.tag,
      createdAt: new Date().toISOString(),
      recordCount: records.length,
    },
    records,
    decisions,
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

function toManagedCategory(category: SearchCategory): DerivedTagManagedCategory {
  return expectDerivedTagManagedCategory(category, "Derived-tag migration session builder");
}

function buildExemplarCleanupWorkset(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  const state = getCurrentDerivedTagAuthoredState();
  const categories = options.category ? [toManagedCategory(options.category)] : [...DERIVED_TAG_MANAGED_CATEGORIES];
  type ExemplarSet = (typeof state.exemplars)[DerivedTagManagedCategory]["exemplars"][number];
  const tagsToReview = categories.flatMap((category) =>
    state.exemplars[category].exemplars
      .filter((entry: ExemplarSet) => matchesDerivedTagFamilyFilter(category, entry.tag, options.family))
      .filter(
        (entry: ExemplarSet) => !options.tag || normalizeDerivedTag(entry.tag) === normalizeDerivedTag(options.tag),
      )
      .filter((entry: ExemplarSet) =>
        options.exemplarLimit === undefined
          ? true
          : (entry.positives?.length ?? 0) + (entry.negatives?.length ?? 0) > options.exemplarLimit,
      )
      .map((entry) => ({ category, exemplarSet: entry })),
  );
  const recordKeys = [
    ...new Set(
      tagsToReview.flatMap(({ exemplarSet }) => [
        ...(exemplarSet.positives ?? []).map((record) => record.recordKey),
        ...(exemplarSet.negatives ?? []).map((record) => record.recordKey),
      ]),
    ),
  ].slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
  const records = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys,
  }).map(toSessionRecord);
  const recordMap = createRecordMap(records);
  const decisionIndex = createDecisionIndex(records);

  for (const { category, exemplarSet } of tagsToReview) {
    for (const positive of exemplarSet.positives ?? []) {
      const record = recordMap.get(positive.recordKey);
      if (!record) {
        continue;
      }
      appendSelectionReason(record, {
        source: "exemplar_cleanup",
        tag: exemplarSet.tag,
        note: "Current exemplar is part of an oversized exemplar set and needs review.",
      });
      decisionIndex.get(positive.recordKey)?.decisions.push({
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
        tag: normalizeDerivedTag(exemplarSet.tag),
        polarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
        action: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
        status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        confidence: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM,
        rationale: `Review whether this ${category} remains a strong positive exemplar for "${normalizeDerivedTag(exemplarSet.tag)}".`,
        source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
        currentPolarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
      });
    }
    for (const negative of exemplarSet.negatives ?? []) {
      const record = recordMap.get(negative.recordKey);
      if (!record) {
        continue;
      }
      appendSelectionReason(record, {
        source: "exemplar_cleanup",
        tag: exemplarSet.tag,
        note: "Current negative exemplar is part of an oversized exemplar set and needs review.",
      });
      decisionIndex.get(negative.recordKey)?.decisions.push({
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
        tag: normalizeDerivedTag(exemplarSet.tag),
        polarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE,
        action: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
        status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        confidence: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM,
        rationale: `Review whether this ${category} remains a strong negative exemplar for "${normalizeDerivedTag(exemplarSet.tag)}".`,
        source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
        currentPolarity: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE,
      });
    }
  }

  return {
    manifest: {
      id: createSessionId(options),
      mode: options.mode,
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      tag: options.tag,
      createdAt: new Date().toISOString(),
      recordCount: records.length,
    },
    records,
    decisions: [...decisionIndex.values()],
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

function buildLegacyRuleWorkset(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  if (!options.category || !options.tag) {
    throw new Error("Legacy rule sessions require both --category and --tag.");
  }

  const candidates = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    requireTag: options.tag,
    limit: options.limit,
  })
    .map(toSessionRecord)
    .filter((record) => {
      const source = record.currentSources[normalizeDerivedTag(options.tag!)] ?? "";
      return source.includes("legacy_rule");
    });

  const family = resolveTagFamily(options.category, options.tag);
  const decisions = candidates.map((record) => ({
    recordKey: record.entityRecord.recordKey,
    name: record.entityRecord.name,
    category: record.entityRecord.category,
    resolutionStatus: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
    decisions: [
      {
        kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
        family,
        tag: normalizeDerivedTag(options.tag!),
        mode: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.ASSIGNMENT_MODE.INCLUDE,
        status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
        confidence: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.CONFIDENCE.MEDIUM,
        rationale:
          "Legacy rule currently supplies this tag; review whether to replace it with an explicit assignment or a future authored rule.",
        source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
      },
    ] satisfies DerivedTagReviewDecision[],
  }));
  for (const record of candidates) {
    appendSelectionReason(record, {
      source: "legacy_rule",
      family,
      tag: options.tag,
      note: "Current live tag source includes a legacy rule.",
    });
  }

  return {
    manifest: {
      id: createSessionId(options),
      mode: options.mode,
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      tag: options.tag,
      createdAt: new Date().toISOString(),
      recordCount: candidates.length,
    },
    records: candidates,
    decisions,
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

function buildProposalReviewWorkset(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  const pendingAssignments = listCurrentPendingLlmAssignmentReviews()
    .filter((entry) => !options.category || entry.category === options.category)
    .filter(() => !options.decisionKind || options.decisionKind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT)
    .filter(
      (entry) => !options.family || normalizeDerivedTag(entry.decision.family) === normalizeDerivedTag(options.family),
    )
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const pendingExemplarReviews = listCurrentPendingLlmExemplarReviews()
    .filter((entry) => !options.category || entry.category === options.category)
    .filter(() => !options.decisionKind || options.decisionKind === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR)
    .filter((entry) => matchesDerivedTagFamilyFilter(entry.category, entry.decision.tag, options.family))
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const uniqueRecordKeys = [
    ...new Set([
      ...pendingAssignments.map((entry) => entry.decision.recordKey),
      ...pendingExemplarReviews.map((entry) => entry.decision.recordKey),
    ]),
  ].slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
  const records = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: uniqueRecordKeys,
  }).map(toSessionRecord);
  const recordMap = createRecordMap(records);
  const decisionIndex = createDecisionIndex(records);

  for (const entry of pendingAssignments) {
    const record = recordMap.get(entry.decision.recordKey);
    if (!record) {
      continue;
    }
    appendSelectionReason(record, {
      source: "llm_assignment_review_queue",
      family: entry.decision.family,
      tag: entry.decision.tag,
      note: "LLM-generated assignment proposal still needs manual review.",
    });
    decisionIndex.get(entry.decision.recordKey)?.decisions.push({
      kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.ASSIGNMENT,
      family: normalizeDerivedTag(entry.decision.family),
      tag: normalizeDerivedTag(entry.decision.tag),
      mode: entry.decision.mode,
      status: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.STATUS.NEEDS_REVIEW,
      confidence: entry.decision.confidence,
      rationale: entry.decision.rationale,
      source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
    });
  }

  for (const entry of pendingExemplarReviews) {
    const record = recordMap.get(entry.decision.recordKey);
    if (!record) {
      continue;
    }
    appendSelectionReason(record, {
      source: "llm_exemplar_review_queue",
      tag: entry.decision.tag,
      note: "LLM-generated exemplar proposal still needs manual review.",
    });
    decisionIndex.get(entry.decision.recordKey)?.decisions.push({
      kind: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.DECISION_KIND.EXEMPLAR,
      tag: normalizeDerivedTag(entry.decision.tag),
      polarity:
        entry.decision.proposedPolarity === DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
          ? DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.NEGATIVE
          : DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_POLARITY.POSITIVE,
      action: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.EXEMPLAR_ACTION.KEEP,
      status: entry.decision.status,
      confidence: entry.decision.confidence,
      rationale: entry.decision.rationale,
      source: DERIVED_TAG_REVIEW_VOCABULARY.REVIEW.SOURCE.LLM,
      currentPolarity: entry.decision.currentPolarity,
    });
  }

  return {
    manifest: {
      id: createSessionId(options),
      mode: options.mode,
      category: options.category,
      subcategory: options.subcategory,
      family: options.family,
      tag: options.tag,
      createdAt: new Date().toISOString(),
      recordCount: records.length,
    },
    records,
    decisions: [...decisionIndex.values()],
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

export function buildDerivedTagReviewSession(
  db: DatabaseSync,
  options: DerivedTagReviewSessionCreateOptions,
): DerivedTagReviewSession {
  const builders: Record<
    DerivedTagWorkbenchMode,
    (db: DatabaseSync, options: DerivedTagReviewSessionCreateOptions) => DerivedTagReviewSession
  > = {
    review_queue: buildReviewQueueWorkset,
    proposal_review: buildProposalReviewWorkset,
    legacy_seed: buildLegacySeedWorkset,
    legacy_rule: buildLegacyRuleWorkset,
    exemplar_cleanup: buildExemplarCleanupWorkset,
  };

  return builders[options.mode](db, options);
}
