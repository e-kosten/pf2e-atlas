import React from "react";

import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
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
  FilterExplorerBrowserContext,
  FilterExplorerBrowserSnapshot,
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext as FilterExplorerContext,
  FilterExplorerInspectAndOpenMode,
  FilterExplorerInspectResult,
  FilterExplorerModel,
  FilterExplorerNode,
  FilterExplorerOptions,
  FilterExplorerQueryTarget,
} from "./types.js";

function toOntologyExplorerModel(model: FilterExplorerModel): OntologyDomainModel {
  return model as OntologyDomainModel;
}

function toFilterExplorerNode(node: OntologyNode | undefined): FilterExplorerNode | undefined {
  return node as FilterExplorerNode | undefined;
}

function toFilterExplorerBrowserSnapshot(snapshot: ReturnType<typeof createOntologyBrowserSnapshot>): FilterExplorerBrowserSnapshot {
  return snapshot as FilterExplorerBrowserSnapshot;
}

function toFilterExplorerBrowserContext(context: OntologyExplorerControllerContext): FilterExplorerBrowserContext {
  return {
    ...context,
    selection: {
      ancestors: context.selection.ancestors.map((node) => node as FilterExplorerNode),
      currentNodes: context.selection.currentNodes.map((node) => node as FilterExplorerNode),
      currentNode: toFilterExplorerNode(context.selection.currentNode),
      currentParent: toFilterExplorerNode(context.selection.currentParent),
    },
    currentNode: toFilterExplorerNode(context.currentNode),
  };
}

function markQueryToOpenInResults(query: FilterExplorerQueryTarget): FilterExplorerQueryTarget {
  return {
    ...query,
    openInResults: true,
  };
}

function buildInspectResult(
  model: FilterExplorerModel,
  mode: FilterExplorerInspectAndOpenMode,
  node: FilterExplorerNode | undefined,
): FilterExplorerInspectResult | undefined {
  if (!node?.query) {
    return undefined;
  }

  const query =
    node.query.kind === "listRecords" && model.id !== "derivedTags" && mode.openListRecordQueriesInResults !== false
      ? markQueryToOpenInResults(node.query)
      : node.query;

  return {
    node,
    query,
    target: mode.resolveInspectTarget?.(node),
    openIntent: query.openInResults ? "results" : "browse",
  };
}

function openInspectResult(
  options: FilterExplorerOptions,
  keyContext: Parameters<typeof createOntologyBrowserSnapshot>[0],
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = toFilterExplorerBrowserSnapshot(createOntologyBrowserSnapshot(keyContext));
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(result, snapshot);
    return true;
  }
  if (options.mode.onOpenQuery) {
    options.mode.onOpenQuery(result.query, snapshot);
    return true;
  }
  return false;
}

function openInspectQuery(
  options: FilterExplorerOptions,
  keyContext: Parameters<typeof createOntologyBrowserSnapshot>[0],
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = toFilterExplorerBrowserSnapshot(createOntologyBrowserSnapshot(keyContext));
  const query = { ...result.query, openInResults: false };
  if (options.mode.onOpenQuery) {
    options.mode.onOpenQuery(query, snapshot);
    return true;
  }
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(
      {
        ...result,
        query,
        openIntent: query.kind === "listRecords" ? "browse" : result.openIntent,
      },
      snapshot,
    );
    return true;
  }
  return false;
}

function getNodeInspectResult(
  options: FilterExplorerOptions,
  node: OntologyNode | undefined,
): FilterExplorerInspectResult | undefined {
  return options.mode.kind === "inspect-and-open"
    ? buildInspectResult(options.model, options.mode, toFilterExplorerNode(node))
    : undefined;
}

function shouldOpenImmediateInspectResult(
  model: FilterExplorerModel,
  node: OntologyNode | undefined,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  return Boolean(
    result &&
      result.query.kind === "listRecords" &&
      model.id !== "derivedTags" &&
      node?.kind !== "record",
  );
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
  const currentNode = toFilterExplorerNode(args.ontology.currentNode);
  const selectedTarget = composeMode?.resolveSelectionTarget(currentNode);

  return {
    model: args.options.model,
    mode: args.options.mode,
    screenTitle: resolveScreenTitle(args.options),
    browser: toFilterExplorerBrowserContext(args.ontology),
    draft: args.draft,
    selection: args.draft.selection,
    selectedTarget,
    selectedPolicyState: getFilterExplorerTargetState(selectedTarget, args.draft.selection),
    selectedScalarClause: getFilterExplorerScalarClause(selectedTarget, args.draft),
    selectedInspectResult:
      args.options.mode.kind === "inspect-and-open"
        ? buildInspectResult(args.options.model, args.options.mode, currentNode)
        : undefined,
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

  const target = composeMode.resolveSelectionTarget(toFilterExplorerNode(keyContext.currentNode));
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
    initialSnapshot: options.initialSnapshot as Parameters<typeof useOntologyExplorerController>[0]["initialSnapshot"],
    model: toOntologyExplorerModel(options.model),
    onExit: options.onExit,
    getInteractionActions: (controller) => getFilterExplorerInteractionActions(options.mode, controller),
    getDetailLines:
      composeMode
        ? ({ selection: ontologySelection }) =>
            buildFilterExplorerComposeDetailLines({
              mode: composeMode,
              draft,
              currentNodeLabel: ontologySelection.currentNode?.label,
              selectedTarget: composeMode.resolveSelectionTarget(toFilterExplorerNode(ontologySelection.currentNode)),
              selectedPolicyState: getFilterExplorerTargetState(
                composeMode.resolveSelectionTarget(toFilterExplorerNode(ontologySelection.currentNode)),
                draft.selection,
              ),
              selectedScalarClause: getFilterExplorerScalarClause(
                composeMode.resolveSelectionTarget(toFilterExplorerNode(ontologySelection.currentNode)),
                draft,
              ),
              baseDetailLines:
                ontologySelection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }],
            })
        : undefined,
    getDetailTitle:
      composeMode
        ? () => composeMode.detailTitle ?? "Detail"
        : undefined,
    onConfirm: (keyContext) => {
      if (composeMode) {
        const target = composeMode.resolveSelectionTarget(toFilterExplorerNode(keyContext.currentNode));
        return (
          openComposeScalarEditor(composeMode, target, draft, updateDraft) ||
          applyComposeCycleSelection(composeMode, keyContext, draft, updateDraft)
        );
      }

      const inspectResult = getNodeInspectResult(options, keyContext.currentNode);
      if (shouldOpenImmediateInspectResult(options.model, keyContext.currentNode, inspectResult)) {
        return openInspectResult(options, keyContext, inspectResult);
      }

      return !keyContext.currentNodeHasChildren && inspectResult
        ? openInspectResult(options, keyContext, inspectResult)
        : false;
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
        const target = composeMode.resolveSelectionTarget(toFilterExplorerNode(keyContext.currentNode));
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
          if (selected === "openSelection" || selected === "openResults") {
            openInspectResult(options, keyContext, getNodeInspectResult(options, keyContext.currentNode));
            return;
          }
          if (selected === "openQuery") {
            openInspectQuery(options, keyContext, getNodeInspectResult(options, keyContext.currentNode));
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
