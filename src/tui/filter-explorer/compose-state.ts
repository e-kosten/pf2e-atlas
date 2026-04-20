import type { OntologyNode } from "../../domain/ontology-types.js";
import type {
  FilterExplorerComposeDraft,
  FilterExplorerComposeTarget,
  FilterExplorerDiscreteComposeTarget,
  FilterExplorerPolicyState,
  FilterExplorerScalarClause,
  FilterExplorerScalarClauseMap,
  FilterExplorerScalarComposeTarget,
  FilterExplorerSelectionMap,
} from "./types.js";

function cloneSelection(selection: { any: string[]; all: string[]; exclude: string[] }): {
  any: string[];
  all: string[];
  exclude: string[];
} {
  return {
    any: [...selection.any],
    all: [...selection.all],
    exclude: [...selection.exclude],
  };
}

function sortUnique(values: readonly string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

export function createEmptyFilterExplorerSelection(): { any: string[]; all: string[]; exclude: string[] } {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

export function cloneFilterExplorerSelectionMap(selection: FilterExplorerSelectionMap | undefined): FilterExplorerSelectionMap {
  if (!selection) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(selection).map(([field, valueSelection]) => [field, cloneSelection(valueSelection)]),
  );
}

function cloneScalarClause(clause: FilterExplorerScalarClause): FilterExplorerScalarClause {
  return clause.operator === "between" ? { ...clause } : { ...clause };
}

export function cloneFilterExplorerScalarClauseMap(
  scalarClauses: FilterExplorerScalarClauseMap | undefined,
): FilterExplorerScalarClauseMap {
  if (!scalarClauses) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(scalarClauses).map(([key, clause]) => [key, cloneScalarClause(clause)]),
  );
}

export function normalizeFilterExplorerSelectionMap(selection: FilterExplorerSelectionMap | undefined): FilterExplorerSelectionMap {
  if (!selection) {
    return {};
  }

  return Object.fromEntries(
    Object.entries(selection).map(([field, valueSelection]) => {
      const exclude = sortUnique(valueSelection.exclude);
      const all = sortUnique(valueSelection.all).filter((value) => !exclude.includes(value));
      const any = sortUnique(valueSelection.any).filter((value) => !exclude.includes(value) && !all.includes(value));

      return [
        field,
        {
          any,
          all,
          exclude,
        },
      ];
    }),
  );
}

export function createEmptyFilterExplorerComposeDraft(): FilterExplorerComposeDraft {
  return {
    selection: {},
    scalarClauses: {},
  };
}

export function cloneFilterExplorerComposeDraft(
  draft: FilterExplorerComposeDraft | undefined,
): FilterExplorerComposeDraft {
  return {
    selection: cloneFilterExplorerSelectionMap(draft?.selection),
    scalarClauses: cloneFilterExplorerScalarClauseMap(draft?.scalarClauses),
  };
}

export function normalizeFilterExplorerComposeDraft(
  draft: FilterExplorerComposeDraft | undefined,
  fallbackSelection?: FilterExplorerSelectionMap,
): FilterExplorerComposeDraft {
  return {
    selection: normalizeFilterExplorerSelectionMap(draft?.selection ?? fallbackSelection),
    scalarClauses: cloneFilterExplorerScalarClauseMap(draft?.scalarClauses),
  };
}

export function hasFilterExplorerSelection(selection: FilterExplorerSelectionMap): boolean {
  return Object.values(selection).some(
    (valueSelection) =>
      valueSelection.any.length > 0 || valueSelection.all.length > 0 || valueSelection.exclude.length > 0,
  );
}

export function hasFilterExplorerScalarClause(
  target: FilterExplorerScalarComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
): boolean {
  return Boolean(target && draft.scalarClauses[target.key]);
}

export function hasFilterExplorerComposeDraftEntries(draft: FilterExplorerComposeDraft): boolean {
  return hasFilterExplorerSelection(draft.selection) || Object.keys(draft.scalarClauses).length > 0;
}

export function getFilterExplorerTargetState(
  target: FilterExplorerComposeTarget | undefined,
  selection: FilterExplorerSelectionMap,
): FilterExplorerPolicyState | undefined {
  if (!target || target.kind === "scalar") {
    return undefined;
  }

  const fieldSelection = selection[target.field];
  if (!fieldSelection) {
    return undefined;
  }
  if (fieldSelection.any.includes(target.value)) {
    return "any";
  }
  if (fieldSelection.all.includes(target.value)) {
    return "all";
  }
  if (fieldSelection.exclude.includes(target.value)) {
    return "exclude";
  }
  return undefined;
}

export function cycleFilterExplorerPolicyState(
  currentState: FilterExplorerPolicyState | undefined,
  allowedStates: readonly FilterExplorerPolicyState[],
  direction: 1 | -1 = 1,
): FilterExplorerPolicyState | undefined {
  const states: Array<FilterExplorerPolicyState | undefined> = [undefined, ...allowedStates];
  const currentIndex = states.findIndex((state) => state === currentState);
  return states[(((currentIndex + direction) % states.length) + states.length) % states.length];
}

export function toggleFilterExplorerTargetSelection(
  target: FilterExplorerComposeTarget | undefined,
  selection: FilterExplorerSelectionMap,
  direction: 1 | -1 = 1,
): FilterExplorerSelectionMap {
  if (!target || target.kind === "scalar") {
    return selection;
  }

  const next = cloneFilterExplorerSelectionMap(selection);
  const fieldSelection = next[target.field] ?? createEmptyFilterExplorerSelection();
  fieldSelection.any = fieldSelection.any.filter((value) => value !== target.value);
  fieldSelection.all = fieldSelection.all.filter((value) => value !== target.value);
  fieldSelection.exclude = fieldSelection.exclude.filter((value) => value !== target.value);

  const nextState = cycleFilterExplorerPolicyState(getFilterExplorerTargetState(target, selection), target.allowedStates, direction);
  if (nextState) {
    fieldSelection[nextState].push(target.value);
  }

  next[target.field] = {
    any: sortUnique(fieldSelection.any),
    all: sortUnique(fieldSelection.all),
    exclude: sortUnique(fieldSelection.exclude),
  };
  return next;
}

export function getFilterExplorerScalarClause(
  target: FilterExplorerComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
): FilterExplorerScalarClause | undefined {
  return target?.kind === "scalar" ? draft.scalarClauses[target.key] : undefined;
}

export function setFilterExplorerScalarClause(
  target: FilterExplorerScalarComposeTarget,
  clause: FilterExplorerScalarClause | null | undefined,
  draft: FilterExplorerComposeDraft,
): FilterExplorerComposeDraft {
  const next = cloneFilterExplorerComposeDraft(draft);
  if (clause) {
    next.scalarClauses[target.key] = cloneScalarClause(clause);
  } else {
    delete next.scalarClauses[target.key];
  }
  return next;
}

export function getFilterExplorerSelectableNodeTarget(
  node: OntologyNode | undefined,
  resolveTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined,
): FilterExplorerComposeTarget | undefined {
  return resolveTarget(node);
}

export function isFilterExplorerDiscreteTarget(
  target: FilterExplorerComposeTarget | undefined,
): target is FilterExplorerDiscreteComposeTarget {
  return Boolean(target && target.kind !== "scalar");
}

export function isFilterExplorerScalarTarget(
  target: FilterExplorerComposeTarget | undefined,
): target is FilterExplorerScalarComposeTarget {
  return target?.kind === "scalar";
}
