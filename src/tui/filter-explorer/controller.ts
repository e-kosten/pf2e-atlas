import React from "react";

import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { getTerminalInteractionCycleDirection } from "../interaction-bindings.js";
import {
  createOntologyBrowserSnapshot,
  useOntologyExplorerController,
  type OntologyExplorerControllerContext,
  type OntologyExplorerKeyContext,
} from "../ontology-explorer/controller.js";
import {
  cloneFilterExplorerSelectionMap,
  getFilterExplorerTargetState,
  normalizeFilterExplorerSelectionMap,
  toggleFilterExplorerTargetSelection,
} from "./compose-state.js";
import {
  buildFilterExplorerCommandEntries,
  buildFilterExplorerComposeDetailLines,
  buildFilterExplorerHelpLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type {
  FilterExplorerComposeMode,
  FilterExplorerControllerContext as FilterExplorerContext,
  FilterExplorerOptions,
  FilterExplorerSelectionMap,
} from "./types.js";

type OntologyResultReaderLaunchQuery = OntologyNodeQuery & {
  openInResults?: boolean;
};

function markQueryToOpenInResults(query: OntologyNodeQuery): OntologyNodeQuery {
  return {
    ...query,
    openInResults: true,
  } as OntologyResultReaderLaunchQuery;
}

function resolveScreenTitle(options: FilterExplorerOptions): string {
  return options.title ?? (options.mode.kind === "compose" ? "Filter Explorer" : options.model.label);
}

function useComposeSelectionState(mode: FilterExplorerComposeMode | null): [
  FilterExplorerSelectionMap,
  (updater: (current: FilterExplorerSelectionMap) => FilterExplorerSelectionMap) => void,
] {
  const isControlled = mode?.selection !== undefined;
  const [internalSelection, setInternalSelection] = React.useState<FilterExplorerSelectionMap>(() =>
    normalizeFilterExplorerSelectionMap(mode?.selection ?? mode?.initialSelection),
  );

  React.useEffect(() => {
    setInternalSelection(normalizeFilterExplorerSelectionMap(mode?.selection ?? mode?.initialSelection));
  }, [mode?.initialSelection, mode?.selection]);

  const currentSelection = React.useMemo(
    () => normalizeFilterExplorerSelectionMap(mode?.selection ?? internalSelection),
    [internalSelection, mode?.selection],
  );

  const updateSelection = React.useCallback(
    (updater: (current: FilterExplorerSelectionMap) => FilterExplorerSelectionMap) => {
      if (!mode) {
        return;
      }

      const next = normalizeFilterExplorerSelectionMap(updater(currentSelection));
      if (!isControlled) {
        setInternalSelection(next);
      }
      mode.onSelectionChange?.(cloneFilterExplorerSelectionMap(next));
    },
    [currentSelection, isControlled, mode],
  );

  return [currentSelection, updateSelection];
}

function createFilterExplorerContext(args: {
  options: FilterExplorerOptions;
  ontology: OntologyExplorerControllerContext;
  selection: FilterExplorerSelectionMap;
}): FilterExplorerContext {
  const composeMode = args.options.mode.kind === "compose" ? args.options.mode : null;
  const selectedTarget = composeMode?.resolveSelectionTarget(args.ontology.currentNode);
  return {
    model: args.options.model,
    mode: args.options.mode,
    screenTitle: resolveScreenTitle(args.options),
    ontology: args.ontology,
    selection: args.selection,
    selectedTarget,
    selectedPolicyState: getFilterExplorerTargetState(selectedTarget, args.selection),
    selectedQuery: args.ontology.selectedQuery,
  };
}

function applyComposeCycleSelection(
  composeMode: FilterExplorerComposeMode,
  keyContext: Pick<OntologyExplorerKeyContext, "currentNode" | "event">,
  selection: FilterExplorerSelectionMap,
  updateSelection: (updater: (current: FilterExplorerSelectionMap) => FilterExplorerSelectionMap) => void,
): boolean {
  const cycleDirection = getTerminalInteractionCycleDirection(keyContext.event, { id: "cycle" });
  if (!cycleDirection) {
    return false;
  }
  const target = composeMode.resolveSelectionTarget(keyContext.currentNode);
  if (!target) {
    return false;
  }
  updateSelection((current) => toggleFilterExplorerTargetSelection(target, current, cycleDirection));
  return true;
}

function openInspectQuery(
  options: FilterExplorerOptions,
  keyContext: Pick<OntologyExplorerKeyContext, "selectedQuery"> & Parameters<typeof createOntologyBrowserSnapshot>[0],
  query: OntologyNodeQuery | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !options.mode.onOpenQuery || !query) {
    return false;
  }

  const nextQuery =
    query.kind === "listRecords" &&
    options.model.id !== "derivedTags" &&
    options.mode.openListRecordQueriesInResults !== false
      ? markQueryToOpenInResults(query)
      : query;
  options.mode.onOpenQuery(nextQuery, createOntologyBrowserSnapshot(keyContext));
  return true;
}

function shouldExitAtRootDepth(options: FilterExplorerOptions, keyContext: OntologyExplorerKeyContext): boolean {
  return (
    options.mode.kind === "compose" &&
    options.exitAtRootDepth === true &&
    keyContext.state.activePane === "list" &&
    keyContext.effectiveState.depth === (options.rootDepth ?? 0)
  );
}

export function useFilterExplorerController(options: FilterExplorerOptions): FilterExplorerContext {
  const adapters = useTerminalInteractionContextAdapters();
  const composeMode = options.mode.kind === "compose" ? options.mode : null;
  const [selection, updateSelection] = useComposeSelectionState(composeMode);

  const ontology = useOntologyExplorerController({
    initialSnapshot: options.initialSnapshot,
    model: options.model,
    onExit: options.onExit,
    getInteractionActions: (controller) => getFilterExplorerInteractionActions(options.mode, controller),
    getDetailLines:
      composeMode
        ? ({ selection: ontologySelection }) =>
            buildFilterExplorerComposeDetailLines({
              mode: composeMode,
              selection,
              currentNodeLabel: ontologySelection.currentNode?.label,
              selectedTarget: composeMode.resolveSelectionTarget(ontologySelection.currentNode),
              selectedPolicyState: getFilterExplorerTargetState(
                composeMode.resolveSelectionTarget(ontologySelection.currentNode),
                selection,
              ),
              baseDetailLines: ontologySelection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }],
            })
        : undefined,
    getDetailTitle:
      composeMode
        ? () => composeMode.detailTitle ?? "Detail"
        : undefined,
    onConfirm: (keyContext) => {
      if (composeMode) {
        return applyComposeCycleSelection(composeMode, keyContext, selection, updateSelection);
      }

      if (
        keyContext.currentNode?.query?.kind === "listRecords" &&
        options.model.id !== "derivedTags" &&
        keyContext.currentNode?.kind !== "record"
      ) {
        return openInspectQuery(options, keyContext, keyContext.currentNode.query);
      }

      return !keyContext.currentNodeHasChildren && openInspectQuery(options, keyContext, keyContext.currentNode?.query);
    },
    onAction: (action, keyContext) => {
      const context = createFilterExplorerContext({
        options,
        ontology: keyContext,
        selection,
      });

      if (action.id === "back" && shouldExitAtRootDepth(options, keyContext)) {
        options.onExit();
        return true;
      }

      if (action.id === "cycle" && composeMode) {
        return applyComposeCycleSelection(composeMode, keyContext, selection, updateSelection);
      }

      if (action.id === "help") {
        void showTerminalReturnDialog(adapters, `${context.screenTitle} Help`, buildFilterExplorerHelpLines(context));
        return true;
      }

      if (action.id !== "commands" || options.mode.kind !== "inspect-and-open") {
        return false;
      }

      const commandEntries = buildFilterExplorerCommandEntries(context);
      if (commandEntries.length === 0) {
        return true;
      }

      void adapters
        .promptCommandPalette({
          title: `${context.screenTitle} Commands`,
          prompt: "Filter explorer commands",
          entries: commandEntries,
        })
        .then((selected) => {
          if (selected === "openQuery") {
            openInspectQuery(options, keyContext, keyContext.selectedQuery);
          }
        });
      return true;
    },
  });

  return createFilterExplorerContext({
    options,
    ontology,
    selection,
  });
}
