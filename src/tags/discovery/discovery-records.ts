import { DatabaseSync } from "node:sqlite";

import { buildPlaceholders } from "../../data/rows.js";
import {
  categorySupportsSubcategory,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { SearchCategory, SearchSubcategory, SourceCategory } from "../../types.js";
import { normalizeText } from "../../utils.js";

export type DiscoveryReferenceRecord = {
  targetRecordKey: string;
  targetName: string;
  targetCategory: SearchCategory;
  targetSubcategory: SearchSubcategory | null;
  fromPackName: string;
  fromRecordType: string;
  fromSourceCategory: SourceCategory;
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
    return segments.length >= 2 ? (segments[0] ?? null) : null;
  }

  const segments = normalizedPath.split("/").filter(Boolean);
  const packNameIndex = segments.lastIndexOf(packName);
  if (packNameIndex >= 0 && segments.length > packNameIndex + 2) {
    return segments[packNameIndex + 1] ?? null;
  }

  return null;
}

function parseStringArrayJson(value: string | null | undefined, fieldName: string, recordKey: string): string[] {
  if (!value) {
    return [];
  }

  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed)) {
    throw new Error(`Expected ${fieldName} for "${recordKey}" to be a JSON string array.`);
  }

  const result: string[] = [];
  for (const entry of parsed) {
    if (typeof entry !== "string") {
      throw new Error(`Expected ${fieldName} for "${recordKey}" to be a JSON string array.`);
    }
    if (entry.length > 0) {
      result.push(entry);
    }
  }

  return result;
}

function parseDiscoveryCategory(category: string, recordKey: string): SearchCategory {
  const normalized = normalizeSearchCategory(category);
  if (!normalized) {
    throw new Error(`Invalid discovery category "${category}" for "${recordKey}".`);
  }

  return normalized;
}

function parseDiscoverySubcategory(
  category: SearchCategory,
  subcategory: string | null,
  recordKey: string,
): SearchSubcategory | null {
  if (!subcategory) {
    return null;
  }

  const normalized = normalizeSearchSubcategory(subcategory);
  if (!normalized) {
    throw new Error(`Invalid discovery subcategory "${subcategory}" for "${recordKey}".`);
  }
  if (!categorySupportsSubcategory(category, normalized)) {
    throw new Error(`Invalid discovery subcategory "${subcategory}" for ${category} record "${recordKey}".`);
  }

  return normalized;
}

function parseSourceCategory(sourceCategory: string, recordKey: string): SourceCategory {
  switch (normalizeText(sourceCategory)) {
    case "core":
      return "core";
    case "rules":
      return "rules";
    case "adventure":
      return "adventure";
    case "unknown":
      return "unknown";
    default:
      throw new Error(`Invalid discovery source category "${sourceCategory}" for "${recordKey}".`);
  }
}

function toDiscoveryRecord(row: LoadedRecordRow, references: DiscoveryReferenceRecord[]): DiscoveryAnalysisRecord {
  const separatorIndex = row.recordKey.indexOf(":");
  const sourceKey = separatorIndex >= 0 ? row.recordKey.slice(0, separatorIndex) : row.recordKey;
  const packName = row.packName ?? sourceKey;
  const category = parseDiscoveryCategory(row.category, row.recordKey);
  return {
    recordKey: row.recordKey,
    sourceKey,
    packName,
    publicationTitle: row.publicationTitle,
    folderId: row.folderId,
    sourcePath: row.sourcePath,
    sourcePathSlice: deriveSourcePathSlice(packName, row.sourcePath),
    name: row.name,
    category,
    subcategory: parseDiscoverySubcategory(category, row.subcategory, row.recordKey),
    variantFamilyKey: row.variantFamilyKey,
    variantBaseName: row.variantBaseName,
    variantLabel: row.variantLabel,
    variantAxes: parseStringArrayJson(row.variantAxesJson, "variantAxesJson", row.recordKey),
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: parseStringArrayJson(row.traitsJson, "traitsJson", row.recordKey),
    derivedTags: parseStringArrayJson(row.derivedTagsJson, "derivedTagsJson", row.recordKey),
    descriptionText: row.descriptionText,
    vector: decodeDiscoveryVector(row.vectorBlob),
    references,
  };
}

function loadReferencesForRecords(db: DatabaseSync, recordKeys: string[]): Map<string, DiscoveryReferenceRecord[]> {
  if (recordKeys.length === 0) {
    return new Map();
  }

  const placeholders = buildPlaceholders(recordKeys);
  const rows = db
    .prepare(
      `
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
  `,
    )
    .all(...recordKeys) as LoadedReferenceRow[];

  const referencesByRecordKey = new Map<string, DiscoveryReferenceRecord[]>();
  for (const row of rows) {
    const bucket = referencesByRecordKey.get(row.fromRecordKey) ?? [];
    const targetCategory = parseDiscoveryCategory(row.targetCategory, row.targetRecordKey);
    bucket.push({
      targetRecordKey: row.targetRecordKey,
      targetName: row.targetName,
      targetCategory,
      targetSubcategory: parseDiscoverySubcategory(targetCategory, row.targetSubcategory, row.targetRecordKey),
      fromPackName: row.fromPackName,
      fromRecordType: row.fromRecordType,
      fromSourceCategory: parseSourceCategory(row.fromSourceCategory, row.fromRecordKey),
    });
    referencesByRecordKey.set(row.fromRecordKey, bucket);
  }

  return referencesByRecordKey;
}

export function loadDiscoveryRecords(db: DatabaseSync, options: DiscoveryRecordLoadOptions): DiscoveryAnalysisRecord[] {
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
    sql.push(
      `AND EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag IN (${buildPlaceholders(options.requireAnyDerivedTags)}))`,
    );
    params.push(...options.requireAnyDerivedTags);
  }
  if (options.excludeDerivedTag) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(options.excludeDerivedTag);
  }
  if (options.excludeAnyDerivedTags && options.excludeAnyDerivedTags.length > 0) {
    sql.push(
      `AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag IN (${buildPlaceholders(options.excludeAnyDerivedTags)}))`,
    );
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
  const referencesByRecordKey = loadReferencesForRecords(
    db,
    rows.map((row) => row.recordKey),
  );
  return rows.map((row) => toDiscoveryRecord(row, referencesByRecordKey.get(row.recordKey) ?? []));
}

function resolveNameExemplar(db: DatabaseSync, query: string, options: DiscoveryExemplarOptions): ResolvedExemplarRow {
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

function loadSingleDiscoveryRecord(
  db: DatabaseSync,
  options: DiscoveryRecordLoadOptions & { recordKey: string },
): DiscoveryAnalysisRecord {
  const records = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: [options.recordKey],
    includeVectors: options.includeVectors,
    includeDerivedTags: options.includeDerivedTags,
  });
  const record = records[0];
  if (!record) {
    throw new Error(`Could not resolve exemplar record key "${options.recordKey}" within the requested scope.`);
  }

  return record;
}

export function resolveDiscoveryExemplars(
  db: DatabaseSync,
  options: DiscoveryExemplarOptions,
): ResolvedDiscoveryExemplar[] {
  const rows: ResolvedExemplarRow[] = [];
  for (const recordKey of options.exemplarRecordKeys ?? []) {
    const record = loadSingleDiscoveryRecord(db, {
      category: options.category,
      subcategory: options.subcategory,
      recordKey,
      includeVectors: true,
    });
    rows.push({
      query: recordKey,
      matchedBy: "recordKey",
      recordKey: record.recordKey,
      packName: record.packName,
      publicationTitle: record.publicationTitle,
      folderId: record.folderId,
      sourcePath: record.sourcePath,
      name: record.name,
      category: record.category,
      subcategory: record.subcategory,
      variantFamilyKey: record.variantFamilyKey,
      variantBaseName: record.variantBaseName,
      variantLabel: record.variantLabel,
      variantAxesJson: JSON.stringify(record.variantAxes),
      level: record.level,
      traitsJson: JSON.stringify(record.traits),
      derivedTagsJson: JSON.stringify(record.derivedTags),
      descriptionText: record.descriptionText,
      vectorBlob: record.vector.length > 0 ? new Uint8Array(record.vector.buffer.slice(0)) : null,
    });
  }

  for (const name of options.exemplarNames ?? []) {
    rows.push(resolveNameExemplar(db, name, options));
  }

  const uniqueRows = dedupeExemplarRows(rows);
  const referencesByRecordKey = loadReferencesForRecords(
    db,
    uniqueRows.map((row) => row.recordKey),
  );
  return uniqueRows.map((row) => ({
    query: row.query,
    matchedBy: row.matchedBy,
    ...toDiscoveryRecord(row, referencesByRecordKey.get(row.recordKey) ?? []),
  }));
}
