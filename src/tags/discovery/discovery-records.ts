import { DatabaseSync } from "node:sqlite";

import { buildPlaceholders } from "../../data/rows.js";
import { SearchCategory, SearchSubcategory } from "../../types.js";
import { normalizeText } from "../../utils.js";

export type DiscoveryReferenceRecord = {
  targetRecordKey: string;
  targetName: string;
  targetCategory: SearchCategory;
  targetSubcategory: SearchSubcategory | null;
  fromPackName: string;
  fromRecordType: string;
  fromSourceCategory: string;
};

export type DiscoveryAnalysisRecord = {
  recordKey: string;
  sourceKey: string;
  packName: string;
  publicationTitle: string | null;
  folderId: string | null;
  sourcePath: string | null;
  sourcePathSlice: string | null;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  variantFamilyKey: string | null;
  variantBaseName: string | null;
  variantLabel: string | null;
  variantAxes: string[];
  level: number | null;
  traits: string[];
  derivedTags: string[];
  descriptionText: string | null;
  vector: Float32Array;
  references: DiscoveryReferenceRecord[];
};

export type DiscoveryRecordLoadOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  recordKeys?: string[];
  excludeRecordKeys?: string[];
  requireTag?: string;
  requireAnyDerivedTags?: string[];
  excludeDerivedTag?: string;
  excludeAnyDerivedTags?: string[];
  untaggedOnly?: boolean;
  includeVectors?: boolean;
  includeDerivedTags?: boolean;
};

export type DiscoveryExemplarOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  exemplarNames?: string[];
  exemplarRecordKeys?: string[];
};

type LoadedRecordRow = {
  recordKey: string;
  packName: string | null;
  publicationTitle: string | null;
  folderId: string | null;
  sourcePath: string | null;
  name: string;
  category: string;
  subcategory: string | null;
  level: number | bigint | null;
  variantFamilyKey: string | null;
  variantBaseName: string | null;
  variantLabel: string | null;
  variantAxesJson: string | null;
  traitsJson: string;
  derivedTagsJson: string;
  descriptionText: string | null;
  vectorBlob: Uint8Array | null;
};

type LoadedReferenceRow = {
  fromRecordKey: string;
  targetRecordKey: string;
  targetName: string;
  targetCategory: string;
  targetSubcategory: string | null;
  fromPackName: string;
  fromRecordType: string;
  fromSourceCategory: string;
};

type ResolvedExemplarRow = LoadedRecordRow & {
  query: string;
  matchedBy: "recordKey" | "name" | "alias";
};

export type ResolvedDiscoveryExemplar = DiscoveryAnalysisRecord & {
  query: string;
  matchedBy: "recordKey" | "name" | "alias";
};

export function decodeDiscoveryVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

export function deriveSourcePathSlice(packName: string, sourcePath: string | null): string | null {
  if (!sourcePath) {
    return null;
  }

  const normalizedPath = sourcePath.replace(/\\/g, "/");
  const packMarker = `/${packName}/`;
  const packIndex = normalizedPath.lastIndexOf(packMarker);
  if (packIndex >= 0) {
    const relative = normalizedPath.slice(packIndex + packMarker.length);
    const segments = relative.split("/").filter(Boolean);
    return segments.length >= 2 ? segments[0] ?? null : null;
  }

  const segments = normalizedPath.split("/").filter(Boolean);
  const packNameIndex = segments.lastIndexOf(packName);
  if (packNameIndex >= 0 && segments.length > packNameIndex + 2) {
    return segments[packNameIndex + 1] ?? null;
  }

  return null;
}

function toDiscoveryRecord(
  row: LoadedRecordRow,
  references: DiscoveryReferenceRecord[],
): DiscoveryAnalysisRecord {
  const separatorIndex = row.recordKey.indexOf(":");
  const sourceKey = separatorIndex >= 0 ? row.recordKey.slice(0, separatorIndex) : row.recordKey;
  const packName = row.packName ?? sourceKey;
  return {
    recordKey: row.recordKey,
    sourceKey,
    packName,
    publicationTitle: row.publicationTitle,
    folderId: row.folderId,
    sourcePath: row.sourcePath,
    sourcePathSlice: deriveSourcePathSlice(packName, row.sourcePath),
    name: row.name,
    category: row.category as SearchCategory,
    subcategory: (row.subcategory ?? null) as SearchSubcategory | null,
    variantFamilyKey: row.variantFamilyKey,
    variantBaseName: row.variantBaseName,
    variantLabel: row.variantLabel,
    variantAxes: row.variantAxesJson ? (JSON.parse(row.variantAxesJson) as string[]) : [],
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: JSON.parse(row.traitsJson) as string[],
    derivedTags: row.derivedTagsJson ? (JSON.parse(row.derivedTagsJson) as string[]) : [],
    descriptionText: row.descriptionText,
    vector: decodeDiscoveryVector(row.vectorBlob),
    references,
  };
}

function loadReferencesForRecords(
  db: DatabaseSync,
  recordKeys: string[],
): Map<string, DiscoveryReferenceRecord[]> {
  if (recordKeys.length === 0) {
    return new Map();
  }

  const placeholders = buildPlaceholders(recordKeys);
  const rows = db.prepare(`
    SELECT
      re.from_record_key AS fromRecordKey,
      re.to_record_key AS targetRecordKey,
      target.name AS targetName,
      target.category AS targetCategory,
      target.subcategory AS targetSubcategory,
      re.from_pack_name AS fromPackName,
      re.from_record_type AS fromRecordType,
      re.from_source_category AS fromSourceCategory
    FROM reference_edges re
    JOIN records target ON target.record_key = re.to_record_key
    WHERE re.from_record_key IN (${placeholders})
  `).all(...recordKeys) as LoadedReferenceRow[];

  const referencesByRecordKey = new Map<string, DiscoveryReferenceRecord[]>();
  for (const row of rows) {
    const bucket = referencesByRecordKey.get(row.fromRecordKey) ?? [];
    bucket.push({
      targetRecordKey: row.targetRecordKey,
      targetName: row.targetName,
      targetCategory: row.targetCategory as SearchCategory,
      targetSubcategory: (row.targetSubcategory ?? null) as SearchSubcategory | null,
      fromPackName: row.fromPackName,
      fromRecordType: row.fromRecordType,
      fromSourceCategory: row.fromSourceCategory,
    });
    referencesByRecordKey.set(row.fromRecordKey, bucket);
  }

  return referencesByRecordKey;
}

export function loadDiscoveryRecords(
  db: DatabaseSync,
  options: DiscoveryRecordLoadOptions,
): DiscoveryAnalysisRecord[] {
  const sql = [
    "SELECT",
    "  r.record_key AS recordKey,",
    "  r.pack_name AS packName,",
    "  r.publication_title AS publicationTitle,",
    "  r.folder_id AS folderId,",
    "  r.source_path AS sourcePath,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.variant_family_key AS variantFamilyKey,",
    "  r.variant_base_name AS variantBaseName,",
    "  r.variant_label AS variantLabel,",
    "  r.variant_axes_json AS variantAxesJson,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    `  ${options.includeDerivedTags === false ? "NULL" : "r.derived_tags_json"} AS derivedTagsJson,`,
    "  r.description_text AS descriptionText,",
    `  ${options.includeVectors === false ? "NULL" : "e.vector_blob"} AS vectorBlob`,
    "FROM records r",
    options.includeVectors === false ? "" : "LEFT JOIN embeddings e ON e.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
  ].filter(Boolean);
  const params: Array<string | number> = [];

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
    params.push(options.requireTag);
  }
  if (options.requireAnyDerivedTags && options.requireAnyDerivedTags.length > 0) {
    sql.push(`AND EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag IN (${buildPlaceholders(options.requireAnyDerivedTags)}))`);
    params.push(...options.requireAnyDerivedTags);
  }
  if (options.excludeDerivedTag) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(options.excludeDerivedTag);
  }
  if (options.excludeAnyDerivedTags && options.excludeAnyDerivedTags.length > 0) {
    sql.push(`AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag IN (${buildPlaceholders(options.excludeAnyDerivedTags)}))`);
    params.push(...options.excludeAnyDerivedTags);
  }
  if (options.untaggedOnly) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key)");
  }
  if (options.recordKeys && options.recordKeys.length > 0) {
    sql.push(`AND r.record_key IN (${buildPlaceholders(options.recordKeys)})`);
    params.push(...options.recordKeys);
  }
  if (options.excludeRecordKeys && options.excludeRecordKeys.length > 0) {
    sql.push(`AND r.record_key NOT IN (${buildPlaceholders(options.excludeRecordKeys)})`);
    params.push(...options.excludeRecordKeys);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as LoadedRecordRow[];
  const referencesByRecordKey = loadReferencesForRecords(db, rows.map((row) => row.recordKey));
  return rows.map((row) => toDiscoveryRecord(row, referencesByRecordKey.get(row.recordKey) ?? []));
}

function resolveNameExemplar(
  db: DatabaseSync,
  query: string,
  options: DiscoveryExemplarOptions,
): ResolvedExemplarRow {
  const normalized = normalizeText(query);
  const sql = [
    "SELECT",
    "  ? AS query,",
    "  match_source.matchedBy AS matchedBy,",
    "  r.record_key AS recordKey,",
    "  r.pack_name AS packName,",
    "  r.publication_title AS publicationTitle,",
    "  r.folder_id AS folderId,",
    "  r.source_path AS sourcePath,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.variant_family_key AS variantFamilyKey,",
    "  r.variant_base_name AS variantBaseName,",
    "  r.variant_label AS variantLabel,",
    "  r.variant_axes_json AS variantAxesJson,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.derived_tags_json AS derivedTagsJson,",
    "  r.description_text AS descriptionText,",
    "  e.vector_blob AS vectorBlob",
    "FROM (",
    "  SELECT record_key, 'name' AS matchedBy",
    "  FROM records",
    "  WHERE normalized_name = ?",
    "  UNION ALL",
    "  SELECT canonical_record_key AS record_key, 'alias' AS matchedBy",
    "  FROM record_aliases",
    "  WHERE normalized_alias = ?",
    ") match_source",
    "JOIN records r ON r.record_key = match_source.record_key",
    "LEFT JOIN embeddings e ON e.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
  ];
  const params: Array<string | number> = [query, normalized, normalized];

  if (options.category) {
    sql.push("AND r.category = ?");
    params.push(options.category);
  }
  if (options.subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(options.subcategory);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as ResolvedExemplarRow[];
  const uniqueRows = dedupeExemplarRows(rows);
  if (uniqueRows.length === 0) {
    throw new Error(`Could not resolve exemplar "${query}" within the requested scope.`);
  }
  if (uniqueRows.length > 1) {
    throw new Error(`Exemplar "${query}" is ambiguous within the requested scope.`);
  }

  return uniqueRows[0]!;
}

function dedupeExemplarRows(rows: ResolvedExemplarRow[]): ResolvedExemplarRow[] {
  const byRecordKey = new Map<string, ResolvedExemplarRow>();
  for (const row of rows) {
    const existing = byRecordKey.get(row.recordKey);
    if (!existing) {
      byRecordKey.set(row.recordKey, row);
      continue;
    }

    if (existing.matchedBy === "alias" && row.matchedBy === "name") {
      byRecordKey.set(row.recordKey, row);
    }
  }

  return [...byRecordKey.values()];
}

export function resolveDiscoveryExemplars(
  db: DatabaseSync,
  options: DiscoveryExemplarOptions,
): ResolvedDiscoveryExemplar[] {
  const rows: ResolvedExemplarRow[] = [];
  for (const recordKey of options.exemplarRecordKeys ?? []) {
    const records = loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      recordKeys: [recordKey],
      includeVectors: true,
    });
    if (records.length === 0) {
      throw new Error(`Could not resolve exemplar record key "${recordKey}" within the requested scope.`);
    }

    rows.push({
      query: recordKey,
      matchedBy: "recordKey",
      recordKey: records[0]!.recordKey,
      packName: records[0]!.packName,
      publicationTitle: records[0]!.publicationTitle,
      folderId: records[0]!.folderId,
      sourcePath: records[0]!.sourcePath,
      name: records[0]!.name,
      category: records[0]!.category,
      subcategory: records[0]!.subcategory,
      variantFamilyKey: records[0]!.variantFamilyKey,
      variantBaseName: records[0]!.variantBaseName,
      variantLabel: records[0]!.variantLabel,
      variantAxesJson: JSON.stringify(records[0]!.variantAxes),
      level: records[0]!.level,
      traitsJson: JSON.stringify(records[0]!.traits),
      derivedTagsJson: JSON.stringify(records[0]!.derivedTags),
      descriptionText: records[0]!.descriptionText,
      vectorBlob: records[0]!.vector.length > 0 ? new Uint8Array(records[0]!.vector.buffer.slice(0)) : null,
    });
  }

  for (const name of options.exemplarNames ?? []) {
    rows.push(resolveNameExemplar(db, name, options));
  }

  const uniqueRows = dedupeExemplarRows(rows);
  const referencesByRecordKey = loadReferencesForRecords(db, uniqueRows.map((row) => row.recordKey));
  return uniqueRows.map((row) => ({
    query: row.query,
    matchedBy: row.matchedBy,
    ...toDiscoveryRecord(row, referencesByRecordKey.get(row.recordKey) ?? []),
  }));
}
