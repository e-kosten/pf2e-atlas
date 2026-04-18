import React from "react";

import type { OntologyDomainModel, OntologyNode, OntologySelectionState } from "../../types.js";
import {
  TerminalTwoPaneScreen,
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
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
} from "../two-pane-state.js";
import {
  buildOntologyBrowserBreadcrumb,
  buildOntologyBrowserListRows,
  canDrillIntoOntologyNode,
  createOntologyBrowserUiState,
  drillIntoOntologyBrowser,
  getOntologyBrowserSelection,
  jumpOntologyBrowserSelection,
  moveOntologyBrowserSelection,
  moveOntologyBrowserSelectionToBoundary,
  normalizeOntologyBrowserState,
  popOntologyBrowserDepth,
  setOntologyBrowserFilter,
  type OntologyBrowserUiState,
} from "./ui.js";

export type OntologyPickerFieldSelection = {
  any: string[];
  all: string[];
  exclude: string[];
};

export type OntologyPickerSelectionMap = Record<string, OntologyPickerFieldSelection>;

type PickerAction =
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
  | { type: "cycle_selection" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number };

function cloneSelectionMap(selection: OntologyPickerSelectionMap): OntologyPickerSelectionMap {
  return Object.fromEntries(
    Object.entries(selection).map(([field, values]) => [field, {
      any: [...values.any],
      all: [...values.all],
      exclude: [...values.exclude],
    }]),
  );
}

function policyStateLabel(state: OntologySelectionState | undefined): string {
  return state ?? "off";
}

function getNodeSelectionState(
  node: OntologyNode | undefined,
  selections: OntologyPickerSelectionMap,
): OntologySelectionState | undefined {
  if (!node?.selection) {
    return undefined;
  }
  const fieldSelection = selections[node.selection.field];
  if (!fieldSelection) {
    return undefined;
  }
  if (fieldSelection.any.includes(node.selection.value)) {
    return "any";
  }
  if (fieldSelection.all.includes(node.selection.value)) {
    return "all";
  }
  if (fieldSelection.exclude.includes(node.selection.value)) {
    return "exclude";
  }
  return undefined;
}

function cycleSelectionState(
  currentState: OntologySelectionState | undefined,
  allowedStates: OntologySelectionState[],
): OntologySelectionState | undefined {
  const stateOrder: Array<OntologySelectionState | undefined> = [undefined, ...allowedStates];
  const currentIndex = stateOrder.findIndex((state) => state === currentState);
  return stateOrder[(currentIndex + 1) % stateOrder.length];
}

function createEmptySelectionMap(model: OntologyDomainModel): OntologyPickerSelectionMap {
  const fields = new Set<string>();
  const visit = (node: OntologyNode): void => {
    if (node.selection) {
      fields.add(node.selection.field);
    }
    node.children?.forEach(visit);
  };
  model.rootNodes.forEach(visit);

  return Object.fromEntries(
    [...fields].map((field) => [field, { any: [], all: [], exclude: [] }]),
  );
}

function toggleNodeSelection(
  node: OntologyNode | undefined,
  selections: OntologyPickerSelectionMap,
): OntologyPickerSelectionMap {
  if (!node?.selection) {
    return selections;
  }
  const next = cloneSelectionMap(selections);
  const fieldSelection = next[node.selection.field] ?? { any: [], all: [], exclude: [] };
  fieldSelection.any = fieldSelection.any.filter((value) => value !== node.selection!.value);
  fieldSelection.all = fieldSelection.all.filter((value) => value !== node.selection!.value);
  fieldSelection.exclude = fieldSelection.exclude.filter((value) => value !== node.selection!.value);

  const nextState = cycleSelectionState(getNodeSelectionState(node, selections), node.selection.allowedStates);
  if (nextState) {
    fieldSelection[nextState].push(node.selection.value);
    fieldSelection[nextState].sort((left, right) => left.localeCompare(right));
  }
  next[node.selection.field] = fieldSelection;
  return next;
}

function buildSelectionSummaryLines(
  selections: OntologyPickerSelectionMap,
  focusedNode: OntologyNode | undefined,
): DerivedTagTerminalLine[] {
  const lines: DerivedTagTerminalLine[] = [
    { text: "" },
    { text: "Current selection", tone: "section" },
  ];
  if (focusedNode?.selection) {
    lines.push({ text: `${focusedNode.selection.fieldLabel}: ${focusedNode.label}` });
    lines.push({ text: `Focused policy: ${policyStateLabel(getNodeSelectionState(focusedNode, selections))}`, tone: "accent" });
  } else {
    lines.push({ text: "Focused node is not selectable.", tone: "dim" });
  }

  const selectionEntries = Object.entries(selections).filter(([, selection]) =>
    selection.any.length > 0 || selection.all.length > 0 || selection.exclude.length > 0,
  );
  lines.push({ text: "" });
  lines.push({ text: "Applied draft preview", tone: "section" });
  if (selectionEntries.length === 0) {
    lines.push({ text: "No facet values selected yet.", tone: "dim" });
    return lines;
  }

  for (const [field, selection] of selectionEntries) {
    const parts = [
      selection.any.length > 0 ? `any=${selection.any.join(", ")}` : null,
      selection.all.length > 0 ? `all=${selection.all.join(", ")}` : null,
      selection.exclude.length > 0 ? `exclude=${selection.exclude.join(", ")}` : null,
    ].filter((value): value is string => Boolean(value));
    lines.push({ text: `${field}: ${parts.join(" | ")}` });
  }
  return lines;
}

function buildPickerDetailLines(
  model: OntologyDomainModel,
  state: OntologyBrowserUiState,
  selections: OntologyPickerSelectionMap,
): DerivedTagTerminalLine[] {
  const selection = getOntologyBrowserSelection(model, state.browserState);
  return [
    ...(selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }]),
    ...buildSelectionSummaryLines(selections, selection.currentNode),
  ];
}

function buildPickerListLines(
  model: OntologyDomainModel,
  state: OntologyBrowserUiState,
  bodyHeight: number,
  selections: OntologyPickerSelectionMap,
): DerivedTagTerminalLine[] {
  return buildOntologyBrowserListRows(model, state.browserState, bodyHeight, (node, isSelected) => {
    const stateLabel = node.selection ? `[${policyStateLabel(getNodeSelectionState(node, selections))}] ` : "";
    return {
      text: `${stateLabel}${node.listLabel ?? node.label}`,
      tone: isSelected ? "selected" : "default",
      noWrap: true,
    };
  }).map((row) => row.line);
}

function reducePickerTwoPaneState(
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

function pickerReducer(
  model: OntologyDomainModel,
  state: OntologyBrowserUiState,
  action: PickerAction,
): OntologyBrowserUiState {
  switch (action.type) {
    case "toggle_focus":
    case "toggle_layout":
    case "leave_detail":
      return {
        ...state,
        ...reducePickerTwoPaneState(state, action),
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
        ...reducePickerTwoPaneState(state, action),
        browserState: {
          ...state.browserState,
          detailScroll: Math.max(0, Math.min(action.maxDetailScroll, state.browserState.detailScroll + action.delta)),
        },
      };
    case "detail_boundary":
      return {
        ...state,
        ...reducePickerTwoPaneState(state, action),
        browserState: {
          ...state.browserState,
          detailScroll: action.boundary === "start" ? 0 : action.maxDetailScroll,
        },
      };
    default:
      return state;
  }
}

export function OntologyPickerScreen({
  model,
  initialSelections,
  onApply,
  onCancel,
}: {
  model: OntologyDomainModel;
  initialSelections?: OntologyPickerSelectionMap;
  onApply: (selection: OntologyPickerSelectionMap) => void;
  onCancel: () => void;
}): React.JSX.Element {
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    (current: OntologyBrowserUiState, action: PickerAction) => pickerReducer(model, current, action),
    model,
    createOntologyBrowserUiState,
  );
  const [selections, setSelections] = React.useState<OntologyPickerSelectionMap>(() => {
    const emptySelections = createEmptySelectionMap(model);
    return initialSelections
      ? { ...emptySelections, ...cloneSelectionMap(initialSelections) }
      : emptySelections;
  });

  React.useEffect(() => {
    dispatch({ type: "normalize" });
    const emptySelections = createEmptySelectionMap(model);
    setSelections(initialSelections ? { ...emptySelections, ...cloneSelectionMap(initialSelections) } : emptySelections);
  }, [initialSelections, model]);

  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode({
    activePane: state.activePane,
    detailScroll: state.browserState.detailScroll,
    layoutMode: state.layoutMode,
  });
  const normalizedBrowserState = normalizeOntologyBrowserState(model, state.browserState);
  const selection = getOntologyBrowserSelection(model, normalizedBrowserState);
  const metrics = {
    bodyHeight: Math.max(1, getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    })),
    detailWidth: getTerminalTwoPaneDetailWidth(size.width, layoutMode, 48),
  };
  const detailLines = buildPickerDetailLines(model, { ...state, browserState: normalizedBrowserState }, selections);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, metrics.detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - metrics.bodyHeight);
  const effectiveState = normalizedBrowserState.detailScroll > maxDetailScroll
    ? { ...normalizedBrowserState, detailScroll: maxDetailScroll }
    : normalizedBrowserState;
  const breadcrumb = buildOntologyBrowserBreadcrumb(model, effectiveState);
  const listNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const detailNavigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const currentNode = selection.currentNode;

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const printable = key.ctrl || key.meta ? undefined : input.length === 1 ? input : undefined;
    const listNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: Math.max(1, metrics.bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(metrics.bodyHeight / 2)),
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
      includeVimHorizontalConfirmKeys: true,
    }, listNavigationStateRef.current);
    listNavigationStateRef.current = listNavigation.state;
    const detailNavigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: Math.max(1, metrics.bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(metrics.bodyHeight / 2)),
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
      includeVimHorizontalCancelKeys: true,
    }, detailNavigationStateRef.current);
    detailNavigationStateRef.current = detailNavigation.state;

    if (normalized === "ctrl_c" || normalized === "q") {
      onCancel();
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

    if (normalized === "a") {
      onApply(selections);
      return;
    }
    if (normalized === "slash") {
      dispatch({ type: "set_search_mode", searchMode: true, searchInput: state.searchInput });
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
    if ((normalized === "space" || normalized === "enter") && currentNode?.selection) {
      setSelections((current) => toggleNodeSelection(currentNode, current));
      return;
    }

    if (state.activePane === "detail") {
      if (detailNavigation.action?.kind === "move") {
        dispatch({ type: "move_detail", delta: detailNavigation.action.delta, maxDetailScroll });
        return;
      }
      if (detailNavigation.action?.kind === "boundary") {
        dispatch({ type: "detail_boundary", boundary: detailNavigation.action.boundary, maxDetailScroll });
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
      if (currentNode?.selection) {
        setSelections((current) => toggleNodeSelection(currentNode, current));
      } else if (canDrillIntoOntologyNode(currentNode)) {
        dispatch({ type: "drill_in" });
      } else {
        dispatch({ type: "toggle_focus" });
      }
      return;
    }
    if (normalized === "left" || normalized === "h" || normalized === "backspace" || normalized === "escape") {
      const nextState = popOntologyBrowserDepth(effectiveState);
      if (nextState.depth === effectiveState.depth) {
        onCancel();
        return;
      }
      dispatch({ type: "pop_depth" });
      return;
    }
  });

  return (
    <TerminalTwoPaneScreen
      title="Facet Picker"
      subtitle={`${model.label} | ${breadcrumb}`}
      left={{
        title: state.activePane === "list" ? "[VALUES]" : "Values",
        lines: buildPickerListLines(model, { ...state, browserState: effectiveState }, metrics.bodyHeight, selections),
        active: state.activePane === "list",
      }}
      right={{
        title: state.activePane === "detail" ? "[DETAIL]" : "Detail",
        lines: sliceRenderedTerminalLines(detailLines, metrics.detailWidth, effectiveState.detailScroll, metrics.bodyHeight),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: "Up/Down move  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  Enter/Space cycle  Enter drills non-selectable branches  / filter  Tab focus  z layout  a apply  Esc/backspace cancel",
          tone: "dim",
        },
        {
          text: `Focused: ${currentNode?.label ?? "(none)"} | Policy ${policyStateLabel(getNodeSelectionState(currentNode, selections))} | Detail scroll ${effectiveState.detailScroll}/${maxDetailScroll}`,
          tone: "accent",
        },
      ]}
      leftWidth={48}
    />
  );
}
