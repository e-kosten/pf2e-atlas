import { showTerminalReturnDialog, type TerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { getTerminalInteractionCycleDirection, type TerminalInteractionAction } from "../interaction-bindings.js";
import {
  cycleFilterExplorerDiscreteClause,
  cloneFilterExplorerComposeDraft,
  getFilterExplorerScalarClause,
  isFilterExplorerScalarTarget,
  setFilterExplorerScalarClause,
} from "./compose-state.js";
import {
  buildFilterExplorerCommandEntries,
  buildFilterExplorerHelpLines,
} from "./screen-models.js";
import type {
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
  composeMode: FilterExplorerComposeMode,
  keyContext: Pick<FilterExplorerKeyContext, "currentNode" | "event">,
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
    ...cycleFilterExplorerDiscreteClause(target, current, cycleDirection),
  }));
  return true;
}

export function openComposeScalarEditor(
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
    options.onExit();
    return true;
  }

  if (action.id === "cycle" && options.mode.kind === "compose") {
    const target = options.mode.resolveSelectionTarget(keyContext.currentNode);
    return (
      openComposeScalarEditor(options.mode, target, draft, updateDraft) ||
      applyComposeCycleSelection(options.mode, keyContext, updateDraft)
    );
  }

  if (action.id === "help") {
    void showTerminalReturnDialog(adapters, `${context.screenTitle} Help`, buildFilterExplorerHelpLines(context));
    return true;
  }

  if (action.id === "commands" && options.mode.kind === "inspect-and-open") {
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
        const inspectResult = context.selectedInspectResult;
        if (selected === "switchToMatching") {
          options.discovery?.onModeChange?.("matching");
          return;
        }
        if (selected === "switchToCatalog") {
          options.discovery?.onModeChange?.("catalog");
          return;
        }
        if (selected === "openSelection" || selected === "openResults") {
          onOpenInspectResult(inspectResult);
          return;
        }
        if (selected === "openQuery") {
          onOpenInspectQuery(inspectResult);
        }
      });
    return true;
  }

  if (action.id === "commands") {
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
        if (selected === "switchToMatching") {
          options.discovery?.onModeChange?.("matching");
        } else if (selected === "switchToCatalog") {
          options.discovery?.onModeChange?.("catalog");
        }
      });
    return true;
  }

  return false;
}
