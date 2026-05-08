import {
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import {
  SEARCH_FILTER_NODE_KIND,
  SEARCH_REQUEST_VOCABULARY,
  type SearchFilterNode,
  type SearchRequest,
} from "../domain/search-request-types.js";
import type {
  SearchExecutionFilterNode,
  SearchExecutionFilters,
  SearchExecutionNumericMatch,
  SearchExecutionNullableNumericMatch,
  SearchExecutionNullableStringMatch,
  SearchExecutionScopeSubcategoryMatch,
} from "./contracts.js";

function compileScopeSubcategoryMatch(
  subcategory: Extract<SearchFilterNode, { kind: typeof SEARCH_FILTER_NODE_KIND["SCOPE"] }>["subcategory"],
): SearchExecutionScopeSubcategoryMatch {
  if (
    subcategory.kind === SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.ANY ||
    subcategory.kind === SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.IS_NULL ||
    subcategory.kind === SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.IS_NOT_NULL
  ) {
    return subcategory;
  }

  const normalized = normalizeSearchSubcategory(subcategory.value);
  if (!normalized) {
    throw new Error(`Unknown subcategory "${subcategory.value}".`);
  }

  return {
    kind: SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.EQ,
    value: normalized,
  };
}

function compileScopeFilter(node: Extract<SearchFilterNode, { kind: typeof SEARCH_FILTER_NODE_KIND["SCOPE"] }>): SearchExecutionFilterNode {
  const normalizedCategory = normalizeSearchCategory(node.category);
  if (!normalizedCategory) {
    throw new Error(`Unknown category "${node.category}".`);
  }

  return {
    kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE,
    category: normalizedCategory,
    subcategory: compileScopeSubcategoryMatch(node.subcategory),
  };
}

function compileNumericMatch(
  match: Extract<SearchFilterNode, { kind: typeof SEARCH_FILTER_NODE_KIND["LEVEL"] | typeof SEARCH_FILTER_NODE_KIND["PRICE"] }>["match"],
): SearchExecutionNumericMatch {
  return match;
}

function compileNullableNumericMatch(
  match: Extract<SearchFilterNode, { kind: typeof SEARCH_FILTER_NODE_KIND["ACTION_COST"] }>["match"],
): SearchExecutionNullableNumericMatch {
  return match;
}

function compileNullableStringMatch(
  match: Extract<SearchFilterNode, { kind: typeof SEARCH_FILTER_NODE_KIND["RARITY"] }>["match"],
): SearchExecutionNullableStringMatch {
  return match;
}

function compileFilterNode(node: SearchFilterNode): SearchExecutionFilterNode {
  switch (node.kind) {
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK:
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LINKS_TO:
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LINKED_FROM:
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC:
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE:
      return node;
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.SCOPE:
      return compileScopeFilter(node);
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL:
      return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.LEVEL, match: compileNumericMatch(node.match) };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PRICE:
      return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PRICE, match: compileNumericMatch(node.match) };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY:
      return {
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.RARITY,
        match: compileNullableStringMatch(node.match),
      };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST:
      return {
        kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ACTION_COST,
        match: compileNullableNumericMatch(node.match),
      };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE:
      return node;
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF:
      return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ANY_OF, children: node.children.map(compileFilterNode) };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF:
      return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.ALL_OF, children: node.children.map(compileFilterNode) };
    case SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT:
      return { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.NOT, child: compileFilterNode(node.child) };
  }
}

function compileSort(request: SearchRequest): Pick<SearchExecutionFilters, "sort" | "sortSeed"> {
  if (request.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
    if (!request.sort) {
      return {};
    }

    if (request.sort.kind === SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM) {
      return {
        sort: SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM,
        sortSeed: request.sort.seed,
      };
    }

    return {
      sort: request.sort.kind,
    };
  }

  if (request.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP) {
    if (!request.sort) {
      return {};
    }

    return {
      sort: request.sort.kind,
    };
  }

  return {};
}

export function compileSearchRequest(request: SearchRequest): SearchExecutionFilters {
  const filter = request.filter ? compileFilterNode(request.filter) : undefined;
  const searchQuery = request.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE ? undefined : request.search.query.trim();

  return {
    ...compileSort(request),
    explain: request.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? request.explain : undefined,
    nameQuery: request.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP ? searchQuery : undefined,
    query: request.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? searchQuery : undefined,
    excludeQuery: request.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? request.search.exclude : undefined,
    searchProfile: request.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH ? request.search.profile : undefined,
    filter,
    offset: request.offset,
    limit: request.limit,
  };
}
