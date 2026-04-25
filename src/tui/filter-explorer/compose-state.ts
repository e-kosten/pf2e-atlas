import type {
  FilterExplorerComposeDraft,
  FilterExplorerComposeTarget,
  FilterExplorerDiscreteClause,
  FilterExplorerDiscreteClauseOperator,
  FilterExplorerDiscreteComposeTarget,
  FilterExplorerNode,
  FilterExplorerScalarClause,
  FilterExplorerScalarClauseMap,
  FilterExplorerScalarComposeTarget,
} from "./types.js";

function sortUnique(values: readonly string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function getDiscreteClauseIdentity(clause: Pick<FilterExplorerDiscreteClause, "field" | "value">): string {
  return `${clause.field}\u0000${clause.value}`;
}

function compareDiscreteClauses(left: FilterExplorerDiscreteClause, right: FilterExplorerDiscreteClause): number {
  return (
    left.field.localeCompare(right.field) ||
    left.value.localeCompare(right.value) ||
    left.operator.localeCompare(right.operator)
  );
}

function cloneDiscreteClause(clause: FilterExplorerDiscreteClause): FilterExplorerDiscreteClause {
  return { ...clause };
}

export function cloneFilterExplorerDiscreteClauses(
  discreteClauses: readonly FilterExplorerDiscreteClause[] | undefined,
): FilterExplorerDiscreteClause[] {
  return [...(discreteClauses ?? [])].map(cloneDiscreteClause);
}

function normalizeFilterExplorerDiscreteClauses(
  discreteClauses: readonly FilterExplorerDiscreteClause[] | undefined,
): FilterExplorerDiscreteClause[] {
  const clauseByIdentity = new Map<string, FilterExplorerDiscreteClause>();

  for (const clause of discreteClauses ?? []) {
    const field = clause.field.trim();
    const value = clause.value.trim();
    if (!field || !value) {
      continue;
    }

    clauseByIdentity.set(getDiscreteClauseIdentity({ field, value }), {
      field,
      value,
      operator: clause.operator,
    });
  }

  return [...clauseByIdentity.values()].sort(compareDiscreteClauses);
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

export function createEmptyFilterExplorerComposeDraft(): FilterExplorerComposeDraft {
  return {
    discreteClauses: [],
    scalarClauses: {},
  };
}

export function cloneFilterExplorerComposeDraft(
  draft: FilterExplorerComposeDraft | undefined,
): FilterExplorerComposeDraft {
  return {
    discreteClauses: cloneFilterExplorerDiscreteClauses(draft?.discreteClauses),
    scalarClauses: cloneFilterExplorerScalarClauseMap(draft?.scalarClauses),
  };
}

export function normalizeFilterExplorerComposeDraft(
  draft: FilterExplorerComposeDraft | undefined,
): FilterExplorerComposeDraft {
  return {
    discreteClauses: normalizeFilterExplorerDiscreteClauses(draft?.discreteClauses),
    scalarClauses: cloneFilterExplorerScalarClauseMap(draft?.scalarClauses),
  };
}

export function hasFilterExplorerDiscreteClauses(
  discreteClauses: readonly FilterExplorerDiscreteClause[],
): boolean {
  return discreteClauses.length > 0;
}

export function hasFilterExplorerScalarClause(
  target: FilterExplorerScalarComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
): boolean {
  return Boolean(target && draft.scalarClauses[target.key]);
}

export function hasFilterExplorerComposeDraftEntries(draft: FilterExplorerComposeDraft): boolean {
  return hasFilterExplorerDiscreteClauses(draft.discreteClauses) || Object.keys(draft.scalarClauses).length > 0;
}

export function getFilterExplorerDiscreteClause(
  target: FilterExplorerComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
): FilterExplorerDiscreteClause | undefined {
  if (!target || target.kind === "scalar") {
    return undefined;
  }

  return draft.discreteClauses.find((clause) => clause.field === target.field && clause.value === target.value);
}

export function getFilterExplorerDiscreteClauseOperator(
  target: FilterExplorerComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
): FilterExplorerDiscreteClauseOperator | undefined {
  return getFilterExplorerDiscreteClause(target, draft)?.operator;
}

export function cycleFilterExplorerDiscreteClauseOperator(
  currentOperator: FilterExplorerDiscreteClauseOperator | undefined,
  allowedOperators: readonly FilterExplorerDiscreteClauseOperator[],
  direction: 1 | -1 = 1,
): FilterExplorerDiscreteClauseOperator | undefined {
  const operators: Array<FilterExplorerDiscreteClauseOperator | undefined> = [undefined, ...allowedOperators];
  const currentIndex = operators.findIndex((operator) => operator === currentOperator);
  return operators[(((currentIndex + direction) % operators.length) + operators.length) % operators.length];
}

export function cycleFilterExplorerDiscreteClause(
  target: FilterExplorerComposeTarget | undefined,
  draft: FilterExplorerComposeDraft,
  direction: 1 | -1 = 1,
): FilterExplorerComposeDraft {
  if (!target || target.kind === "scalar") {
    return draft;
  }

  const currentOperator = getFilterExplorerDiscreteClauseOperator(target, draft);
  const nextOperator = cycleFilterExplorerDiscreteClauseOperator(currentOperator, target.allowedOperators, direction);
  const nextDraft = cloneFilterExplorerComposeDraft(draft);
  nextDraft.discreteClauses = nextDraft.discreteClauses.filter(
    (clause) => clause.field !== target.field || clause.value !== target.value,
  );

  if (nextOperator) {
    nextDraft.discreteClauses.push({
      field: target.field,
      value: target.value,
      operator: nextOperator,
    });
  }

  nextDraft.discreteClauses = normalizeFilterExplorerDiscreteClauses(nextDraft.discreteClauses);
  return nextDraft;
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
  node: FilterExplorerNode | undefined,
  resolveTarget: (node: FilterExplorerNode | undefined) => FilterExplorerComposeTarget | undefined,
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
