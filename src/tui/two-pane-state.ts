import {
  normalizeTerminalTwoPaneLayoutMode,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
  type DerivedTagTerminalTwoPaneFocus,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "./terminal-ui.js";

export type DerivedTagTerminalTwoPaneState = {
  activePane: DerivedTagTerminalTwoPaneFocus;
  detailScroll: number;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
};

export type DerivedTagTerminalTwoPaneAction =
  | { type: "toggle_focus" }
  | { type: "toggle_layout" }
  | { type: "leave_detail" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number };

export function createDerivedTagTerminalTwoPaneState(): DerivedTagTerminalTwoPaneState {
  return {
    activePane: "list",
    detailScroll: 0,
    layoutMode: "split",
  };
}

export function reduceDerivedTagTerminalTwoPaneState<TState extends DerivedTagTerminalTwoPaneState>(
  state: TState,
  action: DerivedTagTerminalTwoPaneAction,
): TState {
  switch (action.type) {
    case "toggle_focus": {
      const activePane = toggleTerminalTwoPaneFocus(state.activePane);
      return {
        ...state,
        activePane,
        layoutMode: normalizeTerminalTwoPaneLayoutMode(state.layoutMode, activePane),
      };
    }
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
    default:
      return state;
  }
}

export function getDerivedTagTerminalTwoPaneLayoutMode(
  state: DerivedTagTerminalTwoPaneState,
): DerivedTagTerminalTwoPaneLayoutMode {
  return normalizeTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane);
}
