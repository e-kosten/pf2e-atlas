import { DatabaseSync } from "node:sqlite";

import type { DerivedTagExemplarReviewDecision, SearchCategory } from "../../types.js";
import {
  listDerivedTagLegacySeedMigrations,
} from "../index.js";
import type { DerivedTagAssignmentReviewDecision } from "../runtime/assignments.js";
import { normalizeDerivedTag } from "../runtime/shared.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";
import { loadDerivedTagMigrationRecords } from "./record-loader.js";
import { deriveCurrentTagSources, getPublishedDerivedTagMigrationOntology } from "./runtime-state.js";
import type {
  DerivedTagMigrationDecision,
  DerivedTagManagedCategory,
  DerivedTagMigrationRecordDecision,
  DerivedTagMigrationSelectionReason,
  DerivedTagMigrationSession,
  DerivedTagMigrationSessionCreateOptions,
  DerivedTagMigrationSessionRecord,
} from "./types.js";

function createSessionId(options: DerivedTagMigrationSessionCreateOptions): string {
  const stamp = new Date().toISOString().replace(/[:.]/g, "-");
  const scope = [options.mode, options.category ?? "all", options.tag ?? options.family ?? "batch"]
    .map((part) => normalizeDerivedTag(part))
    .join("-");
  return `${stamp}-${scope}`;
}

function createRecordMap(records: DerivedTagMigrationSessionRecord[]): Map<string, DerivedTagMigrationSessionRecord> {
  return new Map(records.map((record) => [record.recordKey, record]));
}

function appendSelectionReason(
  record: DerivedTagMigrationSessionRecord,
  reason: DerivedTagMigrationSelectionReason,
): void {
  record.selectionReasons.push(reason);
}

function toSessionRecord(record: ReturnType<typeof loadDerivedTagMigrationRecords>[number]): DerivedTagMigrationSessionRecord {
  return {
    recordKey: record.recordKey,
    name: record.name,
    category: record.category,
    subcategory: record.subcategory,
    packName: record.packName,
    level: record.level,
    traits: record.traits,
    families: record.families,
    currentDerivedTags: record.derivedTags,
    currentSources: deriveCurrentTagSources({
      recordKey: record.recordKey,
      name: record.name,
      category: record.category,
      subcategory: record.subcategory,
      descriptionText: record.descriptionText,
      blurbText: record.blurbText,
      traits: record.traits,
      families: record.families,
      references: record.references,
    }),
    descriptionText: record.descriptionText,
    blurbText: record.blurbText,
    selectionReasons: [],
  };
}

function flattenCurrentPendingReviewAssignments(): Array<{
  category: SearchCategory;
  decision: DerivedTagAssignmentReviewDecision;
}> {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const pending: Array<{
    category: SearchCategory;
    decision: DerivedTagAssignmentReviewDecision;
  }> = [];

  for (const [category, assignmentReviewCategory] of Object.entries(state.assignmentReviews) as Array<[SearchCategory, { decisions: DerivedTagAssignmentReviewDecision[] }]>) {
    for (const decision of assignmentReviewCategory.decisions) {
      pending.push({ category, decision });
    }
  }

  return pending;
}

function flattenCurrentPendingExemplarReviews(): Array<{
  category: SearchCategory;
  decision: DerivedTagExemplarReviewDecision;
}> {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const pending: Array<{
    category: SearchCategory;
    decision: DerivedTagExemplarReviewDecision;
  }> = [];

  for (const [category, exemplarReviewCategory] of Object.entries(state.exemplarReviews) as Array<[SearchCategory, { decisions: DerivedTagExemplarReviewDecision[] }]>) {
    for (const decision of exemplarReviewCategory.decisions) {
      if (decision.status !== "needs_review") {
        continue;
      }
      pending.push({ category, decision });
    }
  }

  return pending;
}

function createDecisionIndex(
  records: DerivedTagMigrationSessionRecord[],
): Map<string, DerivedTagMigrationRecordDecision> {
  return new Map(records.map((record) => [record.recordKey, {
    recordKey: record.recordKey,
    name: record.name,
    category: record.category,
    resolutionStatus: "needs_review",
    decisions: [],
  }]));
}

function resolveTagFamily(category: SearchCategory, tag: string): string {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizeDerivedTag(tag)}`);
  if (!ontologyTag) {
    throw new Error(`Could not resolve ontology family for tag "${tag}" in category "${category}".`);
  }
  return normalizeDerivedTag(ontologyTag.family);
}

function buildLegacySeedWorkset(
  db: DatabaseSync,
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  const scope = { category: options.category, subcategory: options.subcategory };
  const definitions = listDerivedTagLegacySeedMigrations(scope)
    .filter((definition) => !options.tag || normalizeDerivedTag(definition.tag) === normalizeDerivedTag(options.tag));

  const recordKeys = [...new Set(definitions.flatMap((definition) => definition.recordKeys))]
    .slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
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
          kind: "assignment",
          family,
          tag: normalizeDerivedTag(definition.tag),
          mode: "include",
          status: "needs_review",
          confidence: "medium",
          rationale: "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
          source: "llm",
        });
      }
      decisionIndex.get(recordKey)?.decisions.push({
        kind: "exemplar",
        tag: normalizeDerivedTag(definition.tag),
        polarity: "positive",
        action: "keep",
        status: "needs_review",
        confidence: "medium",
        rationale: "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
        source: "llm",
        currentPolarity: "none",
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
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  const pendingAssignments = flattenCurrentPendingReviewAssignments()
    .filter((entry) => !options.category || entry.category === options.category)
    .filter(() => !options.decisionKind || options.decisionKind === "assignment")
    .filter((entry) => !options.family || normalizeDerivedTag(entry.decision.family) === normalizeDerivedTag(options.family))
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const pendingExemplarReviews = flattenCurrentPendingExemplarReviews()
    .filter((entry) => !options.category || entry.category === options.category)
    .filter(() => !options.decisionKind || options.decisionKind === "exemplar")
    .filter((entry) => !options.tag || normalizeDerivedTag(entry.decision.tag) === normalizeDerivedTag(options.tag));
  const uniqueRecordKeys = [...new Set([
    ...pendingAssignments.map((entry) => entry.decision.recordKey),
    ...pendingExemplarReviews.map((entry) => entry.decision.recordKey),
  ])]
    .slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
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
      kind: "assignment",
      family: normalizeDerivedTag(entry.decision.family),
      tag: normalizeDerivedTag(entry.decision.tag),
      mode: entry.decision.mode,
      status: "needs_review",
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
      kind: "exemplar",
      tag: normalizeDerivedTag(entry.decision.tag),
      polarity: entry.decision.proposedPolarity === "negative" ? "negative" : "positive",
      action: entry.decision.proposedPolarity === "drop" ? "drop" : "keep",
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
  if (
    category !== "affliction"
    && category !== "creature"
    && category !== "equipment"
    && category !== "hazard"
    && category !== "spell"
  ) {
    throw new Error(`Derived-tag migration session builder does not manage category "${category}".`);
  }
  return category;
}

function buildExemplarCleanupWorkset(
  db: DatabaseSync,
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const categories = options.category ? [toManagedCategory(options.category)] : (Object.keys(state.exemplars) as DerivedTagManagedCategory[]);
  type ExemplarSet = (typeof state.exemplars)[DerivedTagManagedCategory]["exemplars"][number];
  const tagsToReview = categories
    .flatMap((category) => state.exemplars[category].exemplars
      .filter((entry: ExemplarSet) =>
        !options.tag || normalizeDerivedTag(entry.tag) === normalizeDerivedTag(options.tag))
      .filter((entry: ExemplarSet) => options.exemplarLimit === undefined
        ? true
        : ((entry.positives?.length ?? 0) + (entry.negatives?.length ?? 0)) > options.exemplarLimit)
      .map((entry) => ({ category, exemplarSet: entry })));
  const recordKeys = [...new Set(tagsToReview.flatMap(({ exemplarSet }) => [
    ...(exemplarSet.positives ?? []).map((record) => record.recordKey),
    ...(exemplarSet.negatives ?? []).map((record) => record.recordKey),
  ]))].slice(0, options.limit ?? Number.MAX_SAFE_INTEGER);
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
        kind: "exemplar",
        tag: normalizeDerivedTag(exemplarSet.tag),
        polarity: "positive",
        action: "keep",
        status: "needs_review",
        confidence: "medium",
        rationale: `Review whether this ${category} remains a strong positive exemplar for "${normalizeDerivedTag(exemplarSet.tag)}".`,
        source: "llm",
        currentPolarity: "positive",
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
        kind: "exemplar",
        tag: normalizeDerivedTag(exemplarSet.tag),
        polarity: "negative",
        action: "keep",
        status: "needs_review",
        confidence: "medium",
        rationale: `Review whether this ${category} remains a strong negative exemplar for "${normalizeDerivedTag(exemplarSet.tag)}".`,
        source: "llm",
        currentPolarity: "negative",
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
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  if (!options.category || !options.tag) {
    throw new Error("Legacy rule sessions require both --category and --tag.");
  }

  const candidates = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    requireTag: options.tag,
    limit: options.limit,
  }).map(toSessionRecord)
    .filter((record) => {
      const source = record.currentSources[normalizeDerivedTag(options.tag!)] ?? "";
      return source.includes("legacy_rule");
    });

  const family = resolveTagFamily(options.category, options.tag);
  const decisions = candidates.map((record) => ({
    recordKey: record.recordKey,
    name: record.name,
    category: record.category,
    resolutionStatus: "needs_review" as const,
    decisions: [
      {
        kind: "assignment",
        family,
        tag: normalizeDerivedTag(options.tag!),
        mode: "include" as const,
        status: "needs_review" as const,
        confidence: "medium" as const,
        rationale: "Legacy rule currently supplies this tag; review whether to replace it with an explicit assignment or a future authored rule.",
        source: "llm" as const,
      },
    ] satisfies DerivedTagMigrationDecision[],
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

function buildNewTaggingWorkset(
  db: DatabaseSync,
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  const records = loadDerivedTagMigrationRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    untaggedOnly: true,
    limit: options.limit,
  }).map(toSessionRecord);

  for (const record of records) {
    appendSelectionReason(record, {
      source: "untagged",
      note: "Canonical record currently has no live derived tags.",
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
    decisions: records.map((record) => ({
      recordKey: record.recordKey,
      name: record.name,
      category: record.category,
      resolutionStatus: "needs_review",
      decisions: [],
    })),
    reviewState: {
      currentIndex: 0,
      unresolvedOnly: true,
      updatedAt: new Date().toISOString(),
    },
  };
}

export function buildDerivedTagMigrationSession(
  db: DatabaseSync,
  options: DerivedTagMigrationSessionCreateOptions,
): DerivedTagMigrationSession {
  if (options.mode === "review_queue") {
    return buildReviewQueueWorkset(db, options);
  }
  if (options.mode === "legacy_seed") {
    return buildLegacySeedWorkset(db, options);
  }
  if (options.mode === "exemplar_cleanup") {
    return buildExemplarCleanupWorkset(db, options);
  }
  if (options.mode === "legacy_rule") {
    return buildLegacyRuleWorkset(db, options);
  }
  return buildNewTaggingWorkset(db, options);
}
