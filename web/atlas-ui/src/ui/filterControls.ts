import type { FilterEditorFieldView, FilterValueOption } from "../generated/atlas";
import type { NumericRangeState, SearchFormState } from "../state/searchState";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export type FilterSelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
};

export type FilterControlKind = "option" | "range" | "boolean";

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
            !visible.has(field.id),
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
  switch (fieldId) {
    case "kind":
      return search.kinds;
    case "rarity":
      return search.rarity;
    case "traits":
      return search.traits;
    case "pack":
      return search.packLabels;
    case "publication_title":
      return search.publicationTitles;
    default:
      return search.optionFilters[fieldId] ?? [];
  }
}

export function setValuesForField(
  search: SearchFormState,
  fieldId: string,
  values: string[],
): SearchFormState {
  switch (fieldId) {
    case "kind":
      return { ...search, kinds: values };
    case "rarity":
      return { ...search, rarity: values };
    case "traits":
      return { ...search, traits: values };
    case "pack":
      return { ...search, packLabels: values };
    case "publication_title":
      return { ...search, publicationTitles: values };
    default:
      return {
        ...search,
        optionFilters: { ...search.optionFilters, [fieldId]: values },
      };
  }
}

export function rangeForField(
  search: SearchFormState,
  fieldId: string,
): NumericRangeState {
  if (fieldId === "level") {
    return { min: search.levelMin, max: search.levelMax };
  }
  return search.rangeFilters[fieldId] ?? { min: null, max: null };
}

export function setRangeForField(
  search: SearchFormState,
  fieldId: string,
  range: NumericRangeState,
): SearchFormState {
  if (fieldId === "level") {
    return {
      ...search,
      levelMin: range.min,
      levelMax: range.max,
    };
  }
  return {
    ...search,
    rangeFilters: { ...search.rangeFilters, [fieldId]: range },
  };
}

export function booleanForField(
  search: SearchFormState,
  fieldId: string,
): string | null {
  return search.booleanFilters[fieldId] ?? null;
}

export function setBooleanForField(
  search: SearchFormState,
  fieldId: string,
  value: string | null,
): SearchFormState {
  return {
    ...search,
    booleanFilters: { ...search.booleanFilters, [fieldId]: value },
  };
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
    default:
      return "option";
  }
}

function clearFieldValue(search: SearchFormState, fieldId: string): SearchFormState {
  const { [fieldId]: _option, ...optionFilters } = search.optionFilters;
  const { [fieldId]: _range, ...rangeFilters } = search.rangeFilters;
  const { [fieldId]: _boolean, ...booleanFilters } = search.booleanFilters;
  return {
    ...setValuesForField(search, fieldId, []),
    optionFilters,
    rangeFilters,
    booleanFilters,
  };
}
