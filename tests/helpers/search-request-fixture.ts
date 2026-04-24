import {
  buildAllOfFilter,
  buildAnyOfFilter,
  buildScopeFilter,
  type BrowseRequest,
  type LookupRequest,
  type SearchFilterNode,
  type SearchNumericMatch,
  type SearchNullableNumericMatch,
  type SearchNullableStringMatch,
  type SearchModeRequest,
} from "../../src/domain/search-request-types.js";
import type { MetadataAtomicPredicate } from "../../src/domain/search-filter-metadata.js";
import type { MetricOperator, NumericMetricOperator } from "../../src/domain/search-filter-operators.js";
import type { SearchCategoryInput, SearchSubcategoryInput } from "../../src/domain/search-types.js";

export type BrowseRequestFixtureInput = Omit<BrowseRequest, "mode">;
export type SearchRequestFixtureInput = Omit<SearchModeRequest, "mode">;
export type LookupRequestFixtureInput = Omit<LookupRequest, "mode">;

export const allOfFilter = buildAllOfFilter;
export const anyOfFilter = buildAnyOfFilter;

export function packFilter(value: string): SearchFilterNode {
  return { kind: "pack", value };
}

export function scopeFilter(
  category: SearchCategoryInput,
  subcategory?: SearchSubcategoryInput | null,
): SearchFilterNode {
  return buildScopeFilter(category, subcategory);
}

export function linksToFilter(target: string): SearchFilterNode {
  return { kind: "linksTo", target };
}

export function levelFilter(match: SearchNumericMatch): SearchFilterNode {
  return { kind: "level", match };
}

export function priceFilter(match: SearchNumericMatch): SearchFilterNode {
  return { kind: "price", match };
}

export function rarityFilter(match: SearchNullableStringMatch): SearchFilterNode {
  return { kind: "rarity", match };
}

export function actionCostFilter(match: SearchNullableNumericMatch): SearchFilterNode {
  return { kind: "actionCost", match };
}

export function metadataPredicateFilter(predicate: MetadataAtomicPredicate): SearchFilterNode {
  return {
    kind: "metadataPredicate",
    predicate,
  };
}

export function metricFilter(metric: string, op: MetricOperator, value: string | number | boolean): SearchFilterNode {
  return { kind: "metric", metric, op, value };
}

export function metricCompareFilter(leftMetric: string, op: NumericMetricOperator, rightMetric: string): SearchFilterNode {
  return { kind: "metricCompare", leftMetric, op, rightMetric };
}

export function notFilter(child: SearchFilterNode): SearchFilterNode {
  return { kind: "not", child };
}

export function browseRequest(input: BrowseRequestFixtureInput = {}): BrowseRequest {
  return {
    mode: "browse",
    ...input,
  };
}

export function searchRequest(input: SearchRequestFixtureInput): SearchModeRequest {
  return {
    mode: "search",
    ...input,
  };
}

export function lookupRequest(input: LookupRequestFixtureInput): LookupRequest {
  return {
    mode: "lookup",
    ...input,
  };
}

export function browseQuery(label: string, input: BrowseRequestFixtureInput = {}) {
  return {
    label,
    request: browseRequest(input),
  };
}
