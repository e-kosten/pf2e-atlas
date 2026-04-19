import React from "react";

import type { OntologyDomainModel, OntologyNode, OntologySelectionState } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  type DerivedTagTerminalLine,
} from "../terminal-ui.js";
import {
  getTerminalInteractionCycleDirection,
} from "../interaction-bindings.js";
import {
  showTerminalReturnDialog,
  useTerminalInteractionContextAdapters,
} from "../interaction-context-adapters.js";
import { useOntologyExplorerController } from "./controller.js";
import {
  buildFacetPickerHelpLines,
  buildFacetPickerScreenModel,
  getFacetPickerInteractionActions,
} from "./screen-models.js";
import { buildOntologyBrowserListRows } from "./ui.js";

export type OntologyPickerFieldSelection = {
  any: string[];
  all: string[];
  exclude: string[];
};

export type OntologyPickerSelectionMap = Record<string, OntologyPickerFieldSelection>;

function cloneSelectionMap(selection: OntologyPickerSelectionMap): OntologyPickerSelectionMap {
  return Object.fromEntries(
    Object.entries(selection).map(([field, values]) => [
      field,
      {
        any: [...values.any],
        all: [...values.all],
        exclude: [...values.exclude],
      },
    ]),
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
  return stateOrder[(((currentIndex + direction) % stateOrder.length) + stateOrder.length) % stateOrder.length];
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

  return Object.fromEntries([...fields].map((field) => [field, { any: [], all: [], exclude: [] }]));
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

  const nextState = cycleSelectionState(
    getNodeSelectionState(node, selections),
    node.selection.allowedStates,
    direction,
  );
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
  const lines: DerivedTagTerminalLine[] = [{ text: "" }, { text: "Current selection", tone: "section" }];
  if (focusedNode?.selection) {
    lines.push({ text: `${focusedNode.selection.fieldLabel}: ${focusedNode.label}` });
    lines.push({
      text: `Focused policy: ${policyStateLabel(getNodeSelectionState(focusedNode, selections))}`,
      tone: "accent",
    });
  } else {
    lines.push({ text: "Focused node is not selectable.", tone: "dim" });
  }

  const selectionEntries = Object.entries(selections).filter(
    ([, selection]) => selection.any.length > 0 || selection.all.length > 0 || selection.exclude.length > 0,
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

export function OntologyPickerScreen({
  model,
  initialSelections,
  onApply,
}: {
  model: OntologyDomainModel;
  initialSelections?: OntologyPickerSelectionMap;
  onApply: (selection: OntologyPickerSelectionMap) => void;
}): React.JSX.Element {
  const adapters = useTerminalInteractionContextAdapters();
  const [selections, setSelections] = React.useState<OntologyPickerSelectionMap>(() => {
    const emptySelections = createEmptySelectionMap(model);
    return initialSelections ? { ...emptySelections, ...cloneSelectionMap(initialSelections) } : emptySelections;
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

  const updateSelections = React.useCallback(
    (update: (current: OntologyPickerSelectionMap) => OntologyPickerSelectionMap) => {
      const next = update(selectionsRef.current);
      selectionsRef.current = next;
      setSelections(next);
    },
    [],
  );

  const returnWithSelections = React.useCallback(() => {
    onApply(selectionsRef.current);
  }, [onApply]);

  const controller = useOntologyExplorerController({
    model,
    onExit: returnWithSelections,
    getDetailLines: ({ selection }) => buildPickerDetailLines(selections, selection.currentNode),
    getDetailTitle: () => "Detail",
    getInteractionActions: getFacetPickerInteractionActions,
    onConfirm: ({ currentNode, event }) => {
      const cycleDirection = getTerminalInteractionCycleDirection(event, { id: "cycle" });
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
        void showTerminalReturnDialog(adapters, "Facet Picker Help", buildFacetPickerHelpLines(keyContext));
        return true;
      }
      if (action.id !== "cycle" || !keyContext.currentNode?.selection) {
        return false;
      }
      updateSelections((current) => toggleNodeSelection(keyContext.currentNode, current, 1));
      return true;
    },
  });

  const screen = buildFacetPickerScreenModel({
    model,
    controller,
    leftLines: buildPickerListLines(model, controller.bodyHeight, controller, selections),
    focusedPolicyLabel: policyStateLabel(getNodeSelectionState(controller.currentNode, selections)),
  });

  if (screen.kind === "detail-only") {
    return <TerminalPaneScreen {...screen.props} />;
  }

  return <TerminalTwoPaneScreen {...screen.props} />;
}
