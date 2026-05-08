import type { SearchProfile, SearchCategoryInput, SearchSubcategoryInput } from "./search-types.js";
import type { MetadataAtomicPredicate } from "./search-filter-metadata.js";
import type { MetricOperator, NullOperator, NumericMetricOperator } from "./search-filter-operators.js";
import type { RecordKey } from "./record-types.js";

export const SEARCH_REQUEST_VOCABULARY = {
  MODE: {
    BROWSE: "browse",
    SEARCH: "search",
    LOOKUP: "lookup",
  },
  FILTER_NODE_KIND: {
    PACK: "pack",
    SCOPE: "scope",
    LEVEL: "level",
    PRICE: "price",
    RARITY: "rarity",
    ACTION_COST: "actionCost",
    LINKS_TO: "linksTo",
    LINKED_FROM: "linkedFrom",
    METADATA_PREDICATE: "metadataPredicate",
    METRIC: "metric",
    METRIC_COMPARE: "metricCompare",
    ANY_OF: "anyOf",
    ALL_OF: "allOf",
    NOT: "not",
  },
  SCOPE_SUBCATEGORY_MATCH_KIND: {
    ANY: "any",
    EQ: "eq",
    IS_NULL: "isNull",
    IS_NOT_NULL: "isNotNull",
  },
  FILTER_MATCH_KIND: {
    EQ: "eq",
    IN: "in",
    NOT_IN: "notIn",
    IS_NULL: "isNull",
    IS_NOT_NULL: "isNotNull",
    GT: "gt",
    GTE: "gte",
    LT: "lt",
    LTE: "lte",
    BETWEEN: "between",
  },
  SORT_KIND: {
    ALPHABETICAL: "alphabetical",
    LEVEL_ASC: "levelAsc",
    LEVEL_DESC: "levelDesc",
    RANDOM: "random",
  },
  LOOKUP_SORT_POLICY: {
    TIERED: "tiered",
    GLOBAL: "global",
  },
} as const;

export const SEARCH_FILTER_NODE_KIND = SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND;
export const SEARCH_SCOPE_SUBCATEGORY_MATCH_KIND = SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND;
export const SEARCH_FILTER_MATCH_KIND = SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND;
export const SEARCH_SORT_KIND = SEARCH_REQUEST_VOCABULARY.SORT_KIND;
export const SEARCH_LOOKUP_SORT_POLICY = SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY;

type SearchFilterNodeKind = typeof SEARCH_FILTER_NODE_KIND;
type SearchScopeSubcategoryMatchKind = typeof SEARCH_SCOPE_SUBCATEGORY_MATCH_KIND;
type SearchFilterMatchKind = typeof SEARCH_FILTER_MATCH_KIND;
type SearchSortKind = typeof SEARCH_SORT_KIND;
type SearchLookupSortPolicy = typeof SEARCH_LOOKUP_SORT_POLICY;

export type SearchRequestMode = (typeof SEARCH_REQUEST_VOCABULARY.MODE)[keyof typeof SEARCH_REQUEST_VOCABULARY.MODE];

export type SearchScopeSubcategoryMatch =
  | { kind: SearchScopeSubcategoryMatchKind["ANY"] }
  | { kind: SearchScopeSubcategoryMatchKind["EQ"]; value: SearchSubcategoryInput }
  | { kind: SearchScopeSubcategoryMatchKind["IS_NULL"] }
  | { kind: SearchScopeSubcategoryMatchKind["IS_NOT_NULL"] };

export type SearchNumericMatch =
  | { kind: SearchFilterMatchKind["EQ"]; value: number }
  | { kind: SearchFilterMatchKind["GT"]; value: number }
  | { kind: SearchFilterMatchKind["GTE"]; value: number }
  | { kind: SearchFilterMatchKind["LT"]; value: number }
  | { kind: SearchFilterMatchKind["LTE"]; value: number }
  | { kind: SearchFilterMatchKind["BETWEEN"]; min: number; max: number };

export type SearchNullableNumericMatch = SearchNumericMatch | { kind: NullOperator };
export type SearchNullableStringMatch =
  | { kind: SearchFilterMatchKind["EQ"]; value: string }
  | { kind: SearchFilterMatchKind["IN"]; values: string[] }
  | { kind: SearchFilterMatchKind["NOT_IN"]; values: string[] }
  | { kind: NullOperator };

export type SearchFilterNode =
  | { kind: SearchFilterNodeKind["PACK"]; value: string }
  | {
      kind: SearchFilterNodeKind["SCOPE"];
      category: SearchCategoryInput;
      subcategory: SearchScopeSubcategoryMatch;
    }
  | {
      kind: SearchFilterNodeKind["LEVEL"];
      match: SearchNumericMatch;
    }
  | {
      kind: SearchFilterNodeKind["PRICE"];
      match: SearchNumericMatch;
    }
  | {
      kind: SearchFilterNodeKind["RARITY"];
      match: SearchNullableStringMatch;
    }
  | {
      kind: SearchFilterNodeKind["ACTION_COST"];
      match: SearchNullableNumericMatch;
    }
  | { kind: SearchFilterNodeKind["LINKS_TO"]; target: RecordKey }
  | { kind: SearchFilterNodeKind["LINKED_FROM"]; source: RecordKey }
  | { kind: SearchFilterNodeKind["METADATA_PREDICATE"]; predicate: MetadataAtomicPredicate }
  | { kind: SearchFilterNodeKind["METRIC"]; metric: string; op: MetricOperator; value: string | number | boolean }
  | { kind: SearchFilterNodeKind["METRIC_COMPARE"]; leftMetric: string; op: NumericMetricOperator; rightMetric: string }
  | { kind: SearchFilterNodeKind["ANY_OF"]; children: SearchFilterNode[] }
  | { kind: SearchFilterNodeKind["ALL_OF"]; children: SearchFilterNode[] }
  | { kind: SearchFilterNodeKind["NOT"]; child: SearchFilterNode };

export type BrowseSortSpec =
  | { kind: SearchSortKind["ALPHABETICAL"] }
  | { kind: SearchSortKind["LEVEL_ASC"] }
  | { kind: SearchSortKind["LEVEL_DESC"] }
  | { kind: SearchSortKind["RANDOM"]; seed?: number };

export type LookupSortSpec = {
  kind: SearchSortKind["ALPHABETICAL"] | SearchSortKind["LEVEL_ASC"] | SearchSortKind["LEVEL_DESC"] | SearchSortKind["RANDOM"];
  policy?: SearchLookupSortPolicy[keyof SearchLookupSortPolicy];
};

export type SearchRequestBase = {
  filter?: SearchFilterNode;
  offset?: number;
  limit?: number;
};

export type BrowseRequest = SearchRequestBase & {
  mode: typeof SEARCH_REQUEST_VOCABULARY.MODE.BROWSE;
  sort?: BrowseSortSpec;
};

export type SearchModeRequest = SearchRequestBase & {
  mode: typeof SEARCH_REQUEST_VOCABULARY.MODE.SEARCH;
  search: {
    query: string;
    exclude?: string;
    profile?: SearchProfile;
  };
  explain?: boolean;
};

export type LookupRequest = SearchRequestBase & {
  mode: typeof SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP;
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
    kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE,
    category,
    subcategory: subcategory
      ? { kind: SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.EQ, value: subcategory }
      : { kind: SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.ANY },
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
  return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children: nodes };
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
  return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF, children: nodes };
}

export function findSearchScopeFilter(
  filter: SearchFilterNode | undefined,
): Extract<SearchFilterNode, { kind: SearchFilterNodeKind["SCOPE"] }> | null {
  if (!filter) {
    return null;
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE) {
    return filter;
  }

  if (filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT) {
    return null;
  }

  if (
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF ||
    filter.kind === SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF
  ) {
    for (const child of filter.children) {
      const scope = findSearchScopeFilter(child);
      if (scope) {
        return scope;
      }
    }
  }

  return null;
}
