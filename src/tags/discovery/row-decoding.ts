import {
  parseSearchCategoryValue,
  parseSearchSubcategoryForCategory,
} from "../../data/sql-row-decoding.js";
import { SearchCategory, SearchSubcategory } from "../../domain/derived-tag-types.js";

export function decodeDiscoveryVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

export function parseDiscoveryCategory(category: string, recordKey: string): SearchCategory {
  try {
    return parseSearchCategoryValue(category, recordKey);
  } catch {
    throw new Error(`Invalid discovery category "${category}"`);
  }
}

export function parseDiscoverySubcategory(
  category: SearchCategory,
  subcategory: string | null,
  recordKey: string,
): SearchSubcategory | null {
  try {
    return parseSearchSubcategoryForCategory(category, subcategory, recordKey);
  } catch {
    throw new Error(`Invalid discovery subcategory "${subcategory}" for ${category} record`);
  }
}

export function parseDiscoveryStringArrayJson(value: string, fieldName: string, recordKey: string): string[] {
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

const RESOLVED_EXEMPLAR_MATCH_TYPE_BY_TEXT = {
  recordKey: "recordKey",
  name: "name",
  alias: "alias",
} as const satisfies Record<string, "recordKey" | "name" | "alias">;

export function parseResolvedExemplarMatchType(
  matchedBy: string | null | undefined,
  recordKey: string,
): "recordKey" | "name" | "alias" {
  const normalizedKey = (matchedBy ?? "recordKey") as keyof typeof RESOLVED_EXEMPLAR_MATCH_TYPE_BY_TEXT;
  const parsed = RESOLVED_EXEMPLAR_MATCH_TYPE_BY_TEXT[normalizedKey];
  if (!parsed) {
    throw new Error(`Invalid exemplar match type "${matchedBy}" for "${recordKey}".`);
  }

  return parsed;
}
