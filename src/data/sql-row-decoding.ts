import {
  categorySupportsSubcategory,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import type { SourceCategory } from "../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../domain/search-types.js";

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

export function parseSourceCategoryValue(value: string, context: string): SourceCategory {
  switch (value) {
    case "core":
      return "core";
    case "rules":
      return "rules";
    case "adventure":
      return "adventure";
    case "unknown":
      return "unknown";
    default:
      throw new Error(`Invalid source category "${value}" for ${context}.`);
  }
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

export function toSqliteNumber(value: number | bigint, context: string): number {
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "number") {
    return value;
  }

  throw new Error(`Expected numeric SQLite value for ${context}.`);
}
