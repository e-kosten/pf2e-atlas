import React from "react";

import type { OntologyDomainModel, OntologyNode, OntologyNodeQuery } from "../../domain/index.js";
import {
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
} from "../framework/rendering.js";
import { useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalInputEvent, DerivedTagTerminalLine } from "../framework/types.js";
import type { TerminalInteractionAction } from "../interaction-bindings.js";
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
} from "../two-pane-state.js";
import { useOntologyExplorerInteractionRouter, type OntologyExplorerInteractionRoute } from "./interactions.js";
import {
  cloneOntologyBrowserSnapshot,
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
  type OntologyBrowserSnapshot,
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

export type OntologyExplorerKeyContext = OntologyExplorerControllerContext & {
  event: DerivedTagTerminalInputEvent;
};

type OntologyExplorerControllerOptions = {
  model: OntologyDomainModel;
  initialSnapshot?: OntologyBrowserSnapshot;
  onExit: () => void;
  onConfirm?: (context: OntologyExplorerKeyContext) => boolean;
  getInteractionActions?: (context: OntologyExplorerControllerContext) => TerminalInteractionAction[];
  onAction?: (action: TerminalInteractionAction, context: OntologyExplorerKeyContext) => boolean;
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void;
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
  nestedDetailBackAction?: "leave_detail" | "pop_depth";
};

export type OntologyExplorerBackNavigationOutcome = "leave_detail" | "pop_depth" | "exit";

export function resolveOntologyExplorerBackNavigation(
  context: Pick<OntologyExplorerControllerContext, "state" | "effectiveState">,
  options: Pick<OntologyExplorerControllerOptions, "nestedDetailBackAction"> = {},
): OntologyExplorerBackNavigationOutcome {
  if (context.state.activePane === "detail") {
    return context.effectiveState.depth > 0 && options.nestedDetailBackAction === "pop_depth"
      ? "pop_depth"
      : "leave_detail";
  }

  const nextState = popOntologyBrowserDepth(context.effectiveState);
  return nextState.depth === context.effectiveState.depth ? "exit" : "pop_depth";
}

export function createOntologyBrowserSnapshot(
  context: Pick<OntologyExplorerControllerContext, "state" | "effectiveState" | "layoutMode">,
): OntologyBrowserSnapshot {
  return cloneOntologyBrowserSnapshot({
    activePane: context.state.activePane,
    browserState: context.effectiveState,
    layoutMode: context.layoutMode,
    searchInput: context.state.searchInput,
    searchMode: context.state.searchMode,
  });
}

function reduceExplorerTwoPaneState(
  state: OntologyBrowserUiState,
  action: DerivedTagTerminalTwoPaneAction,
): Pick<OntologyBrowserUiState, "activePane" | "layoutMode" | "browserState"> {
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
        browserState: moveOntologyBrowserDetailScrollToBoundary(
          state.browserState,
          action.boundary,
          action.maxDetailScroll,
        ),
      };
    default:
      return state;
  }
}

function buildOntologyBrowserBreadcrumb(model: OntologyDomainModel, selection: OntologyBrowserSelection): string {
  const segments = [model.label, ...selection.ancestors.map((node) => node.label)];
  if (selection.currentNode && selection.currentNodes.length > 0) {
    segments.push(selection.currentNode.label);
  }
  return segments.join(" > ");
}

export function useOntologyExplorerController(
  options: OntologyExplorerControllerOptions,
): OntologyExplorerControllerContext {
  const { initialSnapshot, model } = options;
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    (current: OntologyBrowserUiState, action: OntologyExplorerAction) =>
      ontologyExplorerReducer(model, current, action),
    { model, initialSnapshot },
    ({ model: initialModel, initialSnapshot: snapshot }) => createOntologyBrowserUiState(initialModel, snapshot),
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
  const detailLines =
    options.getDetailLines?.({
      model,
      state: { ...state, browserState: normalizedBrowserState },
      selection,
    }) ?? buildOntologyBrowserDetailLines(model, normalizedBrowserState);
  const detailTitle =
    options.getDetailTitle?.({
      model,
      state: { ...state, browserState: normalizedBrowserState },
      selection,
    }) ?? getOntologyBrowserDetailTitle(model, normalizedBrowserState);
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );
  const detailWidth = getTerminalTwoPaneDetailWidth(size.width, layoutMode, 46);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const effectiveState =
    normalizedBrowserState.detailScroll > maxDetailScroll
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

  const interactionActions = options.getInteractionActions?.(context) ?? [];
  const handleRoute = React.useCallback(
    (route: OntologyExplorerInteractionRoute) => {
      const {
        detailNavigationAction,
        event,
        interactionAction,
        listNavigationAction,
        searchModeAction,
        textEntryIntent,
      } = route;
      const keyContext: OntologyExplorerKeyContext = {
        ...context,
        event,
      };

      if (context.state.searchMode) {
        if (textEntryIntent?.kind === "submit") {
          dispatch({ type: "set_search_mode", searchMode: false });
          return;
        }
        if (textEntryIntent?.kind === "deleteBackward") {
          dispatch({ type: "backspace_search" });
          return;
        }
        if (searchModeAction?.id === "cancel") {
          dispatch({ type: "clear_search" });
          dispatch({ type: "set_search_mode", searchMode: false, searchInput: "" });
          return;
        }
        if (textEntryIntent?.kind === "append") {
          dispatch({ type: "append_search", character: textEntryIntent.text });
        }
        return;
      }

      if (context.state.activePane === "detail") {
        if (detailNavigationAction?.kind === "move") {
          dispatch({
            type: "move_detail",
            delta: detailNavigationAction.delta,
            maxDetailScroll: context.maxDetailScroll,
          });
          return;
        }
        if (detailNavigationAction?.kind === "boundary") {
          dispatch({
            type: "detail_boundary",
            boundary: detailNavigationAction.boundary,
            maxDetailScroll: context.maxDetailScroll,
          });
          return;
        }
        if (interactionAction && options.onAction?.(interactionAction, keyContext)) {
          return;
        }
        if (interactionAction?.id === "quit") {
          options.onExit();
          return;
        }
        if (interactionAction?.id === "focus") {
          dispatch({ type: "toggle_focus" });
          return;
        }
        if (interactionAction?.id === "layout") {
          dispatch({ type: "toggle_layout" });
          return;
        }
        if (interactionAction?.id === "back" || interactionAction?.id === "return") {
          const backNavigation = resolveOntologyExplorerBackNavigation(context, options);
          if (backNavigation === "pop_depth") {
            dispatch({ type: "pop_depth" });
            return;
          }
          if (backNavigation === "leave_detail") {
            dispatch({ type: "leave_detail" });
            return;
          }
          options.onExit();
          return;
        }
        return;
      }

      if (listNavigationAction?.kind === "move") {
        const isJump = Math.abs(listNavigationAction.delta) > 1;
        dispatch(
          isJump
            ? { type: "jump_selection", delta: listNavigationAction.delta }
            : { type: "move_selection", delta: listNavigationAction.delta },
        );
        return;
      }
      if (listNavigationAction?.kind === "boundary") {
        dispatch({ type: "selection_boundary", boundary: listNavigationAction.boundary });
        return;
      }
      if (listNavigationAction?.kind === "confirm") {
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

      if (interactionAction?.id === "cancel") {
        if ((options.escapeClearsFilterBeforeExit ?? true) && context.effectiveState.filter) {
          dispatch({ type: "clear_search" });
          return;
        }
        return;
      }
      if (interactionAction && options.onAction?.(interactionAction, keyContext)) {
        return;
      }
      if (interactionAction?.id === "focus") {
        dispatch({ type: "toggle_focus" });
        return;
      }
      if (interactionAction?.id === "layout") {
        dispatch({ type: "toggle_layout" });
        return;
      }
      if (interactionAction?.id === "back" || interactionAction?.id === "return") {
        const backNavigation = resolveOntologyExplorerBackNavigation(context, options);
        if (backNavigation === "pop_depth") {
          dispatch({ type: "pop_depth" });
        } else if (backNavigation === "leave_detail") {
          dispatch({ type: "leave_detail" });
        } else {
          options.onExit();
        }
        return;
      }
      if (interactionAction?.id === "quit") {
        options.onExit();
        return;
      }
      if (interactionAction?.id === "search") {
        dispatch({
          type: "set_search_mode",
          searchMode: true,
          searchInput: context.effectiveState.filter,
        });
      }
    },
    [context, dispatch, options],
  );

  useOntologyExplorerInteractionRouter({
    context: {
      searchMode: state.searchMode,
      activePane: state.activePane,
      detailPageSize: context.detailPageSize,
      selectionJumpSize: context.selectionJumpSize,
      detailJumpSize: context.detailJumpSize,
      interactionActions,
    },
    onRoute: handleRoute,
  });

  return context;
}
