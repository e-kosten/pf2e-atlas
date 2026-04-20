import type { SearchCountResult } from "../../domain/index.js";
import type { Pf2eTerminalSearchQuery, Pf2eTerminalSearchSession, Pf2eTerminalSearchSort } from "../search/service.js";
import { moveSelection } from "../framework/input.js";
import { reduceDerivedTagTerminalTwoPaneState } from "../two-pane-state.js";
export type {
  SearchQueryFieldBuilderDraft,
  SearchQueryFieldBuilderOutcome,
  SearchQueryFieldBuilderSession,
  SearchQueryFieldBuilderStep,
  SearchQueryFieldPickerSession,
} from "./query-field-builder-session.js";
export type {
  SearchStructuredDraftAnchor,
  SearchStructuredDraftEntry,
  SearchStructuredDraftEntryKind,
  SearchStructuredDraftSession,
} from "../search/structured-draft-session.js";

export type SearchScreenLayout = "editor" | "results";
export type SearchScreenPane = "list" | "detail";

export type SearchCountState = {
  status: "idle" | "loading" | "ready" | "error";
  result: SearchCountResult | null;
  message: string | null;
};

export type SearchScreenState = {
  layout: SearchScreenLayout;
  activePane: SearchScreenPane;
  detailScroll: number;
  layoutMode: "split";
  query: Pf2eTerminalSearchQuery;
  workspaceSelectedIndex: number;
  resultSelectedIndex: number;
  session: Pf2eTerminalSearchSession | null;
};

export type SearchScreenAction =
  | { type: "set_layout"; layout: SearchScreenLayout; pane?: SearchScreenPane }
  | { type: "set_active_pane"; pane: SearchScreenPane }
  | { type: "move_workspace_selection"; delta: number; itemCount: number }
  | { type: "workspace_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_result_selection"; delta: number; itemCount: number }
  | { type: "set_result_selection"; index: number; itemCount: number }
  | { type: "result_selection_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "set_query"; query: Pf2eTerminalSearchQuery }
  | { type: "set_session"; session: Pf2eTerminalSearchSession; showResults?: boolean; preserveSelection?: boolean }
  | { type: "clear_results" };

export const SEARCH_LEFT_WIDTH = 44;
export const LIVE_COUNT_DEBOUNCE_MS = 150;
const EAGER_RESULT_BUFFER_LIMIT = 250;
const MIN_RESULT_WINDOW_LIMIT = 120;
const RESULT_WINDOW_PAGE_MULTIPLIER = 8;
const RESULT_PRELOAD_PAGE_MULTIPLIER = 4;
const RESULT_PRELOAD_JUMP_MULTIPLIER = 6;
export const RESULT_WINDOW_FETCH_DEBOUNCE_MS = 40;

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function formatSort(sort: Pf2eTerminalSearchSort): string {
  switch (sort) {
    case "ranked":
    case "alphabetical":
    case "random":
      return humanizeIdentifier(sort);
    case "levelAsc":
      return "Level Low-High";
    case "levelDesc":
      return "Level High-Low";
  }
}

export function formatCount(value: number): string {
  return value.toLocaleString("en-US");
}

export function createInitialSearchScreenState(initialQuery: Pf2eTerminalSearchQuery): SearchScreenState {
  return {
    layout: "editor",
    activePane: "list",
    detailScroll: 0,
    layoutMode: "split",
    query: initialQuery,
    workspaceSelectedIndex: 0,
    resultSelectedIndex: 0,
    session: null,
  };
}

function reduceSearchTwoPaneState(
  state: SearchScreenState,
  action:
    | { type: "leave_detail" }
    | { type: "toggle_focus" }
    | { type: "move_detail"; delta: number; maxDetailScroll: number }
    | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number },
): Pick<SearchScreenState, "activePane" | "detailScroll" | "layoutMode"> {
  const next = reduceDerivedTagTerminalTwoPaneState(
    {
      activePane: state.activePane,
      detailScroll: state.detailScroll,
      layoutMode: state.layoutMode,
    },
    action,
  );

  return {
    activePane: next.activePane,
    detailScroll: next.detailScroll,
    layoutMode: "split",
  };
}

export function getSearchResultWindowMetrics(bodyHeight: number): {
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
    Math.max(pageSize * RESULT_PRELOAD_PAGE_MULTIPLIER, selectionJumpSize * RESULT_PRELOAD_JUMP_MULTIPLIER),
  );
  return {
    selectionJumpSize,
    pageSize,
    windowLimit,
    preloadThreshold,
  };
}

export function clampAbsoluteSelection(index: number, total: number): number {
  return Math.max(0, Math.min(index, Math.max(0, total - 1)));
}

export function getSearchResultWindowTarget(
  session: Pf2eTerminalSearchSession,
  selectedIndex: number,
  metrics: {
    windowLimit: number;
    preloadThreshold: number;
  },
): { offset: number; limit: number } | null {
  if (session.total <= 0) {
    return null;
  }

  if (session.total <= EAGER_RESULT_BUFFER_LIMIT) {
    return session.windowOffset === 0 && session.results.length >= session.total
      ? null
      : { offset: 0, limit: session.total };
  }

  const windowSize = Math.min(session.total, Math.max(session.query.limit, metrics.windowLimit));
  const windowStart = session.windowOffset;
  const windowEnd = session.windowOffset + session.results.length;
  const minimumBuffer = Math.min(metrics.preloadThreshold, Math.max(1, Math.floor(windowSize / 3)));
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

export function getSessionRecordAtIndex(
  session: Pf2eTerminalSearchSession | null,
  selectedIndex: number,
): Pf2eTerminalSearchSession["results"][number] | null {
  if (!session || session.results.length === 0) {
    return null;
  }

  const localIndex = selectedIndex - session.windowOffset;
  return localIndex >= 0 && localIndex < session.results.length ? (session.results[localIndex] ?? null) : null;
}

export function getSessionBufferRange(session: Pf2eTerminalSearchSession): string {
  if (session.results.length === 0) {
    return "empty";
  }

  return `${formatCount(session.windowOffset + 1)}-${formatCount(session.windowOffset + session.results.length)}`;
}

export function formatResultPosition(selectedIndex: number, total: number): string {
  if (total <= 0) {
    return "0/0";
  }

  return `${formatCount(selectedIndex + 1)}/${formatCount(total)}`;
}

export function searchScreenReducer(state: SearchScreenState, action: SearchScreenAction): SearchScreenState {
  switch (action.type) {
    case "set_layout":
      return {
        ...state,
        layout: action.layout,
        activePane: action.layout === "editor" ? "list" : (action.pane ?? "list"),
        detailScroll: 0,
        layoutMode: "split",
      };
    case "set_active_pane":
      if (state.layout === "editor" || action.pane === "list") {
        return {
          ...state,
          ...reduceSearchTwoPaneState(state, { type: "leave_detail" }),
        };
      }
      return {
        ...state,
        ...reduceSearchTwoPaneState({ ...state, activePane: "list" }, { type: "toggle_focus" }),
      };
    case "move_workspace_selection":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex:
          action.itemCount <= 0 ? 0 : moveSelection(state.workspaceSelectedIndex, action.delta, action.itemCount),
      };
    case "workspace_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        workspaceSelectedIndex: action.itemCount <= 0 ? 0 : action.boundary === "start" ? 0 : action.itemCount - 1,
      };
    case "move_result_selection":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex:
          action.itemCount <= 0 ? 0 : moveSelection(state.resultSelectedIndex, action.delta, action.itemCount),
      };
    case "set_result_selection":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0 ? 0 : clampAbsoluteSelection(action.index, action.itemCount),
      };
    case "result_selection_boundary":
      return {
        ...state,
        detailScroll: 0,
        resultSelectedIndex: action.itemCount <= 0 ? 0 : action.boundary === "start" ? 0 : action.itemCount - 1,
      };
    case "move_detail":
      return { ...state, ...reduceSearchTwoPaneState(state, action) };
    case "detail_boundary":
      return { ...state, ...reduceSearchTwoPaneState(state, action) };
    case "set_query":
      return {
        ...state,
        query: action.query,
      };
    case "set_session": {
      const maxIndex = Math.max(0, action.session.total - 1);
      return {
        ...state,
        layout: action.showResults === false ? state.layout : "results",
        activePane: action.showResults === false ? state.activePane : "list",
        detailScroll: 0,
        layoutMode: "split",
        query: action.session.query,
        resultSelectedIndex: action.preserveSelection ? Math.min(state.resultSelectedIndex, maxIndex) : 0,
        session: action.session,
      };
    }
    case "clear_results":
      return {
        ...state,
        layout: "editor",
        activePane: "list",
        detailScroll: 0,
        layoutMode: "split",
        resultSelectedIndex: 0,
        session: null,
      };
    default:
      return state;
  }
}
