import { DatabaseSync } from "node:sqlite";

import type { SearchCategory } from "../../domain/search-types.js";
import { parseSearchCategoryValue } from "../../data/sql-row-decoding.js";
import { compareDisplayText, getVisibleDerivedTagOntology } from "../../tags/editorial.js";
import { normalizeDerivedTag } from "../../tags/runtime.js";
import type { DerivedTagOntologyExplorerData } from "./derived-tag-explorer.js";
import {
  buildOntologyExplorerEntityRecordSelectColumns,
  mapOntologyExplorerEntityRecordRow,
  type OntologyExplorerEntityRecord,
  type OntologyExplorerEntityRecordRow,
} from "./entity-record.js";

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

type DerivedTagOntologyExplorerCacheCountRow = {
  kind: "tag" | "family" | "category";
  key: string;
  count: number;
};

type DerivedTagOntologyExplorerCacheRecordKeyRow = {
  tagKey: string;
  recordKey: string;
};

const ONTOLOGY_EXPLORER_CACHE_ROW_ID = 1;
const explorerDbCacheByKey = new Map<string, DerivedTagOntologyExplorerData>();

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

function buildLiveCountMaps(rows: ExplorerCountRow[]): {
  tagCounts: Map<string, number>;
  familyCounts: Map<string, number>;
  categoryCounts: Map<SearchCategory, number>;
  recordKeysByTagKey: Map<string, string[]>;
} {
  const ontology = getVisibleDerivedTagOntology();
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
): Omit<DerivedTagOntologyExplorerData, "recordsByTagKey"> {
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

function buildExplorerRecordsByTagKey(
  db: DatabaseSync,
  rowsByTagKey: Map<string, string[]>,
): Map<string, OntologyExplorerEntityRecord[]> {
  const uniqueRecordKeys = [...new Set([...rowsByTagKey.values()].flat())];
  const recordsByKey = new Map(
    queryExplorerRecordRows(db, uniqueRecordKeys)
      .map((row) => mapOntologyExplorerEntityRecordRow(row))
      .map((record) => [record.recordKey, record] as const),
  );
  const recordsByTagKey = new Map<string, OntologyExplorerEntityRecord[]>();

  for (const [tagKey, recordKeys] of rowsByTagKey.entries()) {
    const records = recordKeys
      .map((recordKey) => recordsByKey.get(recordKey))
      .filter((record): record is OntologyExplorerEntityRecord => Boolean(record))
      .sort((left, right) => compareDisplayText(left.name, right.name) || left.recordKey.localeCompare(right.recordKey));
    recordsByTagKey.set(tagKey, records);
  }

  return recordsByTagKey;
}

function buildDerivedTagOntologyExplorerData(db: DatabaseSync): DerivedTagOntologyExplorerData {
  const liveCounts = buildLiveCountMaps(queryCanonicalTagRows(db));
  const recordsByTagKey = buildExplorerRecordsByTagKey(db, liveCounts.recordKeysByTagKey);

  return {
    ...toSerializableLiveCounts(liveCounts),
    recordsByTagKey: Object.fromEntries(recordsByTagKey),
  };
}

function readDerivedTagOntologyExplorerDbCache(db: DatabaseSync): DerivedTagOntologyExplorerData | null {
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
      recordsByTagKey: Object.fromEntries(buildExplorerRecordsByTagKey(db, rowsByTagKey)),
    };
  } catch {
    return null;
  }
}

export function writeDerivedTagOntologyExplorerDbCache(db: DatabaseSync): void {
  const data = buildDerivedTagOntologyExplorerData(db);
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

  for (const [key, count] of Object.entries(data.tagCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "tag", key, count);
  }
  for (const [key, count] of Object.entries(data.familyCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "family", key, count);
  }
  for (const [key, count] of Object.entries(data.categoryCounts)) {
    insertCount.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, "category", key, count);
  }
  for (const [tagKey, records] of Object.entries(data.recordsByTagKey)) {
    for (const record of records) {
      insertRecordKey.run(ONTOLOGY_EXPLORER_CACHE_ROW_ID, tagKey, record.recordKey);
    }
  }
}

export function loadDerivedTagOntologyExplorerData(
  db: DatabaseSync,
  options: { cacheKey?: string } = {},
): DerivedTagOntologyExplorerData {
  if (options.cacheKey) {
    const cached = explorerDbCacheByKey.get(options.cacheKey);
    if (cached) {
      return cached;
    }
  }

  const resolved = readDerivedTagOntologyExplorerDbCache(db) ?? buildDerivedTagOntologyExplorerData(db);
  if (options.cacheKey) {
    explorerDbCacheByKey.set(options.cacheKey, resolved);
  }
  return resolved;
}
