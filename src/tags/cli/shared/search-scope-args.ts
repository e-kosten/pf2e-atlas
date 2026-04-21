import {
  parseSearchCategoryValue,
  parseSearchSubcategoryForCategory,
  parseSearchSubcategoryValue,
} from "../../../data/sql-row-decoding.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/derived-tag-types.js";

export function parseOptionalSearchCategoryArg(
  value: string | undefined,
  flagName: string,
): SearchCategory | undefined {
  if (!value) {
    return undefined;
  }

  return parseSearchCategoryValue(value, flagName);
}

export function parseRequiredSearchCategoryArg(value: string | undefined, flagName: string): SearchCategory {
  if (!value) {
    throw new Error(`Missing required ${flagName} <category> argument.`);
  }

  return parseSearchCategoryValue(value, flagName);
}

export function parseOptionalSearchSubcategoryArg(
  value: string | undefined,
  flagName: string,
): SearchSubcategory | undefined {
  if (!value) {
    return undefined;
  }

  return parseSearchSubcategoryValue(value, flagName);
}

export function parseOptionalScopedSearchSubcategoryArg(
  category: SearchCategory | undefined,
  value: string | undefined,
  flagName: string,
): SearchSubcategory | undefined {
  if (!value) {
    return undefined;
  }

  if (!category) {
    return parseSearchSubcategoryValue(value, flagName);
  }

  return parseSearchSubcategoryForCategory(category, value, flagName) ?? undefined;
}
