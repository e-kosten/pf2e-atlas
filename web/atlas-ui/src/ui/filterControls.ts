import type { FilterFieldView, FilterValueOption } from "../generated/atlas";
import {
  STANDARD_FILTER_IDS,
  type SearchFormState,
} from "../state/searchState";
import type { AtlasWorkspaceState } from "./useAtlasWorkspace";

export type FilterSelectOption = {
  value: string;
  label: string;
  disabled?: boolean;
};

const OPTION_FIELD_IDS = [
  "kind",
  "rarity",
  "traits",
  "pack",
  "publication_title",
];

export function optionFieldIds(search: SearchFormState): string[] {
  return search.visibleFilterIds.filter((fieldId) =>
    OPTION_FIELD_IDS.includes(fieldId),
  );
}

export function additionalFilterOptions(
  workspace: AtlasWorkspaceState,
): FilterSelectOption[] {
  const visible = new Set(workspace.search.visibleFilterIds);
  return (workspace.filterFields?.fields ?? [])
    .filter((field) => !visible.has(field.id))
    .filter((field) => OPTION_FIELD_IDS.includes(field.id))
    .map((field) => ({
      value: field.id,
      label: field.label,
    }));
}

export function discoveredOptions(
  workspace: AtlasWorkspaceState,
  fieldId: string,
  fallback: FilterSelectOption[],
): FilterSelectOption[] {
  const options = workspace.filterValuesByField[fieldId]?.options;
  if (!options || options.length === 0) {
    return fallback;
  }
  return options.map(valueOption);
}

export function addVisibleFilter(
  workspace: AtlasWorkspaceState,
  fieldId: string | null,
) {
  if (!fieldId || workspace.search.visibleFilterIds.includes(fieldId)) {
    return;
  }
  workspace.setSearch({
    ...workspace.search,
    visibleFilterIds: [...workspace.search.visibleFilterIds, fieldId],
  });
}

export function removeVisibleFilter(
  workspace: AtlasWorkspaceState,
  fieldId: string,
) {
  if (STANDARD_FILTER_IDS.includes(fieldId)) {
    return;
  }
  workspace.setSearch({
    ...setValuesForField(workspace.search, fieldId, []),
    visibleFilterIds: workspace.search.visibleFilterIds.filter(
      (visibleFieldId) => visibleFieldId !== fieldId,
    ),
  });
}

export function valuesForField(
  search: SearchFormState,
  fieldId: string,
): string[] {
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
      return [];
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
      return search;
  }
}

export function labelForField(
  fields: FilterFieldView[] | undefined,
  fieldId: string,
): string {
  return fields?.find((field) => field.id === fieldId)?.label ?? fallbackLabel(fieldId);
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

function fallbackLabel(fieldId: string): string {
  switch (fieldId) {
    case "kind":
      return "Kinds";
    case "rarity":
      return "Rarity";
    case "traits":
      return "Traits";
    case "pack":
      return "Pack";
    case "publication_title":
      return "Publication";
    default:
      return "Filter";
  }
}
