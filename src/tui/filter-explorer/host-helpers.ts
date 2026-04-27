import { promptNumericScalarClause, type NumericScalarClauseDraft } from "./scalar-editor.js";
import type {
  FilterExplorerComposeMode,
  FilterExplorerDiscoveryState,
  FilterExplorerOptions,
  FilterExplorerScalarClause,
  FilterExplorerSelectTargetOutcome,
} from "./types.js";
import type { SearchTerminalPromptAdapters } from "../interaction-context-adapters.js";
import type { DerivedTagTerminalApp } from "../framework/types.js";

type ScalarPrompts = Pick<SearchTerminalPromptAdapters, "promptTextInput">;
type ScalarTerminal = Pick<DerivedTagTerminalApp, "pauseForAnyKey">;

function toNumericScalarDraft(clause: FilterExplorerScalarClause | undefined): NumericScalarClauseDraft | null {
  if (!clause) {
    return null;
  }
  if (clause.operator === "between") {
    return { op: "between", min: clause.min, max: clause.max };
  }
  if (typeof clause.value !== "number") {
    return null;
  }
  return { op: clause.operator, value: clause.value };
}

function toFilterExplorerScalarClause(
  clause: NumericScalarClauseDraft | null | undefined,
): FilterExplorerScalarClause | null | undefined {
  if (clause === undefined) {
    return undefined;
  }
  if (clause === null) {
    return null;
  }

  return clause.op === "between"
    ? { operator: "between", min: clause.min, max: clause.max }
    : { operator: clause.op, value: clause.value };
}

export function createFilterExplorerNumericScalarEditHandler(
  prompts: ScalarPrompts,
  terminal: ScalarTerminal,
): NonNullable<FilterExplorerComposeMode["onEditScalarTarget"]> {
  return async ({ target, currentClause }) => {
    if (target.valueType !== "number") {
      return undefined;
    }

    const nextClause = await promptNumericScalarClause(prompts, terminal, {
      title: target.editorLabel ?? `${target.fieldLabel} / ${target.subjectLabel}`,
      currentClause: toNumericScalarDraft(currentClause),
    });
    return toFilterExplorerScalarClause(nextClause);
  };
}

export function createFilterExplorerDiscoveryState<TMode extends string>(options: {
  mode: TMode;
  modes: FilterExplorerDiscoveryState<TMode>["modes"];
  pendingMode?: TMode;
  isRefreshing?: boolean;
  onModeChange: NonNullable<FilterExplorerDiscoveryState<TMode>["onModeChange"]>;
  enabled?: boolean;
}): FilterExplorerDiscoveryState<TMode> | undefined {
  if (options.enabled === false) {
    return undefined;
  }

  return {
    mode: options.mode,
    modes: options.modes,
    pendingMode: options.pendingMode,
    isRefreshing: options.isRefreshing,
    onModeChange: options.onModeChange,
  };
}

export function createFilterExplorerOutcomeHandler(args: {
  onBack?: (snapshot: Parameters<FilterExplorerOptions["onOutcome"]>[1]) => void;
  onExitRoot?: (snapshot: Parameters<FilterExplorerOptions["onOutcome"]>[1]) => void;
  onCancel?: (snapshot: Parameters<FilterExplorerOptions["onOutcome"]>[1]) => void;
  onSelectTarget?: (
    outcome: FilterExplorerSelectTargetOutcome,
    snapshot: Parameters<FilterExplorerOptions["onOutcome"]>[1],
  ) => void;
}): FilterExplorerOptions["onOutcome"] {
  return (outcome, snapshot) => {
    if (outcome.kind === "selectTarget") {
      args.onSelectTarget?.(outcome, snapshot);
      return;
    }

    if (outcome.kind === "back") {
      args.onBack?.(snapshot);
      return;
    }

    if (outcome.kind === "exitRoot") {
      args.onExitRoot?.(snapshot);
      return;
    }

    args.onCancel?.(snapshot);
  };
}
