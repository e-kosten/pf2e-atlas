import type {
  FilterClause,
  FilterClauseOperator,
  FilterEditorFieldView,
  FilterValueOption,
} from "../generated/atlas";
import type {
  MetricComparisonState,
  NumericRangeState,
  SearchFormState,
} from "../state/searchState";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export type FilterSelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
};

export type FilterControlKind = "option" | "range" | "boolean" | "metric";

export type FilterSelectGroup = {
  label: string;
  options: FilterSelectOption[];
};

export function additionalFilterGroups(
  workspace: AtlasWorkspaceState,
): FilterSelectGroup[] {
  const visible = new Set(workspace.search.visibleFilterIds);
  const hidden = new Set(workspace.search.hiddenFilterIds);
  return (workspace.filterEditor?.groups ?? [])
    .map((group) => ({
      label: group.label,
      options: group.fields
        .filter(
          (field) =>
            (field.placement === "addable" ||
              (field.placement === "initially_visible" && hidden.has(field.id))) &&
            !visible.has(field.id) &&
            field.applicability === "applicable",
        )
        .map((field) => ({ value: field.id, label: field.label })),
    }))
    .filter((group) => group.options.length > 0);
}

export function visibleEditorFilterFields(
  workspace: AtlasWorkspaceState,
): FilterEditorFieldView[] {
  const hidden = new Set(workspace.search.hiddenFilterIds);
  return (workspace.filterEditor?.groups ?? [])
    .flatMap((group) => group.fields)
    .filter(
      (field) =>
        field.placement === "always_visible" ||
        (field.placement === "initially_visible" && !hidden.has(field.id)),
    );
}

export function additionalVisibleFilterIds(workspace: AtlasWorkspaceState): string[] {
  return workspace.search.visibleFilterIds.filter((fieldId) => {
    const field = fieldById(workspace, fieldId);
    return field && field.placement === "addable";
  });
}

export function discoveredOptions(
  workspace: AtlasWorkspaceState,
  fieldId: string,
): FilterSelectOption[] {
  const options = workspace.filterValuesByField[fieldId]?.options;
  if (!options || options.length === 0) {
    return [];
  }
  return options.map(valueOption);
}

export function addVisibleFilter(
  workspace: AtlasWorkspaceState,
  fieldId: string | null,
) {
  if (!fieldId) {
    return;
  }
  const field = fieldById(workspace, fieldId);
  if (!field || workspace.search.visibleFilterIds.includes(fieldId)) {
    return;
  }
  if (field.placement === "initially_visible") {
    workspace.setSearch({
      ...workspace.search,
      hiddenFilterIds: workspace.search.hiddenFilterIds.filter(
        (hiddenFieldId) => hiddenFieldId !== fieldId,
      ),
    });
    return;
  }
  workspace.setSearch({
    ...workspace.search,
    visibleFilterIds: [...workspace.search.visibleFilterIds, fieldId],
  });
}

export function removeVisibleFilter(workspace: AtlasWorkspaceState, fieldId: string) {
  const field = fieldById(workspace, fieldId);
  if (field?.placement === "always_visible") {
    return;
  }
  const hiddenFilterIds =
    field?.placement === "initially_visible" &&
    !workspace.search.hiddenFilterIds.includes(fieldId)
      ? [...workspace.search.hiddenFilterIds, fieldId]
      : workspace.search.hiddenFilterIds;
  workspace.setSearch({
    ...clearFieldValue(workspace.search, fieldId),
    hiddenFilterIds,
    visibleFilterIds: workspace.search.visibleFilterIds.filter(
      (visibleFieldId) => visibleFieldId !== fieldId,
    ),
  });
}

export function valuesForField(search: SearchFormState, fieldId: string): string[] {
  return valueClause(search, fieldId)?.values ?? [];
}

export function setValuesForField(
  search: SearchFormState,
  fieldId: string,
  values: string[],
): SearchFormState {
  const operator =
    valueClause(search, fieldId)?.operator ?? defaultValueOperator(fieldId);
  return setValuesClause(search, fieldId, operator, values);
}

export function includeOperatorForField(
  search: SearchFormState,
  fieldId: string,
): "include_all" | "include_any" {
  const operator = valueClause(search, fieldId)?.operator;
  return operator === "include_any" ? "include_any" : "include_all";
}

export function setIncludeOperatorForField(
  search: SearchFormState,
  fieldId: string,
  operator: "include_all" | "include_any",
): SearchFormState {
  const values = valuesForField(search, fieldId);
  return setValuesClause(
    removeClauses(search, fieldId, ["include_all", "include_any"]),
    fieldId,
    operator,
    values,
  );
}

export function excludedValuesForField(
  search: SearchFormState,
  fieldId: string,
): string[] {
  return clauseForField(search, fieldId, "exclude_any")?.values ?? [];
}

export function setExcludedValuesForField(
  search: SearchFormState,
  fieldId: string,
  values: string[],
): SearchFormState {
  return setValuesClause(search, fieldId, "exclude_any", values);
}

export function rangeForField(
  search: SearchFormState,
  fieldId: string,
): NumericRangeState {
  const range = clauseForField(search, fieldId, "range")?.range;
  return { min: range?.min ?? null, max: range?.max ?? null };
}

export function setRangeForField(
  search: SearchFormState,
  fieldId: string,
  range: NumericRangeState,
): SearchFormState {
  if (range.min === null && range.max === null) {
    return removeClauses(search, fieldId, ["range"]);
  }
  return upsertClause(search, {
    id: `${fieldId}-range`,
    field: fieldId,
    operator: "range",
    values: [],
    range: {
      min: range.min ?? undefined,
      max: range.max ?? undefined,
    },
  });
}

export function booleanForField(
  search: SearchFormState,
  fieldId: string,
): string | null {
  const value = valuesForField(search, fieldId)[0] ?? null;
  return value === "true" || value === "false" ? value : null;
}

export function setBooleanForField(
  search: SearchFormState,
  fieldId: string,
  value: string | null,
): SearchFormState {
  return setValuesForField(search, fieldId, value ? [value] : []);
}

export function metricComparisonForField(
  search: SearchFormState,
  fieldId: string,
): MetricComparisonState {
  const metric = clauseForField(search, fieldId, "metric_compare")?.metric;
  return {
    key: metric?.key ?? null,
    op: metric?.op ?? "gte",
    value: metric?.value ?? null,
  };
}

export function setMetricComparisonForField(
  search: SearchFormState,
  fieldId: string,
  comparison: MetricComparisonState,
): SearchFormState {
  if (
    !comparison.key ||
    comparison.value === null ||
    !Number.isFinite(comparison.value)
  ) {
    return removeClauses(search, fieldId, ["metric_compare"]);
  }
  return upsertClause(search, {
    id: `${fieldId}-metric_compare`,
    field: fieldId,
    operator: "metric_compare",
    values: [],
    metric: {
      key: comparison.key,
      op: comparison.op,
      value: comparison.value,
    },
  });
}

export function controlKindForField(
  workspace: AtlasWorkspaceState,
  fieldId: string,
): FilterControlKind {
  const field = fieldById(workspace, fieldId);
  return controlKind(field);
}

export function labelForField(workspace: AtlasWorkspaceState, fieldId: string): string {
  return fieldById(workspace, fieldId)?.label ?? fieldId;
}

export function editorFieldForId(
  workspace: AtlasWorkspaceState,
  fieldId: string,
): FilterEditorFieldView | undefined {
  return fieldById(workspace, fieldId);
}

function valueOption(option: FilterValueOption): FilterSelectOption {
  const count = option.count;
  return {
    value: option.value,
    label:
      count === undefined
        ? option.label
        : `${option.label} (${count.toLocaleString()})`,
    disabled: option.disabled,
  };
}

function fieldById(
  workspace: AtlasWorkspaceState,
  fieldId: string,
): FilterEditorFieldView | undefined {
  return (workspace.filterEditor?.groups ?? [])
    .flatMap((group) => group.fields)
    .find((field) => field.id === fieldId);
}

function controlKind(field: FilterEditorFieldView | undefined): FilterControlKind {
  switch (field?.control.kind) {
    case "range":
      return "range";
    case "boolean":
      return "boolean";
    case "metric_comparison":
      return "metric";
    default:
      return "option";
  }
}

function valueClause(
  search: SearchFormState,
  fieldId: string,
): FilterClause | undefined {
  if (fieldId === "traits") {
    return (
      clauseForField(search, fieldId, "include_all") ??
      clauseForField(search, fieldId, "include_any")
    );
  }
  return clauseForField(search, fieldId, "include_any");
}

function clauseForField(
  search: SearchFormState,
  fieldId: string,
  operator: FilterClauseOperator,
): FilterClause | undefined {
  return search.filterClauses.find(
    (clause) => clause.field === fieldId && clause.operator === operator,
  );
}

function setValuesClause(
  search: SearchFormState,
  fieldId: string,
  operator: FilterClauseOperator,
  values: string[],
): SearchFormState {
  if (values.length === 0) {
    return removeClauses(search, fieldId, [operator]);
  }
  return upsertClause(search, {
    id: `${fieldId}-${operator}`,
    field: fieldId,
    operator,
    values,
  });
}

function upsertClause(search: SearchFormState, clause: FilterClause): SearchFormState {
  let replaced = false;
  const filterClauses = search.filterClauses.map((existing) => {
    if (existing.field === clause.field && existing.operator === clause.operator) {
      replaced = true;
      return clause;
    }
    return existing;
  });
  if (!replaced) {
    filterClauses.push(clause);
  }
  return { ...search, filterClauses };
}

function removeClauses(
  search: SearchFormState,
  fieldId: string,
  operators?: FilterClauseOperator[],
): SearchFormState {
  const operatorSet = operators ? new Set<FilterClauseOperator>(operators) : null;
  return {
    ...search,
    filterClauses: search.filterClauses.filter(
      (clause) =>
        clause.field !== fieldId ||
        (operatorSet !== null && !operatorSet.has(clause.operator)),
    ),
  };
}

function clearFieldValue(search: SearchFormState, fieldId: string): SearchFormState {
  return removeClauses(search, fieldId);
}

function defaultValueOperator(fieldId: string): FilterClauseOperator {
  return fieldId === "traits" ? "include_all" : "include_any";
}
