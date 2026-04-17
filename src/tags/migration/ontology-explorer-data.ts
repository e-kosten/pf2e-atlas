import { DatabaseSync } from "node:sqlite";

import type {
  DerivedTagAssignmentMode,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
  SearchCategory,
  SearchSubcategory,
} from "../../types.js";
import {
  getDerivedTagExemplars,
  listDerivedTagLegacySeedMigrations,
  normalizeDerivedTag,
} from "../index.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";
import { getPublishedDerivedTagMigrationOntology } from "./runtime-state.js";

type ExplorerCountRow = {
  category: SearchCategory;
  tag: string;
  recordKey: string;
};

export type DerivedTagOntologyExplorerTagNode = {
  kind: "tag";
  key: string;
  category: SearchCategory;
  family: string;
  subcategories?: SearchSubcategory[];
  tag: string;
  description: string;
  assignmentMode: DerivedTagAssignmentMode;
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
  variantInheritance?: boolean;
  liveRecordCount: number;
  authoredRuleCount: number;
  exemplarPositiveCount: number;
  exemplarNegativeCount: number;
  legacyMigrationDefinitionCount: number;
  legacyMigrationRecordCount: number;
  filterText: string;
};

export type DerivedTagOntologyExplorerFamilyNode = {
  kind: "family";
  key: string;
  category: SearchCategory;
  family: string;
  description: string;
  subcategories?: SearchSubcategory[];
  variantInheritance?: boolean;
  tagCount: number;
  liveRecordCount: number;
  tags: DerivedTagOntologyExplorerTagNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerCategoryNode = {
  kind: "category";
  key: SearchCategory;
  category: SearchCategory;
  familyCount: number;
  tagCount: number;
  taggedRecordCount: number;
  families: DerivedTagOntologyExplorerFamilyNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerModel = {
  categories: DerivedTagOntologyExplorerCategoryNode[];
};

function buildTagFilterText(tag: DerivedTagOntologyTag): string {
  return [
    tag.category,
    tag.family,
    tag.tag,
    tag.description,
    ...(tag.appliesWhen ?? []),
    ...(tag.doesNotApplyWhen ?? []),
    ...(tag.positiveSignals ?? []),
    ...(tag.negativeSignals ?? []),
    ...(tag.adjacentTags ?? []),
    ...(tag.compositeOfAnyTags ?? []),
  ].join(" ").toLowerCase();
}

function buildFamilyFilterText(family: DerivedTagOntologyFamily): string {
  return [
    family.category,
    family.family,
    family.description,
    ...(family.subcategories ?? []),
  ].join(" ").toLowerCase();
}

function queryCanonicalTagRows(db: DatabaseSync): ExplorerCountRow[] {
  return db.prepare(`
    SELECT r.category AS category, d.tag AS tag, d.record_key AS recordKey
    FROM record_derived_tags d
    JOIN records r ON r.record_key = d.record_key
    WHERE r.is_search_canonical = 1
  `).all() as ExplorerCountRow[];
}

function buildAuthoredRuleCounts(): Map<string, number> {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const counts = new Map<string, number>();

  for (const [category, rules] of Object.entries(state.authoredRules) as Array<[SearchCategory, typeof state.authoredRules[keyof typeof state.authoredRules]]>) {
    for (const rule of rules) {
      const key = `${category}:${normalizeDerivedTag(rule.tag)}`;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }

  return counts;
}

function buildExemplarCounts(): Map<string, { positive: number; negative: number }> {
  const counts = new Map<string, { positive: number; negative: number }>();
  const state = getCurrentDerivedTagMigrationAuthoredState();

  for (const [category, exemplarCategory] of Object.entries(state.exemplars) as Array<[SearchCategory, typeof state.exemplars[keyof typeof state.exemplars]]>) {
    for (const exemplar of exemplarCategory.exemplars) {
      const key = `${category}:${normalizeDerivedTag(exemplar.tag)}`;
      counts.set(key, {
        positive: exemplar.positives?.length ?? 0,
        negative: exemplar.negatives?.length ?? 0,
      });
    }
  }

  return counts;
}

function buildLegacyMigrationCounts(): Map<string, { definitions: number; records: number }> {
  const counts = new Map<string, { definitions: number; records: number }>();

  for (const definition of listDerivedTagLegacySeedMigrations()) {
    const key = `${definition.category}:${normalizeDerivedTag(definition.tag)}`;
    const current = counts.get(key) ?? { definitions: 0, records: 0 };
    current.definitions += 1;
    current.records += definition.recordKeys.length;
    counts.set(key, current);
  }

  return counts;
}

function buildLiveCountMaps(rows: ExplorerCountRow[]): {
  tagCounts: Map<string, number>;
  familyCounts: Map<string, number>;
  categoryCounts: Map<SearchCategory, number>;
} {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const tagCounts = new Map<string, number>();
  const familyRecordKeys = new Map<string, Set<string>>();
  const categoryRecordKeys = new Map<SearchCategory, Set<string>>();

  for (const row of rows) {
    const normalizedTag = normalizeDerivedTag(row.tag);
    const tagKey = `${row.category}:${normalizedTag}`;
    tagCounts.set(tagKey, (tagCounts.get(tagKey) ?? 0) + 1);

    const ontologyTag = ontology.tagByKey.get(tagKey as `${SearchCategory}:${string}`);
    if (!ontologyTag) {
      continue;
    }

    const familyKey = `${row.category}:${normalizeDerivedTag(ontologyTag.family)}`;
    const familyBucket = familyRecordKeys.get(familyKey) ?? new Set<string>();
    familyBucket.add(row.recordKey);
    familyRecordKeys.set(familyKey, familyBucket);

    const categoryBucket = categoryRecordKeys.get(row.category) ?? new Set<string>();
    categoryBucket.add(row.recordKey);
    categoryRecordKeys.set(row.category, categoryBucket);
  }

  const familyCounts = new Map<string, number>();
  for (const [key, recordKeys] of familyRecordKeys.entries()) {
    familyCounts.set(key, recordKeys.size);
  }

  const categoryCounts = new Map<SearchCategory, number>();
  for (const [category, recordKeys] of categoryRecordKeys.entries()) {
    categoryCounts.set(category, recordKeys.size);
  }

  return { tagCounts, familyCounts, categoryCounts };
}

export function buildDerivedTagOntologyExplorerModel(db: DatabaseSync): DerivedTagOntologyExplorerModel {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const liveCounts = buildLiveCountMaps(queryCanonicalTagRows(db));
  const authoredRuleCounts = buildAuthoredRuleCounts();
  const exemplarCounts = buildExemplarCounts();
  const legacyMigrationCounts = buildLegacyMigrationCounts();

  const categories = ontology.families
    .reduce<Map<SearchCategory, DerivedTagOntologyExplorerCategoryNode>>((bucket, family) => {
      const categoryNode = bucket.get(family.category) ?? {
        kind: "category" as const,
        key: family.category,
        category: family.category,
        familyCount: 0,
        tagCount: 0,
        taggedRecordCount: liveCounts.categoryCounts.get(family.category) ?? 0,
        families: [],
        filterText: family.category.toLowerCase(),
      };

      const familyKey = `${family.category}:${normalizeDerivedTag(family.family)}`;
      const tags = (ontology.tagsByFamilyKey.get(familyKey as `${SearchCategory}:${string}`) ?? [])
        .map((tag) => {
          const tagKey = `${tag.category}:${normalizeDerivedTag(tag.tag)}`;
          const exemplars = exemplarCounts.get(tagKey) ?? { positive: 0, negative: 0 };
          const migrations = legacyMigrationCounts.get(tagKey) ?? { definitions: 0, records: 0 };
          return {
            kind: "tag" as const,
            key: tagKey,
            category: tag.category,
            family: tag.family,
            subcategories: family.subcategories,
            tag: tag.tag,
            description: tag.description,
            assignmentMode: tag.assignmentMode,
            nativeOntologyPolicy: tag.nativeOntologyPolicy,
            appliesWhen: tag.appliesWhen,
            doesNotApplyWhen: tag.doesNotApplyWhen,
            positiveSignals: tag.positiveSignals,
            negativeSignals: tag.negativeSignals,
            adjacentTags: tag.adjacentTags,
            compositeOfAnyTags: tag.compositeOfAnyTags,
            variantInheritance: tag.variantInheritance,
            liveRecordCount: liveCounts.tagCounts.get(tagKey) ?? 0,
            authoredRuleCount: authoredRuleCounts.get(tagKey) ?? 0,
            exemplarPositiveCount: exemplars.positive,
            exemplarNegativeCount: exemplars.negative,
            legacyMigrationDefinitionCount: migrations.definitions,
            legacyMigrationRecordCount: migrations.records,
            filterText: buildTagFilterText(tag),
          };
        });

      const familyNode: DerivedTagOntologyExplorerFamilyNode = {
        kind: "family",
        key: familyKey,
        category: family.category,
        family: family.family,
        description: family.description,
        subcategories: family.subcategories,
        variantInheritance: family.variantInheritance,
        tagCount: tags.length,
        liveRecordCount: liveCounts.familyCounts.get(familyKey) ?? 0,
        tags,
        filterText: buildFamilyFilterText(family),
      };

      categoryNode.familyCount += 1;
      categoryNode.tagCount += tags.length;
      categoryNode.families.push(familyNode);
      bucket.set(family.category, categoryNode);
      return bucket;
    }, new Map())
    .values();

  return {
    categories: [...categories].map((category) => ({
      ...category,
      filterText: [
        category.category,
        ...category.families.map((family) => `${family.family} ${family.description}`),
      ].join(" ").toLowerCase(),
    })),
  };
}

export function filterOntologyExplorerNodes<T extends { filterText: string }>(
  nodes: T[],
  filter: string,
): T[] {
  const normalized = filter.trim().toLowerCase();
  if (!normalized) {
    return nodes;
  }
  return nodes.filter((node) => node.filterText.includes(normalized));
}

export function getPublishedExemplarPresence(
  category: SearchCategory,
  tag: string,
): { positive: number; negative: number } {
  const exemplars = getDerivedTagExemplars(tag, { category });
  return exemplars.reduce((totals, exemplarSet) => ({
    positive: totals.positive + exemplarSet.positives.length,
    negative: totals.negative + exemplarSet.negatives.length,
  }), { positive: 0, negative: 0 });
}
