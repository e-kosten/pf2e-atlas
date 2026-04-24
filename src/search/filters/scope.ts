import {
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { SearchScope } from "../../domain/search-types.js";
import { normalizeText, uniqueSorted } from "../../shared/utils.js";
import type {
  NormalizedSearchFilters,
  SearchExecutionNumericMatch,
  SearchExecutionFilterNode,
  SearchExecutionScopeSubcategoryMatch,
} from "../contracts.js";
import {
  recordMatchesMetadataAtomicPredicate,
  recordMatchesMetricComparePredicate,
  recordMatchesMetricPredicate,
} from "./metadata.js";

export function normalizeSearchScope(scope: SearchScope): {
  category: ReturnType<typeof normalizeSearchCategory> extends infer T ? Exclude<T, null> : never;
  subcategories?: ReturnType<typeof normalizeSearchSubcategory>[];
} {
  const category = normalizeSearchCategory(scope.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(scope.category)));
  }

  const subcategories = scope.subcategories?.map((subcategory) => {
    const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
    if (!canonicalSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(subcategory)));
    }
    return canonicalSubcategory;
  });

  const uniqueSubcategories = subcategories ? uniqueSorted(subcategories) : undefined;

  return {
    category,
    subcategories: uniqueSubcategories && uniqueSubcategories.length > 0 ? uniqueSubcategories : undefined,
  };
}

function recordMatchesScopeSubcategory(
  record: NormalizedRecord,
  match: SearchExecutionScopeSubcategoryMatch,
): boolean {
  if (match.kind === "any") {
    return true;
  }
  if (match.kind === "isNull") {
    return record.subcategory === null;
  }
  if (match.kind === "isNotNull") {
    return record.subcategory !== null;
  }
  return record.subcategory === match.value;
}

function recordMatchesNumericMatch(
  value: number | null,
  match: SearchExecutionNumericMatch,
): boolean {
  if (value === null) {
    return false;
  }
  if (match.kind === "eq") {
    return value === match.value;
  }
  if (match.kind === "gte") {
    return value >= match.value;
  }
  if (match.kind === "lte") {
    return value <= match.value;
  }
  return value >= match.min && value <= match.max;
}

function recordMatchesNullableNumericMatch(
  value: number | null,
  match: Extract<SearchExecutionFilterNode, { kind: "actionCost" }>["match"],
): boolean {
  if (match.kind === "isNull") {
    return value === null;
  }
  if (match.kind === "isNotNull") {
    return value !== null;
  }
  return recordMatchesNumericMatch(value, match as SearchExecutionNumericMatch);
}

function recordMatchesNullableStringMatch(
  value: string | null,
  match: Extract<SearchExecutionFilterNode, { kind: "rarity" }>["match"],
): boolean {
  const normalizedValue = value ? normalizeText(value) : null;
  if (match.kind === "isNull") {
    return normalizedValue === null || normalizedValue === "";
  }
  if (match.kind === "isNotNull") {
    return normalizedValue !== null && normalizedValue !== "";
  }
  return normalizedValue === normalizeText((match as Extract<typeof match, { kind: "eq" }>).value);
}

function recordMatchesFilterNode(record: NormalizedRecord, filter: SearchExecutionFilterNode): boolean {
  switch (filter.kind) {
    case "pack": {
      const normalizedPack = normalizeText(filter.value);
      return normalizeText(record.packName) === normalizedPack || normalizeText(record.packLabel) === normalizedPack;
    }
    case "scope":
      return record.category === filter.category && recordMatchesScopeSubcategory(record, filter.subcategory);
    case "level":
      return recordMatchesNumericMatch(record.level, filter.match);
    case "price":
      return recordMatchesNumericMatch(record.priceCp, filter.match);
    case "rarity":
      return recordMatchesNullableStringMatch(record.rarity, filter.match);
    case "actionCost":
      return recordMatchesNullableNumericMatch(record.actionCost, filter.match);
    case "linksTo":
      return true;
    case "metadataPredicate":
      return recordMatchesMetadataAtomicPredicate(record, filter.predicate);
    case "metric":
      return recordMatchesMetricPredicate(record, filter.metric, filter.op, filter.value);
    case "metricCompare":
      return recordMatchesMetricComparePredicate(record, filter.leftMetric, filter.op, filter.rightMetric);
    case "anyOf":
      return filter.children.some((child) => recordMatchesFilterNode(record, child));
    case "allOf":
      return filter.children.every((child) => recordMatchesFilterNode(record, child));
    case "not":
      return !recordMatchesFilterNode(record, filter.child);
  }
}

export function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (!filters.filter) {
    return true;
  }

  return recordMatchesFilterNode(record, filters.filter);
}
