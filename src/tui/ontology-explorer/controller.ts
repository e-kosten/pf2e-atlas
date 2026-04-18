import React from "react";

import type { Key } from "ink";

import type {
  OntologyDomainModel,
  OntologyNode,
  OntologyNodeQuery,
} from "../../types.js";
import {
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  resolveDerivedTagTerminalListNavigationAction,
  sliceRenderedTerminalLines,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "../terminal-ui.js";
import { isBackNavigationKey } from "../keymap.js";
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
} from "../two-pane-state.js";
import {
  buildOntologyBrowserDetailLines,
  canDrillIntoOntologyNode,
  createOntologyBrowserUiState,
  drillIntoOntologyBrowser,
  getOntologyBrowserDetailTitle,
  getOntologyBrowserSelection,
  jumpOntologyBrowserSelection,
  moveOntologyBrowserDetailScroll,
  moveOntologyBrowserDetailScrollToBoundary,
  moveOntologyBrowserSelection,
  moveOntologyBrowserSelectionToBoundary,
  normalizeOntologyBrowserState,
  popOntologyBrowserDepth,
  setOntologyBrowserFilter,
  type OntologyBrowserSelection,
  type OntologyBrowserUiState,
} from "./ui.js";

export type OntologyExplorerAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "normalize" }
  | { type: "set_search_mode"; searchMode: boolean; searchInput?: string }
  | { type: "append_search"; character: string }
  | { type: "backspace_search" }
  | { type: "clear_search" }
  | { type: "move_selection"; delta: number }
  | { type: "jump_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "drill_in" }
  | { type: "pop_depth" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number };

export type OntologyExplorerControllerContext = {
  state: OntologyBrowserUiState;
  effectiveState: OntologyBrowserUiState["browserState"];
  selection: OntologyBrowserSelection;
  currentNode?: OntologyNode;
  currentNodeHasChildren: boolean;
  selectedQuery?: OntologyNodeQuery;
  breadcrumb: string;
  bodyHeight: number;
  detailWidth: number;
  detailLines: DerivedTagTerminalLine[];
  visibleDetailLines: DerivedTagTerminalLine[];
  detailTitle: string;
  layoutMode: ReturnType<typeof getDerivedTagTerminalTwoPaneLayoutMode>;
  maxDetailScroll: number;
  detailJumpSize: number;
  detailPageSize: number;
  selectionJumpSize: number;
  searchIndicator: string;
};

type OntologyExplorerKeyContext = OntologyExplorerControllerContext & {
  dispatch: React.Dispatch<OntologyExplorerAction>;
  input: string;
  key: Key;
  normalizedKey: string;
  printable?: string;
};

type OntologyExplorerControllerOptions = {
  model: OntologyDomainModel;
  onExit: () => void;
  onConfirm?: (context: OntologyExplorerKeyContext) => boolean;
  onKey?: (context: OntologyExplorerKeyContext) => boolean;
  onOpenQuery?: (query: OntologyNodeQuery) => void;
  escapeClearsFilterBeforeExit?: boolean;
  getDetailLines?: (context: {
    model: OntologyDomainModel;
    state: OntologyBrowserUiState;
    selection: OntologyBrowserSelection;
  }) => DerivedTagTerminalLine[];
  getDetailTitle?: (context: {
    model: OntologyDomainModel;
    state: OntologyBrowserUiState;
    selection: OntologyBrowserSelection;
  }) => string;
};

function reduceExplorerTwoPaneState(
  state: OntologyBrowserUiState,
  action: DerivedTagTerminalTwoPaneAction,
): Pick<OntologyBrowserUiState, "activePane" | "layoutMode" | "browserState"> {
  const next = reduceDerivedTagTerminalTwoPaneState({
    activePane: state.activePane,
    detailScroll: state.browserState.detailScroll,
    layoutMode: state.layoutMode,
  }, action);

  return {
    activePane: next.activePane,
    layoutMode: next.layoutMode,
    browserState: {
      ...state.browserState,
      detailScroll: next.detailScroll,
    },
  };
}

function ontologyExplorerReducer(
  model: OntologyDomainModel,
  state: OntologyBrowserUiState,
  action: OntologyExplorerAction,
): OntologyBrowserUiState {
  switch (action.type) {
    case "toggle_focus":
    case "toggle_layout":
    case "leave_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
      };
    case "normalize":
      return {
        ...state,
        browserState: normalizeOntologyBrowserState(model, state.browserState),
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
        browserState: setOntologyBrowserFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "backspace_search": {
      const searchInput = state.searchInput.slice(0, -1);
      return {
        ...state,
        browserState: setOntologyBrowserFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "clear_search":
      return {
        ...state,
        browserState: setOntologyBrowserFilter(model, state.browserState, ""),
        searchInput: "",
      };
    case "move_selection":
      return {
        ...state,
        browserState: moveOntologyBrowserSelection(model, state.browserState, action.delta),
      };
    case "jump_selection":
      return {
        ...state,
        browserState: jumpOntologyBrowserSelection(model, state.browserState, action.delta),
      };
    case "selection_boundary":
      return {
        ...state,
        browserState: moveOntologyBrowserSelectionToBoundary(model, state.browserState, action.boundary),
      };
    case "drill_in":
      return {
        ...state,
        activePane: "list",
        browserState: drillIntoOntologyBrowser(model, state.browserState),
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "pop_depth":
      return {
        ...state,
        activePane: "list",
        browserState: popOntologyBrowserDepth(state.browserState),
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "move_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveOntologyBrowserDetailScroll(state.browserState, action.delta, action.maxDetailScroll),
      };
    case "detail_boundary":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveOntologyBrowserDetailScrollToBoundary(state.browserState, action.boundary, action.maxDetailScroll),
      };
    default:
      return state;
  }
}

function buildOntologyBrowserBreadcrumb(
  model: OntologyDomainModel,
  selection: OntologyBrowserSelection,
): string {
  const segments = [model.label, ...selection.ancestors.map((node) => node.label)];
  if (selection.currentNode && selection.currentNodes.length > 0) {
    segments.push(selection.currentNode.label);
  }
  return segments.join(" > ");
}

export function useOntologyExplorerController(
  options: OntologyExplorerControllerOptions,
): OntologyExplorerControllerContext {
  const { model } = options;
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    (current: OntologyBrowserUiState, action: OntologyExplorerAction) =>
      ontologyExplorerReducer(model, current, action),
    model,
    createOntologyBrowserUiState,
  );

  React.useEffect(() => {
    dispatch({ type: "normalize" });
  }, [model]);

  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode({
    activePane: state.activePane,
    detailScroll: state.browserState.detailScroll,
    layoutMode: state.layoutMode,
  });
  const normalizedBrowserState = normalizeOntologyBrowserState(model, state.browserState);
  const selection = getOntologyBrowserSelection(model, normalizedBrowserState);
  const detailLines = options.getDetailLines?.({
    model,
    state: { ...state, browserState: normalizedBrowserState },
    selection,
  }) ?? buildOntologyBrowserDetailLines(model, normalizedBrowserState);
  const detailTitle = options.getDetailTitle?.({
    model,
    state: { ...state, browserState: normalizedBrowserState },
    selection,
  }) ?? getOntologyBrowserDetailTitle(model, normalizedBrowserState);
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const detailWidth = getTerminalTwoPaneDetailWidth(size.width, layoutMode, 46);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const effectiveState = normalizedBrowserState.detailScroll > maxDetailScroll
    ? { ...normalizedBrowserState, detailScroll: maxDetailScroll }
    : normalizedBrowserState;
  const selectedQuery = selection.currentNode?.query;
  const currentNodeHasChildren = canDrillIntoOntologyNode(selection.currentNode);
  const searchIndicator = state.searchMode
    ? ` | /${state.searchInput}`
    : effectiveState.filter
      ? ` | /${effectiveState.filter}`
      : "";

  const context: OntologyExplorerControllerContext = {
    state,
    effectiveState,
    selection,
    currentNode: selection.currentNode,
    currentNodeHasChildren,
    selectedQuery,
    breadcrumb: buildOntologyBrowserBreadcrumb(model, selection),
    bodyHeight,
    detailWidth,
    detailLines,
    visibleDetailLines: sliceRenderedTerminalLines(detailLines, detailWidth, effectiveState.detailScroll, bodyHeight),
    detailTitle,
    layoutMode,
    maxDetailScroll,
    detailJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    detailPageSize: Math.max(1, bodyHeight - 1),
    selectionJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    searchIndicator,
  };

  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const detailNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());

  useDerivedTagTerminalInput((input, key) => {
    const normalizedKey = getNormalizedKeyName(input, key);
    const printable = key.ctrl || key.meta ? undefined : input.length === 1 ? input : undefined;
    const listNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: context.detailPageSize,
      jumpSize: context.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    }, listNavigationStateRef.current);
    listNavigationStateRef.current = listNavigation.state;
    const detailNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: context.detailPageSize,
      jumpSize: context.detailJumpSize,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    }, detailNavigationStateRef.current);
    detailNavigationStateRef.current = detailNavigation.state;

    const keyContext: OntologyExplorerKeyContext = {
      ...context,
      dispatch,
      input,
      key,
      normalizedKey,
      printable,
    };

    if (normalizedKey === "ctrl_c") {
      options.onExit();
      return;
    }

    if (state.searchMode) {
      if (normalizedKey === "enter") {
        dispatch({ type: "set_search_mode", searchMode: false });
        return;
      }
      if (normalizedKey === "backspace") {
        dispatch({ type: "backspace_search" });
        return;
      }
      if (normalizedKey === "escape") {
        dispatch({ type: "clear_search" });
        dispatch({ type: "set_search_mode", searchMode: false, searchInput: "" });
        return;
      }
      if (printable) {
        dispatch({ type: "append_search", character: printable });
      }
      return;
    }

    if (options.onKey?.(keyContext)) {
      return;
    }

    if (normalizedKey === "q") {
      options.onExit();
      return;
    }
    if (normalizedKey === "tab" || normalizedKey === "shift_tab" || normalizedKey === "w") {
      dispatch({ type: "toggle_focus" });
      return;
    }
    if (normalizedKey === "z") {
      dispatch({ type: "toggle_layout" });
      return;
    }

    if (state.activePane === "detail") {
      if (detailNavigation.action?.kind === "move") {
        dispatch({ type: "move_detail", delta: detailNavigation.action.delta, maxDetailScroll: context.maxDetailScroll });
        return;
      }
      if (detailNavigation.action?.kind === "boundary") {
        dispatch({ type: "detail_boundary", boundary: detailNavigation.action.boundary, maxDetailScroll: context.maxDetailScroll });
        return;
      }
      if (detailNavigation.action?.kind === "cancel") {
        dispatch({ type: "leave_detail" });
        return;
      }
      return;
    }

    if (listNavigation.action?.kind === "move") {
      const isJump = Math.abs(listNavigation.action.delta) > 1;
      dispatch(isJump
        ? { type: "jump_selection", delta: listNavigation.action.delta }
        : { type: "move_selection", delta: listNavigation.action.delta });
      return;
    }
    if (listNavigation.action?.kind === "boundary") {
      dispatch({ type: "selection_boundary", boundary: listNavigation.action.boundary });
      return;
    }
    if (listNavigation.action?.kind === "confirm") {
      if (options.onConfirm?.(keyContext)) {
        return;
      }
      if (context.currentNodeHasChildren) {
        dispatch({ type: "drill_in" });
      } else {
        dispatch({ type: "toggle_focus" });
      }
      return;
    }
    if (isBackNavigationKey(normalizedKey)) {
      const nextState = popOntologyBrowserDepth(context.effectiveState);
      if (nextState.depth === context.effectiveState.depth) {
        options.onExit();
      } else {
        dispatch({ type: "pop_depth" });
      }
      return;
    }
    if (normalizedKey === "escape") {
      if ((options.escapeClearsFilterBeforeExit ?? true) && context.effectiveState.filter) {
        dispatch({ type: "clear_search" });
        return;
      }
      const nextState = popOntologyBrowserDepth(context.effectiveState);
      if (nextState.depth === context.effectiveState.depth) {
        options.onExit();
      } else {
        dispatch({ type: "pop_depth" });
      }
      return;
    }
    if (normalizedKey === "o" && context.selectedQuery && options.onOpenQuery) {
      options.onOpenQuery(context.selectedQuery);
      return;
    }
    if (normalizedKey === "slash") {
      dispatch({
        type: "set_search_mode",
        searchMode: true,
        searchInput: context.effectiveState.filter,
      });
    }
  });

  return context;
}
