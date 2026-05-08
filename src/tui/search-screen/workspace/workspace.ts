import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import { getSearchQueryText } from "../../search/query-state.js";
import {
  buildSearchQuerySummary,
  formatMode,
  getVisibleSearchQuerySummaryEntries,
  isStructuredSearchQuerySummaryEntry,
  type SearchQuerySummary,
  type SearchQuerySummaryAnchor,
  type SearchQuerySummaryEntry,
} from "./query-summary.js";
import type { DerivedTagTerminalLine } from "../../framework/types.js";
import { clampWindowStart } from "../../list-utils.js";
import { SEARCH_COUNT_STATUS, type SearchCountState, type SearchScreenState } from "../state.js";
import { formatCount, formatResultPosition, formatSort, getSessionBufferRange } from "../state.js";
import type { SearchFilterRenderOptions } from "../../search/query-core.js";
import { SEARCH_REQUEST_VOCABULARY } from "../../../domain/search-request-types.js";
import { SEARCH_VOCABULARY } from "../../../domain/search-types.js";
export { parseLevelRangeInput } from "../../filter-explorer/scalar-editor.js";
export {
  formatFilterSelection,
  formatLevelRange,
  formatMode,
  formatSearchCategory,
  formatSearchScope,
  formatSearchSubcategory,
  hasFilterSelection,
} from "./query-summary.js";

export type SearchWorkspaceAction =
  | "mode"
  | "query"
  | "exclude"
  | "profile"
  | "queryTreeRoot"
  | "addQueryPart"
  | "clearClauses"
  | "reset"
  | "clearResults"
  | "execute"
  | `queryNode:${string}`;

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
  const prefix = "  ".repeat(entry.indent ?? 0);
  const body = entry.label === "Filter" ? entry.value : `${entry.label} | ${entry.value}`;
  return `${prefix}${body}${entry.disabled ? " | unavailable" : ""}`;
}

function encodeQueryNodePath(path: number[]): string {
  return path.length > 0 ? path.join(".") : "root";
}

function encodeSummaryAnchorAction(anchor: SearchQuerySummaryAnchor): SearchWorkspaceAction {
  switch (anchor.kind) {
    case "mode":
      return "mode";
    case "query":
      return "query";
    case "exclude":
      return "exclude";
    case "profile":
      return "profile";
    case "queryTreeRoot":
      return "queryTreeRoot";
    case "addQueryPart":
      return "addQueryPart";
    case "queryNode":
      return `queryNode:${encodeQueryNodePath(anchor.path)}`;
  }
}

export function isQueryNodeAction(action: SearchWorkspaceAction): action is `queryNode:${string}` {
  return action.startsWith("queryNode:");
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

export function getExecuteAvailability(request: Pf2eTerminalSearchQuery): {
  disabled: boolean;
  reason: string | null;
} {
  const queryText = getSearchQueryText(request).trim();

  if (request.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP && !queryText) {
    return {
      disabled: true,
      reason: "Unavailable until you enter a lookup name.",
    };
  }

  if (request.mode === SEARCH_REQUEST_VOCABULARY.MODE.SEARCH && !queryText) {
    return {
      disabled: true,
      reason: "Enter search text to enable this search.",
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
    const noun = request.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP ? "candidate" : "match";
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

function buildWorkspaceEntryFromSummary(
  entry: SearchQuerySummaryEntry,
  options?: { indentOffset?: number },
): SearchWorkspaceEntry {
  return {
    action: encodeSummaryAnchorAction(entry.anchor),
    label: entry.label,
    value: entry.value,
    description: entry.description,
    indent: (entry.indent ?? 0) + (options?.indentOffset ?? 0),
  };
}

function buildSummaryLines(summary: SearchQuerySummary, title: string): DerivedTagTerminalLine[] {
  const lines: DerivedTagTerminalLine[] = [{ text: title, tone: "section" }];
  const visibleEntries = getVisibleSearchQuerySummaryEntries(summary);

  for (const entry of visibleEntries.filter((entry) => entry.kind !== "filterNode")) {
    lines.push({
      text: `${entry.label}: ${entry.value}`,
    });
  }

  lines.push({ text: `Top-level filters: ${summary.activeStructuredPartCount}` });
  if (summary.metadataPredicateCount > 0) {
    lines.push({ text: `Metadata predicates: ${summary.metadataPredicateCount}` });
  }
  for (const entry of visibleEntries.filter((entry) => entry.kind === "filterNode")) {
    lines.push({
      text: `${entry.label}: ${entry.value}`,
      indent: 2 + (entry.indent ?? 0) * 2,
    });
  }
  return lines;
}

export function buildWorkspaceEntries(
  state: SearchScreenState,
  countState: SearchCountState,
  renderOptions: SearchFilterRenderOptions = {},
): SearchWorkspaceEntry[] {
  const summary = buildSearchQuerySummary(state.query, renderOptions);
  const executeAvailability = getExecuteAvailability(state.query);
  const modeEntry = summary.entries.find((entry) => entry.kind === "mode");
  const queryEntry = summary.entries.find((entry) => entry.kind === "query");
  const excludeEntry = summary.entries.find((entry) => entry.kind === "exclude");
  const profileEntry = summary.entries.find((entry) => entry.kind === "profile");
  if (!modeEntry) {
    throw new Error("Search query summary must include mode entries.");
  }
  const entries: SearchWorkspaceEntry[] = [
    buildWorkspaceEntryFromSummary(modeEntry),
    ...(queryEntry?.visible ? [buildWorkspaceEntryFromSummary(queryEntry)] : []),
    ...(excludeEntry?.visible ? [buildWorkspaceEntryFromSummary(excludeEntry)] : []),
    ...(profileEntry?.visible ? [buildWorkspaceEntryFromSummary(profileEntry)] : []),
    {
      action: "addQueryPart",
      label: "Filters >",
      value: summary.activeStructuredPartCount > 0 ? `${summary.activeStructuredPartCount} active` : "None yet",
      description: "Open the dedicated filter builder for the canonical filter tree.",
    },
  ];

  const structuredEntries = getVisibleSearchQuerySummaryEntries(summary)
    .filter((entry) => isStructuredSearchQuerySummaryEntry(entry) && entry.kind !== "filterTreeRoot")
    .map((entry) => buildWorkspaceEntryFromSummary(entry));
  if (summary.metadataPredicateCount > 0) {
    structuredEntries.push({
      action: "clearClauses",
      label: "Clear Query Clauses",
      value: `${summary.metadataPredicateCount} active`,
      description: "Remove every explicit metadata clause while keeping the rest of the query editor state intact.",
      indent: 1,
    });
  }
  entries.push(
    ...structuredEntries,
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
  );

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

export function buildStructuredQuerySummaryLines(
  query: Pf2eTerminalSearchQuery,
  renderOptions: SearchFilterRenderOptions = {},
): DerivedTagTerminalLine[] {
  return buildSummaryLines(buildSearchQuerySummary(query, renderOptions), "Live Structured Query");
}

export function buildStructuredWorkspaceEntryFocusLines(entry: SearchWorkspaceEntry): DerivedTagTerminalLine[] {
  return [
    { text: "Focused Entry", tone: "section" },
    { text: entry.label, tone: "accent" },
    { text: `Current value: ${entry.value}` },
    {
      text: entry.disabled ? `Unavailable: ${entry.disabledReason ?? entry.description}` : entry.description,
      tone: entry.disabled ? "warning" : "default",
    },
  ];
}

export function buildQuerySummaryLines(
  state: SearchScreenState,
  countState: SearchCountState,
  renderOptions: SearchFilterRenderOptions = {},
): DerivedTagTerminalLine[] {
  const summary = buildSearchQuerySummary(state.query, renderOptions);
  const executeAvailability = getExecuteAvailability(state.query);
  const lines = buildSummaryLines(summary, "Query Summary");

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
  renderOptions: SearchFilterRenderOptions = {},
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
    ...buildQuerySummaryLines(state, countState, renderOptions),
  ];
}
