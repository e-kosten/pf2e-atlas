import type { ActorMetricMap } from "../domain/actor-metrics.js";
import type { ItemMetricMap } from "../domain/item-metrics.js";
import {
  categorySupportsSubcategory,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import type { SourceCategory, VariantSource } from "../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";
import { normalizeText } from "../shared/utils.js";

export function parseSearchCategoryValue(value: string, context: string): SearchCategory {
  const normalized = normalizeSearchCategory(value);
  if (!normalized) {
    throw new Error(`Invalid search category "${value}" for ${context}.`);
  }

  return normalized;
}

export function parseSearchSubcategoryValue(value: string, context: string): SearchSubcategory {
  const normalized = normalizeSearchSubcategory(value);
  if (!normalized) {
    throw new Error(`Invalid search subcategory "${value}" for ${context}.`);
  }

  return normalized;
}

export function parseSearchSubcategoryForCategory(
  category: SearchCategory,
  value: string | null,
  context: string,
): SearchSubcategory | null {
  if (!value) {
    return null;
  }

  const normalized = parseSearchSubcategoryValue(value, context);
  if (!categorySupportsSubcategory(category, normalized)) {
    throw new Error(`Invalid search subcategory "${value}" for ${category} ${context}.`);
  }

  return normalized;
}

const SOURCE_CATEGORY_BY_TEXT = {
  core: "core",
  rules: "rules",
  adventure: "adventure",
  unknown: "unknown",
} as const satisfies Record<string, SourceCategory>;

const VARIANT_SOURCE_BY_TEXT = {
  baseitem: "baseItem",
  slug: "slug",
  namepattern: "namePattern",
  sourcepath: "sourcePath",
  composite: "composite",
  none: "none",
} as const satisfies Record<string, VariantSource>;

export function parseSourceCategoryValue(value: string, context: string): SourceCategory {
  const normalized = normalizeText(value);
  const parsed = SOURCE_CATEGORY_BY_TEXT[normalized as keyof typeof SOURCE_CATEGORY_BY_TEXT];
  if (!parsed) {
    throw new Error(`Invalid source category "${value}" for ${context}.`);
  }

  return parsed;
}

export function parseVariantSourceValue(value: string | null | undefined, context: string): VariantSource {
  const parsed = VARIANT_SOURCE_BY_TEXT[normalizeText(value ?? "none") as keyof typeof VARIANT_SOURCE_BY_TEXT];
  if (!parsed) {
    throw new Error(`Invalid variant source "${value}" for ${context}.`);
  }

  return parsed;
}

export function parseStringArrayJson(value: string | null | undefined, fieldName: string, context: string): string[] {
  if (!value) {
    return [];
  }

  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed)) {
    throw new Error(`Expected ${fieldName} for ${context} to be a JSON string array.`);
  }

  const result: string[] = [];
  for (const entry of parsed) {
    if (typeof entry !== "string") {
      throw new Error(`Expected ${fieldName} for ${context} to be a JSON string array.`);
    }
    if (entry.length > 0) {
      result.push(entry);
    }
  }

  return result;
}

type MetricJsonRow = {
  metricKey: string;
  valueType: "number" | "text" | "boolean";
  numberValue: number | null;
  textValue: string | null;
  boolValue: number | null;
};

function isMetricJsonRow(value: unknown): value is MetricJsonRow {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }

  const row = value as Record<string, unknown>;
  return (
    typeof row.metricKey === "string" &&
    (row.valueType === "number" || row.valueType === "text" || row.valueType === "boolean") &&
    (typeof row.numberValue === "number" || row.numberValue === null) &&
    (typeof row.textValue === "string" || row.textValue === null) &&
    (typeof row.boolValue === "number" || row.boolValue === null)
  );
}

function parseMetricMapJson(
  value: string | null | undefined,
  fieldName: string,
  context: string,
): ActorMetricMap {
  if (!value) {
    return {};
  }

  const parsed: unknown = JSON.parse(value);
  if (!Array.isArray(parsed) || !parsed.every(isMetricJsonRow)) {
    throw new Error(`Expected ${fieldName} for ${context} to be a JSON metric row array.`);
  }

  const metrics: ActorMetricMap = {};
  for (const row of parsed) {
    if (row.valueType === "number" && row.numberValue !== null) {
      metrics[row.metricKey] = row.numberValue;
      continue;
    }
    if (row.valueType === "text" && row.textValue !== null) {
      metrics[row.metricKey] = row.textValue;
      continue;
    }
    if (row.valueType === "boolean") {
      metrics[row.metricKey] = Boolean(row.boolValue);
    }
  }

  return metrics;
}

export function parseActorMetricMapJson(
  value: string | null | undefined,
  fieldName: string,
  context: string,
): ActorMetricMap {
  return parseMetricMapJson(value, fieldName, context);
}

export function parseItemMetricMapJson(
  value: string | null | undefined,
  fieldName: string,
  context: string,
): ItemMetricMap {
  return parseMetricMapJson(value, fieldName, context);
}

export function parseJsonObject(
  value: string | null | undefined,
  fieldName: string,
  context: string,
): Record<string, unknown> {
  if (!value) {
    return {};
  }

  const parsed: unknown = JSON.parse(value);
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error(`Expected ${fieldName} for ${context} to be a JSON object.`);
  }

  return parsed as Record<string, unknown>;
}

export function toSqliteNumber(value: number | bigint, context: string): number {
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "number") {
    return value;
  }

  throw new Error(`Expected numeric SQLite value for ${context}.`);
}
