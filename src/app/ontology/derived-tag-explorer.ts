import type { DerivedTagAssignmentMode } from "../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import {
  compareDisplayText,
  compareManagedCategory,
  getCurrentDerivedTagAuthoredState,
  getCurrentDerivedTagAuthoredStateRevision,
  getVisibleDerivedTagOntology,
} from "../../tags/editorial.js";
import {
  getDerivedTagExemplars,
  listDerivedTagLegacySeedMigrations,
  normalizeDerivedTag,
} from "../../tags/runtime.js";
import type { OntologyExplorerEntityRecord } from "./entity-record.js";

export type DerivedTagOntologyExplorerRecordNode = {
  kind: "record";
  key: string;
  category: SearchCategory;
  tag: string;
  record: OntologyExplorerEntityRecord;
  filterText: string;
};

export type DerivedTagOntologyExplorerTagNode = {
  kind: "tag";
  key: string;
  category: SearchCategory;
  family: string;
  subcategories?: SearchSubcategory[];
  tag: string;
  label: string;
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
  records: DerivedTagOntologyExplorerRecordNode[];
  filterText: string;
};

export type DerivedTagOntologyExplorerFamilyNode = {
  kind: "family";
  key: string;
  category: SearchCategory;
  family: string;
  label: string;
  axis: string;
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

export type DerivedTagOntologyExplorerData = {
  tagCounts: Record<string, number>;
  familyCounts: Record<string, number>;
  categoryCounts: Record<string, number>;
  recordsByTagKey: Record<string, OntologyExplorerEntityRecord[]>;
};

type DerivedTagOntologyExplorerModelCacheEntry = {
  authoredStateRevision: number;
  model: DerivedTagOntologyExplorerModel;
};

const explorerModelCacheByKey = new Map<string, DerivedTagOntologyExplorerModelCacheEntry>();

function buildTagFilterText(tag: {
  category: SearchCategory;
  family: string;
  tag: string;
  label?: string;
  description: string;
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
}): string {
  return [
    tag.category,
    tag.family,
    tag.tag,
    tag.label ?? "",
    tag.description,
    ...(tag.appliesWhen ?? []),
    ...(tag.doesNotApplyWhen ?? []),
    ...(tag.positiveSignals ?? []),
    ...(tag.negativeSignals ?? []),
    ...(tag.adjacentTags ?? []),
    ...(tag.compositeOfAnyTags ?? []),
  ]
    .join(" ")
    .toLowerCase();
}

function buildFamilyFilterText(
  family: {
    category: SearchCategory;
    family: string;
    label?: string;
    axis: string;
    description: string;
    subcategories?: SearchSubcategory[];
  },
  tags: Array<{ filterText: string }>,
): string {
  return [
    family.category,
    family.family,
    family.label ?? "",
    family.axis,
    family.description,
    ...(family.subcategories ?? []),
    ...tags.map((tag) => tag.filterText),
  ]
    .join(" ")
    .toLowerCase();
}

function buildCategoryFilterText(
  category: SearchCategory,
  families: Array<{ family: string; label?: string; description: string; filterText: string }>,
): string {
  return [
    category,
    ...families.map((family) => `${family.family} ${family.label ?? ""} ${family.description}`),
    ...families.map((family) => family.filterText),
  ]
    .join(" ")
    .toLowerCase();
}

function buildRecordFilterText(tag: string, record: OntologyExplorerEntityRecord): string {
  return [
    tag,
    record.name,
    record.recordKey,
    record.category,
    record.subcategory ?? "",
    record.type,
    record.documentType,
    record.rarity ?? "",
    ...record.traits,
    ...record.derivedTags,
    ...record.families,
    record.blurbText ?? "",
    record.descriptionText ?? "",
  ]
    .join(" ")
    .toLowerCase();
}

function buildAuthoredRuleCounts(
  state: ReturnType<typeof getCurrentDerivedTagAuthoredState>,
): Map<string, number> {
  const counts = new Map<string, number>();

  for (const [category, rules] of Object.entries(state.authoredRules) as Array<
    [SearchCategory, (typeof state.authoredRules)[keyof typeof state.authoredRules]]
  >) {
    for (const rule of rules) {
      const key = `${category}:${normalizeDerivedTag(rule.tag)}`;
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
  }

  return counts;
}

function buildExemplarCounts(
  state: ReturnType<typeof getCurrentDerivedTagAuthoredState>,
): Map<string, { positive: number; negative: number }> {
  const counts = new Map<string, { positive: number; negative: number }>();

  for (const [category, exemplarCategory] of Object.entries(state.exemplars) as Array<
    [SearchCategory, (typeof state.exemplars)[keyof typeof state.exemplars]]
  >) {
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

function mapFromCountRecord(record: Record<string, number>): Map<string, number> {
  return new Map<string, number>(Object.entries(record));
}

function buildRecordNodesByTagKey(
  recordsByTagKey: Map<string, OntologyExplorerEntityRecord[]>,
): Map<string, DerivedTagOntologyExplorerRecordNode[]> {
  const nodesByTagKey = new Map<string, DerivedTagOntologyExplorerRecordNode[]>();

  for (const [tagKey, records] of recordsByTagKey.entries()) {
    const [, normalizedTag = ""] = tagKey.split(":", 2);
    const recordNodes = records.map((record) => ({
      kind: "record" as const,
      key: record.recordKey,
      category: record.category,
      tag: normalizedTag,
      record,
      filterText: buildRecordFilterText(normalizedTag, record),
    }));
    nodesByTagKey.set(tagKey, recordNodes);
  }

  return nodesByTagKey;
}

export function buildDerivedTagOntologyExplorerModel(
  data: DerivedTagOntologyExplorerData,
  options: { cacheKey?: string } = {},
): DerivedTagOntologyExplorerModel {
  const authoredStateRevision = getCurrentDerivedTagAuthoredStateRevision();
  if (options.cacheKey) {
    const cached = explorerModelCacheByKey.get(options.cacheKey);
    if (cached && cached.authoredStateRevision === authoredStateRevision) {
      return cached.model;
    }
  }

  const ontology = getVisibleDerivedTagOntology();
  const authoredState = getCurrentDerivedTagAuthoredState();
  const tagCounts = mapFromCountRecord(data.tagCounts);
  const familyCounts = mapFromCountRecord(data.familyCounts);
  const categoryCounts = new Map(Object.entries(data.categoryCounts) as Array<[SearchCategory, number]>);
  const recordNodesByTagKey = buildRecordNodesByTagKey(new Map(Object.entries(data.recordsByTagKey)));
  const authoredRuleCounts = buildAuthoredRuleCounts(authoredState);
  const exemplarCounts = buildExemplarCounts(authoredState);
  const legacyMigrationCounts = buildLegacyMigrationCounts();

  const categories = ontology.families
    .reduce<Map<SearchCategory, DerivedTagOntologyExplorerCategoryNode>>((bucket, family) => {
      const categoryNode = bucket.get(family.category) ?? {
        kind: "category" as const,
        key: family.category,
        category: family.category,
        familyCount: 0,
        tagCount: 0,
        taggedRecordCount: categoryCounts.get(family.category) ?? 0,
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
            label: tag.label ?? tag.tag,
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
            liveRecordCount: tagCounts.get(tagKey) ?? 0,
            authoredRuleCount: authoredRuleCounts.get(tagKey) ?? 0,
            exemplarPositiveCount: exemplars.positive,
            exemplarNegativeCount: exemplars.negative,
            legacyMigrationDefinitionCount: migrations.definitions,
            legacyMigrationRecordCount: migrations.records,
            records: recordNodesByTagKey.get(tagKey) ?? [],
            filterText: buildTagFilterText(tag),
          };
        })
        .sort((left, right) => compareDisplayText(left.tag, right.tag) || left.key.localeCompare(right.key));

      const familyNode: DerivedTagOntologyExplorerFamilyNode = {
        kind: "family",
        key: familyKey,
        category: family.category,
        family: family.family,
        label: family.label ?? family.family,
        axis: family.axis,
        description: family.description,
        subcategories: family.subcategories,
        variantInheritance: family.variantInheritance,
        tagCount: tags.length,
        liveRecordCount: familyCounts.get(familyKey) ?? 0,
        tags,
        filterText: buildFamilyFilterText(family, tags),
      };

      categoryNode.familyCount += 1;
      categoryNode.tagCount += tags.length;
      categoryNode.families.push(familyNode);
      bucket.set(family.category, categoryNode);
      return bucket;
    }, new Map())
    .values();

  const model = {
    categories: [...categories]
      .map((category) => ({
        ...category,
        families: [...category.families].sort(
          (left, right) =>
            compareDisplayText(left.axis, right.axis) ||
            compareDisplayText(left.family, right.family) ||
            left.key.localeCompare(right.key),
        ),
        filterText: buildCategoryFilterText(category.category, category.families),
      }))
      .sort((left, right) => compareManagedCategory(left.category, right.category)),
  };

  if (options.cacheKey) {
    explorerModelCacheByKey.set(options.cacheKey, {
      authoredStateRevision,
      model,
    });
  }

  return model;
}

export function filterOntologyExplorerNodes<T extends { filterText: string }>(nodes: T[], filter: string): T[] {
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
  return exemplars.reduce(
    (totals, exemplarSet) => ({
      positive: totals.positive + exemplarSet.positives.length,
      negative: totals.negative + exemplarSet.negatives.length,
    }),
    { positive: 0, negative: 0 },
  );
}
