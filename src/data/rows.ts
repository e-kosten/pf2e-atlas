import type {
  NormalizedRecord,
  SourceCategory,
  VariantSource,
} from "../domain/record-types.js";
import type { SearchCategory, SearchSubcategory, LookupResult } from "../domain/search-types.js";
import type { RuleReferenceEdge } from "../domain/rule-types.js";
import type { ActorMetricMap } from "../domain/actor-metrics.js";
import type { ItemMetricMap } from "../domain/item-metrics.js";
import {
  categorySupportsSubcategory,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import { METADATA_FIELD_REGISTRY } from "../domain/metadata-field-registry.js";
import { normalizeText } from "../shared/utils.js";

export type CandidateRow = {
  recordKey: string;
  canonicalRecordKey?: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traitsJson: string;
  derivedTagsJson: string;
  publicationTitle: string | null;
  publicationRemaster: number;
  descriptionText: string | null;
  blurbText?: string | null;
  hasDescription: number;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  familiesJson: string | null;
  variantFamilyKey: string | null;
  variantBaseName: string | null;
  variantLabel: string | null;
  variantAxesJson: string | null;
  variantConfidence: number | null;
  variantSource: string;
  sourcePath: string;
  isUnique: number;
  isSearchCanonical?: number;
  size: string | null;
  itemCategory: string | null;
  baseItem: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  usage: string | null;
  hands: number | null;
  itemMetricsJson: string | null;
  damageTypesJson: string | null;
  weaponGroup: string | null;
  armorGroup: string | null;
  traditionsJson: string | null;
  spellKindsJson: string | null;
  rangeText: string | null;
  saveType: string | null;
  areaType: string | null;
  durationText: string | null;
  durationUnit: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: number | null;
  basicSave: number | null;
  languagesJson: string | null;
  speedTypesJson: string | null;
  sensesJson: string | null;
  immunitiesJson: string | null;
  resistancesJson: string | null;
  weaknessesJson: string | null;
  disableText: string | null;
  disableSkillsJson: string | null;
  isComplex: number | null;
  actorMetricsJson: string | null;
  rangeValue: number | null;
  rawJson?: string | null;
  searchText?: string | null;
  embeddingBlob?: Uint8Array | null;
};

export type ReferenceEdgeRow = {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  fromPackName: string;
  fromRecordType: string;
  fromDocumentType: string;
  fromSourceCategory: SourceCategory;
};

export type ValueCountRow = {
  value: string;
  count: number;
};

type ActorMetricJsonRow = {
  metricKey: string;
  valueType: "number" | "text" | "boolean";
  numberValue: number | null;
  textValue: string | null;
  boolValue: number | null;
};

function readCandidateRowValue(row: CandidateRow, key: string): CandidateRow[keyof CandidateRow] {
  return row[key as keyof CandidateRow];
}

function readStringRowValue(row: CandidateRow, key: string): string | null | undefined {
  const value = readCandidateRowValue(row, key);
  if (value == null || typeof value === "string") {
    return value;
  }

  throw new Error(`Expected row field "${key}" to be a string value.`);
}

function readNumberRowValue(row: CandidateRow, key: string): number | null | undefined {
  const value = readCandidateRowValue(row, key);
  if (value == null || typeof value === "number") {
    return value;
  }

  throw new Error(`Expected row field "${key}" to be a numeric value.`);
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

function isActorMetricJsonRow(value: unknown): value is ActorMetricJsonRow {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }

  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.metricKey === "string" &&
    (candidate.valueType === "number" || candidate.valueType === "text" || candidate.valueType === "boolean") &&
    (typeof candidate.numberValue === "number" ||
      candidate.numberValue === null ||
      candidate.numberValue === undefined) &&
    (typeof candidate.textValue === "string" || candidate.textValue === null || candidate.textValue === undefined) &&
    (typeof candidate.boolValue === "number" || candidate.boolValue === null || candidate.boolValue === undefined)
  );
}

function parseMetricRowsJson(
  value: string | null | undefined,
  fieldName: string,
  recordKey: string,
): ActorMetricJsonRow[] {
  if (!value) {
    return [];
  }

  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed) || !parsed.every(isActorMetricJsonRow)) {
    throw new Error(`Expected ${fieldName} for "${recordKey}" to be a JSON metric row array.`);
  }

  return parsed;
}

function parseMetricsJson(
  value: string | null | undefined,
  fieldName: string,
  recordKey: string,
): ActorMetricMap | ItemMetricMap {
  const rows = parseMetricRowsJson(value, fieldName, recordKey);
  const metrics: ActorMetricMap = {};
  for (const row of rows) {
    if (row.valueType === "number" && typeof row.numberValue === "number") {
      metrics[row.metricKey] = row.numberValue;
      continue;
    }

    if (row.valueType === "boolean") {
      metrics[row.metricKey] = Boolean(row.boolValue);
      continue;
    }

    if (row.valueType === "text" && typeof row.textValue === "string") {
      metrics[row.metricKey] = row.textValue;
    }
  }

  return metrics;
}

function parseCategory(category: string, recordKey: string): SearchCategory {
  const normalized = normalizeSearchCategory(category);
  if (!normalized) {
    throw new Error(`Invalid row category "${category}" for "${recordKey}".`);
  }

  return normalized;
}

function parseSubcategory(
  category: SearchCategory,
  subcategory: string | null,
  recordKey: string,
): SearchSubcategory | null {
  if (!subcategory) {
    return null;
  }

  const normalized = normalizeSearchSubcategory(subcategory);
  if (!normalized) {
    throw new Error(`Invalid row subcategory "${subcategory}" for "${recordKey}".`);
  }
  if (!categorySupportsSubcategory(category, normalized)) {
    throw new Error(`Invalid row subcategory "${subcategory}" for ${category} record "${recordKey}".`);
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
      throw new Error(`Invalid row source category "${sourceCategory}" for "${recordKey}".`);
  }
}

function parseVariantSource(variantSource: string | null | undefined, recordKey: string): VariantSource {
  const resolved = variantSource ?? "none";
  switch (resolved) {
    case "baseItem":
      return "baseItem";
    case "slug":
      return "slug";
    case "namePattern":
      return "namePattern";
    case "sourcePath":
      return "sourcePath";
    case "composite":
      return "composite";
    case "none":
      return "none";
    default:
      throw new Error(`Invalid row variant source "${resolved}" for "${recordKey}".`);
  }
}

function parseRawRecordJson(rawJson: string | null | undefined, recordKey: string): Record<string, unknown> {
  if (!rawJson) {
    return {};
  }

  const parsed: unknown = JSON.parse(rawJson);
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`Expected rawJson for "${recordKey}" to be a JSON object.`);
  }

  return parsed as Record<string, unknown>;
}

function extractMetadataValuesFromRow(row: CandidateRow): Partial<NormalizedRecord> {
  const metadata: Record<string, unknown> = {};

  for (const spec of METADATA_FIELD_REGISTRY) {
    switch (spec.rowValueSource.kind) {
      case "jsonArray":
        metadata[spec.recordProperty] = parseStringArrayJson(
          readStringRowValue(row, spec.rowValueSource.key),
          spec.rowValueSource.key,
          row.recordKey,
        );
        break;
      case "booleanNumber":
        metadata[spec.recordProperty] = Boolean(readNumberRowValue(row, spec.rowValueSource.key));
        break;
      case "number":
        metadata[spec.recordProperty] = readNumberRowValue(row, spec.rowValueSource.key) ?? null;
        break;
      case "string":
        metadata[spec.recordProperty] = readStringRowValue(row, spec.rowValueSource.key) ?? null;
        break;
    }
  }

  return metadata as Partial<NormalizedRecord>;
}

export function rowToRecord(row: CandidateRow, raw: Record<string, unknown> | null = null): NormalizedRecord {
  const category = parseCategory(row.category, row.recordKey);
  const resolvedRaw = raw ?? parseRawRecordJson(row.rawJson, row.recordKey);
  const metadata = extractMetadataValuesFromRow(row);
  return {
    recordKey: row.recordKey,
    id: row.id,
    name: row.name,
    normalizedName: row.normalizedName,
    type: row.type,
    category,
    subcategory: parseSubcategory(category, row.subcategory, row.recordKey),
    packName: row.packName,
    packLabel: row.packLabel,
    documentType: row.documentType,
    descriptionText: row.descriptionText,
    blurbText: row.blurbText ?? null,
    descriptionSnippet: row.descriptionSnippet,
    folderId: row.folderId,
    variantConfidence: row.variantConfidence,
    variantSource: parseVariantSource(row.variantSource, row.recordKey),
    sourcePath: row.sourcePath,
    ...metadata,
    itemMetrics: parseMetricsJson(row.itemMetricsJson, "itemMetricsJson", row.recordKey),
    actorMetrics: parseMetricsJson(row.actorMetricsJson, "actorMetricsJson", row.recordKey),
    aliases: [],
    legacyRecordLinks: [],
    raw: resolvedRaw,
  } as NormalizedRecord;
}

export function buildPlaceholders(values: readonly unknown[]): string {
  return values.map(() => "?").join(", ");
}

export function getLookupMatchType(query: string, record: NormalizedRecord | null): LookupResult["matchType"] {
  if (!record) {
    return "none";
  }

  const normalizedQuery = normalizeText(query);
  if (normalizeText(record.name) === normalizedQuery) {
    return normalizeText(query) === normalizeText(record.name) && query.trim() === record.name
      ? "exact"
      : "normalized_exact";
  }

  return "fuzzy";
}

export function edgeRowToReferenceEdge(
  row: ReferenceEdgeRow,
  direction: RuleReferenceEdge["direction"],
): RuleReferenceEdge {
  return {
    fromRecordKey: row.fromRecordKey,
    toRecordKey: row.toRecordKey,
    displayText: row.displayText,
    referenceText: row.referenceText,
    direction,
    relationshipType: direction === "outgoing" ? "references" : "referenced_by",
    sourcePackName: row.fromPackName,
    sourceRecordType: row.fromRecordType,
    sourceDocumentType: row.fromDocumentType,
    sourceCategory: parseSourceCategory(row.fromSourceCategory, row.fromRecordKey),
  };
}

export function sqliteRowCount(row: Record<string, unknown> | undefined, field = "total"): number {
  const value = row?.[field];
  return typeof value === "number" ? value : Number(value ?? 0);
}

export function backlinkTypeRank(recordType: string): number {
  if (recordType === "action") {
    return 0;
  }
  if (recordType === "feat") {
    return 1;
  }
  if (recordType === "classfeature") {
    return 2;
  }
  return 3;
}

export function sourceCategoryRank(sourceCategory: SourceCategory): number {
  if (sourceCategory === "core") {
    return 0;
  }
  if (sourceCategory === "rules") {
    return 1;
  }
  if (sourceCategory === "adventure") {
    return 2;
  }
  return 3;
}

export function extractQuestionRuleNames(question: string): string[] {
  const quoted = [...question.matchAll(/"([^"]+)"/g)]
    .map((match) => match[1]?.trim() ?? "")
    .filter((value) => value.length > 0);
  if (quoted.length > 0) {
    return quoted;
  }

  const cleaned = question
    .replace(/\?+$/g, "")
    .replace(/^(how|what|when|why|can)\s+(does|do|is|are)\s+/i, "")
    .replace(/^(how|what|when|why|can)\s+/i, "")
    .trim();

  const parts = cleaned
    .split(/\b(?:interplay with|interact with|works with|work with|with|vs\.?|versus|and)\b/i)
    .map((part) => part.trim().replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, ""))
    .filter((part) => part.length > 0);

  return [...new Set(parts)].slice(0, 5);
}
