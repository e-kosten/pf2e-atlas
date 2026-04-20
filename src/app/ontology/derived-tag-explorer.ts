import { DatabaseSync } from "node:sqlite";

import type { DerivedTagAssignmentMode } from "../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import {
  getCurrentDerivedTagMigrationAuthoredState,
  getCurrentDerivedTagMigrationAuthoredStateRevision,
} from "../../tags/migration/authored-state.js";
import { getDerivedTagExemplars, listDerivedTagLegacySeedMigrations, normalizeDerivedTag } from "../../tags/index.js";
import {
  buildOntologyExplorerEntityRecordSelectColumns,
  mapOntologyExplorerEntityRecordRow,
  type OntologyExplorerEntityRecord,
  type OntologyExplorerEntityRecordRow,
} from "./entity-record.js";
import { compareDisplayText, compareManagedCategory } from "../../tags/migration/list-sorting.js";
import { getPublishedDerivedTagMigrationOntology } from "../../tags/migration/runtime-state.js";
import { parseSearchCategoryValue } from "../../data/sql-row-decoding.js";

type ExplorerCountRow = {
  category: SearchCategory;
  tag: string;
  recordKey: string;
};

type RawExplorerCountRow = {
  category: string;
  tag: string;
  recordKey: string;
};

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

type DerivedTagOntologyExplorerDbCache = {
  tagCounts: Record<string, number>;
  familyCounts: Record<string, number>;
  categoryCounts: Record<string, number>;
  recordNodesByTagKey: Record<string, DerivedTagOntologyExplorerRecordNode[]>;
};

type DerivedTagOntologyExplorerCacheCountRow = {
  kind: "tag" | "family" | "category";
  key: string;
  count: number;
};

type DerivedTagOntologyExplorerCacheRecordKeyRow = {
  tagKey: string;
  recordKey: string;
};

type DerivedTagOntologyExplorerModelCacheEntry = {
  authoredStateRevision: number;
  model: DerivedTagOntologyExplorerModel;
};

const ONTOLOGY_EXPLORER_CACHE_ROW_ID = 1;
const explorerDbCacheByKey = new Map<string, DerivedTagOntologyExplorerDbCache>();
const explorerModelCacheByKey = new Map<string, DerivedTagOntologyExplorerModelCacheEntry>();

function buildTagFilterText(tag: {
  category: SearchCategory;
  family: string;
  tag: string;
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
    axis: string;
    description: string;
    subcategories?: SearchSubcategory[];
  },
  tags: Array<{ filterText: string }>,
): string {
  return [
    family.category,
    family.family,
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
  families: Array<{ family: string; description: string; filterText: string }>,
): string {
  return [
    category,
    ...families.map((family) => `${family.family} ${family.description}`),
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

function queryCanonicalTagRows(db: DatabaseSync): ExplorerCountRow[] {
  return (
    db
      .prepare(
        `
    SELECT r.category AS category, d.tag AS tag, d.record_key AS recordKey
    FROM record_derived_tags d
    JOIN records r ON r.record_key = d.record_key
    WHERE r.is_search_canonical = 1
  `,
      )
      .all() as RawExplorerCountRow[]
  ).map((row) => ({
    ...row,
    category: parseSearchCategoryValue(row.category, `ontology explorer live count row "${row.recordKey}"`),
  }));
}

function queryExplorerRecordRows(db: DatabaseSync, recordKeys: string[]): OntologyExplorerEntityRecordRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = recordKeys.map(() => "?").join(", ");
  const selectColumns = buildOntologyExplorerEntityRecordSelectColumns();
  return db
    .prepare(
      `
    SELECT
      ${selectColumns.join(",\n      ")}
    FROM records r
    LEFT JOIN actor_records a ON a.record_key = r.record_key
    LEFT JOIN item_records i ON i.record_key = r.record_key
    LEFT JOIN spell_records s ON s.record_key = r.record_key
    WHERE r.record_key IN (${placeholders})
  `,
    )
    .all(...recordKeys) as OntologyExplorerEntityRecordRow[];
}

function buildAuthoredRuleCounts(
  state: ReturnType<typeof getCurrentDerivedTagMigrationAuthoredState>,
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
  state: ReturnType<typeof getCurrentDerivedTagMigrationAuthoredState>,
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

function buildLiveCountMaps(rows: ExplorerCountRow[]): {
  tagCounts: Map<string, number>;
  familyCounts: Map<string, number>;
  categoryCounts: Map<SearchCategory, number>;
  recordKeysByTagKey: Map<string, string[]>;
} {
  const ontology = getPublishedDerivedTagMigrationOntology();
  const tagCounts = new Map<string, number>();
  const familyRecordKeys = new Map<string, Set<string>>();
  const categoryRecordKeys = new Map<SearchCategory, Set<string>>();
  const tagRecordKeys = new Map<string, Set<string>>();

  for (const row of rows) {
    const normalizedTag = normalizeDerivedTag(row.tag);
    const tagKey = `${row.category}:${normalizedTag}`;
    tagCounts.set(tagKey, (tagCounts.get(tagKey) ?? 0) + 1);

    const tagBucket = tagRecordKeys.get(tagKey) ?? new Set<string>();
    tagBucket.add(row.recordKey);
    tagRecordKeys.set(tagKey, tagBucket);

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

  const recordKeysByTagKey = new Map<string, string[]>();
  for (const [key, recordKeys] of tagRecordKeys.entries()) {
    recordKeysByTagKey.set(key, [...recordKeys]);
  }

  return { tagCounts, familyCounts, categoryCounts, recordKeysByTagKey };
}

function ensureOntologyExplorerCacheTable(db: DatabaseSync): void {
  db.exec(`
    CREATE TABLE IF NOT EXISTS derived_tag_ontology_explorer_cache_counts (
      cache_id INTEGER NOT NULL CHECK (cache_id = ${ONTOLOGY_EXPLORER_CACHE_ROW_ID}),
      count_kind TEXT NOT NULL CHECK (count_kind IN ('tag', 'family', 'category')),
      cache_key TEXT NOT NULL,
      count_value INTEGER NOT NULL,
      PRIMARY KEY (cache_id, count_kind, cache_key)
    );

    CREATE TABLE IF NOT EXISTS derived_tag_ontology_explorer_cache_record_keys (
      cache_id INTEGER NOT NULL CHECK (cache_id = ${ONTOLOGY_EXPLORER_CACHE_ROW_ID}),
      tag_key TEXT NOT NULL,
      record_key TEXT NOT NULL,
      PRIMARY KEY (cache_id, tag_key, record_key)
    )
  `);
}

function toSerializableLiveCounts(
  liveCounts: ReturnType<typeof buildLiveCountMaps>,
): Omit<DerivedTagOntologyExplorerDbCache, "recordNodesByTagKey"> {
  return {
    tagCounts: Object.fromEntries(liveCounts.tagCounts),
    familyCounts: Object.fromEntries(liveCounts.familyCounts),
    categoryCounts: Object.fromEntries(liveCounts.categoryCounts),
  };
}

function mapFromCountRows(
  rows: DerivedTagOntologyExplorerCacheCountRow[],
  kind: DerivedTagOntologyExplorerCacheCountRow["kind"],
): Map<string, number> {
  return new Map(rows.filter((row) => row.kind === kind).map((row) => [row.key, row.count] as const));
}

function mapFromCountRecord(record: Record<string, number>): Map<string, number> {
  return new Map<string, number>(Object.entries(record));
}

function buildDerivedTagOntologyExplorerDbCache(db: DatabaseSync): DerivedTagOntologyExplorerDbCache {
  const liveCounts = buildLiveCountMaps(queryCanonicalTagRows(db));
  const recordNodesByTagKey = buildRecordNodesByTagKey(db, liveCounts.recordKeysByTagKey);

  return {
    ...toSerializableLiveCounts(liveCounts),
    recordNodesByTagKey: Object.fromEntries(recordNodesByTagKey),
  };
}

function readDerivedTagOntologyExplorerDbCache(db: DatabaseSync): DerivedTagOntologyExplorerDbCache | null {
  try {
    const countRows = db
      .prepare(
        `
      SELECT count_kind AS kind, cache_key AS key, count_value AS count
      FROM derived_tag_ontology_explorer_cache_counts
      WHERE cache_id = ?
    `,
      )
      .all(ONTOLOGY_EXPLORER_CACHE_ROW_ID) as DerivedTagOntologyExplorerCacheCountRow[];
    const recordKeyRows = db
      .prepare(
        `
      SELECT tag_key AS tagKey, record_key AS recordKey
      FROM derived_tag_ontology_explorer_cache_record_keys
      WHERE cache_id = ?
      ORDER BY tag_key, record_key
    `,
      )
      .all(ONTOLOGY_EXPLORER_CACHE_ROW_ID) as DerivedTagOntologyExplorerCacheRecordKeyRow[];

    if (countRows.length === 0 && recordKeyRows.length === 0) {
      return null;
    }

    const rowsByTagKey = new Map<string, string[]>();
    for (const row of recordKeyRows) {
      const recordKeys = rowsByTagKey.get(row.tagKey);
      if (recordKeys) {
        recordKeys.push(row.recordKey);
        continue;
      }

      rowsByTagKey.set(row.tagKey, [row.recordKey]);
    }

    return {
      tagCounts: Object.fromEntries(mapFromCountRows(countRows, "tag")),
      familyCounts: Object.fromEntries(mapFromCountRows(countRows, "family")),
      categoryCounts: Object.fromEntries(
        Array.from(mapFromCountRows(countRows, "category"), ([category, count]) => [
          parseSearchCategoryValue(category, `ontology explorer cached category count "${category}"`),
          count,
        ]),
      ),
      recordNodesByTagKey: Object.fromEntries(buildRecordNodesByTagKey(db, rowsByTagKey)),
    };
  } catch {
    return null;
  }
}

export function writeDerivedTagOntologyExplorerDbCache(db: DatabaseSync): void {
  const cache = buildDerivedTagOntologyExplorerDbCache(db);
  ensureOntologyExplorerCacheTable(db);
  const deleteCounts = db.prepare(`
    DELETE FROM derived_tag_ontology_explorer_cache_counts
    WHERE cache_id = ?
  `);
  const deleteRecordKeys = db.prepare(`
    DELETE FROM derived_tag_ontology_explorer_cache_record_keys
    WHERE cache_id = ?
  `);
  const insertCount = db.prepare(`
    INSERT INTO derived_tag_ontology_explorer_cache_counts (cache_id, count_kind, cache_key, count_value)
    VALUES (?, ?, ?, ?)
  `);
  const insertRecordKey = db.prepare(`
    INSERT INTO derived_tag_ontology_explorer_cache_record_keys (cache_id, tag_key, record_key)
    VALUES (?, ?, ?)
  `);

  deleteCounts.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID);
  deleteRecordKeys.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID);

  for (const [key, count] of Object.entries(cache.tagCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "tag", key, count);
  }
  for (const [key, count] of Object.entries(cache.familyCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "family", key, count);
  }
  for (const [key, count] of Object.entries(cache.categoryCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "category", key, count);
  }
  for (const [tagKey, recordNodes] of Object.entries(cache.recordNodesByTagKey)) {
    for (const recordNode of recordNodes) {
      insertRecordKey.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, tagKey, recordNode.key);
    }
  }
}

function loadDerivedTagOntologyExplorerDbCache(db: DatabaseSync, cacheKey?: string): DerivedTagOntologyExplorerDbCache {
  if (cacheKey) {
    const cached = explorerDbCacheByKey.get(cacheKey);
    if (cached) {
      return cached;
    }
  }

  const resolved = readDerivedTagOntologyExplorerDbCache(db) ?? buildDerivedTagOntologyExplorerDbCache(db);
  if (cacheKey) {
    explorerDbCacheByKey.set(cacheKey, resolved);
  }
  return resolved;
}

function buildRecordNodesByTagKey(
  db: DatabaseSync,
  rowsByTagKey: Map<string, string[]>,
): Map<string, DerivedTagOntologyExplorerRecordNode[]> {
  const uniqueRecordKeys = [...new Set([...rowsByTagKey.values()].flat())];
  const recordsByKey = new Map(
    queryExplorerRecordRows(db, uniqueRecordKeys)
      .map((row) => mapOntologyExplorerEntityRecordRow(row))
      .map((record) => [record.recordKey, record] as const),
  );
  const nodesByTagKey = new Map<string, DerivedTagOntologyExplorerRecordNode[]>();

  for (const [tagKey, recordKeys] of rowsByTagKey.entries()) {
    const [, normalizedTag = ""] = tagKey.split(":", 2);
    const records = recordKeys
      .map((recordKey) => recordsByKey.get(recordKey))
      .filter((record): record is OntologyExplorerEntityRecord => Boolean(record))
      .sort((left, right) => compareDisplayText(left.name, right.name) || left.recordKey.localeCompare(right.recordKey))
      .map((record) => ({
        kind: "record" as const,
        key: record.recordKey,
        category: record.category,
        tag: normalizedTag,
        record,
        filterText: buildRecordFilterText(normalizedTag, record),
      }));
    nodesByTagKey.set(tagKey, records);
  }

  return nodesByTagKey;
}

export function buildDerivedTagOntologyExplorerModel(
  db: DatabaseSync,
  options: { cacheKey?: string } = {},
): DerivedTagOntologyExplorerModel {
  const authoredStateRevision = getCurrentDerivedTagMigrationAuthoredStateRevision();
  if (options.cacheKey) {
    const cached = explorerModelCacheByKey.get(options.cacheKey);
    if (cached && cached.authoredStateRevision === authoredStateRevision) {
      return cached.model;
    }
  }

  const ontology = getPublishedDerivedTagMigrationOntology();
  const authoredState = getCurrentDerivedTagMigrationAuthoredState();
  const dbCache = loadDerivedTagOntologyExplorerDbCache(db, options.cacheKey);
  const tagCounts = mapFromCountRecord(dbCache.tagCounts);
  const familyCounts = mapFromCountRecord(dbCache.familyCounts);
  const categoryCounts = new Map(Object.entries(dbCache.categoryCounts) as Array<[SearchCategory, number]>);
  const recordNodesByTagKey = new Map<string, DerivedTagOntologyExplorerRecordNode[]>(
    Object.entries(dbCache.recordNodesByTagKey),
  );
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
