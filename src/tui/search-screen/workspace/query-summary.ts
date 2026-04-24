import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import { humanizeOntologySearchIdentifier } from "../../../domain/presentation-vocabulary.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type {
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
  getSearchQuerySearchProfile,
  getSearchQuerySubcategory,
  getSearchQueryText,
} from "../../search/query-state.js";
import { formatFilterExplorerPolicySummary } from "../../framework/policy-presentation.js";
import { countMetadataPredicates, flattenMetadataTree } from "../../search/query-core.js";
import { extractLegacyQueryPartsFromCanonicalFilter } from "../../search/query-parts.js";
import type { SearchStructuredDraftAnchor } from "../../search/structured-draft-session.js";

const DEFAULT_SEARCH_PROFILE = "balanced";

export type SearchQuerySummaryAnchor =
  | { kind: "mode" }
  | { kind: "query" }
  | { kind: "profile" }
  | SearchStructuredDraftAnchor;

export type SearchQuerySummaryEntryKind =
  | "mode"
  | "query"
  | "profile"
  | "category"
  | "subcategory"
  | "levelRange"
  | "rarity"
  | "actionCost"
  | "metadata";

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

function countStructuredQueryParts(request: Pf2eTerminalSearchQuery): number {
  const legacyState = extractLegacyQueryPartsFromCanonicalFilter(request.filter);
  return (legacyState.category ? 1 : 0) + legacyState.parts.length;
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

function buildMetadataSummaryEntries(
  node: MetadataFilterNode,
  category: SearchCategory | null,
): SearchQuerySummaryEntry[] {
  return flattenMetadataTree(node, { rootLabel: "query", category }).map((entry) => ({
    kind: "metadata",
    key: `queryNode:${entry.path.length > 0 ? entry.path.join(".") : "root"}`,
    anchor: { kind: "queryNode", path: entry.path },
    label: entry.summary.label,
    value: entry.summary.value,
    description: entry.summary.description,
    visible: true,
    indent: entry.depth,
    metadataPath: entry.path,
  }));
}

export function buildSearchQuerySummary(query: Pf2eTerminalSearchQuery): SearchQuerySummary {
  const category = getSearchQueryCategory(query);
  const queryText = getSearchQueryText(query);
  const searchProfile = getSearchQuerySearchProfile(query);
  const subcategory = getSearchQuerySubcategory(query);
  const levelRange = getSearchQueryLevelRange(query);
  const rarityPolicy = getSearchQueryRarityPolicy(query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(query);
  const metadataTree = getSearchQueryMetadataTree(query);
  const metadataPredicateCount = countMetadataPredicates(metadataTree);

  return {
    activeStructuredPartCount: countStructuredQueryParts(query),
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
        kind: "profile",
        key: "profile",
        anchor: { kind: "profile" },
        label: "Profile",
        value: searchProfile ?? DEFAULT_SEARCH_PROFILE,
        description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
        visible: query.mode === "search",
      },
      {
        kind: "category",
        key: "queryPart:category",
        anchor: { kind: "queryPart", part: "category" },
        label: "Category",
        value: formatSearchCategory(category),
        description: "Set the root category. Changing category clears every other active query part.",
        visible: true,
      },
      {
        kind: "subcategory",
        key: "queryPart:subcategory",
        anchor: { kind: "queryPart", part: "subcategory" },
        label: "Subcategory",
        value: formatSearchSubcategory(subcategory),
        description: "Refine the current category with an optional subcategory boundary.",
        visible: Boolean(subcategory),
      },
      {
        kind: "levelRange",
        key: "queryPart:levelRange",
        anchor: { kind: "queryPart", part: "levelRange" },
        label: "Level Range",
        value: formatLevelRange(query),
        description: "Adjust the current level band or clear it.",
        visible: levelRange.levelMin !== null || levelRange.levelMax !== null,
      },
      {
        kind: "rarity",
        key: "queryPart:rarity",
        anchor: { kind: "queryPart", part: "rarity" },
        label: "Rarity",
        value: formatFilterPolicy(rarityPolicy),
        description: "Adjust the rarity filter policy.",
        visible: hasFilterPolicy(rarityPolicy),
      },
      {
        kind: "actionCost",
        key: "queryPart:actionCost",
        anchor: { kind: "queryPart", part: "actionCost" },
        label: "Action Cost",
        value: formatFilterPolicy(actionCostPolicy),
        description: "Adjust the action-cost filter policy for the current scope.",
        visible: hasFilterPolicy(actionCostPolicy),
      },
      ...(metadataTree ? buildMetadataSummaryEntries(metadataTree, category) : []),
    ],
  };
}

export function getVisibleSearchQuerySummaryEntries(summary: SearchQuerySummary): SearchQuerySummaryEntry[] {
  return summary.entries.filter((entry) => entry.visible);
}

export function isStructuredSearchQuerySummaryEntry(entry: SearchQuerySummaryEntry): boolean {
  return entry.kind !== "mode" && entry.kind !== "query";
}
