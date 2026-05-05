import type { SearchProfile, SearchCategoryInput, SearchSubcategoryInput } from "./search-types.js";
import type { MetadataAtomicPredicate } from "./search-filter-metadata.js";
import type { MetricOperator, NullOperator, NumericMetricOperator } from "./search-filter-operators.js";
import type { RecordKey } from "./record-types.js";

export type SearchRequestMode = "browse" | "search" | "lookup";

export type SearchScopeSubcategoryMatch =
  | { kind: "any" }
  | { kind: "eq"; value: SearchSubcategoryInput }
  | { kind: "isNull" }
  | { kind: "isNotNull" };

export type SearchNumericMatch =
  | { kind: "eq"; value: number }
  | { kind: "gt"; value: number }
  | { kind: "gte"; value: number }
  | { kind: "lt"; value: number }
  | { kind: "lte"; value: number }
  | { kind: "between"; min: number; max: number };

export type SearchNullableNumericMatch = SearchNumericMatch | { kind: NullOperator };
export type SearchNullableStringMatch =
  | { kind: "eq"; value: string }
  | { kind: "in"; values: string[] }
  | { kind: "notIn"; values: string[] }
  | { kind: NullOperator };

export type SearchFilterNode =
  | { kind: "pack"; value: string }
  | {
      kind: "scope";
      category: SearchCategoryInput;
      subcategory: SearchScopeSubcategoryMatch;
    }
  | {
      kind: "level";
      match: SearchNumericMatch;
    }
  | {
      kind: "price";
      match: SearchNumericMatch;
    }
  | {
      kind: "rarity";
      match: SearchNullableStringMatch;
    }
  | {
      kind: "actionCost";
      match: SearchNullableNumericMatch;
    }
  | { kind: "linksTo"; target: RecordKey }
  | { kind: "linkedFrom"; source: RecordKey }
  | { kind: "metadataPredicate"; predicate: MetadataAtomicPredicate }
  | { kind: "metric"; metric: string; op: MetricOperator; value: string | number | boolean }
  | { kind: "metricCompare"; leftMetric: string; op: NumericMetricOperator; rightMetric: string }
  | { kind: "anyOf"; children: SearchFilterNode[] }
  | { kind: "allOf"; children: SearchFilterNode[] }
  | { kind: "not"; child: SearchFilterNode };

export type BrowseSortSpec =
  | { kind: "alphabetical" | "levelAsc" | "levelDesc" }
  | { kind: "random"; seed?: number };

export type LookupSortSpec = {
  kind: "alphabetical" | "levelAsc" | "levelDesc";
  policy?: "tiered" | "global";
};

export type SearchRequestBase = {
  filter?: SearchFilterNode;
  offset?: number;
  limit?: number;
};

export type BrowseRequest = SearchRequestBase & {
  mode: "browse";
  sort?: BrowseSortSpec;
};

export type SearchModeRequest = SearchRequestBase & {
  mode: "search";
  search: {
    query: string;
    exclude?: string;
    profile?: SearchProfile;
  };
  explain?: boolean;
};

export type LookupRequest = SearchRequestBase & {
  mode: "lookup";
  search: {
    query: string;
  };
  sort?: LookupSortSpec;
};

export type SearchRequest = BrowseRequest | SearchModeRequest | LookupRequest;

export function buildScopeFilter(
  category: SearchCategoryInput,
  subcategory?: SearchSubcategoryInput | null,
): SearchFilterNode {
  return {
    kind: "scope",
    category,
    subcategory: subcategory ? { kind: "eq", value: subcategory } : { kind: "any" },
  };
}

export function buildAllOfFilter(
  children: Array<SearchFilterNode | null | undefined>,
): SearchFilterNode | undefined {
  const nodes = children.filter((child): child is SearchFilterNode => Boolean(child));
  if (nodes.length === 0) {
    return undefined;
  }
  if (nodes.length === 1) {
    return nodes[0];
  }
  return { kind: "allOf", children: nodes };
}

export function buildAnyOfFilter(
  children: Array<SearchFilterNode | null | undefined>,
): SearchFilterNode | undefined {
  const nodes = children.filter((child): child is SearchFilterNode => Boolean(child));
  if (nodes.length === 0) {
    return undefined;
  }
  if (nodes.length === 1) {
    return nodes[0];
  }
  return { kind: "anyOf", children: nodes };
}

export function findSearchScopeFilter(filter: SearchFilterNode | undefined): Extract<SearchFilterNode, { kind: "scope" }> | null {
  if (!filter) {
    return null;
  }

  if (filter.kind === "scope") {
    return filter;
  }

  if (filter.kind === "not") {
    return null;
  }

  if (filter.kind === "anyOf" || filter.kind === "allOf") {
    for (const child of filter.children) {
      const scope = findSearchScopeFilter(child);
      if (scope) {
        return scope;
      }
    }
  }

  return null;
}
