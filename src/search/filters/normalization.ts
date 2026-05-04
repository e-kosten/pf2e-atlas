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

  if (node.subcategory.kind !== "eq") {
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
    kind: "scope",
    category,
    subcategory: {
      kind: "eq",
      value: subcategory,
    },
  };
}

function normalizeNumericMatch<T extends { kind: string } & Record<string, unknown>>(match: T): T {
  if (match.kind === "between") {
    if (!Number.isFinite(match.min as number) || !Number.isFinite(match.max as number)) {
      throw new Error("between match requires finite min/max values.");
    }
    return match;
  }
  if (match.kind !== "isNull" && match.kind !== "isNotNull" && !Number.isFinite(match.value as number)) {
    throw new Error(`${match.kind} match requires a finite numeric value.`);
  }
  return match;
}

function normalizeFilterNode(
  node: SearchExecutionFilterNode,
  resolvePackName: (packValue: string) => string | undefined,
): SearchExecutionFilterNode {
  switch (node.kind) {
    case "pack":
      return {
        kind: "pack",
        value: resolvePackName(node.value) ?? node.value,
      };
    case "scope":
      return normalizeScopeFilterNode(node);
    case "level":
      return { kind: "level", match: normalizeNumericMatch(node.match) };
    case "price":
      return { kind: "price", match: normalizeNumericMatch(node.match) };
    case "rarity":
      if (node.match.kind !== "eq") {
        return node;
      }
      return {
        kind: "rarity",
        match: {
          kind: "eq",
          value: normalizeSearchPromotedStringValue("rarity", node.match.value) ?? node.match.value,
        },
      };
    case "actionCost":
      if (node.match.kind === "isNull" || node.match.kind === "isNotNull") {
        return node;
      }
      if (node.match.kind === "between") {
        return { kind: "actionCost", match: normalizeNumericMatch(node.match) };
      }
      return {
        kind: "actionCost",
        match: {
          kind: node.match.kind,
          value: normalizeSearchPromotedNumberValue(
            "actionCost",
            (node.match as Extract<typeof node.match, { kind: "eq" | "gt" | "gte" | "lt" | "lte" }>).value,
          ) ?? (node.match as Extract<typeof node.match, { kind: "eq" | "gt" | "gte" | "lt" | "lte" }>).value,
        },
      };
    case "linksTo":
      return {
        kind: "linksTo",
        target: normalizeRecordKey(node.target, "linksTo"),
      };
    case "linkedFrom":
      return {
        kind: "linkedFrom",
        source: normalizeRecordKey(node.source, "linkedFrom"),
      };
    case "metadataPredicate":
      return {
        kind: "metadataPredicate",
        predicate: normalizeMetadataAtomicPredicate(node.predicate),
      };
    case "metric":
      return {
        ...node,
        metric: normalizeSearchMetricKey(node.metric),
      };
    case "metricCompare":
      return {
        ...node,
        leftMetric: normalizeSearchMetricKey(node.leftMetric),
        rightMetric: normalizeSearchMetricKey(node.rightMetric),
      };
    case "anyOf":
    case "allOf": {
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
    case "not":
      return {
        kind: "not",
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

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (mode === "structured" && filters.query) {
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
