import { humanizeOntologySearchIdentifier } from "../../../domain/presentation-vocabulary.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type {
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalValueSelection,
} from "../../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQueryExcludeText,
  getSearchQueryLevelMatch,
  getSearchQueryPredicateFilter,
  getSearchQueryRootOperator,
  getSearchQuerySearchProfile,
  getSearchQueryText,
} from "../../search/query-state.js";
import {
  countSearchFilterPredicateNodes,
  formatSearchFilterNodePresentationAlias,
  type SearchFilterRenderOptions,
} from "../../search/query-core.js";
import type { SearchStructuredDraftAnchor } from "../../search/structured-draft-session.js";

const DEFAULT_SEARCH_PROFILE = "balanced";

export type SearchQuerySummaryAnchor =
  | { kind: "mode" }
  | { kind: "query" }
  | { kind: "exclude" }
  | { kind: "profile" }
  | { kind: "queryTreeRoot" }
  | SearchStructuredDraftAnchor;

export type SearchQuerySummaryEntryKind =
  | "mode"
  | "query"
  | "exclude"
  | "profile"
  | "filterTreeRoot"
  | "filterNode";

export type SearchQuerySummaryEntry = {
  kind: SearchQuerySummaryEntryKind;
  key: string;
  anchor: SearchQuerySummaryAnchor;
  label: string;
  value: string;
  description: string;
  visible: boolean;
  indent?: number;
  metadataPath?: number[];
};

export type SearchQuerySummary = {
  activeStructuredPartCount: number;
  metadataPredicateCount: number;
  entries: SearchQuerySummaryEntry[];
};

export function formatSearchCategory(category: SearchCategory | null): string {
  return category ? humanizeOntologySearchIdentifier(category) : "Any Category";
}

export function formatSearchSubcategory(subcategory: SearchSubcategory | null): string {
  return subcategory ? humanizeOntologySearchIdentifier(subcategory) : "Any Subcategory";
}

export function formatSearchScope(category: SearchCategory | null, subcategory: SearchSubcategory | null): string {
  if (!category) {
    return "Any Category";
  }
  return subcategory
    ? `${formatSearchCategory(category)} / ${formatSearchSubcategory(subcategory)}`
    : formatSearchCategory(category);
}

export function formatMode(mode: Pf2eTerminalSearchMode): string {
  return humanizeOntologySearchIdentifier(mode);
}

function formatSelectionValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeOntologySearchIdentifier(value);
}

export function formatFilterSelection<T extends number | string>(selection: Pf2eTerminalValueSelection<T>): string {
  const parts: string[] = [];
  if (selection.include.length > 0) {
    parts.push(`include ${selection.include.map((value) => formatSelectionValue(value)).join(", ")}`);
  }
  if (selection.exclude.length > 0) {
    parts.push(`exclude ${selection.exclude.map((value) => formatSelectionValue(value)).join(", ")}`);
  }
  return parts.length > 0 ? parts.join(" | ") : "(none)";
}

export function hasFilterSelection<T extends number | string>(selection: Pf2eTerminalValueSelection<T>): boolean {
  return selection.include.length > 0 || selection.exclude.length > 0;
}

export function formatLevelRange(request: Pf2eTerminalSearchQuery): string {
  const match = getSearchQueryLevelMatch(request);
  if (!match) {
    return "(any)";
  }

  switch (match.kind) {
    case "eq":
      return `L${match.value}`;
    case "gt":
      return `> L${match.value}`;
    case "gte":
      return `L${match.value}+`;
    case "lt":
      return `< L${match.value}`;
    case "lte":
      return `<= L${match.value}`;
    case "between":
      return `L${match.min}-L${match.max}`;
  }
}

function getVisibleRootChildren(
  filter: SearchFilterNode | undefined,
): Array<{ node: SearchFilterNode; path: number[] }> {
  if (!filter) {
    return [];
  }

  if (filter.kind === "allOf" || filter.kind === "anyOf") {
    return filter.children.map((child, index) => ({ node: child, path: [index] }));
  }

  return [{ node: filter, path: [] }];
}

function buildFilterSummaryEntries(
  query: Pf2eTerminalSearchQuery,
  renderOptions: SearchFilterRenderOptions = {},
): SearchQuerySummaryEntry[] {
  const category = getSearchQueryCategory(query);
  const children = getVisibleRootChildren(query.filter);
  if (children.length === 0) {
    return [];
  }

  const rootOperator = getSearchQueryRootOperator(query);
  const rootNode: SearchFilterNode =
    rootOperator === "anyOf" ? { kind: "anyOf", children: children.map((child) => child.node) } : { kind: "allOf", children: children.map((child) => child.node) };
  return [
    {
      kind: "filterTreeRoot",
      key: "queryTree:root",
      anchor: { kind: "queryTreeRoot" },
      label: "Query Logic",
      value: formatSearchFilterNodePresentationAlias(rootNode, { ...renderOptions, category, style: "compact" }),
      description: "Open the dedicated filter builder for the full boolean filter tree.",
      visible: true,
    },
    ...children.map((entry) => ({
      kind: "filterNode" as const,
      key: `queryNode:${entry.path.length > 0 ? entry.path.join(".") : "rootNode"}`,
      anchor: { kind: "queryNode" as const, path: entry.path },
      label: "Filter",
      value: formatSearchFilterNodePresentationAlias(entry.node, { ...renderOptions, category, style: "compact" }),
      description: "Open this top-level filter node in the dedicated builder.",
      visible: true,
    })),
  ];
}

export function buildSearchQuerySummary(
  query: Pf2eTerminalSearchQuery,
  renderOptions: SearchFilterRenderOptions = {},
): SearchQuerySummary {
  const queryText = getSearchQueryText(query);
  const excludeText = getSearchQueryExcludeText(query);
  const searchProfile = getSearchQuerySearchProfile(query);
  const metadataTree = getSearchQueryPredicateFilter(query);
  const metadataPredicateCount = countSearchFilterPredicateNodes(metadataTree ?? undefined);
  const rootChildren = getVisibleRootChildren(query.filter);

  return {
    activeStructuredPartCount: rootChildren.length,
    metadataPredicateCount,
    entries: [
      {
        kind: "mode",
        key: "mode",
        anchor: { kind: "mode" },
        label: "Mode",
        value: formatMode(query.mode),
        description:
          "Choose whether this query should browse deterministically, run ranked search, or perform exact lookup-style matching.",
        visible: true,
      },
      {
        kind: "query",
        key: "query",
        anchor: { kind: "query" },
        label: "Query",
        value: queryText || "(none)",
        description:
          query.mode === "lookup"
            ? "Edit the lookup text used to find near-exact record names."
            : "Edit the free-text portion of the query.",
        visible: query.mode !== "browse",
      },
      {
        kind: "exclude",
        key: "exclude",
        anchor: { kind: "exclude" },
        label: "Exclude",
        value: excludeText || "(none)",
        description: "Exclude ranked-search matches containing this text.",
        visible: query.mode === "search",
      },
      {
        kind: "profile",
        key: "profile",
        anchor: { kind: "profile" },
        label: "Profile",
        value: searchProfile ?? DEFAULT_SEARCH_PROFILE,
        description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
        visible: query.mode === "search",
      },
      ...buildFilterSummaryEntries(query, renderOptions),
    ],
  };
}

export function getVisibleSearchQuerySummaryEntries(summary: SearchQuerySummary): SearchQuerySummaryEntry[] {
  return summary.entries.filter((entry) => entry.visible);
}

export function isStructuredSearchQuerySummaryEntry(entry: SearchQuerySummaryEntry): boolean {
  return entry.kind === "filterTreeRoot" || entry.kind === "filterNode";
}
