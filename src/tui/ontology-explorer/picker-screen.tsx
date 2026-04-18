import React from "react";

import type { OntologyDomainModel, OntologyNode, OntologySelectionState } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  useDerivedTagTerminalApp,
  type DerivedTagTerminalLine,
} from "../terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import { getCycleDirection } from "../keymap.js";
import { useOntologyExplorerController } from "./controller.js";
import { buildOntologyBrowserListRows } from "./ui.js";

export type OntologyPickerFieldSelection = {
  any: string[];
  all: string[];
  exclude: string[];
};

export type OntologyPickerSelectionMap = Record<string, OntologyPickerFieldSelection>;

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
  direction: 1 | -1 = 1,
): OntologySelectionState | undefined {
  const stateOrder: Array<OntologySelectionState | undefined> = [undefined, ...allowedStates];
  const currentIndex = stateOrder.findIndex((state) => state === currentState);
  return stateOrder[((currentIndex + direction) % stateOrder.length + stateOrder.length) % stateOrder.length];
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
  direction: 1 | -1 = 1,
): OntologyPickerSelectionMap {
  if (!node?.selection) {
    return selections;
  }
  const next = cloneSelectionMap(selections);
  const fieldSelection = next[node.selection.field] ?? { any: [], all: [], exclude: [] };
  fieldSelection.any = fieldSelection.any.filter((value) => value !== node.selection!.value);
  fieldSelection.all = fieldSelection.all.filter((value) => value !== node.selection!.value);
  fieldSelection.exclude = fieldSelection.exclude.filter((value) => value !== node.selection!.value);

  const nextState = cycleSelectionState(getNodeSelectionState(node, selections), node.selection.allowedStates, direction);
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
  lines.push({ text: "Current filters", tone: "section" });
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
  selections: OntologyPickerSelectionMap,
  currentNode: OntologyNode | undefined,
): DerivedTagTerminalLine[] {
  return [
    ...(currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }]),
    ...buildSelectionSummaryLines(selections, currentNode),
  ];
}

function buildPickerListLines(
  model: OntologyDomainModel,
  bodyHeight: number,
  state: ReturnType<typeof useOntologyExplorerController>,
  selections: OntologyPickerSelectionMap,
): DerivedTagTerminalLine[] {
  return buildOntologyBrowserListRows(model, state.effectiveState, bodyHeight, (node, isSelected) => {
    const stateLabel = node.selection ? `[${policyStateLabel(getNodeSelectionState(node, selections))}] ` : "";
    return {
      text: `${stateLabel}${node.listLabel ?? node.label}`,
      tone: isSelected ? "selected" : "default",
      noWrap: true,
    };
  }).map((row) => row.line);
}

function buildFacetPickerFooterText(
  controller: ReturnType<typeof useOntologyExplorerController>,
): string {
  return formatTerminalInteractionFooter(getFacetPickerInteractionActions(controller));
}

function getFacetPickerInteractionActions(
  controller: Pick<ReturnType<typeof useOntologyExplorerController>, "layoutMode" | "state">,
): TerminalInteractionAction[] {
  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "cycle" },
      { id: "layout", label: "split-view" },
      { id: "back", label: "values" },
      { id: "search" },
      { id: "help" },
      { id: "quit", label: "return" },
    ];
  }

  if (controller.state.activePane === "list") {
    return [
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "cycle" },
      { id: "focus", label: "pane" },
      { id: "layout", label: "detail-only" },
      { id: "back", label: "up" },
      { id: "search" },
      { id: "help" },
      { id: "quit", label: "return" },
    ];
  }

  return [
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    { id: "focus", label: "pane" },
    { id: "layout", label: "detail-only" },
    { id: "back", label: "values" },
    { id: "search" },
    { id: "help" },
    { id: "quit", label: "return" },
  ];
}

function buildFacetPickerHelpLines(
  controller: Pick<ReturnType<typeof useOntologyExplorerController>, "layoutMode" | "state">,
): DerivedTagTerminalLine[] {
  const actionActions = getFacetPickerInteractionActions(controller)
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText: action.id === "cycle"
        ? "cycle the focused policy through off, any, all, and exclude"
        : action.id === "focus"
          ? "switch focus between values and detail"
          : action.id === "layout"
            ? "toggle split and detail-only layouts"
            : action.id === "back"
              ? "move up a level or leave the active pane"
              : action.id === "search"
                ? "start live filtering"
                : action.id === "help"
                  ? "show this help"
                  : "apply the current facet state and return",
      label: action.id === "focus" ? "toggle pane" : action.label,
    }));

  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        { id: controller.state.activePane === "list" && controller.layoutMode !== "detail-only" ? "move" : "scroll", helpText: "move through the active pane" },
        { id: "jump", helpText: "jump through the active pane" },
        { id: "page", helpText: "page through the active pane" },
        { id: "edge", helpText: "jump to the start or end of the active pane" },
      ],
    },
    {
      title: "Actions",
      actions: actionActions,
    },
  ]);
}

export function OntologyPickerScreen({
  model,
  initialSelections,
  onApply,
}: {
  model: OntologyDomainModel;
  initialSelections?: OntologyPickerSelectionMap;
  onApply: (selection: OntologyPickerSelectionMap) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const [selections, setSelections] = React.useState<OntologyPickerSelectionMap>(() => {
    const emptySelections = createEmptySelectionMap(model);
    return initialSelections
      ? { ...emptySelections, ...cloneSelectionMap(initialSelections) }
      : emptySelections;
  });
  const selectionsRef = React.useRef(selections);

  React.useEffect(() => {
    const emptySelections = createEmptySelectionMap(model);
    const nextSelections = initialSelections
      ? { ...emptySelections, ...cloneSelectionMap(initialSelections) }
      : emptySelections;
    selectionsRef.current = nextSelections;
    setSelections(nextSelections);
  }, [initialSelections, model]);

  const updateSelections = React.useCallback((update: (current: OntologyPickerSelectionMap) => OntologyPickerSelectionMap) => {
    const next = update(selectionsRef.current);
    selectionsRef.current = next;
    setSelections(next);
  }, []);

  const returnWithSelections = React.useCallback(() => {
    onApply(selectionsRef.current);
  }, [onApply]);

  const controller = useOntologyExplorerController({
    model,
    onExit: returnWithSelections,
    getDetailLines: ({ selection }) => buildPickerDetailLines(selections, selection.currentNode),
    getDetailTitle: () => "Detail",
    getInteractionActions: getFacetPickerInteractionActions,
    onConfirm: ({ currentNode, normalizedKey }) => {
      const cycleDirection = getCycleDirection(normalizedKey);
      if (!cycleDirection) {
        return false;
      }
      if (!currentNode?.selection) {
        return false;
      }
      updateSelections((current) => toggleNodeSelection(currentNode, current, cycleDirection));
      return true;
    },
    onAction: (action, keyContext) => {
      if (action.id === "help") {
        void terminal.showDialog({
          title: "Facet Picker Help",
          body: buildFacetPickerHelpLines(keyContext),
          footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
        });
        return true;
      }
      if (action.id !== "cycle" || !keyContext.currentNode?.selection) {
        return false;
      }
      updateSelections((current) => toggleNodeSelection(keyContext.currentNode, current, 1));
      return true;
    },
  });

  if (controller.layoutMode === "detail-only") {
    return (
      <TerminalPaneScreen
        title="Facet Picker"
        subtitle={`${model.label} | ${controller.breadcrumb}${controller.searchIndicator}`}
        pane={{
          title: "[FOCUSED DETAIL] Detail",
          lines: controller.visibleDetailLines,
          active: true,
        }}
        footer={[
          {
            text: controller.state.searchMode
              ? TERMINAL_LIVE_FILTER_FOOTER
              : buildFacetPickerFooterText(controller),
            tone: "dim",
          },
          {
            text: controller.state.searchMode
              ? `Search /${controller.state.searchInput}`
              : `detail focus | focused detail view | Policy ${policyStateLabel(getNodeSelectionState(controller.currentNode, selections))} | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title="Facet Picker"
      subtitle={`${model.label} | ${controller.breadcrumb}${controller.searchIndicator}`}
      left={{
        title: controller.state.activePane === "list" ? "[VALUES]" : "Values",
        lines: buildPickerListLines(model, controller.bodyHeight, controller, selections),
        active: controller.state.activePane === "list",
      }}
      right={{
        title: controller.state.activePane === "detail" ? "[DETAIL]" : "Detail",
        lines: controller.visibleDetailLines,
        active: controller.state.activePane === "detail",
      }}
      footer={[
        {
          text: controller.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
            : buildFacetPickerFooterText(controller),
          tone: "dim",
        },
        {
          text: controller.state.searchMode
            ? `Search /${controller.state.searchInput}`
            : `Focused: ${controller.currentNode?.label ?? "(none)"} | Policy ${policyStateLabel(getNodeSelectionState(controller.currentNode, selections))} | ${controller.state.activePane} focus | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
          tone: "accent",
        },
      ]}
      leftWidth={48}
    />
  );
}
