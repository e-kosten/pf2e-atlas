import { DatabaseSync } from "node:sqlite";

import type { SearchCategory } from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_MANAGED_CATEGORIES, expectDerivedTagManagedCategory } from "../../manifest.js";
import { listDerivedTagLegacySeedMigrations } from "../../runtime/derivation/api.js";
import { getCurrentDerivedTagAuthoredState } from "../state/authored-state.js";
import { loadDerivedTagMigrationRecords } from "./record-loader.js";
import { deriveCurrentTagSources, summarizeCurrentDerivedTagReviewQueue } from "../state/runtime-state.js";
import type { DerivedTagManagedCategory, DerivedTagWorkbenchMode } from "../types.js";

export type DerivedTagCategoryScopeSummary = {
  category: DerivedTagManagedCategory;
  detailLines: string[];
};

export type DerivedTagCategoryScopeSummarySet = {
  allCategoriesDetailLines: string[];
  categories: DerivedTagCategoryScopeSummary[];
};

function sum(values: number[]): number {
  return values.reduce((total, value) => total + value, 0);
}

function formatPendingReviewChanges(count: number): string {
  return `${count} pending review change${count === 1 ? "" : "s"}`;
}

function toManagedCategory(category: SearchCategory): DerivedTagManagedCategory {
  return expectDerivedTagManagedCategory(category, "Derived-tag migration session builder");
}

function buildReviewQueueCategoryScopeSummary(): DerivedTagCategoryScopeSummarySet {
  const state = getCurrentDerivedTagAuthoredState();
  const queueSummary = summarizeCurrentDerivedTagReviewQueue();

  const counts = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const assignmentCount = state.assignmentReviews[category].decisions.length;
    const exemplarCount = state.exemplarReviews[category].decisions.filter(
      (decision) => decision.status === "needs_review",
    ).length;
    const queueSliceCount = queueSummary.filter((item) => item.category === category).length;

    return {
      category,
      assignmentCount,
      exemplarCount,
      queueSliceCount,
    };
  });

  return {
    allCategoriesDetailLines: [
      formatPendingReviewChanges(sum(counts.map((entry) => entry.assignmentCount + entry.exemplarCount))),
      `${sum(counts.map((entry) => entry.assignmentCount))} assignment + ${sum(counts.map((entry) => entry.exemplarCount))} exemplar`,
      `${sum(counts.map((entry) => entry.queueSliceCount))} queue slices`,
    ],
    categories: counts.map((entry) => ({
      category: entry.category,
      detailLines: [
        formatPendingReviewChanges(entry.assignmentCount + entry.exemplarCount),
        `${entry.assignmentCount} assignment + ${entry.exemplarCount} exemplar`,
        `${entry.queueSliceCount} queue slice${entry.queueSliceCount === 1 ? "" : "s"}`,
      ],
    })),
  };
}

function buildLegacySeedCategoryScopeSummary(): DerivedTagCategoryScopeSummarySet {
  const counts = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const definitions = listDerivedTagLegacySeedMigrations({ category });
    const recordCount = new Set(definitions.flatMap((definition) => definition.recordKeys)).size;

    return {
      category,
      recordCount,
      tagCount: definitions.length,
    };
  });

  return {
    allCategoriesDetailLines: [
      `${sum(counts.map((entry) => entry.recordCount))} legacy-seed records`,
      `${sum(counts.map((entry) => entry.tagCount))} legacy-seed tags`,
    ],
    categories: counts.map((entry) => ({
      category: entry.category,
      detailLines: [
        `${entry.recordCount} legacy-seed record${entry.recordCount === 1 ? "" : "s"}`,
        `${entry.tagCount} legacy-seed tag${entry.tagCount === 1 ? "" : "s"}`,
      ],
    })),
  };
}

function buildExemplarCleanupCategoryScopeSummary(): DerivedTagCategoryScopeSummarySet {
  const state = getCurrentDerivedTagAuthoredState();
  const counts = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const exemplarSets = state.exemplars[category].exemplars;
    const placementCount = exemplarSets.reduce(
      (total, set) => total + (set.positives?.length ?? 0) + (set.negatives?.length ?? 0),
      0,
    );

    return {
      category,
      exemplarSetCount: exemplarSets.length,
      placementCount,
    };
  });

  return {
    allCategoriesDetailLines: [
      `${sum(counts.map((entry) => entry.exemplarSetCount))} exemplar sets`,
      `${sum(counts.map((entry) => entry.placementCount))} exemplar placements`,
    ],
    categories: counts.map((entry) => ({
      category: entry.category,
      detailLines: [
        `${entry.exemplarSetCount} exemplar set${entry.exemplarSetCount === 1 ? "" : "s"}`,
        `${entry.placementCount} exemplar placement${entry.placementCount === 1 ? "" : "s"}`,
      ],
    })),
  };
}

function buildProposalReviewCategoryScopeSummary(): DerivedTagCategoryScopeSummarySet {
  const state = getCurrentDerivedTagAuthoredState();
  const counts = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const assignmentCount = state.assignmentReviews[category].decisions.filter(
      (decision) => decision.source === "llm",
    ).length;
    const exemplarCount = state.exemplarReviews[category].decisions.filter(
      (decision) => decision.status === "needs_review" && decision.source === "llm",
    ).length;

    return {
      category,
      assignmentCount,
      exemplarCount,
    };
  });

  return {
    allCategoriesDetailLines: [
      formatPendingReviewChanges(sum(counts.map((entry) => entry.assignmentCount + entry.exemplarCount))),
      `${sum(counts.map((entry) => entry.assignmentCount))} assignment + ${sum(counts.map((entry) => entry.exemplarCount))} exemplar`,
      "LLM-origin proposals only",
    ],
    categories: counts.map((entry) => ({
      category: entry.category,
      detailLines: [
        formatPendingReviewChanges(entry.assignmentCount + entry.exemplarCount),
        `${entry.assignmentCount} assignment + ${entry.exemplarCount} exemplar`,
        "LLM-origin proposals only",
      ],
    })),
  };
}

function buildLegacyRuleCategoryScopeSummary(db: DatabaseSync): DerivedTagCategoryScopeSummarySet {
  const counts = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const records = loadDerivedTagMigrationRecords(db, { category });
    const legacyRuleTags = new Set<string>();
    let legacyRuleRecordCount = 0;

    for (const record of records) {
      const entityRecord = record.entityRecord;
      const currentSources = deriveCurrentTagSources({
        recordKey: entityRecord.recordKey,
        name: entityRecord.name,
        category: entityRecord.category,
        subcategory: entityRecord.subcategory,
        descriptionText: entityRecord.descriptionText,
        blurbText: entityRecord.blurbText,
        traits: entityRecord.traits,
        families: entityRecord.families,
        references: record.references,
      });
      const matchedTags = Object.entries(currentSources)
        .filter(([, source]) => source.includes("legacy_rule"))
        .map(([tag]) => tag);

      if (matchedTags.length > 0) {
        legacyRuleRecordCount += 1;
        for (const tag of matchedTags) {
          legacyRuleTags.add(tag);
        }
      }
    }

    return {
      category,
      legacyRuleRecordCount,
      legacyRuleTagCount: legacyRuleTags.size,
    };
  });

  return {
    allCategoriesDetailLines: [
      `${sum(counts.map((entry) => entry.legacyRuleRecordCount))} legacy-rule records`,
      `${sum(counts.map((entry) => entry.legacyRuleTagCount))} legacy-rule tags`,
    ],
    categories: counts.map((entry) => ({
      category: entry.category,
      detailLines: [
        `${entry.legacyRuleRecordCount} legacy-rule record${entry.legacyRuleRecordCount === 1 ? "" : "s"}`,
        `${entry.legacyRuleTagCount} legacy-rule tag${entry.legacyRuleTagCount === 1 ? "" : "s"}`,
      ],
    })),
  };
}

export function summarizeDerivedTagCategoryScopes(
  db: DatabaseSync,
  mode: DerivedTagWorkbenchMode,
): DerivedTagCategoryScopeSummarySet {
  if (mode === "review_queue") {
    return buildReviewQueueCategoryScopeSummary();
  }
  if (mode === "legacy_seed") {
    return buildLegacySeedCategoryScopeSummary();
  }
  if (mode === "exemplar_cleanup") {
    return buildExemplarCleanupCategoryScopeSummary();
  }
  if (mode === "legacy_rule") {
    return buildLegacyRuleCategoryScopeSummary(db);
  }
  return buildProposalReviewCategoryScopeSummary();
}

export function getManagedCategoryScopeSummary(
  set: DerivedTagCategoryScopeSummarySet,
  category: SearchCategory,
): DerivedTagCategoryScopeSummary | undefined {
  return set.categories.find((entry) => entry.category === toManagedCategory(category));
}
