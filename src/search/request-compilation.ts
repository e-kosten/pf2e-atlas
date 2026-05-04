import {
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import type { SearchFilterNode, SearchRequest } from "../domain/search-request-types.js";
import type { SearchCategory, SearchSort, SearchSubcategory } from "../domain/search-types.js";
import type {
  SearchExecutionFilterNode,
  SearchExecutionFilters,
  SearchExecutionNumericMatch,
  SearchExecutionNullableNumericMatch,
  SearchExecutionNullableStringMatch,
  SearchExecutionScopeSubcategoryMatch,
} from "./contracts.js";

function compileScopeSubcategoryMatch(
  subcategory: Extract<SearchFilterNode, { kind: "scope" }>["subcategory"],
): SearchExecutionScopeSubcategoryMatch {
  if (subcategory.kind === "any" || subcategory.kind === "isNull" || subcategory.kind === "isNotNull") {
    return subcategory;
  }

  const normalized = normalizeSearchSubcategory(subcategory.value);
  if (!normalized) {
    throw new Error(`Unknown subcategory "${subcategory.value}".`);
  }

  return {
    kind: "eq",
    value: normalized,
  };
}

function compileScopeFilter(node: Extract<SearchFilterNode, { kind: "scope" }>): SearchExecutionFilterNode {
  const normalizedCategory = normalizeSearchCategory(node.category);
  if (!normalizedCategory) {
    throw new Error(`Unknown category "${node.category}".`);
  }

  return {
    kind: "scope",
    category: normalizedCategory,
    subcategory: compileScopeSubcategoryMatch(node.subcategory),
  };
}

function compileNumericMatch(
  match: Extract<SearchFilterNode, { kind: "level" | "price" }>["match"],
): SearchExecutionNumericMatch {
  return match;
}

function compileNullableNumericMatch(
  match: Extract<SearchFilterNode, { kind: "actionCost" }>["match"],
): SearchExecutionNullableNumericMatch {
  return match;
}

function compileNullableStringMatch(
  match: Extract<SearchFilterNode, { kind: "rarity" }>["match"],
): SearchExecutionNullableStringMatch {
  return match;
}

function compileFilterNode(node: SearchFilterNode): SearchExecutionFilterNode {
  switch (node.kind) {
    case "pack":
    case "linksTo":
    case "linkedFrom":
    case "metric":
    case "metricCompare":
      return node;
    case "scope":
      return compileScopeFilter(node);
    case "level":
      return { kind: "level", match: compileNumericMatch(node.match) };
    case "price":
      return { kind: "price", match: compileNumericMatch(node.match) };
    case "rarity":
      return { kind: "rarity", match: compileNullableStringMatch(node.match) };
    case "actionCost":
      return { kind: "actionCost", match: compileNullableNumericMatch(node.match) };
    case "metadataPredicate":
      return node;
    case "anyOf":
      return { kind: "anyOf", children: node.children.map(compileFilterNode) };
    case "allOf":
      return { kind: "allOf", children: node.children.map(compileFilterNode) };
    case "not":
      return { kind: "not", child: compileFilterNode(node.child) };
  }
}

function compileSort(request: SearchRequest): Pick<SearchExecutionFilters, "sort" | "sortSeed"> {
  if (request.mode === "browse") {
    if (!request.sort) {
      return {};
    }

    if (request.sort.kind === "random") {
      return {
        sort: "random",
        sortSeed: request.sort.seed,
      };
    }

    return {
      sort: request.sort.kind,
    };
  }

  if (request.mode === "lookup") {
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
  const searchQuery = request.mode === "browse" ? undefined : request.search.query.trim();

  return {
    ...compileSort(request),
    explain: request.mode === "search" ? request.explain : undefined,
    nameQuery: request.mode === "lookup" ? searchQuery : undefined,
    query: request.mode === "search" ? searchQuery : undefined,
    excludeQuery: request.mode === "search" ? request.search.exclude : undefined,
    searchProfile: request.mode === "search" ? request.search.profile : undefined,
    filter,
    offset: request.offset,
    limit: request.limit,
  };
}
