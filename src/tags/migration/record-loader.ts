import { DatabaseSync } from "node:sqlite";

import { buildPlaceholders } from "../../data/rows.js";
import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { normalizeText } from "../../utils.js";

export type DerivedTagMigrationReference = {
  recordKey: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  traits: string[];
};

export type DerivedTagMigrationRecord = {
  recordKey: string;
  packName: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  traits: string[];
  families: string[];
  derivedTags: string[];
  descriptionText: string | null;
  blurbText: string | null;
  references: DerivedTagMigrationReference[];
};

type LoadedRecordRow = {
  recordKey: string;
  packName: string | null;
  name: string;
  category: string;
  subcategory: string | null;
  level: number | bigint | null;
  traitsJson: string;
  familiesJson: string | null;
  derivedTagsJson: string;
  descriptionText: string | null;
  blurbText: string | null;
};

type LoadedReferenceRow = {
  fromRecordKey: string;
  targetRecordKey: string;
  targetPackName: string | null;
  targetName: string;
  targetCategory: string;
  targetSubcategory: string | null;
  targetTraitsJson: string;
};

function loadReferences(
  db: DatabaseSync,
  recordKeys: string[],
): Map<string, DerivedTagMigrationReference[]> {
  if (recordKeys.length === 0) {
    return new Map();
  }

  const rows = db.prepare(`
    SELECT
      re.from_record_key AS fromRecordKey,
      re.to_record_key AS targetRecordKey,
      target.pack_name AS targetPackName,
      target.name AS targetName,
      target.category AS targetCategory,
      target.subcategory AS targetSubcategory,
      target.traits_json AS targetTraitsJson
    FROM reference_edges re
    JOIN records target ON target.record_key = re.to_record_key
    WHERE re.from_record_key IN (${buildPlaceholders(recordKeys)})
  `).all(...recordKeys) as LoadedReferenceRow[];

  const referencesByRecordKey = new Map<string, DerivedTagMigrationReference[]>();
  for (const row of rows) {
    const bucket = referencesByRecordKey.get(row.fromRecordKey) ?? [];
    bucket.push({
      recordKey: row.targetRecordKey,
      packName: row.targetPackName ?? row.targetRecordKey.split(":")[0] ?? "",
      name: row.targetName,
      category: row.targetCategory as SearchCategory,
      subcategory: (row.targetSubcategory ?? null) as SearchSubcategory | null,
      traits: JSON.parse(row.targetTraitsJson) as string[],
    });
    referencesByRecordKey.set(row.fromRecordKey, bucket);
  }

  return referencesByRecordKey;
}

export type LoadDerivedTagMigrationRecordsOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  recordKeys?: string[];
  requireTag?: string;
  untaggedOnly?: boolean;
  limit?: number;
};

function appendRecordFilters(
  sql: string[],
  params: Array<string | number>,
  options: LoadDerivedTagMigrationRecordsOptions,
): void {
  if (options.category) {
    sql.push("AND r.category = ?");
    params.push(options.category);
  }
  if (options.subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(options.subcategory);
  }
  if (options.requireTag) {
    sql.push("AND EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(normalizeText(options.requireTag));
  }
  if (options.untaggedOnly) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key)");
  }
  if (options.recordKeys && options.recordKeys.length > 0) {
    sql.push(`AND r.record_key IN (${buildPlaceholders(options.recordKeys)})`);
    params.push(...options.recordKeys);
  }
}

export function countDerivedTagMigrationRecords(
  db: DatabaseSync,
  options: LoadDerivedTagMigrationRecordsOptions,
): number {
  const sql = [
    "SELECT COUNT(*) AS count",
    "FROM records r",
    "WHERE r.is_search_canonical = 1",
  ];
  const params: Array<string | number> = [];
  appendRecordFilters(sql, params, options);

  const row = db.prepare(sql.join("\n")).get(...params) as { count: number | bigint } | undefined;
  return typeof row?.count === "bigint" ? Number(row.count) : (row?.count ?? 0);
}

export function loadDerivedTagMigrationRecords(
  db: DatabaseSync,
  options: LoadDerivedTagMigrationRecordsOptions,
): DerivedTagMigrationRecord[] {
  const sql = [
    "SELECT",
    "  r.record_key AS recordKey,",
    "  r.pack_name AS packName,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.families_json AS familiesJson,",
    "  r.derived_tags_json AS derivedTagsJson,",
    "  r.description_text AS descriptionText,",
    "  r.blurb_text AS blurbText",
    "FROM records r",
    "WHERE r.is_search_canonical = 1",
  ];
  const params: Array<string | number> = [];
  appendRecordFilters(sql, params, options);
  sql.push("ORDER BY r.name ASC, r.record_key ASC");
  if (options.limit && options.limit > 0) {
    sql.push(`LIMIT ${Math.trunc(options.limit)}`);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as LoadedRecordRow[];
  const referencesByRecordKey = loadReferences(db, rows.map((row) => row.recordKey));

  return rows.map((row) => ({
    recordKey: row.recordKey,
    packName: row.packName ?? row.recordKey.split(":")[0] ?? "",
    name: row.name,
    category: row.category as SearchCategory,
    subcategory: (row.subcategory ?? null) as SearchSubcategory | null,
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: JSON.parse(row.traitsJson) as string[],
    families: row.familiesJson ? (JSON.parse(row.familiesJson) as string[]) : [],
    derivedTags: JSON.parse(row.derivedTagsJson) as string[],
    descriptionText: row.descriptionText,
    blurbText: row.blurbText,
    references: referencesByRecordKey.get(row.recordKey) ?? [],
  }));
}
