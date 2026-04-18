import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
} from "../terminal-ui.js";
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
} from "../two-pane-state.js";
import {
  buildOntologyBrowserBreadcrumb,
  canDrillIntoOntologyNode,
  buildOntologyBrowserDetailLines,
  buildOntologyBrowserHelpLines,
  buildOntologyBrowserListLines,
  buildVisibleOntologyBrowserDetailLines,
  createOntologyBrowserUiState,
  drillIntoOntologyBrowser,
  getOntologyBrowserDetailMetrics,
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
  type OntologyBrowserUiState,
} from "./ui.js";

type ExplorerAction =
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

function explorerReducer(
  model: OntologyDomainModel,
  state: OntologyBrowserUiState,
  action: ExplorerAction,
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

export function OntologyBrowserScreen({
  model,
  onExit,
  onOpenQuery,
}: {
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    (current: OntologyBrowserUiState, action: ExplorerAction) => explorerReducer(model, current, action),
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
  const selectedQuery = selection.currentNode?.query;
  const metrics = getOntologyBrowserDetailMetrics(model, normalizedBrowserState, layoutMode, size.width, size.height);
  const effectiveState = normalizedBrowserState.detailScroll > metrics.maxDetailScroll
    ? { ...normalizedBrowserState, detailScroll: metrics.maxDetailScroll }
    : normalizedBrowserState;
  const breadcrumb = buildOntologyBrowserBreadcrumb(model, effectiveState);
  const currentNodeHasChildren = canDrillIntoOntologyNode(selection.currentNode);
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const detailNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const searchIndicator = state.searchMode
    ? ` | /${state.searchInput}`
    : effectiveState.filter
      ? ` | /${effectiveState.filter}`
      : "";

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const printable = key.ctrl || key.meta ? undefined : input.length === 1 ? input : undefined;
    const listNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: metrics.detailPageSize,
      jumpSize: metrics.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
      includeVimHorizontalConfirmKeys: true,
    }, listNavigationStateRef.current);
    listNavigationStateRef.current = listNavigation.state;
    const detailNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: metrics.detailPageSize,
      jumpSize: metrics.detailJumpSize,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
      includeVimHorizontalCancelKeys: true,
    }, detailNavigationStateRef.current);
    detailNavigationStateRef.current = detailNavigation.state;

    if (normalized === "ctrl_c") {
      onExit();
      return;
    }

    if (state.searchMode) {
      if (normalized === "enter") {
        dispatch({ type: "set_search_mode", searchMode: false });
        return;
      }
      if (normalized === "backspace") {
        dispatch({ type: "backspace_search" });
        return;
      }
      if (normalized === "escape") {
        dispatch({ type: "clear_search" });
        dispatch({ type: "set_search_mode", searchMode: false, searchInput: "" });
        return;
      }
      if (printable) {
        dispatch({ type: "append_search", character: printable });
      }
      return;
    }

    if (normalized === "q") {
      onExit();
      return;
    }

    if (normalized === "tab" || normalized === "shift_tab" || normalized === "w") {
      dispatch({ type: "toggle_focus" });
      return;
    }
    if (normalized === "z") {
      dispatch({ type: "toggle_layout" });
      return;
    }
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Ontology Browser Help",
        body: buildOntologyBrowserHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }

    if (state.activePane === "detail") {
      if (detailNavigation.action?.kind === "move") {
        dispatch({ type: "move_detail", delta: detailNavigation.action.delta, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (detailNavigation.action?.kind === "boundary") {
        dispatch({ type: "detail_boundary", boundary: detailNavigation.action.boundary, maxDetailScroll: metrics.maxDetailScroll });
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
      if (currentNodeHasChildren) {
        dispatch({ type: "drill_in" });
      } else {
        dispatch({ type: "toggle_focus" });
      }
      return;
    }
    if (normalized === "left" || normalized === "h" || normalized === "backspace") {
      const nextState = popOntologyBrowserDepth(effectiveState);
      if (nextState.depth === effectiveState.depth) {
        onExit();
      } else {
        dispatch({ type: "pop_depth" });
      }
      return;
    }
    if (normalized === "escape") {
      if (effectiveState.filter) {
        dispatch({ type: "clear_search" });
        return;
      }
      const nextState = popOntologyBrowserDepth(effectiveState);
      if (nextState.depth === effectiveState.depth) {
        onExit();
      } else {
        dispatch({ type: "pop_depth" });
      }
      return;
    }
    if (normalized === "o" && selectedQuery && onOpenQuery) {
      onOpenQuery(selectedQuery);
      return;
    }
    if (normalized === "slash") {
      dispatch({
        type: "set_search_mode",
        searchMode: true,
        searchInput: effectiveState.filter,
      });
    }
  });

  if (layoutMode === "detail-only") {
    return (
      <TerminalPaneScreen
        title={model.label}
        subtitle={`${breadcrumb} | depth ${effectiveState.depth} | focused detail${searchIndicator}`}
        pane={{
          title: `[FOCUSED DETAIL] ${getOntologyBrowserDetailTitle(model, effectiveState)}`,
          lines: buildVisibleOntologyBrowserDetailLines(model, effectiveState, layoutMode, size.width, metrics.bodyHeight),
          active: true,
        }}
        footer={[
          {
            text: state.searchMode
              ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
              : "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Left/backspace/esc list  o open query  / search  ? help  q back",
            tone: "dim",
          },
          {
            text: state.searchMode
              ? `Search /${state.searchInput}`
              : `detail focus | focused detail view | Detail scroll ${effectiveState.detailScroll}/${metrics.maxDetailScroll}${selectedQuery ? " | query ready" : ""}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title={model.label}
      subtitle={`${breadcrumb} | depth ${effectiveState.depth}${searchIndicator}`}
      left={{
        title: state.activePane === "list" ? "[LIST] Ontology Entries" : "Ontology Entries",
        lines: buildOntologyBrowserListLines(model, effectiveState, metrics.bodyHeight),
        active: state.activePane === "list",
      }}
      right={{
        title: state.activePane === "detail"
          ? `[DETAIL] ${getOntologyBrowserDetailTitle(model, effectiveState)}`
          : getOntologyBrowserDetailTitle(model, effectiveState),
        lines: buildVisibleOntologyBrowserDetailLines(
          model,
          effectiveState,
          layoutMode,
          size.width,
          metrics.bodyHeight,
        ),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: state.searchMode
            ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
            : "Tab/w focus  z detail-only  Up/Down or j/k move-scroll  Ctrl+U/D jump  Space/b page  gg/G edge  Enter/right open  o open query  Left/backspace up  / search  Esc back/clear  ? help  q back",
          tone: "dim",
        },
        {
          text: state.searchMode
            ? `Search /${state.searchInput}`
            : `${state.activePane} focus | ${layoutMode} layout | Detail scroll ${effectiveState.detailScroll}/${metrics.maxDetailScroll}${selectedQuery ? " | query ready" : ""}`,
          tone: "accent",
        },
      ]}
      leftWidth={46}
    />
  );
}
