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
  cloneFilterExplorerComposeDraft,
  cloneFilterExplorerSelectionMap,
  getFilterExplorerScalarClause,
  getFilterExplorerTargetState,
  isFilterExplorerScalarTarget,
  normalizeFilterExplorerComposeDraft,
  setFilterExplorerScalarClause,
  toggleFilterExplorerTargetSelection,
} from "./compose-state.js";
import {
  buildFilterExplorerCommandEntries,
  buildFilterExplorerComposeDetailLines,
  buildFilterExplorerHelpLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type {
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext as FilterExplorerContext,
  FilterExplorerOptions,
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
  FilterExplorerComposeDraft,
  (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
] {
  const isControlled = mode?.draft !== undefined;
  const [internalDraft, setInternalDraft] = React.useState<FilterExplorerComposeDraft>(() =>
    normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft, mode?.selection ?? mode?.initialSelection),
  );

  React.useEffect(() => {
    setInternalDraft(
      normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft, mode?.selection ?? mode?.initialSelection),
    );
  }, [mode?.draft, mode?.initialDraft, mode?.initialSelection, mode?.selection]);

  const currentDraft = React.useMemo(
    () => normalizeFilterExplorerComposeDraft(mode?.draft ?? internalDraft, mode?.selection),
    [internalDraft, mode?.draft, mode?.selection],
  );
  const updateSelection = React.useCallback(
    (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => {
      if (!mode) {
        return;
      }

      const next = normalizeFilterExplorerComposeDraft(updater(currentDraft));
      if (!isControlled) {
        setInternalDraft(next);
      }
      mode.onDraftChange?.(cloneFilterExplorerComposeDraft(next));
      mode.onSelectionChange?.(cloneFilterExplorerSelectionMap(next.selection));
    },
    [currentDraft, isControlled, mode],
  );

  return [currentDraft, updateSelection];
}

function createFilterExplorerContext(args: {
  options: FilterExplorerOptions;
  ontology: OntologyExplorerControllerContext;
  draft: FilterExplorerComposeDraft;
}): FilterExplorerContext {
  const composeMode = args.options.mode.kind === "compose" ? args.options.mode : null;
  const selectedTarget = composeMode?.resolveSelectionTarget(args.ontology.currentNode);
  return {
    model: args.options.model,
    mode: args.options.mode,
    screenTitle: resolveScreenTitle(args.options),
    ontology: args.ontology,
    draft: args.draft,
    selection: args.draft.selection,
    selectedTarget,
    selectedPolicyState: getFilterExplorerTargetState(selectedTarget, args.draft.selection),
    selectedScalarClause: getFilterExplorerScalarClause(selectedTarget, args.draft),
    selectedQuery: args.ontology.selectedQuery,
  };
}

function applyComposeCycleSelection(
  composeMode: FilterExplorerComposeMode,
  keyContext: Pick<OntologyExplorerKeyContext, "currentNode" | "event">,
  draft: FilterExplorerComposeDraft,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
): boolean {
  const cycleDirection = getTerminalInteractionCycleDirection(keyContext.event, { id: "cycle" });
  if (!cycleDirection) {
    return false;
  }
  const target = composeMode.resolveSelectionTarget(keyContext.currentNode);
  if (!target || target.kind === "scalar") {
    return false;
  }
  updateDraft((current) => ({
    ...current,
    selection: toggleFilterExplorerTargetSelection(target, current.selection, cycleDirection),
  }));
  return true;
}

function openComposeScalarEditor(
  composeMode: FilterExplorerComposeMode,
  target: ReturnType<FilterExplorerComposeMode["resolveSelectionTarget"]>,
  draft: FilterExplorerComposeDraft,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
): boolean {
  if (!isFilterExplorerScalarTarget(target) || !composeMode.onEditScalarTarget) {
    return false;
  }

  void Promise.resolve(
    composeMode.onEditScalarTarget({
      target,
      currentClause: getFilterExplorerScalarClause(target, draft),
      draft: cloneFilterExplorerComposeDraft(draft),
    }),
  ).then((nextClause) => {
    if (nextClause === undefined) {
      return;
    }
    updateDraft((current) => setFilterExplorerScalarClause(target, nextClause, current));
  });
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
  const [draft, updateDraft] = useComposeSelectionState(composeMode);

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
              draft,
              currentNodeLabel: ontologySelection.currentNode?.label,
              selectedTarget: composeMode.resolveSelectionTarget(ontologySelection.currentNode),
              selectedPolicyState: getFilterExplorerTargetState(
                composeMode.resolveSelectionTarget(ontologySelection.currentNode),
                draft.selection,
              ),
              selectedScalarClause: getFilterExplorerScalarClause(
                composeMode.resolveSelectionTarget(ontologySelection.currentNode),
                draft,
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
        const target = composeMode.resolveSelectionTarget(keyContext.currentNode);
        return (
          openComposeScalarEditor(composeMode, target, draft, updateDraft) ||
          applyComposeCycleSelection(composeMode, keyContext, draft, updateDraft)
        );
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
        draft,
      });

      if (action.id === "back" && shouldExitAtRootDepth(options, keyContext)) {
        options.onExit();
        return true;
      }

      if (action.id === "cycle" && composeMode) {
        const target = composeMode.resolveSelectionTarget(keyContext.currentNode);
        return (
          openComposeScalarEditor(composeMode, target, draft, updateDraft) ||
          applyComposeCycleSelection(composeMode, keyContext, draft, updateDraft)
        );
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
    draft,
  });
}
