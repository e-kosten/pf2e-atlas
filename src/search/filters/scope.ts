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
import { SEARCH_EXECUTION_VOCABULARY } from "../contracts.js";
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
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.ANY) {
    return true;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.IS_NULL) {
    return record.subcategory === null;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.IS_NOT_NULL) {
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
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.EQ) {
    return value === match.value;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.GT) {
    return value > match.value;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.GTE) {
    return value >= match.value;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.LT) {
    return value < match.value;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.LTE) {
    return value <= match.value;
  }
  return value >= match.min && value <= match.max;
}

function recordMatchesNullableNumericMatch(
  value: number | null,
  match: Extract<SearchExecutionFilterNode, { kind: "actionCost" }>["match"],
): boolean {
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NULL) {
    return value === null;
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NOT_NULL) {
    return value !== null;
  }
  return recordMatchesNumericMatch(value, match as SearchExecutionNumericMatch);
}

function recordMatchesNullableStringMatch(
  value: string | null,
  match: Extract<SearchExecutionFilterNode, { kind: "rarity" }>["match"],
): boolean {
  const normalizedValue = value ? normalizeText(value) : null;
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NULL) {
    return normalizedValue === null || normalizedValue === "";
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NOT_NULL) {
    return normalizedValue !== null && normalizedValue !== "";
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IN) {
    return match.values.map((value) => normalizeText(value)).includes(normalizedValue ?? "");
  }
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.NOT_IN) {
    return !match.values.map((value) => normalizeText(value)).includes(normalizedValue ?? "");
  }
  return normalizedValue === normalizeText((match as Extract<typeof match, { kind: typeof SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.EQ }>).value);
}

function recordMatchesFilterNode(record: NormalizedRecord, filter: SearchExecutionFilterNode): boolean {
  switch (filter.kind) {
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PACK: {
      const normalizedPack = normalizeText(filter.value);
      return normalizeText(record.packName) === normalizedPack || normalizeText(record.packLabel) === normalizedPack;
    }
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.SCOPE:
      return record.category === filter.category && recordMatchesScopeSubcategory(record, filter.subcategory);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LEVEL:
      return recordMatchesNumericMatch(record.level, filter.match);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PRICE:
      return recordMatchesNumericMatch(record.priceCp, filter.match);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.RARITY:
      return recordMatchesNullableStringMatch(record.rarity, filter.match);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ACTION_COST:
      return recordMatchesNullableNumericMatch(record.actionCost, filter.match);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKS_TO:
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKED_FROM:
      return true;
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE:
      return recordMatchesMetadataAtomicPredicate(record, filter.predicate);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC:
      return recordMatchesMetricPredicate(record, filter.metric, filter.op, filter.value);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE:
      return recordMatchesMetricComparePredicate(record, filter.leftMetric, filter.op, filter.rightMetric);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ANY_OF:
      return filter.children.some((child) => recordMatchesFilterNode(record, child));
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ALL_OF:
      return filter.children.every((child) => recordMatchesFilterNode(record, child));
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.NOT:
      return !recordMatchesFilterNode(record, filter.child);
  }
}

export function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (!filters.filter) {
    return true;
  }

  return recordMatchesFilterNode(record, filters.filter);
}
