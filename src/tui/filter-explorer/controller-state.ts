import { reduceDerivedTagTerminalTwoPaneState, type DerivedTagTerminalTwoPaneAction } from "../two-pane-state.js";
import {
  cloneFilterExplorerBrowserSnapshot,
  drillIntoFilterExplorerBrowser,
  jumpFilterExplorerSelection,
  moveFilterExplorerDetailScroll,
  moveFilterExplorerDetailScrollToBoundary,
  moveFilterExplorerSelection,
  moveFilterExplorerSelectionToBoundary,
  normalizeFilterExplorerBrowserState,
  popFilterExplorerDepth,
  setFilterExplorerFilter,
} from "./browser.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerBrowserSnapshot,
  FilterExplorerBrowserUiState,
  FilterExplorerModel,
} from "./types.js";

export type FilterExplorerAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "normalize" }
  | { type: "set_search_mode"; searchMode: boolean; searchInput?: string }
  | { type: "append_search"; character: string }
  | { type: "backspace_search" }
  | { type: "clear_search" }
  | { type: "set_child_loading"; nodeId?: string; expectedNodeId?: string }
  | { type: "move_selection"; delta: number }
  | { type: "jump_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "drill_in"; nodeId?: string }
  | { type: "pop_depth" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number };

export type FilterExplorerBackNavigationOutcome = "leave_detail" | "pop_depth" | "exit";

export function createFilterExplorerBrowserSnapshot(
  context: Pick<FilterExplorerBrowserContext, "state" | "effectiveState" | "layoutMode">,
): FilterExplorerBrowserSnapshot {
  return cloneFilterExplorerBrowserSnapshot({
    activePane: context.state.activePane,
    browserState: context.effectiveState,
    layoutMode: context.layoutMode,
    searchInput: context.state.searchInput,
    searchMode: context.state.searchMode,
  });
}

export function resolveFilterExplorerBackNavigation(
  context: Pick<FilterExplorerBrowserContext, "state" | "effectiveState">,
): FilterExplorerBackNavigationOutcome {
  if (context.state.activePane === "detail") {
    return "leave_detail";
  }

  const nextState = popFilterExplorerDepth(context.effectiveState);
  return nextState.depth === context.effectiveState.depth ? "exit" : "pop_depth";
}

function reduceExplorerTwoPaneState(
  state: FilterExplorerBrowserUiState,
  action: DerivedTagTerminalTwoPaneAction,
): Pick<FilterExplorerBrowserUiState, "activePane" | "layoutMode" | "browserState"> {
  const next = reduceDerivedTagTerminalTwoPaneState(
    {
      activePane: state.activePane,
      detailScroll: state.browserState.detailScroll,
      layoutMode: state.layoutMode,
    },
    action,
  );

  return {
    activePane: next.activePane,
    layoutMode: next.layoutMode,
    browserState: {
      ...state.browserState,
      detailScroll: next.detailScroll,
    },
  };
}

export function filterExplorerReducer(
  model: FilterExplorerModel,
  state: FilterExplorerBrowserUiState,
  action: FilterExplorerAction,
): FilterExplorerBrowserUiState {
  switch (action.type) {
    case "toggle_focus":
    case "set_focus":
    case "toggle_layout":
    case "leave_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
      };
    case "normalize":
      return {
        ...state,
        browserState: normalizeFilterExplorerBrowserState(model, state.browserState),
      };
    case "set_search_mode":
      return {
        ...state,
        searchInput: action.searchInput ?? state.searchInput,
        searchMode: action.searchMode,
      };
    case "append_search": {
      const searchInput = state.searchInput + action.character;
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "backspace_search": {
      const searchInput = state.searchInput.slice(0, -1);
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "clear_search":
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, ""),
        searchInput: "",
      };
    case "set_child_loading":
      if (action.expectedNodeId && state.loadingChildNodeId !== action.expectedNodeId) {
        return state;
      }
      return {
        ...state,
        loadingChildNodeId: action.nodeId,
      };
    case "move_selection":
      return {
        ...state,
        browserState: moveFilterExplorerSelection(model, state.browserState, action.delta),
      };
    case "jump_selection":
      return {
        ...state,
        browserState: jumpFilterExplorerSelection(model, state.browserState, action.delta),
      };
    case "selection_boundary":
      return {
        ...state,
        browserState: moveFilterExplorerSelectionToBoundary(model, state.browserState, action.boundary),
      };
    case "drill_in":
      return {
        ...state,
        activePane: "list",
        browserState: drillIntoFilterExplorerBrowser(model, state.browserState, action.nodeId),
        loadingChildNodeId: undefined,
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "pop_depth":
      return {
        ...state,
        activePane: "list",
        browserState: popFilterExplorerDepth(state.browserState),
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "move_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveFilterExplorerDetailScroll(state.browserState, action.delta, action.maxDetailScroll),
      };
    case "detail_boundary":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveFilterExplorerDetailScrollToBoundary(
          state.browserState,
          action.boundary,
          action.maxDetailScroll,
        ),
      };
    default:
      return state;
  }
}
