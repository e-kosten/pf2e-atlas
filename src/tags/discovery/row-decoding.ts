import {
  categorySupportsSubcategory,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import { SearchCategory, SearchSubcategory } from "../../types.js";

export function decodeDiscoveryVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

export function parseDiscoveryCategory(category: string, recordKey: string): SearchCategory {
  const normalized = normalizeSearchCategory(category);
  if (!normalized) {
    throw new Error(`Invalid discovery category "${category}" for "${recordKey}".`);
  }

  return normalized;
}

export function parseDiscoverySubcategory(
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

export function parseResolvedExemplarMatchType(
  matchedBy: string | null | undefined,
  recordKey: string,
): "recordKey" | "name" | "alias" {
  switch (matchedBy ?? "recordKey") {
    case "recordKey":
      return "recordKey";
    case "name":
      return "name";
    case "alias":
      return "alias";
    default:
      throw new Error(`Invalid exemplar match type "${matchedBy}" for "${recordKey}".`);
  }
}
