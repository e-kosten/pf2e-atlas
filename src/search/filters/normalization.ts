import {
  categorySupportsSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import {
  normalizeSearchPromotedNumberValue,
  normalizeSearchPromotedStringValue,
} from "../../domain/search-field-domains.js";
import type {
  NormalizedSearchFilters,
  SearchExecutionFilterNode,
  SearchExecutionFilters,
} from "../contracts.js";
import { hasStructuredFilterSignal, resolveSearchMode } from "../ranking.js";
import { normalizeMetadataAtomicPredicate, normalizeSearchMetricKey } from "./metadata.js";
import { SEARCH_EXECUTION_VOCABULARY } from "../contracts.js";

export type SearchFilterContext = "list" | "search";

function normalizeRecordKey(value: string, context: "linksTo" | "linkedFrom"): string {
  const normalized = value.trim();
  if (!normalized) {
    throw new Error(`${context} ${context === "linksTo" ? "target" : "source"} must not be empty.`);
  }
  return normalized;
}

function normalizeScopeFilterNode(node: Extract<SearchExecutionFilterNode, { kind: "scope" }>): SearchExecutionFilterNode {
  const category = normalizeSearchCategory(node.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(node.category)));
  }

  if (node.subcategory.kind !== SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.EQ) {
    return {
      ...node,
      category,
    };
  }

  const subcategory = normalizeSearchSubcategory(node.subcategory.value);
  if (!subcategory) {
    throw new Error(getSearchSubcategoryErrorMessage(String(node.subcategory.value)));
  }
  if (!categorySupportsSubcategory(category, subcategory)) {
    throw new Error(`Subcategory "${subcategory}" does not belong to category "${category}".`);
  }

  return {
    kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.SCOPE,
    category,
    subcategory: {
      kind: SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND.EQ,
      value: subcategory,
    },
  };
}

function normalizeNumericMatch<T extends { kind: string } & Record<string, unknown>>(match: T): T {
  if (match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.BETWEEN) {
    if (!Number.isFinite(match.min as number) || !Number.isFinite(match.max as number)) {
      throw new Error("between match requires finite min/max values.");
    }
    return match;
  }
  if (
    match.kind !== SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NULL &&
    match.kind !== SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NOT_NULL &&
    !Number.isFinite(match.value as number)
  ) {
    throw new Error(`${match.kind} match requires a finite numeric value.`);
  }
  return match;
}

function normalizeFilterNode(
  node: SearchExecutionFilterNode,
  resolvePackName: (packValue: string) => string | undefined,
): SearchExecutionFilterNode {
  switch (node.kind) {
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PACK:
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PACK,
        value: resolvePackName(node.value) ?? node.value,
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.SCOPE:
      return normalizeScopeFilterNode(node);
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LEVEL:
      return { kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LEVEL, match: normalizeNumericMatch(node.match) };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PRICE:
      return { kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.PRICE, match: normalizeNumericMatch(node.match) };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.RARITY:
      if (node.match.kind !== SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.EQ) {
        if (
          node.match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IN ||
          node.match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.NOT_IN
        ) {
          return {
            kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.RARITY,
            match: {
              kind: node.match.kind,
              values: node.match.values.map(
                (value) => normalizeSearchPromotedStringValue("rarity", value) ?? value,
              ),
            },
          };
        }
        return node;
      }
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.RARITY,
        match: {
          kind: SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.EQ,
          value: normalizeSearchPromotedStringValue("rarity", node.match.value) ?? node.match.value,
        },
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ACTION_COST:
      if (
        node.match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NULL ||
        node.match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.IS_NOT_NULL
      ) {
        return node;
      }
      if (node.match.kind === SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND.BETWEEN) {
        return { kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ACTION_COST, match: normalizeNumericMatch(node.match) };
      }
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ACTION_COST,
        match: {
          kind: node.match.kind,
          value: normalizeSearchPromotedNumberValue(
            "actionCost",
            (node.match as Extract<typeof node.match, { kind: "eq" | "gt" | "gte" | "lt" | "lte" }>).value,
          ) ?? (node.match as Extract<typeof node.match, { kind: "eq" | "gt" | "gte" | "lt" | "lte" }>).value,
        },
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKS_TO:
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKS_TO,
        target: normalizeRecordKey(node.target, "linksTo"),
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKED_FROM:
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.LINKED_FROM,
        source: normalizeRecordKey(node.source, "linkedFrom"),
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE:
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
        predicate: normalizeMetadataAtomicPredicate(node.predicate),
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC:
      return {
        ...node,
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC,
        metric: normalizeSearchMetricKey(node.metric),
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE:
      return {
        ...node,
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.METRIC_COMPARE,
        leftMetric: normalizeSearchMetricKey(node.leftMetric),
        rightMetric: normalizeSearchMetricKey(node.rightMetric),
      };
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ANY_OF:
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.ALL_OF: {
      const children = node.children.map((child) => normalizeFilterNode(child, resolvePackName));
      if (children.length === 0) {
        throw new Error(`${node.kind} must contain at least 2 child filters.`);
      }
      if (children.length === 1) {
        return children[0]!;
      }
      return {
        kind: node.kind,
        children,
      };
    }
    case SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.NOT:
      return {
        kind: SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND.NOT,
        child: normalizeFilterNode(node.child, resolvePackName),
      };
  }
}

function validateFilterNode(node: SearchExecutionFilterNode): void {
  if (node.kind === "anyOf" || node.kind === "allOf") {
    if (node.children.length < 2) {
      throw new Error(`${node.kind} must contain at least 2 child filters.`);
    }
    node.children.forEach(validateFilterNode);
    return;
  }

  if (node.kind === "not") {
    validateFilterNode(node.child);
  }
}

export function validateSearchFilters(filters: NormalizedSearchFilters, context: SearchFilterContext): void {
  const mode = resolveSearchMode(filters, context);

  if (context === "list" && filters.searchProfile) {
    throw new Error("searchProfile is only supported for pf2e_search.");
  }

  if (context === "list" && filters.query) {
    throw new Error("query is only supported for pf2e_search.");
  }

  if (context === "list" && filters.nameQuery) {
    throw new Error("nameQuery is only supported for pf2e_lookup/pf2e_search.");
  }

  if (context === "list" && filters.excludeQuery) {
    throw new Error("excludeQuery is only supported for pf2e_search.");
  }

  if (context === "list" && mode !== SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED) {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (mode === SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED && filters.query) {
    throw new Error("query requires a themed search profile such as balanced or concept.");
  }

  if (context === "search" && !filters.query?.trim() && !filters.nameQuery?.trim() && !hasStructuredFilterSignal(filters)) {
    throw new Error("pf2e_search requires search text and/or at least one structured filter.");
  }

  if (filters.filter) {
    validateFilterNode(filters.filter);
  }
}

export function normalizeSearchFilters(
  filters: SearchExecutionFilters,
  resolvePackName: (packValue: string) => string | undefined,
): NormalizedSearchFilters {
  return {
    ...filters,
    filter: filters.filter ? normalizeFilterNode(filters.filter, resolvePackName) : undefined,
  };
}
