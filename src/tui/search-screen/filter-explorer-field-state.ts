import type {
  FilterExplorerComposeDraft,
  FilterExplorerComposeTarget,
  FilterExplorerDiscreteClauseOperator,
  FilterExplorerScalarClause,
  FilterExplorerScalarClauseMap,
} from "../filter-explorer/types.js";
import type { Pf2eTerminalFilterExplorerDraft, Pf2eTerminalQueryFieldSelectionMap } from "../search/service-types.js";

export type SearchFilterExplorerFieldState = {
  discreteSelections: Pf2eTerminalQueryFieldSelectionMap;
  scalarClauses: FilterExplorerScalarClauseMap;
};

function sortUnique(values: readonly string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function cloneScalarClauses(scalarClauses: FilterExplorerScalarClauseMap): FilterExplorerScalarClauseMap {
  return Object.fromEntries(Object.entries(scalarClauses).map(([key, clause]) => [key, { ...clause }]));
}

function cloneDiscreteSelections(
  discreteSelections: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  return Object.fromEntries(
    Object.entries(discreteSelections).map(([field, selection]) => [
      field,
      {
        include: [...selection.include],
        exclude: [...selection.exclude],
      },
    ]),
  );
}

function normalizeSelectionMap(
  selections: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  return Object.fromEntries(
    Object.entries(selections).map(([field, selection]) => {
      const exclude = sortUnique(selection.exclude);
      const include = sortUnique(selection.include).filter((value) => !exclude.includes(value));
      return [field, { include, exclude }];
    }),
  );
}

export function buildSearchFilterExplorerFieldState(
  draft: Pf2eTerminalFilterExplorerDraft,
): SearchFilterExplorerFieldState {
  const discreteSelections: Pf2eTerminalQueryFieldSelectionMap = {};
  for (const clause of draft.discreteClauses) {
    const selection = discreteSelections[clause.field] ?? { include: [], exclude: [] };
    if (clause.operator === "include") {
      selection.include.push(clause.value);
    } else {
      selection.exclude.push(clause.value);
    }
    discreteSelections[clause.field] = selection;
  }

  return {
    discreteSelections: normalizeSelectionMap(discreteSelections),
    scalarClauses: cloneScalarClauses(draft.scalarClauses),
  };
}

export function buildSearchFilterExplorerComposeDraft(
  fieldState: SearchFilterExplorerFieldState,
): FilterExplorerComposeDraft {
  const discreteClauses: Pf2eTerminalFilterExplorerDraft["discreteClauses"] = [];
  const selections = normalizeSelectionMap(fieldState.discreteSelections);

  for (const field of Object.keys(selections).sort((left, right) => left.localeCompare(right))) {
    const selection = selections[field]!;
    for (const value of selection.include) {
      discreteClauses.push({ field, value, operator: "include" });
    }
    for (const value of selection.exclude) {
      discreteClauses.push({ field, value, operator: "exclude" });
    }
  }

  return {
    discreteClauses,
    scalarClauses: cloneScalarClauses(fieldState.scalarClauses),
  };
}

export function getSearchFilterExplorerDiscreteOperator(
  fieldState: SearchFilterExplorerFieldState,
  target: Extract<FilterExplorerComposeTarget, { kind?: "discrete" }>,
): FilterExplorerDiscreteClauseOperator | undefined {
  const selection = fieldState.discreteSelections[target.field];
  if (!selection) {
    return undefined;
  }
  if (selection.include.includes(target.value)) {
    return "include";
  }
  if (selection.exclude.includes(target.value)) {
    return "exclude";
  }
  return undefined;
}

export function cycleSearchFilterExplorerDiscreteSelection(
  fieldState: SearchFilterExplorerFieldState,
  target: Extract<FilterExplorerComposeTarget, { kind?: "discrete" }>,
  step = 1,
): SearchFilterExplorerFieldState {
  const currentOperator = getSearchFilterExplorerDiscreteOperator(fieldState, target);
  const states: (FilterExplorerDiscreteClauseOperator | undefined)[] = [
    undefined,
    ...target.allowedOperators,
  ];
  const currentIndex = Math.max(0, states.indexOf(currentOperator));
  const nextIndex = ((currentIndex + step) % states.length + states.length) % states.length;
  const nextOperator = states[nextIndex];

  const nextSelections = cloneDiscreteSelections(fieldState.discreteSelections);
  const selection = nextSelections[target.field] ?? { include: [], exclude: [] };
  selection.include = selection.include.filter((value) => value !== target.value);
  selection.exclude = selection.exclude.filter((value) => value !== target.value);
  if (nextOperator === "include") {
    selection.include.push(target.value);
  } else if (nextOperator === "exclude") {
    selection.exclude.push(target.value);
  }
  if (selection.include.length === 0 && selection.exclude.length === 0) {
    delete nextSelections[target.field];
  } else {
    nextSelections[target.field] = selection;
  }

  return {
    discreteSelections: normalizeSelectionMap(nextSelections),
    scalarClauses: cloneScalarClauses(fieldState.scalarClauses),
  };
}

export function getSearchFilterExplorerScalarClause(
  fieldState: SearchFilterExplorerFieldState,
  target: Extract<FilterExplorerComposeTarget, { kind: "scalar" }>,
): FilterExplorerScalarClause | undefined {
  return fieldState.scalarClauses[target.key];
}

export function setSearchFilterExplorerScalarClause(
  fieldState: SearchFilterExplorerFieldState,
  target: Extract<FilterExplorerComposeTarget, { kind: "scalar" }>,
  nextClause: FilterExplorerScalarClause | null,
): SearchFilterExplorerFieldState {
  const nextScalarClauses = cloneScalarClauses(fieldState.scalarClauses);
  if (nextClause === null) {
    delete nextScalarClauses[target.key];
  } else {
    nextScalarClauses[target.key] = { ...nextClause };
  }

  return {
    discreteSelections: cloneDiscreteSelections(fieldState.discreteSelections),
    scalarClauses: nextScalarClauses,
  };
}
