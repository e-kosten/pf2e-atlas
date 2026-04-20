import type { OntologyNode } from "../../domain/ontology-types.js";
import type { FilterExplorerComposeTarget, FilterExplorerPolicyState, FilterExplorerSelectionMap } from "./types.js";

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

export function hasFilterExplorerSelection(selection: FilterExplorerSelectionMap): boolean {
  return Object.values(selection).some(
    (valueSelection) =>
      valueSelection.any.length > 0 || valueSelection.all.length > 0 || valueSelection.exclude.length > 0,
  );
}

export function getFilterExplorerTargetState(
  target: FilterExplorerComposeTarget | undefined,
  selection: FilterExplorerSelectionMap,
): FilterExplorerPolicyState | undefined {
  if (!target) {
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
  if (!target) {
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

export function getFilterExplorerSelectableNodeTarget(
  node: OntologyNode | undefined,
  resolveTarget: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined,
): FilterExplorerComposeTarget | undefined {
  return resolveTarget(node);
}
