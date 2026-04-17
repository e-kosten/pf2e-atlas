import React from "react";
import { DatabaseSync } from "node:sqlite";

import { buildDerivedTagOntologyExplorerModel } from "./ontology-explorer-data.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  normalizeTerminalTwoPaneLayoutMode,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "./terminal-ui.js";
import {
  buildDerivedTagOntologyExplorerBreadcrumb,
  buildDerivedTagOntologyExplorerDetailLines,
  buildDerivedTagOntologyExplorerHelpLines,
  buildDerivedTagOntologyExplorerListLines,
  buildVisibleDerivedTagOntologyExplorerDetailLines,
  createDerivedTagOntologyExplorerUiState,
  drillIntoDerivedTagOntologyExplorer,
  getDerivedTagOntologyExplorerDetailMetrics,
  getDerivedTagOntologyExplorerSelection,
  isExactPrintableOntologyExplorerKey,
  jumpDerivedTagOntologyExplorerSelection,
  moveDerivedTagOntologyExplorerDetailScroll,
  moveDerivedTagOntologyExplorerDetailScrollToBoundary,
  moveDerivedTagOntologyExplorerSelection,
  moveDerivedTagOntologyExplorerSelectionToBoundary,
  normalizeDerivedTagOntologyExplorerState,
  popDerivedTagOntologyExplorerDepth,
  setDerivedTagOntologyExplorerFilter,
  type DerivedTagOntologyExplorerUiState,
} from "./ontology-explorer-ui.js";

type ExplorerAction =
  | { type: "toggle_focus" }
  | { type: "toggle_layout" }
  | { type: "leave_detail" }
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
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "set_pending_g"; pending: boolean };

function explorerReducer(
  model: ReturnType<typeof buildDerivedTagOntologyExplorerModel>,
  state: DerivedTagOntologyExplorerUiState,
  action: ExplorerAction,
): DerivedTagOntologyExplorerUiState {
  switch (action.type) {
    case "toggle_focus":
      return {
        ...state,
        activePane: toggleTerminalTwoPaneFocus(state.activePane),
        layoutMode: normalizeTerminalTwoPaneLayoutMode(state.layoutMode, toggleTerminalTwoPaneFocus(state.activePane)),
      };
    case "toggle_layout":
      return {
        ...state,
        layoutMode: toggleTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane),
      };
    case "leave_detail":
      return {
        ...state,
        activePane: "list",
        layoutMode: "split",
      };
    case "normalize":
      return {
        ...state,
        explorerState: normalizeDerivedTagOntologyExplorerState(model, state.explorerState),
      };
    case "set_search_mode":
      return {
        ...state,
        pendingListCommand: null,
        searchInput: action.searchInput ?? state.searchInput,
        searchMode: action.searchMode,
      };
    case "append_search": {
      const searchInput = state.searchInput + action.character;
      return {
        ...state,
        explorerState: setDerivedTagOntologyExplorerFilter(model, state.explorerState, searchInput),
        searchInput,
      };
    }
    case "backspace_search": {
      const searchInput = state.searchInput.slice(0, -1);
      return {
        ...state,
        explorerState: setDerivedTagOntologyExplorerFilter(model, state.explorerState, searchInput),
        searchInput,
      };
    }
    case "clear_search":
      return {
        ...state,
        explorerState: setDerivedTagOntologyExplorerFilter(model, state.explorerState, ""),
        pendingListCommand: null,
        searchInput: "",
      };
    case "move_selection":
      return {
        ...state,
        explorerState: moveDerivedTagOntologyExplorerSelection(model, state.explorerState, action.delta),
        pendingListCommand: null,
      };
    case "jump_selection":
      return {
        ...state,
        explorerState: jumpDerivedTagOntologyExplorerSelection(model, state.explorerState, action.delta),
        pendingListCommand: null,
      };
    case "selection_boundary":
      return {
        ...state,
        explorerState: moveDerivedTagOntologyExplorerSelectionToBoundary(model, state.explorerState, action.boundary),
        pendingListCommand: null,
      };
    case "drill_in":
      return {
        ...state,
        activePane: "list",
        explorerState: drillIntoDerivedTagOntologyExplorer(model, state.explorerState),
        layoutMode: "split",
        pendingListCommand: null,
        searchInput: "",
        searchMode: false,
      };
    case "pop_depth":
      return {
        ...state,
        activePane: "list",
        explorerState: popDerivedTagOntologyExplorerDepth(state.explorerState),
        layoutMode: "split",
        pendingListCommand: null,
        searchInput: "",
        searchMode: false,
      };
    case "move_detail":
      return {
        ...state,
        explorerState: moveDerivedTagOntologyExplorerDetailScroll(state.explorerState, action.delta, action.maxDetailScroll),
        pendingListCommand: null,
      };
    case "detail_boundary":
      return {
        ...state,
        explorerState: moveDerivedTagOntologyExplorerDetailScrollToBoundary(state.explorerState, action.boundary, action.maxDetailScroll),
        pendingListCommand: null,
      };
    case "set_pending_g":
      return {
        ...state,
        pendingListCommand: action.pending ? "g" : null,
      };
    default:
      return state;
  }
}

export function DerivedTagOntologyExplorerScreen({
  db,
  options = {},
  onExit,
}: {
  db: DatabaseSync;
  options?: { cacheKey?: string };
  onExit: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const model = React.useMemo(() => buildDerivedTagOntologyExplorerModel(db, options), [db, options]);
  const [state, dispatch] = React.useReducer(
    (current: DerivedTagOntologyExplorerUiState, action: ExplorerAction) => explorerReducer(model, current, action),
    model,
    createDerivedTagOntologyExplorerUiState,
  );

  React.useEffect(() => {
    dispatch({ type: "normalize" });
  }, [model]);

  const layoutMode = normalizeTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane);
  const normalizedExplorerState = normalizeDerivedTagOntologyExplorerState(model, state.explorerState);
  const selection = getDerivedTagOntologyExplorerSelection(model, normalizedExplorerState);
  const metrics = getDerivedTagOntologyExplorerDetailMetrics(model, normalizedExplorerState, layoutMode, size.width, size.height);
  const effectiveState = normalizedExplorerState.detailScroll > metrics.maxDetailScroll
    ? { ...normalizedExplorerState, detailScroll: metrics.maxDetailScroll }
    : normalizedExplorerState;
  const searchIndicator = state.searchMode
    ? ` | /${state.searchInput}`
    : effectiveState.filter
      ? ` | /${effectiveState.filter}`
      : "";

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const printable = key.ctrl || key.meta ? undefined : input.length === 1 ? input : undefined;

    if (state.pendingListCommand && printable !== "g") {
      dispatch({ type: "set_pending_g", pending: false });
    }

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
        title: "Ontology Search Help",
        body: buildDerivedTagOntologyExplorerHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }

    if (state.activePane === "detail") {
      if (normalized === "up" || normalized === "k") {
        dispatch({ type: "move_detail", delta: -1, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "down" || normalized === "j") {
        dispatch({ type: "move_detail", delta: 1, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "ctrl_d") {
        dispatch({ type: "move_detail", delta: metrics.detailJumpSize, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "ctrl_u") {
        dispatch({ type: "move_detail", delta: -metrics.detailJumpSize, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "page_down" || normalized === "space") {
        dispatch({ type: "move_detail", delta: metrics.detailPageSize, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "page_up" || normalized === "b") {
        dispatch({ type: "move_detail", delta: -metrics.detailPageSize, maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "home") {
        dispatch({ type: "detail_boundary", boundary: "start", maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "end") {
        dispatch({ type: "detail_boundary", boundary: "end", maxDetailScroll: metrics.maxDetailScroll });
        return;
      }
      if (normalized === "left" || normalized === "h" || normalized === "backspace" || normalized === "escape") {
        dispatch({ type: "leave_detail" });
      }
      return;
    }

    if (normalized === "up" || normalized === "k") {
      dispatch({ type: "move_selection", delta: -1 });
      return;
    }
    if (normalized === "down" || normalized === "j") {
      dispatch({ type: "move_selection", delta: 1 });
      return;
    }
    if (normalized === "ctrl_d") {
      dispatch({ type: "jump_selection", delta: metrics.selectionJumpSize });
      return;
    }
    if (normalized === "ctrl_u") {
      dispatch({ type: "jump_selection", delta: -metrics.selectionJumpSize });
      return;
    }
    if (normalized === "space" || normalized === "page_down") {
      dispatch({ type: "jump_selection", delta: metrics.detailPageSize });
      return;
    }
    if (normalized === "b" || normalized === "page_up") {
      dispatch({ type: "jump_selection", delta: -metrics.detailPageSize });
      return;
    }
    if (normalized === "home") {
      dispatch({ type: "selection_boundary", boundary: "start" });
      return;
    }
    if (normalized === "end") {
      dispatch({ type: "selection_boundary", boundary: "end" });
      return;
    }
    if (normalized === "g" && isExactPrintableOntologyExplorerKey(input, key, "g")) {
      if (state.pendingListCommand === "g") {
        dispatch({ type: "selection_boundary", boundary: "start" });
      } else {
        dispatch({ type: "set_pending_g", pending: true });
      }
      return;
    }
    if (normalized === "g" && isExactPrintableOntologyExplorerKey(input, key, "G")) {
      dispatch({ type: "selection_boundary", boundary: "end" });
      return;
    }
    if (normalized === "right" || normalized === "l" || normalized === "enter") {
      dispatch({ type: "drill_in" });
      return;
    }
    if (normalized === "left" || normalized === "h" || normalized === "backspace") {
      const nextState = popDerivedTagOntologyExplorerDepth(effectiveState);
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
      const nextState = popDerivedTagOntologyExplorerDepth(effectiveState);
      if (nextState.depth === effectiveState.depth) {
        onExit();
      } else {
        dispatch({ type: "pop_depth" });
      }
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
        title="Ontology Search"
        subtitle={`${buildDerivedTagOntologyExplorerBreadcrumb(selection, effectiveState)} | depth ${effectiveState.depth} | focused detail${searchIndicator}`}
        pane={{
          title: `[FOCUSED DETAIL] ${effectiveState.depth === "category"
            ? "Category Details"
            : effectiveState.depth === "family"
              ? "Family Details"
              : effectiveState.depth === "tag"
                ? "Tag Details"
                : "Record Details"}`,
          lines: buildVisibleDerivedTagOntologyExplorerDetailLines(model, effectiveState, layoutMode, size.width, metrics.bodyHeight),
          active: true,
        }}
        footer={[
          {
            text: state.searchMode
              ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
              : "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Left/backspace/esc list  / search  ? help  q back",
            tone: "dim",
          },
          {
            text: state.searchMode
              ? `Search /${state.searchInput}`
              : `detail focus | focused detail view | Detail scroll ${effectiveState.detailScroll}/${metrics.maxDetailScroll}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title="Ontology Search"
      subtitle={`${buildDerivedTagOntologyExplorerBreadcrumb(selection, effectiveState)} | depth ${effectiveState.depth}${searchIndicator}`}
      left={{
        title: `${state.activePane === "list" ? "[LIST] " : "List: "}${effectiveState.depth === "category"
          ? "Categories"
          : effectiveState.depth === "family"
            ? "Families"
            : effectiveState.depth === "tag"
              ? "Tags"
              : "Records"}`,
        lines: buildDerivedTagOntologyExplorerListLines(model, effectiveState, metrics.bodyHeight),
        active: state.activePane === "list",
      }}
      right={{
        title: `${state.activePane === "detail" ? "[DETAIL] " : "Detail: "}${effectiveState.depth === "category"
          ? "Category Details"
          : effectiveState.depth === "family"
            ? "Family Details"
            : effectiveState.depth === "tag"
              ? "Tag Details"
              : "Record Details"}`,
        lines: buildVisibleDerivedTagOntologyExplorerDetailLines(model, effectiveState, layoutMode, size.width, metrics.bodyHeight),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: state.searchMode
            ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
            : "Tab/w focus  z detail-only  Up/Down or j/k move-scroll  Ctrl+U/D jump  Space/b page  gg/G edge  Enter/right drill  Left/backspace up  / search  Esc back/clear  ? help  q back",
          tone: "dim",
        },
        {
          text: state.searchMode
            ? `Search /${state.searchInput}`
            : `${state.activePane} focus | ${layoutMode} layout | Detail scroll ${effectiveState.detailScroll}/${metrics.maxDetailScroll}`,
          tone: "accent",
        },
      ]}
      leftWidth={46}
    />
  );
}
