import React from "react";

import type {
  OntologyNodeQuery,
  SearchCategory,
  SearchCountResult,
  SearchProfile,
  SearchSubcategory,
} from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFacetSelection,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchMode,
  Pf2eTerminalSearchRequest,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
} from "./search-service.js";
import {
  TerminalTwoPaneScreen,
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  resolveDerivedTagTerminalListNavigationAction,
  sliceRenderedTerminalLines,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import { buildOntologyExplorerEntityDetailLines } from "./ontology-explorer/entity-page.js";
import { mapNormalizedRecordToOntologyExplorerEntityRecord } from "./ontology-explorer/entity-record.js";
import { clampWindowStart } from "./list-utils.js";

type SearchWorkspaceAction =
  | "execute"
  | "mode"
  | "query"
  | "profile"
  | "category"
  | "subcategory"
  | "levels"
  | "rarity"
  | "addFacet"
  | "removeFacet"
  | "reset"
  | "clearResults";

type SearchWorkspaceEntry = {
  action: SearchWorkspaceAction;
  label: string;
  value: string;
  description: string;
  disabled?: boolean;
};

type SearchScreenLayout = "draft" | "results";
type SearchScreenPane = "list" | "detail";

type SearchCountState = {
  status: "idle" | "loading" | "ready" | "error";
  result: SearchCountResult | null;
  message: string | null;
};

type SearchScreenState = {
  layout: SearchScreenLayout;
  activePane: SearchScreenPane;
  detailScroll: number;
  draft: Pf2eTerminalSearchRequest;
  workspaceSelectedIndex: number;
  resultSelectedIndex: number;
  session: Pf2eTerminalSearchSession | null;
};

type SearchScreenAction =
  | { type: "set_layout"; layout: SearchScreenLayout; pane?: SearchScreenPane }
  | { type: "set_active_pane"; pane: SearchScreenPane }
  | { type: "move_workspace_selection"; delta: number; itemCount: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_result_selection"; delta: number; itemCount: number }
  | { type: "set_result_selection"; index: number; itemCount: number }
  | { type: "result_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "set_draft"; request: Pf2eTerminalSearchRequest }
  | { type: "set_session"; session: Pf2eTerminalSearchSession; showResults?: boolean; preserveSelection?: boolean }
  | { type: "clear_results" };

const SEARCH_LEFT_WIDTH = 44;
const LIVE_COUNT_DEBOUNCE_MS = 150;
const EAGER_RESULT_BUFFER_LIMIT = 250;
const MIN_RESULT_WINDOW_LIMIT = 120;
const RESULT_WINDOW_PAGE_MULTIPLIER = 8;
const RESULT_PRELOAD_PAGE_MULTIPLIER = 4;
const RESULT_PRELOAD_JUMP_MULTIPLIER = 6;
const RESULT_WINDOW_FETCH_DEBOUNCE_MS = 40;

function createInitialSearchScreenState(initialRequest: Pf2eTerminalSearchRequest): SearchScreenState {
  return {
    layout: "draft",
    activePane: "list",
    detailScroll: 0,
    draft: initialRequest,
    workspaceSelectedIndex: 0,
    resultSelectedIndex: 0,
    session: null,
  };
}

function getSearchResultWindowMetrics(bodyHeight: number): {
  selectionJumpSize: number;
  pageSize: number;
  windowLimit: number;
  preloadThreshold: number;
} {
  const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const pageSize = Math.max(1, bodyHeight - 1);
  const windowLimit = Math.max(
    MIN_RESULT_WINDOW_LIMIT,
    pageSize * RESULT_WINDOW_PAGE_MULTIPLIER,
    selectionJumpSize * (RESULT_WINDOW_PAGE_MULTIPLIER + 2),
  );
  const preloadThreshold = Math.min(
    Math.max(1, windowLimit - 1),
    Math.max(
      pageSize * RESULT_PRELOAD_PAGE_MULTIPLIER,
      selectionJumpSize * RESULT_PRELOAD_JUMP_MULTIPLIER,
    ),
  );
  return {
    selectionJumpSize,
    pageSize,
    windowLimit,
    preloadThreshold,
  };
}

function clampAbsoluteSelection(index: number, total: number): number {
  return Math.max(0, Math.min(index, Math.max(0, total - 1)));
}

function getSearchResultWindowTarget(
  session: Pf2eTerminalSearchSession,
  selectedIndex: number,
  metrics: {
    windowLimit: number;
    preloadThreshold: number;
  },
) : { offset: number; limit: number } | null {
  if (session.total <= 0) {
    return null;
  }

  if (session.total <= EAGER_RESULT_BUFFER_LIMIT) {
    return session.windowOffset === 0 && session.results.length >= session.total
      ? null
      : { offset: 0, limit: session.total };
  }

  const windowSize = Math.min(
    session.total,
    Math.max(session.request.limit, metrics.windowLimit),
  );
  const windowStart = session.windowOffset;
  const windowEnd = session.windowOffset + session.results.length;
  const minimumBuffer = Math.min(
    metrics.preloadThreshold,
    Math.max(1, Math.floor(windowSize / 3)),
  );
  const remainingBehind = selectedIndex - windowStart;
  const remainingAhead = windowEnd - selectedIndex - 1;

  if (
    selectedIndex >= windowStart &&
    selectedIndex < windowEnd &&
    remainingBehind >= minimumBuffer &&
    remainingAhead >= minimumBuffer
  ) {
    return null;
  }

  const beforeBuffer = Math.max(minimumBuffer, Math.floor(windowSize / 3));
  const maxOffset = Math.max(0, session.total - windowSize);
  const offset = Math.max(0, Math.min(maxOffset, selectedIndex - beforeBuffer));
  const expectedCount = Math.min(windowSize, session.total - offset);

  if (
    selectedIndex >= windowStart &&
    selectedIndex < windowEnd &&
    session.windowOffset === offset &&
    session.results.length === expectedCount
  ) {
    return null;
  }

  return { offset, limit: windowSize };
}

function getSessionRecordAtIndex(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
): Pf2eTerminalSearchSession["results"][number] | null {
  if (!session || session.results.length === 0) {
    return null;
  }

  const localIndex = selectedIndex - session.windowOffset;
  return localIndex >= 0 && localIndex < session.results.length
    ? session.results[localIndex] ?? null
    : null;
}

function getSessionBufferRange(session: Pf2eTerminalSearchSession): string {
  if (session.results.length === 0) {
    return "empty";
  }

  return `${formatCount(session.windowOffset + 1)}-${formatCount(session.windowOffset + session.results.length)}`;
}

function formatResultPosition(selectedIndex: number, total: number): string {
  if (total <= 0) {
    return "0/0";
  }

  return `${formatCount(selectedIndex + 1)}/${formatCount(total)}`;
}

function formatResultReaderStatus(
  session: Pf2eTerminalSearchSession,
  selectedIndex: number,
): string {
  return `Pos ${formatResultPosition(selectedIndex, session.total)} | Buf ${formatCount(session.loadedCount)} | Win ${getSessionBufferRange(session)}`;
}

function searchScreenReducer(state: SearchScreenState, action: SearchScreenAction): SearchScreenState {
  switch (action.type) {
    case "set_layout":
      return {
        ...state,
        layout: action.layout,
        activePane: action.layout === "draft" ? "list" : (action.pane ?? "list"),
        detailScroll: 0,
      };
    case "set_active_pane":
      return {
        ...state,
        activePane: state.layout === "draft" ? "list" : action.pane,
        detailScroll: action.pane === "detail" ? state.detailScroll : 0,
      };
    case "move_workspace_selection":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelection(state.workspaceSelectedIndex, action.delta, action.itemCount),
      };
    case "workspace_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex: action.itemCount <= 0
          ? 0
          : action.boundary === "start"
            ? 0
            : action.itemCount - 1,
      };
    case "move_result_selection":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0
          ? 0
          : moveSelection(state.resultSelectedIndex, action.delta, action.itemCount),
      };
    case "set_result_selection":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0
          ? 0
          : clampAbsoluteSelection(action.index, action.itemCount),
      };
    case "result_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0
          ? 0
          : action.boundary === "start"
            ? 0
            : action.itemCount - 1,
      };
    case "move_detail":
      return {
        ...state,
        detailScroll: Math.max(0, Math.min(action.maxDetailScroll, state.detailScroll + action.delta)),
      };
    case "detail_boundary":
      return {
        ...state,
        detailScroll: action.boundary === "start" ? 0 : action.maxDetailScroll,
      };
    case "set_draft":
      return {
        ...state,
        draft: action.request,
      };
    case "set_session": {
      const maxIndex = Math.max(0, action.session.total - 1);
      return {
        ...state,
        layout: action.showResults === false ? state.layout : "results",
        activePane: action.showResults === false ? state.activePane : "list",
        detailScroll: 0,
        draft: action.session.request,
        resultSelectedIndex: action.preserveSelection
          ? Math.min(state.resultSelectedIndex, maxIndex)
          : 0,
        session: action.session,
      };
    }
    case "clear_results":
      return {
        ...state,
        layout: "draft",
        activePane: "list",
        detailScroll: 0,
        resultSelectedIndex: 0,
        session: null,
      };
    default:
      return state;
  }
}

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function formatSearchCategory(category: SearchCategory | null): string {
  return category ? humanizeIdentifier(category) : "Any Category";
}

function formatSearchSubcategory(subcategory: SearchSubcategory | null): string {
  return subcategory ? humanizeIdentifier(subcategory) : "Any Subcategory";
}

function formatMode(mode: Pf2eTerminalSearchMode): string {
  return humanizeIdentifier(mode);
}

function formatSort(sort: Pf2eTerminalSearchSort): string {
  switch (sort) {
    case "levelAsc":
      return "Level Low-High";
    case "levelDesc":
      return "Level High-Low";
    default:
      return humanizeIdentifier(sort);
  }
}

function formatPolicyValue(value: number | string): string {
  return typeof value === "number" ? String(value) : humanizeIdentifier(value);
}

function formatCount(value: number): string {
  return value.toLocaleString("en-US");
}

function formatFilterPolicy<T extends number | string>(
  policy: Pf2eTerminalFilterValuePolicy<T>,
): string {
  const parts: string[] = [];
  if (policy.any.length > 0) {
    parts.push(`any: ${policy.any.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  if (policy.all.length > 0) {
    parts.push(`all: ${policy.all.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  if (policy.exclude.length > 0) {
    parts.push(`exclude: ${policy.exclude.map((value) => formatPolicyValue(value)).join(", ")}`);
  }
  return parts.length > 0 ? parts.join(" | ") : "(any)";
}

function hasFilterPolicy<T extends number | string>(policy: Pf2eTerminalFilterValuePolicy<T>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function formatFacetSelection(facet: Pf2eTerminalFacetSelection): string {
  return `${humanizeIdentifier(facet.field)}: ${formatFilterPolicy(facet.policy)}`;
}

function formatLevelRange(request: Pf2eTerminalSearchRequest): string {
  const { levelMin, levelMax } = request.filters;
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

function formatLevelRangeInputValue(request: Pf2eTerminalSearchRequest): string {
  const { levelMin, levelMax } = request.filters;
  if (levelMin === null && levelMax === null) {
    return "";
  }
  if (levelMin !== null && levelMax !== null) {
    return levelMin === levelMax ? String(levelMin) : `${levelMin}-${levelMax}`;
  }
  if (levelMin !== null) {
    return `${levelMin}+`;
  }
  return `<=${levelMax}`;
}

function parseLevelRangeInput(value: string): { levelMin: number | null; levelMax: number | null } | string {
  const trimmed = value.trim();
  if (!trimmed) {
    return { levelMin: null, levelMax: null };
  }

  const betweenMatch = trimmed.match(/^(\d+)\s*-\s*(\d+)$/);
  if (betweenMatch) {
    return {
      levelMin: Number.parseInt(betweenMatch[1]!, 10),
      levelMax: Number.parseInt(betweenMatch[2]!, 10),
    };
  }

  if (/^\d+$/.test(trimmed)) {
    const level = Number.parseInt(trimmed, 10);
    return { levelMin: level, levelMax: level };
  }

  const minMatch = trimmed.match(/^(\d+)\+$/);
  if (minMatch) {
    return { levelMin: Number.parseInt(minMatch[1]!, 10), levelMax: null };
  }

  const maxMatch = trimmed.match(/^<=?\s*(\d+)$/);
  if (maxMatch) {
    return { levelMin: null, levelMax: Number.parseInt(maxMatch[1]!, 10) };
  }

  return "Use `3-8`, `5`, `5+`, or `<=10`.";
}

function buildSearchResultLabel(record: Pf2eTerminalSearchSession["results"][number]): string {
  const scope = record.subcategory ? `${record.category}/${record.subcategory}` : record.category;
  const level = record.level === null ? "-" : String(record.level);
  return `${record.name} | ${scope} | lvl ${level} | ${record.packLabel}`;
}

function hasStructuredSignal(request: Pf2eTerminalSearchRequest): boolean {
  return Boolean(
    request.filters.category ||
    request.filters.subcategory ||
    request.filters.levelMin !== null ||
    request.filters.levelMax !== null ||
    hasFilterPolicy(request.filters.rarity) ||
    hasFilterPolicy(request.filters.actionCost) ||
    request.filters.facets.length > 0,
  );
}

function getExecuteAvailability(request: Pf2eTerminalSearchRequest): { disabled: boolean; reason: string | null } {
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

function formatCountSummary(countState: SearchCountState, request: Pf2eTerminalSearchRequest): string {
  const availability = getExecuteAvailability(request);
  if (availability.disabled) {
    return availability.reason ?? "Unavailable";
  }
  if (countState.status === "loading") {
    return "Counting matches...";
  }
  if (countState.status === "error") {
    return countState.message ?? "Live count unavailable.";
  }
  if (countState.status === "ready" && countState.result) {
    const noun = request.mode === "lookup" ? "candidate" : "match";
    return `${countState.result.total} ${noun}${countState.result.total === 1 ? "" : "es"}`;
  }
  return "Count pending";
}

export function parseJumpToResultInput(input: string, total: number): number | string {
  const normalized = input.replace(/[,_\s]+/g, "");
  if (!/^\d+$/.test(normalized)) {
    return "Enter a result number such as `6000`.";
  }

  const parsed = Number.parseInt(normalized, 10);
  if (!Number.isFinite(parsed) || parsed < 1) {
    return "Result numbers start at 1.";
  }
  if (parsed > total) {
    return `Result ${parsed} is out of range. Valid positions are 1-${total}.`;
  }

  return parsed - 1;
}

function formatDraftStatus(state: SearchScreenState): string {
  if (!state.session) {
    return "Draft not executed yet";
  }
  return JSON.stringify(state.draft) === JSON.stringify(state.session.request)
    ? "Draft matches applied query"
    : "Draft has unapplied changes";
}

function buildWorkspaceEntries(state: SearchScreenState, countState: SearchCountState): SearchWorkspaceEntry[] {
  const executeAvailability = getExecuteAvailability(state.draft);
  const entries: SearchWorkspaceEntry[] = [
    {
      action: "execute",
      label: "Execute Query",
      value: formatCountSummary(countState, state.draft),
      description: executeAvailability.disabled
        ? executeAvailability.reason ?? "Unavailable for the current draft."
        : "Apply the current draft and switch into the results reader.",
      disabled: executeAvailability.disabled,
    },
    {
      action: "mode",
      label: "Mode",
      value: formatMode(state.draft.mode),
      description: "Choose whether this draft should browse deterministically, run ranked search, or perform exact lookup-style matching.",
    },
    {
      action: "query",
      label: "Query",
      value: state.draft.queryText || "(none)",
      description: state.draft.mode === "lookup"
        ? "Edit the lookup text used to find near-exact record names."
        : "Edit the free-text portion of the draft. Browse mode can leave this empty.",
    },
    {
      action: "category",
      label: "Category",
      value: formatSearchCategory(state.draft.filters.category),
      description: "Set the top-level category boundary for the draft.",
    },
    {
      action: "subcategory",
      label: "Subcategory",
      value: formatSearchSubcategory(state.draft.filters.subcategory),
      description: state.draft.filters.category
        ? "Set the within-category boundary for the draft."
        : "Choose a category first, then refine to a subcategory.",
      disabled: !state.draft.filters.category,
    },
    {
      action: "levels",
      label: "Levels",
      value: formatLevelRange(state.draft),
      description: "Constrain the draft to a level band such as `3-8` or `<=5`.",
    },
    {
      action: "rarity",
      label: "Rarity",
      value: formatFilterPolicy(state.draft.filters.rarity),
      description: "Cycle rarity values through include and exclude policies in one view.",
    },
    {
      action: "addFacet",
      label: "Edit Facet Filter",
      value: `${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)} active`,
      description: state.draft.filters.category
        ? "Choose a discoverable metadata field and cycle each value through any, all, or exclude."
        : "Choose a category before editing discoverable facet filters.",
      disabled: !state.draft.filters.category,
    },
    {
      action: "removeFacet",
      label: "Clear Facet Filter",
      value: `${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)} active`,
      description: state.draft.filters.facets.length > 0 || hasFilterPolicy(state.draft.filters.actionCost)
        ? "Remove an entire facet policy block from the current draft."
        : "No facet policies are currently applied.",
      disabled: state.draft.filters.facets.length === 0 && !hasFilterPolicy(state.draft.filters.actionCost),
    },
    {
      action: "reset",
      label: "Reset Draft",
      value: "Restore defaults",
      description: "Discard the current draft filters and return to the default workspace state.",
    },
    {
      action: "clearResults",
      label: "Discard Applied Results",
      value: state.session ? `${state.session.loadedCount}/${state.session.total} loaded` : "No results",
      description: state.session
        ? "Clear the applied result reader while leaving the draft untouched."
        : "There is no applied result reader to discard.",
      disabled: !state.session,
    },
  ];

  if (state.draft.mode === "search") {
    entries.splice(3, 0, {
      action: "profile",
      label: "Profile",
      value: state.draft.searchProfile,
      description: "Choose the lexical, balanced, or concept retrieval profile used by ranked search.",
    });
  }

  return entries;
}

function buildWorkspaceLines(
  entries: SearchWorkspaceEntry[],
  selectedIndex: number,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const visibleCount = Math.max(1, bodyHeight);
  const safeIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, entries.length - 1)));
  const windowStart = clampWindowStart(safeIndex, entries.length, visibleCount);

  return entries.slice(windowStart, windowStart + visibleCount).map((entry, offset) => ({
    text: `${entry.label} | ${entry.value}${entry.disabled ? " | unavailable" : ""}`,
    tone: windowStart + offset === safeIndex ? "selected" : entry.disabled ? "dim" : "default",
    noWrap: true,
  }));
}

function buildResultLines(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
  bodyHeight: number,
  loadingMore: boolean,
): DerivedTagTerminalLine[] {
  if (!session || session.results.length === 0) {
    return [
      { text: "No applied results yet.", tone: "section" },
      { text: "Execute the draft to switch into the result reader.", tone: "dim" },
    ];
  }

  const visibleCount = Math.max(1, bodyHeight);
  const statusRows = loadingMore || session.loadedCount < session.total ? 1 : 0;
  const resultWindowCount = Math.max(1, visibleCount - statusRows);
  const localSelectedIndex = selectedIndex - session.windowOffset;
  const safeIndex = Math.max(0, Math.min(localSelectedIndex, session.results.length - 1));
  const windowStart = clampWindowStart(safeIndex, session.results.length, resultWindowCount);

  const lines: DerivedTagTerminalLine[] = session.results.slice(windowStart, windowStart + resultWindowCount).map((record, offset) => ({
    text: buildSearchResultLabel(record),
    tone: localSelectedIndex >= 0 && localSelectedIndex < session.results.length && windowStart + offset === localSelectedIndex
      ? "selected"
      : "default",
    noWrap: true,
  }));

  if (loadingMore) {
    lines.push({ text: `Loading around ${formatResultPosition(selectedIndex, session.total)}...`, tone: "accent" });
  } else if (session.loadedCount < session.total) {
    lines.push({
      text: formatResultReaderStatus(session, selectedIndex),
      tone: "dim",
    });
  }

  return lines;
}

function buildPendingResultDetailLines(
  session: Pf2eTerminalSearchSession,
  resultIndex: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Result Preview", tone: "section" },
    { text: `Showing result ${formatResultPosition(resultIndex, session.total)}` },
    { text: `Sort: ${formatSort(session.sort)}` },
    { text: "" },
    { text: "Loading the result window around the current selection.", tone: "accent" },
    { text: `Current buffer: ${getSessionBufferRange(session)}`, tone: "dim" },
  ];
}

function buildDraftSummaryLines(
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const executeAvailability = getExecuteAvailability(state.draft);
  const lines: DerivedTagTerminalLine[] = [
    { text: "Draft Summary", tone: "section" },
    { text: `Mode: ${formatMode(state.draft.mode)}` },
    { text: `Query: ${state.draft.queryText || "(none)"}` },
    { text: `Scope: ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}` },
    { text: `Levels: ${formatLevelRange(state.draft)}` },
    { text: `Rarity: ${formatFilterPolicy(state.draft.filters.rarity)}` },
    { text: `Facet filters: ${state.draft.filters.facets.length + (hasFilterPolicy(state.draft.filters.actionCost) ? 1 : 0)}` },
  ];

  if (state.draft.mode === "search") {
    lines.splice(3, 0, { text: `Profile: ${state.draft.searchProfile}` });
  }

  if (hasFilterPolicy(state.draft.filters.actionCost)) {
    lines.push({ text: `Action Cost: ${formatFilterPolicy(state.draft.filters.actionCost)}`, indent: 2 });
  }

  for (const facet of state.draft.filters.facets) {
    lines.push({ text: formatFacetSelection(facet), indent: 2 });
  }

  if (state.draft.sourceLabel) {
    lines.push({ text: `Seeded from: ${state.draft.sourceLabel}` });
  }

  lines.push({ text: "" });
  lines.push({ text: "Live Count", tone: "section" });
  if (executeAvailability.disabled) {
    lines.push({ text: executeAvailability.reason ?? "Unavailable for the current draft.", tone: "warning" });
  } else if (countState.status === "loading") {
    lines.push({ text: "Counting lexical matches for the current draft...", tone: "accent" });
  } else if (countState.status === "error") {
    lines.push({ text: countState.message ?? "Live count unavailable.", tone: "warning" });
  } else if (countState.status === "ready" && countState.result) {
    lines.push({
      text: `${countState.result.total} matching record${countState.result.total === 1 ? "" : "s"} before result ordering.`,
      tone: countState.result.total === 0 ? "warning" : "default",
    });
  } else {
    lines.push({ text: "Count pending.", tone: "dim" });
  }

  lines.push({ text: "Tab executes the draft and opens the results reader.", tone: "accent" });

  lines.push({ text: "" });
  lines.push({ text: "Applied Session", tone: "section" });
  lines.push({ text: formatDraftStatus(state) });
  if (!state.session) {
    lines.push({ text: "No applied query yet.", tone: "dim" });
  } else {
    lines.push({ text: `Sort: ${formatSort(state.session.sort)}` });
    lines.push({ text: `Position: ${formatResultPosition(state.resultSelectedIndex, state.session.total)}` });
    lines.push({ text: `Buffered: ${formatCount(state.session.loadedCount)}` });
    lines.push({ text: `Window: ${getSessionBufferRange(state.session)}` });
    lines.push({ text: `Applied mode: ${formatMode(state.session.request.mode)} | ${state.session.resultMode}` });
  }

  return lines;
}

function buildWorkspaceEntryDetailLines(
  entry: SearchWorkspaceEntry,
  state: SearchScreenState,
  countState: SearchCountState,
): DerivedTagTerminalLine[] {
  const descriptionTone = entry.disabled ? "warning" : "default";
  return [
    { text: entry.label, tone: "section" },
    { text: `Current value: ${entry.value}` },
    {
      text: entry.disabled
        ? `Unavailable: ${entry.description}`
        : entry.description,
      tone: descriptionTone,
    },
    ...(entry.disabled
      ? []
      : entry.action === "execute"
        ? [{ text: "Press Enter or Tab to execute the draft and switch to results.", tone: "accent" as const }]
        : [{ text: "Press Enter to edit or act on this item.", tone: "accent" as const }]),
    { text: "" },
    ...buildDraftSummaryLines(state, countState),
  ];
}

function buildResultDetailLines(
  record: Pf2eTerminalSearchSession["results"][number],
  session: Pf2eTerminalSearchSession,
  resultIndex: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Result Preview", tone: "section" },
    { text: `Showing result ${formatResultPosition(resultIndex, session.total)}` },
    { text: `Sort: ${formatSort(session.sort)}` },
    { text: "" },
    ...buildOntologyExplorerEntityDetailLines(mapNormalizedRecordToOntologyExplorerEntityRecord(record)),
  ];
}

function buildSearchSubtitle(state: SearchScreenState, countState: SearchCountState): string {
  const draft = `${formatMode(state.draft.mode)} | ${formatSearchCategory(state.draft.filters.category)} / ${formatSearchSubcategory(state.draft.filters.subcategory)}`;
  if (state.layout === "draft") {
    return `${draft} | ${formatCountSummary(countState, state.draft)} | ${formatDraftStatus(state)}`;
  }
  if (!state.session) {
    return `${draft} | no applied session`;
  }
  return `${draft} | ${formatSort(state.session.sort)} | ${formatResultPosition(state.resultSelectedIndex, state.session.total)} | ${formatDraftStatus(state)}`;
}

function buildFacetRemovalEntries(
  facets: Pf2eTerminalFacetSelection[],
  actionCost: Pf2eTerminalFilterValuePolicy<number>,
): Array<{
  value: string;
  label: string;
  description: string;
}> {
  const entries = facets.map((facet) => ({
    value: facet.field,
    label: humanizeIdentifier(facet.field),
    description: `Clear ${formatFilterPolicy(facet.policy)} from the current draft filter stack.`,
  }));

  if (hasFilterPolicy(actionCost)) {
    entries.unshift({
      value: "actionCost",
      label: "Action Cost",
      description: `Clear ${formatFilterPolicy(actionCost)} from the current draft filter stack.`,
    });
  }

  return entries;
}

function buildFooterText(
  state: SearchScreenState,
  loadingMore: boolean,
): string {
  if (state.layout === "draft") {
    return "Up/Down select  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  Enter edit  Tab execute  / query  Esc/backspace back  q back";
  }

  if (state.activePane === "list") {
    return loadingMore
      ? "Up/Down select  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  n/: jump-to  Left draft  Right preview  Enter preview  Tab toggle  O sort  Loading more..."
      : "Up/Down select  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  n/: jump-to  Left draft  Right preview  Enter preview  Tab toggle  O sort  q back";
  }

  return "Up/Down scroll  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  n/: jump-to  Left results  Tab toggle  O sort  Esc/backspace results  q back";
}

export function SearchScreen({
  initialQuery,
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  onBack: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const [busy, setBusy] = React.useState(false);
  const [loadingMore, setLoadingMore] = React.useState(false);
  const [countState, setCountState] = React.useState<SearchCountState>({
    status: "idle",
    result: null,
    message: null,
  });
  const initialRequest = React.useMemo(
    () => initialQuery ? user.search.createRequestFromOntologyQuery(initialQuery) : user.search.createDefaultRequest(),
    [initialQuery, user.search],
  );
  const [state, dispatch] = React.useReducer(searchScreenReducer, initialRequest, createInitialSearchScreenState);
  const autoRanInitialQuery = React.useRef(false);
  const loadMoreSessionKeyRef = React.useRef<string | null>(null);
  const loadMoreTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const detailNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());

  const workspaceEntries = buildWorkspaceEntries(state, countState);
  const workspaceSelectedIndex = Math.max(0, Math.min(state.workspaceSelectedIndex, Math.max(0, workspaceEntries.length - 1)));
  const selectedWorkspaceEntry = workspaceEntries[workspaceSelectedIndex] ?? workspaceEntries[0];

  const resultCount = state.session?.total ?? 0;
  const resultSelectedIndex = clampAbsoluteSelection(state.resultSelectedIndex, resultCount);
  const selectedResult = resultCount > 0 ? getSessionRecordAtIndex(state.session, resultSelectedIndex) : null;

  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const {
    selectionJumpSize,
    pageSize,
    windowLimit: resultWindowLimit,
    preloadThreshold,
  } = getSearchResultWindowMetrics(bodyHeight);
  const detailWidth = getTerminalTwoPaneDetailWidth(size.width, "split", SEARCH_LEFT_WIDTH);
  const detailLines = state.layout === "results" && state.session
    ? selectedResult
      ? buildResultDetailLines(selectedResult, state.session, resultSelectedIndex)
      : buildPendingResultDetailLines(state.session, resultSelectedIndex)
    : selectedWorkspaceEntry
      ? buildWorkspaceEntryDetailLines(selectedWorkspaceEntry, state, countState)
      : buildDraftSummaryLines(state, countState);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);

  const applyDraftUpdate = React.useCallback((update: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest) => {
    dispatch({ type: "set_draft", request: user.search.normalizeRequest(update(state.draft)) });
  }, [state.draft, user.search]);

  const executeRequest = React.useCallback(async (request: Pf2eTerminalSearchRequest) => {
    const availability = getExecuteAvailability(request);
    if (availability.disabled) {
      await terminal.pauseForAnyKey(availability.reason ?? "This draft cannot be executed yet.");
      return;
    }

    setBusy(true);
    try {
      const sort = state.session && state.session.request.mode === request.mode
        ? state.session.sort
        : user.search.getDefaultSort(request.mode);
      const session = await user.search.executeQuery(request, {
        sort,
        limit: Math.max(request.limit, resultWindowLimit),
      });
      dispatch({ type: "set_session", session });
    } catch (error) {
      await terminal.pauseForAnyKey(`Workspace query failed.\n\n${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [resultWindowLimit, state.session, terminal, user.search]);

  const chooseResultSort = React.useCallback(async () => {
    if (!state.session) {
      return;
    }

    const selected = await terminal.promptSelectOption({
      title: "Result Sort",
      prompt: "Choose how the current result reader should be ordered",
      entries: user.search.getResultSortOptions(state.session.request.mode).map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.session.sort,
    });

    if (!selected || selected === state.session.sort) {
      return;
    }

    setBusy(true);
    try {
      const session = await user.search.changeSort(state.session, selected as Pf2eTerminalSearchSort);
      dispatch({ type: "set_session", session });
    } catch (error) {
      await terminal.pauseForAnyKey(`Result sort failed.\n\n${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [state.session, terminal, user.search]);

  const jumpToResultPosition = React.useCallback(async () => {
    if (!state.session || state.session.total <= 0) {
      return;
    }

    const input = await terminal.promptTextInput({
      title: "Jump To Result",
      prompt: `Enter a result number between 1 and ${state.session.total}.`,
      hint: `Current: ${resultSelectedIndex + 1}  Example: 6000`,
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseJumpToResultInput(input, state.session.total);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    dispatch({
      type: "set_result_selection",
      index: parsed,
      itemCount: state.session.total,
    });
  }, [resultSelectedIndex, state.session, terminal]);

  React.useEffect(() => {
    if (!initialQuery || autoRanInitialQuery.current) {
      return;
    }
    autoRanInitialQuery.current = true;
    void executeRequest(initialRequest);
  }, [executeRequest, initialQuery, initialRequest]);

  React.useEffect(() => {
    const availability = getExecuteAvailability(state.draft);
    if (availability.disabled) {
      setCountState({
        status: "idle",
        result: null,
        message: availability.reason,
      });
      return;
    }

    let cancelled = false;
    setCountState((current) => ({
      status: "loading",
      result: current.result,
      message: null,
    }));

    const timeout = setTimeout(() => {
      void user.search.countQuery(state.draft)
        .then((result) => {
          if (cancelled) {
            return;
          }
          setCountState({
            status: "ready",
            result,
            message: null,
          });
        })
        .catch((error) => {
          if (cancelled) {
            return;
          }
          setCountState({
            status: "error",
            result: null,
            message: (error as Error).message,
          });
        });
    }, LIVE_COUNT_DEBOUNCE_MS);

    return () => {
      cancelled = true;
      clearTimeout(timeout);
    };
  }, [state.draft, user.search]);

  React.useEffect(() => {
    if (loadMoreTimerRef.current) {
      clearTimeout(loadMoreTimerRef.current);
      loadMoreTimerRef.current = null;
    }
    if (!state.session || state.layout !== "results" || state.session.total <= 0) {
      return;
    }

    const currentSession = state.session;

    const targetWindow = getSearchResultWindowTarget(currentSession, resultSelectedIndex, {
      windowLimit: resultWindowLimit,
      preloadThreshold,
    });
    if (!targetWindow) {
      return;
    }

    const sessionKey =
      `${currentSession.windowId}:${currentSession.sort}:${currentSession.windowOffset}:${currentSession.loadedCount}:` +
      `${targetWindow.offset}:${targetWindow.limit}`;
    if (loadingMore || loadMoreSessionKeyRef.current === sessionKey) {
      return;
    }

    let cancelled = false;
    loadMoreTimerRef.current = setTimeout(() => {
      if (cancelled) {
        return;
      }

      loadMoreSessionKeyRef.current = sessionKey;
      setLoadingMore(true);
      void user.search.readResultWindow(currentSession, targetWindow)
        .then((session) => {
          if (cancelled) {
            return;
          }
          dispatch({ type: "set_session", session, showResults: true, preserveSelection: true });
        })
        .catch(async (error) => {
          if (cancelled) {
            return;
          }
          await terminal.pauseForAnyKey(`Loading more results failed.\n\n${(error as Error).message}`);
        })
        .finally(() => {
          if (loadMoreSessionKeyRef.current === sessionKey) {
            loadMoreSessionKeyRef.current = null;
            setLoadingMore(false);
          }
        });
    }, RESULT_WINDOW_FETCH_DEBOUNCE_MS);

    return () => {
      cancelled = true;
      if (loadMoreTimerRef.current) {
        clearTimeout(loadMoreTimerRef.current);
        loadMoreTimerRef.current = null;
      }
    };
  }, [loadingMore, preloadThreshold, resultSelectedIndex, resultWindowLimit, state.layout, state.session, terminal, user.search]);

  const editQueryText = React.useCallback(async () => {
    const queryText = await terminal.promptTextInput({
      title: "Draft Query",
      prompt: state.draft.mode === "lookup"
        ? "Enter an exact or near-exact record name"
        : "Enter search text for the current draft",
      defaultValue: state.draft.queryText,
      hint: state.draft.mode === "lookup"
        ? "Example: Raise Shield"
        : "Example: ghost ship captain",
    });

    if (queryText === undefined) {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      queryText,
    }));
  }, [applyDraftUpdate, state.draft.mode, state.draft.queryText, terminal]);

  const chooseMode = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Workspace Mode",
      prompt: "Choose how the current draft should execute",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.mode,
    });

    if (!selected) {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      mode: selected,
    }));
  }, [applyDraftUpdate, state.draft.mode, terminal, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the draft profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.searchProfile,
    });

    if (selected) {
      applyDraftUpdate((request) => ({
        ...request,
        searchProfile: selected as SearchProfile,
      }));
    }
  }, [applyDraftUpdate, state.draft.searchProfile, terminal, user.search]);

  const chooseCategoryFilter = React.useCallback(async () => {
    const selected = await terminal.promptSelectOption({
      title: "Category Scope",
      prompt: "Choose the draft category boundary",
      entries: user.search.getCategoryOptions().map((option) => ({
        value: option.value ?? "__all__",
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.category ?? "__all__",
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          category: selected === "__all__" ? null : selected as SearchCategory,
          subcategory: null,
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.category, terminal, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const selected = await terminal.promptSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the draft subcategory boundary",
      entries: user.search.getSubcategoryOptions(state.draft.filters.category).map((option) => ({
        value: option.value ?? "__all__",
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.subcategory ?? "__all__",
    });

    if (selected !== undefined) {
      applyDraftUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          subcategory: selected === "__all__" ? null : selected as SearchSubcategory,
        },
      }));
    }
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(state.draft.filters.category, state.draft.filters.subcategory);
    const selected = await terminal.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle draft rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: state.draft.filters.rarity,
    });

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        rarity: {
          any: selected.any,
          all: [],
          exclude: selected.exclude,
        },
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.rarity, state.draft.filters.subcategory, terminal, user.search]);

  const editLevelRange = React.useCallback(async () => {
    const input = await terminal.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue: formatLevelRangeInputValue(state.draft),
      hint: "Examples: 3-8 or <=5",
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseLevelRangeInput(input);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        levelMin: parsed.levelMin,
        levelMax: parsed.levelMax,
      },
    }));
  }, [applyDraftUpdate, state.draft, terminal]);

  const addFacetFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before editing a discoverable facet filter.");
      return;
    }

    const fieldOptions = user.search.getFacetFieldOptions(state.draft.filters.category, state.draft.filters.subcategory);
    if (fieldOptions.length === 0) {
      await terminal.pauseForAnyKey("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const selectedField = await terminal.promptSelectOption({
      title: "Facet Field",
      prompt: "Choose a discoverable field to edit in the draft",
      entries: fieldOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
    });

    if (!selectedField) {
      return;
    }

    const selectedFieldOption = fieldOptions.find((option) => option.value === selectedField);
    if (!selectedFieldOption) {
      return;
    }

    const valueOptions = selectedField === "actionCost"
      ? user.search.getActionCostOptions(state.draft.filters.category, state.draft.filters.subcategory)
      : user.search.getFacetValueOptions(
        selectedField as Pf2eTerminalFacetField,
        state.draft.filters.category,
        state.draft.filters.subcategory,
      );
    if (valueOptions.length === 0) {
      await terminal.pauseForAnyKey("No live values are available for that field in the current scope.");
      return;
    }

    const selectedPolicy = await terminal.promptPolicySelectOption({
      title: "Facet Policy",
      prompt: `Cycle values for ${humanizeIdentifier(selectedField)}. Press Esc or Left when finished.`,
      allowedStates: selectedFieldOption.fieldType === "set"
        ? ["any", "all", "exclude"]
        : ["any", "exclude"],
      entries: valueOptions.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: selectedField === "actionCost"
        ? {
          any: state.draft.filters.actionCost.any.map((value) => String(value)),
          exclude: state.draft.filters.actionCost.exclude.map((value) => String(value)),
        }
        : state.draft.filters.facets.find((facet) => facet.field === selectedField)?.policy,
    });

    applyDraftUpdate((request) => {
      if (selectedField === "actionCost") {
        return {
          ...request,
          filters: {
            ...request.filters,
            actionCost: {
              any: selectedPolicy.any.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
              all: [],
              exclude: selectedPolicy.exclude.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
            },
          },
        };
      }

      const facets = [...request.filters.facets];
      const currentIndex = facets.findIndex((facet) => facet.field === selectedField);
      if (selectedPolicy.any.length === 0 && selectedPolicy.all.length === 0 && selectedPolicy.exclude.length === 0) {
        return {
          ...request,
          filters: {
            ...request.filters,
            facets: facets.filter((facet) => facet.field !== selectedField),
          },
        };
      }

      if (currentIndex >= 0) {
        facets[currentIndex] = {
          field: facets[currentIndex]!.field,
          policy: selectedPolicy,
        };
      } else {
        facets.push({
          field: selectedField as Pf2eTerminalFacetField,
          policy: selectedPolicy,
        });
      }

      return {
        ...request,
        filters: {
          ...request.filters,
          facets,
        },
      };
    });
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const removeFacetFilter = React.useCallback(async () => {
    if (state.draft.filters.facets.length === 0 && !hasFilterPolicy(state.draft.filters.actionCost)) {
      await terminal.pauseForAnyKey("There are no facet policies to clear from the draft.");
      return;
    }

    const selected = await terminal.promptMultiSelectOption({
      title: "Clear Facet Filter",
      prompt: "Toggle facet fields to clear. Press Esc or Left when finished.",
      entries: buildFacetRemovalEntries(state.draft.filters.facets, state.draft.filters.actionCost),
      selectedValues: [],
    });
    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        actionCost: selected.includes("actionCost")
          ? { any: [], all: [], exclude: [] }
          : request.filters.actionCost,
        facets: request.filters.facets.filter((facet) => !selected.includes(facet.field)),
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.actionCost, state.draft.filters.facets, terminal]);

  const resetDraftWorkspace = React.useCallback(() => {
    dispatch({ type: "set_draft", request: user.search.createDefaultRequest() });
    dispatch({ type: "set_layout", layout: "draft", pane: "list" });
  }, [user.search]);

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }

    switch (selectedWorkspaceEntry.action) {
      case "execute":
        void executeRequest(state.draft);
        return;
      case "mode":
        void chooseMode();
        return;
      case "query":
        void editQueryText();
        return;
      case "profile":
        void chooseSearchProfile();
        return;
      case "category":
        void chooseCategoryFilter();
        return;
      case "subcategory":
        void chooseSubcategoryFilter();
        return;
      case "levels":
        void editLevelRange();
        return;
      case "rarity":
        void chooseRarityFilter();
        return;
      case "addFacet":
        void addFacetFilter();
        return;
      case "removeFacet":
        void removeFacetFilter();
        return;
      case "reset":
        resetDraftWorkspace();
        return;
      case "clearResults":
        dispatch({ type: "clear_results" });
        return;
      default:
        return;
    }
  }, [
    addFacetFilter,
    chooseCategoryFilter,
    chooseMode,
    chooseRarityFilter,
    chooseSearchProfile,
    chooseSubcategoryFilter,
    editLevelRange,
    editQueryText,
    executeRequest,
    removeFacetFilter,
    resetDraftWorkspace,
    selectedWorkspaceEntry,
    state.draft,
  ]);

  useDerivedTagTerminalInput((input, key) => {
    if (busy) {
      return;
    }

    const normalized = getNormalizedKeyName(input, key);
    const listNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize,
      jumpSize: selectionJumpSize,
      includeConfirmKeys: true,
    }, listNavigationStateRef.current);
    listNavigationStateRef.current = listNavigation.state;
    const detailNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize,
      jumpSize: selectionJumpSize,
    }, detailNavigationStateRef.current);
    detailNavigationStateRef.current = detailNavigation.state;

    if (normalized === "ctrl_c" || normalized === "q") {
      onBack();
      return;
    }
    if (normalized === "slash") {
      void editQueryText();
      return;
    }

    if (state.layout === "draft") {
      if (normalized === "tab" || normalized === "shift_tab") {
        void executeRequest(state.draft);
        return;
      }
      if (normalized === "escape" || normalized === "backspace") {
        onBack();
        return;
      }
      if (listNavigation.action?.kind === "move") {
        dispatch({ type: "move_workspace_selection", delta: listNavigation.action.delta, itemCount: workspaceEntries.length });
        return;
      }
      if (listNavigation.action?.kind === "boundary") {
        dispatch({
          type: "workspace_selection_boundary",
          boundary: listNavigation.action.boundary,
          itemCount: workspaceEntries.length,
        });
        return;
      }
      if (listNavigation.action?.kind === "confirm") {
        openSelectedWorkspaceEntry();
        return;
      }
      if (normalized === "m") {
        void chooseMode();
        return;
      }
      if (normalized === "p") {
        void chooseSearchProfile();
        return;
      }
      if (normalized === "c") {
        void chooseCategoryFilter();
        return;
      }
      if (normalized === "s") {
        void chooseSubcategoryFilter();
        return;
      }
      if (normalized === "v") {
        void editLevelRange();
        return;
      }
      if (normalized === "r") {
        void chooseRarityFilter();
        return;
      }
      if (normalized === "f") {
        void addFacetFilter();
        return;
      }
      if (normalized === "d") {
        void removeFacetFilter();
        return;
      }
      if (normalized === "x") {
        resetDraftWorkspace();
        return;
      }
      if (normalized === "e") {
        void executeRequest(state.draft);
      }
      return;
    }

    if (normalized === "o" || normalized === "s") {
      void chooseResultSort();
      return;
    }
    if (normalized === "n" || normalized === ":") {
      void jumpToResultPosition();
      return;
    }
    if (normalized === "tab" || normalized === "shift_tab") {
      dispatch({ type: "set_active_pane", pane: state.activePane === "list" ? "detail" : "list" });
      return;
    }

    if (state.activePane === "list") {
      if (normalized === "escape" || normalized === "backspace" || normalized === "left") {
        dispatch({ type: "set_layout", layout: "draft", pane: "list" });
        return;
      }
      if ((normalized === "right" || listNavigation.action?.kind === "confirm") && selectedResult) {
        dispatch({ type: "set_active_pane", pane: "detail" });
        return;
      }
      if (listNavigation.action?.kind === "move") {
        dispatch({ type: "move_result_selection", delta: listNavigation.action.delta, itemCount: resultCount });
        return;
      }
      if (listNavigation.action?.kind === "boundary") {
        dispatch({ type: "result_selection_boundary", boundary: listNavigation.action.boundary, itemCount: resultCount });
      }
      return;
    }

    if (normalized === "escape" || normalized === "backspace" || normalized === "left") {
      dispatch({ type: "set_active_pane", pane: "list" });
      return;
    }
    if (detailNavigation.action?.kind === "move") {
      dispatch({ type: "move_detail", delta: detailNavigation.action.delta, maxDetailScroll });
      return;
    }
    if (detailNavigation.action?.kind === "boundary") {
      dispatch({ type: "detail_boundary", boundary: detailNavigation.action.boundary, maxDetailScroll });
    }
  }, !busy);

  return (
    <TerminalTwoPaneScreen
      title="Browse/Search"
      subtitle={buildSearchSubtitle(state, countState)}
      left={{
        title: state.layout === "draft"
          ? "[DRAFT] Scope & Filters"
          : state.activePane === "list"
            ? `[RESULTS] ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | ${formatSort(state.session.sort)}` : "No applied session"}`
            : `Results | ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | ${formatSort(state.session.sort)}` : "No applied session"}`,
        lines: state.layout === "draft"
          ? buildWorkspaceLines(workspaceEntries, workspaceSelectedIndex, bodyHeight)
          : buildResultLines(state.session, resultSelectedIndex, bodyHeight, loadingMore),
        active: state.layout === "results" ? state.activePane === "list" : true,
      }}
      right={{
        title: state.layout === "results"
          ? state.activePane === "detail"
            ? `[PREVIEW] ${selectedResult?.name ?? "Results"}`
            : `Preview | ${selectedResult?.name ?? "Results"}`
          : "Draft Status",
        lines: sliceRenderedTerminalLines(
          detailLines,
          detailWidth,
          detailScroll,
          bodyHeight,
        ),
        active: state.layout === "results" && state.activePane === "detail",
      }}
      footer={[
        {
          text: buildFooterText(state, loadingMore),
          tone: "dim",
        },
        {
          text: state.layout === "results" && state.session
            ? `${formatDraftStatus(state)} | ${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | Win ${getSessionBufferRange(state.session)}`
            : `${formatDraftStatus(state)} | ${formatCountSummary(countState, state.draft)} | Draft Workspace`,
          tone: "accent",
        },
      ]}
      leftWidth={SEARCH_LEFT_WIDTH}
    />
  );
}
