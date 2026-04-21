import type { MetadataFilterNode } from "../../../search/filters/types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import type {
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
} from "../../search/query-state.js";
import type { DerivedTagTerminalCommandOption, DerivedTagTerminalLine } from "../../framework/types.js";
import { formatFilterExplorerPolicySummary } from "../../framework/policy-presentation.js";
import { countMetadataPredicates, flattenMetadataTree } from "../../search/query-core.js";
import { humanizeIdentifier } from "../../search/service-options.js";
import { clampWindowStart } from "../../list-utils.js";
import { SEARCH_COUNT_STATUS, type SearchCountState, type SearchScreenState } from "../state.js";
import { formatCount, formatResultPosition, formatSort, getSessionBufferRange } from "../state.js";
export { parseLevelRangeInput } from "../../filter-explorer/scalar-editor.js";

export type SearchWorkspaceAction =
  | "mode"
  | "query"
  | "profile"
  | "addQueryPart"
  | "clearClauses"
  | "reset"
  | "clearResults"
  | "execute"
  | `queryPart:${SearchWorkspaceQueryPart}`
  | `queryNode:${string}`;

type SearchWorkspaceQueryPart = "category" | "subcategory" | "levelRange" | "rarity" | "actionCost";

export type SearchWorkspaceEntry = {
  action: SearchWorkspaceAction;
  label: string;
  value: string;
  description: string;
  indent?: number;
  disabled?: boolean;
  disabledReason?: string;
};

export function formatSearchWorkspaceEntryLine(entry: SearchWorkspaceEntry): string {
  return `${"  ".repeat(entry.indent ?? 0)}${entry.label} | ${entry.value}${entry.disabled ? " | unavailable" : ""}`;
}

export function formatSearchCategory(category: SearchCategory | null): string {
  return category ? humanizeIdentifier(category) : "Any Category";
}

export function formatSearchSubcategory(subcategory: SearchSubcategory | null): string {
  return subcategory ? humanizeIdentifier(subcategory) : "Any Subcategory";
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
  return humanizeIdentifier(mode);
}

function formatPolicyValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeIdentifier(value);
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
  return (request.filters.category ? 1 : 0) + request.filters.parts.length;
}

function encodeQueryNodePath(path: number[]): string {
  return path.length > 0 ? path.join(".") : "root";
}

function encodeQueryPartAction(part: SearchWorkspaceQueryPart): SearchWorkspaceAction {
  return `queryPart:${part}`;
}

export function isQueryNodeAction(action: SearchWorkspaceAction): action is `queryNode:${string}` {
  return action.startsWith("queryNode:");
}

export function isQueryPartAction(action: SearchWorkspaceAction): action is `queryPart:${SearchWorkspaceQueryPart}` {
  return action.startsWith("queryPart:");
}

export function decodeQueryPartAction(action: SearchWorkspaceAction): SearchWorkspaceQueryPart | null {
  if (!isQueryPartAction(action)) {
    return null;
  }
  const part = action.slice("queryPart:".length);
  return part === "category" ||
    part === "subcategory" ||
    part === "levelRange" ||
    part === "rarity" ||
    part === "actionCost"
    ? part
    : null;
}

function shouldShowActionCostQueryPart(request: Pf2eTerminalSearchQuery): boolean {
  return hasFilterPolicy(getSearchQueryActionCostPolicy(request));
}

export function decodeQueryNodeActionPath(action: SearchWorkspaceAction): number[] | null {
  if (!isQueryNodeAction(action)) {
    return null;
  }
  const encodedPath = action.slice("queryNode:".length);
  if (encodedPath === "root") {
    return [];
  }
  const path = encodedPath
    .split(".")
    .map((segment) => Number.parseInt(segment, 10))
    .filter((segment) => Number.isInteger(segment));
  return path.length > 0 ? path : null;
}

function buildMetadataWorkspaceEntries(node: MetadataFilterNode, category: SearchCategory | null): SearchWorkspaceEntry[] {
  return flattenMetadataTree(node, { rootLabel: "query", category }).map((entry) => ({
    action: `queryNode:${encodeQueryNodePath(entry.path)}`,
    label: entry.summary.label,
    value: entry.summary.value,
    description: entry.summary.description,
    indent: entry.depth,
  }));
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

function hasStructuredSignal(request: Pf2eTerminalSearchQuery): boolean {
  return Boolean(request.filters.category || request.filters.parts.length > 0);
}

export function getExecuteAvailability(request: Pf2eTerminalSearchQuery): {
  disabled: boolean;
  reason: string | null;
} {
  if (request.mode === "lookup" && !request.queryText.trim()) {
    return {
      disabled: true,
      reason: "Unavailable until you enter a lookup name.",
    };
  }

  if (request.mode === "search" && !request.queryText.trim() && !hasStructuredSignal(request)) {
    return {
      disabled: true,
      reason: "Unavailable until search mode has text or at least one structured filter.",
    };
  }

  return {
    disabled: false,
    reason: null,
  };
}

export function formatCountSummary(countState: SearchCountState, request: Pf2eTerminalSearchQuery): string {
  const availability = getExecuteAvailability(request);
  if (availability.disabled) {
    return availability.reason ?? "Unavailable";
  }
  if (countState.status === SEARCH_COUNT_STATUS.LOADING) {
    return "Counting matches...";
  }
  if (countState.status === SEARCH_COUNT_STATUS.ERROR) {
    return countState.message ?? "Live count unavailable.";
  }
  if (countState.status === SEARCH_COUNT_STATUS.READY && countState.result) {
    const noun = request.mode === "lookup" ? "candidate" : "match";
    return `${countState.result.total} ${noun}${countState.result.total === 1 ? "" : "es"}`;
  }
  return "Count pending";
}

export function formatQueryStatus(state: SearchScreenState): string {
  if (!state.session) {
    return "No applied query yet";
  }
  return JSON.stringify(state.query) === JSON.stringify(state.session.query)
    ? "Current editor matches applied query"
    : "Current editor has unapplied changes";
}

export function buildWorkspaceEntries(state: SearchScreenState, countState: SearchCountState): SearchWorkspaceEntry[] {
  const executeAvailability = getExecuteAvailability(state.query);
  const activeStructuredPartCount = countStructuredQueryParts(state.query);
  const subcategory = getSearchQuerySubcategory(state.query);
  const levelRange = getSearchQueryLevelRange(state.query);
  const rarityPolicy = getSearchQueryRarityPolicy(state.query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(state.query);
  const metadataTree = getSearchQueryMetadataTree(state.query);
  const entries: SearchWorkspaceEntry[] = [
    {
      action: "mode",
      label: "Mode",
      value: formatMode(state.query.mode),
      description:
        "Choose whether this query should browse deterministically, run ranked search, or perform exact lookup-style matching.",
    },
    {
      action: "query",
      label: "Query",
      value: state.query.queryText || "(none)",
      description:
        state.query.mode === "lookup"
          ? "Edit the lookup text used to find near-exact record names."
          : "Edit the free-text portion of the query. Browse mode can leave this empty.",
    },
    {
      action: "addQueryPart",
      label: "Add Query Part",
      value: activeStructuredPartCount > 0 ? `${activeStructuredPartCount} active` : "None yet",
      description: "Add another root query part or append explicit metadata clauses and logic groups.",
    },
    {
      action: "reset",
      label: "Reset Query",
      value: "Restore defaults",
      description: "Discard the current query editor state and return to the default scope and filters.",
    },
    {
      action: "clearResults",
      label: "Discard Applied Results",
      value: state.session ? `${state.session.loadedCount}/${state.session.total} loaded` : "No results",
      description: "Clear the applied result reader while leaving the current query editor state untouched.",
      disabled: !state.session,
      disabledReason: state.session ? undefined : "There is no applied result reader to discard.",
    },
    {
      action: "execute",
      label: "Execute Query",
      value: formatCountSummary(countState, state.query),
      description: "Apply the current query editor state and switch into the results reader.",
      disabled: executeAvailability.disabled,
      disabledReason: executeAvailability.reason ?? undefined,
    },
  ];

  const resetIndex = entries.findIndex((entry) => entry.action === "reset");
  const structuredEntries: SearchWorkspaceEntry[] = [];
  if (state.query.mode === "search") {
    structuredEntries.push({
      action: "profile",
      label: "Profile",
      value: state.query.searchProfile,
      description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
      indent: 1,
    });
  }
  structuredEntries.push({
    action: encodeQueryPartAction("category"),
    label: "Category",
    value: formatSearchCategory(state.query.filters.category),
    description: "Set the root category. Changing category clears every other active query part.",
    indent: 1,
  });
  if (subcategory) {
    structuredEntries.push({
      action: encodeQueryPartAction("subcategory"),
      label: "Subcategory",
      value: formatSearchSubcategory(subcategory),
      description: "Refine the current category with an optional subcategory boundary.",
      indent: 1,
    });
  }
  if (levelRange.levelMin !== null || levelRange.levelMax !== null) {
    structuredEntries.push({
      action: encodeQueryPartAction("levelRange"),
      label: "Level Range",
      value: formatLevelRange(state.query),
      description: "Adjust the current level band or clear it.",
      indent: 1,
    });
  }
  if (hasFilterPolicy(rarityPolicy)) {
    structuredEntries.push({
      action: encodeQueryPartAction("rarity"),
      label: "Rarity",
      value: formatFilterPolicy(rarityPolicy),
      description: "Adjust the rarity filter policy.",
      indent: 1,
    });
  }
  if (shouldShowActionCostQueryPart(state.query)) {
    structuredEntries.push({
      action: encodeQueryPartAction("actionCost"),
      label: "Action Cost",
      value: formatFilterPolicy(actionCostPolicy),
      description: "Adjust the action-cost filter policy for the current scope.",
      indent: 1,
    });
  }
  if (metadataTree) {
    structuredEntries.push(
      ...buildMetadataWorkspaceEntries(metadataTree, state.query.filters.category).map((entry) => ({
        ...entry,
        indent: (entry.indent ?? 0) + 1,
      })),
    );
    structuredEntries.push({
      action: "clearClauses",
      label: "Clear Query Clauses",
      value: `${countMetadataPredicates(metadataTree)} active`,
      description: "Remove every explicit metadata clause while keeping the rest of the query editor state intact.",
      indent: 1,
    });
  }

  if (structuredEntries.length > 0 && resetIndex >= 0) {
    entries.splice(resetIndex, 0, ...structuredEntries);
  }

  return entries;
}

export function buildWorkspaceLines(
  entries: SearchWorkspaceEntry[],
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, bodyHeight);
  const safeIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, entries.length - 1)));
  const windowStart = clampWindowStart(safeIndex, entries.length, visibleCount);

  return entries.slice(windowStart, windowStart + visibleCount).map((entry, offset) => ({
    text: formatSearchWorkspaceEntryLine(entry),
    tone: windowStart + offset === safeIndex ? "selected" : entry.disabled ? "dim" : "default",
    noWrap: true,
  }));
}

export function buildStructuredQuerySummaryLines(query: Pf2eTerminalSearchQuery): DerivedTagTerminalLine[] {
  const subcategory = getSearchQuerySubcategory(query);
  const levelRange = getSearchQueryLevelRange(query);
  const rarityPolicy = getSearchQueryRarityPolicy(query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(query);
  const metadataTree = getSearchQueryMetadataTree(query);
  const lines: DerivedTagTerminalLine[] = [
    { text: "Staged Structured Query", tone: "section" },
    { text: `Mode: ${formatMode(query.mode)}` },
    { text: `Query: ${query.queryText || "(none)"}` },
    { text: `Category: ${formatSearchCategory(query.filters.category)}` },
  ];

  if (query.mode === "search") {
    lines.splice(3, 0, { text: `Profile: ${query.searchProfile}` });
  }
  if (subcategory) {
    lines.push({ text: `Subcategory: ${formatSearchSubcategory(subcategory)}` });
  }
  if (levelRange.levelMin !== null || levelRange.levelMax !== null) {
    lines.push({ text: `Level Range: ${formatLevelRange(query)}` });
  }
  if (hasFilterPolicy(rarityPolicy)) {
    lines.push({ text: `Rarity: ${formatFilterPolicy(rarityPolicy)}` });
  }
  if (shouldShowActionCostQueryPart(query)) {
    lines.push({ text: `Action Cost: ${formatFilterPolicy(actionCostPolicy)}` });
  }
  lines.push({ text: `Query clauses: ${metadataTree ? countMetadataPredicates(metadataTree) : 0}` });

  if (metadataTree) {
    for (const entry of buildMetadataWorkspaceEntries(metadataTree, query.filters.category)) {
      lines.push({
        text: `${entry.label}: ${entry.value}`,
        indent: 2 + (entry.indent ?? 0) * 2,
      });
    }
  }

  return lines;
}

export function buildStructuredWorkspaceEntryFocusLines(entry: SearchWorkspaceEntry): DerivedTagTerminalLine[] {
  return [
    { text: "Focused Entry", tone: "section" },
    { text: entry.label, tone: "accent" },
    { text: `Current staged value: ${entry.value}` },
    {
      text: entry.disabled ? `Unavailable: ${entry.disabledReason ?? entry.description}` : entry.description,
      tone: entry.disabled ? "warning" : "default",
    },
  ];
}

export function buildQuerySummaryLines(
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const executeAvailability = getExecuteAvailability(state.query);
  const subcategory = getSearchQuerySubcategory(state.query);
  const levelRange = getSearchQueryLevelRange(state.query);
  const rarityPolicy = getSearchQueryRarityPolicy(state.query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(state.query);
  const metadataTree = getSearchQueryMetadataTree(state.query);
  const lines: DerivedTagTerminalLine[] = [
    { text: "Query Summary", tone: "section" },
    { text: `Mode: ${formatMode(state.query.mode)}` },
    { text: `Query: ${state.query.queryText || "(none)"}` },
    { text: `Category: ${formatSearchCategory(state.query.filters.category)}` },
    ...(subcategory ? [{ text: `Subcategory: ${formatSearchSubcategory(subcategory)}` }] : []),
    ...(levelRange.levelMin !== null || levelRange.levelMax !== null
      ? [{ text: `Level Range: ${formatLevelRange(state.query)}` }]
      : []),
    ...(hasFilterPolicy(rarityPolicy) ? [{ text: `Rarity: ${formatFilterPolicy(rarityPolicy)}` }] : []),
    ...(shouldShowActionCostQueryPart(state.query)
      ? [{ text: `Action Cost: ${formatFilterPolicy(actionCostPolicy)}` as const }]
      : []),
    { text: `Query clauses: ${metadataTree ? countMetadataPredicates(metadataTree) : 0}` },
  ];

  if (state.query.mode === "search") {
    lines.splice(3, 0, { text: `Profile: ${state.query.searchProfile}` });
  }

  if (metadataTree) {
    for (const entry of buildMetadataWorkspaceEntries(metadataTree, state.query.filters.category)) {
      lines.push({
        text: `${entry.label}: ${entry.value}`,
        indent: 2 + (entry.indent ?? 0) * 2,
      });
    }
  }

  if (state.query.sourceLabel) {
    lines.push({ text: `Seeded from: ${state.query.sourceLabel}` });
  }

  lines.push({ text: "" });
  lines.push({ text: "Live Count", tone: "section" });
  if (executeAvailability.disabled) {
    lines.push({ text: executeAvailability.reason ?? "Unavailable for the current query.", tone: "warning" });
  } else if (countState.status === SEARCH_COUNT_STATUS.LOADING) {
    lines.push({ text: "Counting lexical matches for the current query...", tone: "accent" });
  } else if (countState.status === SEARCH_COUNT_STATUS.ERROR) {
    lines.push({ text: countState.message ?? "Live count unavailable.", tone: "warning" });
  } else if (countState.status === SEARCH_COUNT_STATUS.READY && countState.result) {
    lines.push({
      text: `${countState.result.total} matching record${countState.result.total === 1 ? "" : "s"} before result ordering.`,
      tone: countState.result.total === 0 ? "warning" : "default",
    });
  } else {
    lines.push({ text: "Count pending.", tone: "dim" });
  }

  lines.push({ text: "Tab executes the current query and opens the results reader.", tone: "accent" });
  lines.push({ text: "" });
  lines.push({ text: "Applied Session", tone: "section" });
  lines.push({ text: formatQueryStatus(state) });
  if (!state.session) {
    lines.push({ text: "No applied query yet.", tone: "dim" });
  } else {
    lines.push({ text: `Sort: ${formatSort(state.session.sort)}` });
    lines.push({ text: `Position: ${formatResultPosition(state.resultSelectedIndex, state.session.total)}` });
    lines.push({ text: `Buffered: ${formatCount(state.session.loadedCount)}` });
    lines.push({ text: `Window: ${getSessionBufferRange(state.session)}` });
    lines.push({ text: `Applied mode: ${formatMode(state.session.query.mode)} | ${state.session.resultMode}` });
  }

  return lines;
}

export function buildWorkspaceEntryDetailLines(
  entry: SearchWorkspaceEntry,
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const descriptionTone = entry.disabled ? "warning" : "default";
  return [
    { text: entry.label, tone: "section" },
    { text: `Current value: ${entry.value}` },
    {
      text: entry.disabled ? `Unavailable: ${entry.disabledReason ?? entry.description}` : entry.description,
      tone: descriptionTone,
    },
    ...(entry.disabled
      ? []
      : entry.action === "execute"
        ? [
            {
              text: "Press Enter, Right, Space, or Tab to execute the current query and switch to results.",
              tone: "accent" as const,
            },
          ]
        : [{ text: "Press Enter, Right, or Space to edit or act on this item.", tone: "accent" as const }]),
    { text: "" },
    ...buildQuerySummaryLines(state, countState),
  ];
}

export function buildEditorCommandPaletteEntries(
  workspaceEntries: SearchWorkspaceEntry[],
): DerivedTagTerminalCommandOption<SearchWorkspaceAction>[] {
  return workspaceEntries
    .filter((entry) => !entry.disabled)
    .map((entry) => ({
      value: entry.action,
      label: entry.label,
      description: entry.description,
      keywords: [entry.value],
      disabled: entry.disabled,
      disabledReason: entry.disabledReason,
    }));
}
