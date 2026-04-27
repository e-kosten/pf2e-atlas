import { showTerminalReturnDialog, type TerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { getTerminalInteractionCycleDirection, type TerminalInteractionAction } from "../interaction-bindings.js";
import {
  cycleFilterExplorerDiscreteClause,
  cloneFilterExplorerComposeDraft,
  getFilterExplorerScalarClause,
  isFilterExplorerScalarTarget,
  setFilterExplorerScalarClause,
} from "./compose-state.js";
import { createFilterExplorerBrowserSnapshot } from "./controller-state.js";
import {
  buildFilterExplorerHelpLines,
} from "./screen-models.js";
import type {
  FilterExplorerActionEntry,
  FilterExplorerBrowserContext,
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerInspectResult,
  FilterExplorerOptions,
} from "./types.js";

type FilterExplorerKeyContext = FilterExplorerBrowserContext & {
  event: Parameters<typeof getTerminalInteractionCycleDirection>[0];
};

export function applyComposeCycleSelection(
  target: FilterExplorerControllerContext["selectedTarget"],
  keyContext: Pick<FilterExplorerKeyContext, "currentNode" | "event">,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
): boolean {
  const cycleDirection = getTerminalInteractionCycleDirection(keyContext.event, { id: "cycle" });
  if (!cycleDirection) {
    return false;
  }

  if (!target || target.kind === "scalar") {
    return false;
  }

  updateDraft((current) => ({
    ...cycleFilterExplorerDiscreteClause(target, current, cycleDirection),
  }));
  return true;
}

export function openComposeScalarEditor(
  composeMode: FilterExplorerComposeMode,
  target: FilterExplorerControllerContext["selectedTarget"],
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

export function shouldExitAtRootDepth(options: FilterExplorerOptions, keyContext: FilterExplorerKeyContext): boolean {
  return (
    options.mode.kind === "compose" &&
    options.exitAtRootDepth === true &&
    keyContext.state.activePane === "list" &&
    keyContext.effectiveState.depth === (options.rootDepth ?? 0)
  );
}

export function handleFilterExplorerAction(args: {
  action: TerminalInteractionAction;
  adapters: TerminalInteractionContextAdapters;
  context: FilterExplorerControllerContext;
  draft: FilterExplorerComposeDraft;
  keyContext: FilterExplorerKeyContext;
  onOpenInspectQuery: (result: FilterExplorerInspectResult | undefined) => void;
  onOpenInspectResult: (result: FilterExplorerInspectResult | undefined) => void;
  options: FilterExplorerOptions;
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void;
}): boolean {
  const { action, adapters, context, draft, keyContext, onOpenInspectQuery, onOpenInspectResult, options, updateDraft } =
    args;

  if (action.id === "back" && shouldExitAtRootDepth(options, keyContext)) {
    options.onOutcome({ kind: "back" }, createFilterExplorerBrowserSnapshot(keyContext));
    return true;
  }

  if (action.id === "cycle" && options.mode.kind === "compose") {
    const target = context.selectedTarget;
    return (
      openComposeScalarEditor(options.mode, target, draft, updateDraft) ||
      applyComposeCycleSelection(target, keyContext, updateDraft)
    );
  }

  if (action.id === "help") {
    void showTerminalReturnDialog(adapters, `${context.screenTitle} Help`, buildFilterExplorerHelpLines(context));
    return true;
  }

  return false;
}

export function applyFilterExplorerActionEntry(args: {
  actionEntry: FilterExplorerActionEntry;
  context: FilterExplorerControllerContext;
  onOpenInspectQuery: (result: FilterExplorerInspectResult | undefined) => void;
  onOpenInspectResult: (result: FilterExplorerInspectResult | undefined) => void;
}): void {
  const { actionEntry, context, onOpenInspectQuery, onOpenInspectResult } = args;
  const inspectResult = context.selectedInspectResult;

  if (actionEntry.action.kind === "setMode") {
    context.discovery?.onModeChange?.(actionEntry.action.mode);
    return;
  }

  if (actionEntry.action.selection === "default") {
    onOpenInspectResult(inspectResult);
    return;
  }

  if (actionEntry.action.selection === "query") {
    onOpenInspectQuery(inspectResult);
  }
}
