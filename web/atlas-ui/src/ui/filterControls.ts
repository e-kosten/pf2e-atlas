import type { FilterFieldView, FilterValueOption } from "../generated/atlas";
import {
  STANDARD_FILTER_IDS,
  type NumericRangeState,
  type SearchFormState,
} from "../state/searchState";
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

type FilterCatalogItem = {
  id: string;
  group: string;
  control: FilterControlKind;
  label?: string;
};

const FILTER_CATALOG: FilterCatalogItem[] = [
  { id: "pack", group: "Source", control: "option" },
  { id: "publication_title", group: "Source", control: "option" },
  { id: "publication_family", group: "Source", control: "option" },
  { id: "publication_remaster", group: "Source", control: "boolean" },
  { id: "traditions", group: "Spells", control: "option" },
  { id: "spell_kinds", group: "Spells", control: "option" },
  { id: "save_type", group: "Spells", control: "option" },
  { id: "basic_save", group: "Spells", control: "boolean" },
  { id: "sustained", group: "Spells", control: "boolean" },
  { id: "damage_types", group: "Spells & Equipment", control: "option" },
  { id: "range_value", group: "Spells", control: "range" },
  { id: "area_type", group: "Spells", control: "option" },
  { id: "area_value", group: "Spells", control: "range" },
  { id: "item_category", group: "Equipment", control: "option" },
  { id: "item_group", group: "Equipment", control: "option" },
  { id: "price_cp", group: "Equipment", control: "range" },
  { id: "bulk_value", group: "Equipment", control: "range" },
  { id: "hands", group: "Equipment", control: "option" },
  { id: "usage", group: "Equipment", control: "option" },
  { id: "base_item", group: "Equipment", control: "option" },
  { id: "size", group: "Creatures", control: "option" },
  { id: "speed_types", group: "Creatures", control: "option" },
  { id: "languages", group: "Creatures", control: "option" },
  { id: "senses", group: "Creatures", control: "option" },
  { id: "immunities", group: "Creatures", control: "option" },
  { id: "resistances", group: "Creatures", control: "option" },
  { id: "weaknesses", group: "Creatures", control: "option" },
];

const OPTION_FIELD_IDS = new Set(
  ["kind", "rarity", "traits"].concat(
    FILTER_CATALOG.filter((field) => field.control === "option").map(
      (field) => field.id,
    ),
  ),
);

export function optionFieldIds(search: SearchFormState): string[] {
  return search.visibleFilterIds.filter((fieldId) => OPTION_FIELD_IDS.has(fieldId));
}

export function additionalFilterOptions(
  workspace: AtlasWorkspaceState,
): FilterSelectOption[] {
  return additionalFilterGroups(workspace).flatMap((group) => group.options);
}

export function additionalFilterGroups(
  workspace: AtlasWorkspaceState,
): FilterSelectGroup[] {
  const visible = new Set(workspace.search.visibleFilterIds);
  const discovered = new Set(
    (workspace.filterFields?.fields ?? []).map((field) => field.id),
  );
  return FILTER_CATALOG.reduce<FilterSelectGroup[]>((groups, item) => {
    if (visible.has(item.id) || !discovered.has(item.id)) {
      return groups;
    }
    let group = groups.find((candidate) => candidate.label === item.group);
    if (!group) {
      group = { label: item.group, options: [] };
      groups.push(group);
    }
    group.options.push({
      value: item.id,
      label: labelForField(workspace.filterFields?.fields, item.id),
    });
    return groups;
  }, []);
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
    ...clearFieldValue(workspace.search, fieldId),
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
  return search.rangeFilters[fieldId] ?? { min: null, max: null };
}

export function setRangeForField(
  search: SearchFormState,
  fieldId: string,
  range: NumericRangeState,
): SearchFormState {
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
  fields: FilterFieldView[] | undefined,
  fieldId: string,
): FilterControlKind {
  const catalogItem = FILTER_CATALOG.find((field) => field.id === fieldId);
  if (catalogItem) {
    return catalogItem.control;
  }
  const field = fields?.find((candidate) => candidate.id === fieldId);
  if (field?.value_kind === "number") {
    return "range";
  }
  if (field?.value_kind === "boolean") {
    return "boolean";
  }
  return "option";
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
    case "publication_family":
      return "Publication Family";
    case "publication_remaster":
      return "Remaster";
    case "traditions":
      return "Traditions";
    case "spell_kinds":
      return "Spell Type";
    case "save_type":
      return "Save Type";
    case "basic_save":
      return "Basic Save";
    case "sustained":
      return "Sustained";
    case "damage_types":
      return "Damage Type";
    case "range_value":
      return "Range";
    case "area_type":
      return "Area Type";
    case "area_value":
      return "Area Size";
    case "item_category":
      return "Category";
    case "item_group":
      return "Group";
    case "price_cp":
      return "Price";
    case "bulk_value":
      return "Bulk";
    case "hands":
      return "Hands";
    case "usage":
      return "Usage";
    case "base_item":
      return "Base Item";
    case "size":
      return "Size";
    case "speed_types":
      return "Speeds";
    case "languages":
      return "Languages";
    case "senses":
      return "Senses";
    case "immunities":
      return "Immunities";
    case "resistances":
      return "Resistances";
    case "weaknesses":
      return "Weaknesses";
    default:
      return "Filter";
  }
}

function clearFieldValue(
  search: SearchFormState,
  fieldId: string,
): SearchFormState {
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
