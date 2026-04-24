import { humanizeOntologySearchIdentifier } from "../../../domain/presentation-vocabulary.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type {
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQueryExcludeText,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryRootOperator,
  getSearchQuerySearchProfile,
  getSearchQueryText,
} from "../../search/query-state.js";
import { formatFilterExplorerPolicySummary } from "../../framework/policy-presentation.js";
import {
  countMetadataPredicates,
  formatSearchFilterNodePresentationAlias,
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

function formatPolicyValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeOntologySearchIdentifier(value);
}

export function formatFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): string {
  return formatFilterExplorerPolicySummary(policy, {
    valueFormatter: (value) => formatPolicyValue(value),
  });
}

export function hasFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

export function formatLevelRange(request: Pf2eTerminalSearchQuery): string {
  const { levelMin, levelMax } = getSearchQueryLevelRange(request);
  if (levelMin === null && levelMax === null) {
    return "(any)";
  }
  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax ? `L${levelMin}` : `L${levelMin}-L${levelMax}`;
  }
  if (levelMin !== null) {
    return `L${levelMin}+`;
  }
  return `<= L${levelMax}`;
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

function buildFilterSummaryEntries(query: Pf2eTerminalSearchQuery): SearchQuerySummaryEntry[] {
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
      value: formatSearchFilterNodePresentationAlias(rootNode, { category, style: "compact" }),
      description: "Open the dedicated filter builder for the full boolean filter tree.",
      visible: true,
    },
    ...children.map((entry) => ({
      kind: "filterNode" as const,
      key: `queryNode:${entry.path.length > 0 ? entry.path.join(".") : "rootNode"}`,
      anchor: { kind: "queryNode" as const, path: entry.path },
      label: "Filter",
      value: formatSearchFilterNodePresentationAlias(entry.node, { category, style: "compact" }),
      description: "Open this top-level filter node in the dedicated builder.",
      visible: true,
      indent: 1,
    })),
  ];
}

export function buildSearchQuerySummary(query: Pf2eTerminalSearchQuery): SearchQuerySummary {
  const queryText = getSearchQueryText(query);
  const excludeText = getSearchQueryExcludeText(query);
  const searchProfile = getSearchQuerySearchProfile(query);
  const metadataTree = getSearchQueryMetadataTree(query);
  const metadataPredicateCount = countMetadataPredicates(metadataTree);
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
      ...buildFilterSummaryEntries(query),
    ],
  };
}

export function getVisibleSearchQuerySummaryEntries(summary: SearchQuerySummary): SearchQuerySummaryEntry[] {
  return summary.entries.filter((entry) => entry.visible);
}

export function isStructuredSearchQuerySummaryEntry(entry: SearchQuerySummaryEntry): boolean {
  return entry.kind !== "mode" && entry.kind !== "query" && entry.kind !== "exclude";
}
