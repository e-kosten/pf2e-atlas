import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import { orderStringValues } from "./service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalValueSelection,
} from "./service-types.js";

export function createEmptyValueSelection<T extends number | string>(): Pf2eTerminalValueSelection<T> {
  return {
    include: [],
    exclude: [],
  };
}

export function createEmptyStringSelection(): Pf2eTerminalValueSelection<string> {
  return createEmptyValueSelection<string>();
}

export function createEmptyNumberSelection(): Pf2eTerminalValueSelection<number> {
  return createEmptyValueSelection<number>();
}

export function hasStringSelection(selection: Pf2eTerminalValueSelection<string>): boolean {
  return selection.include.length > 0 || selection.exclude.length > 0;
}

export function hasNumberSelection(selection: Pf2eTerminalValueSelection<number>): boolean {
  return selection.include.length > 0 || selection.exclude.length > 0;
}

export function cloneStringSelection(
  selection: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalValueSelection<string> {
  return {
    include: [...selection.include],
    exclude: [...selection.exclude],
  };
}

export function cloneNumberSelection(
  selection: Pf2eTerminalValueSelection<number>,
): Pf2eTerminalValueSelection<number> {
  return {
    include: [...selection.include],
    exclude: [...selection.exclude],
  };
}

export function normalizeStringSelection(
  selection: Partial<Pf2eTerminalValueSelection<string>> | undefined,
  ordering?: FilterValueOrdering,
): Pf2eTerminalValueSelection<string> {
  const exclude = [...new Set((selection?.exclude ?? []).map((value) => String(value).trim()).filter(Boolean))];
  const include = [...new Set((selection?.include ?? []).map((value) => String(value).trim()).filter(Boolean))].filter(
    (value) => !exclude.includes(value),
  );

  return {
    include: orderStringValues(include, ordering),
    exclude: orderStringValues(exclude, ordering),
  };
}

export function normalizeNumberSelection(
  selection: Partial<Pf2eTerminalValueSelection<number>> | undefined,
): Pf2eTerminalValueSelection<number> {
  const exclude = [...new Set((selection?.exclude ?? []).filter((value) => Number.isFinite(value)))].sort(
    (left, right) => left - right,
  );
  const include = [...new Set((selection?.include ?? []).filter((value) => Number.isFinite(value)))]
    .filter((value) => !exclude.includes(value))
    .sort((left, right) => left - right);

  return { include, exclude };
}

export function normalizeQueryFieldSelection(
  field: Pf2eTerminalFacetField,
  selection: Partial<Pf2eTerminalValueSelection<string>> | undefined,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalValueSelection<string> | null {
  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (!["set", "enumString", "boolean"].includes(fieldSemantics.fieldType)) {
    return null;
  }

  const normalizedSelection = normalizeStringSelection(selection, fieldSemantics.valueOrdering);
  if (fieldSemantics.fieldType === "boolean") {
    normalizedSelection.include = normalizedSelection.include.filter((value) => value === "true" || value === "false");
    normalizedSelection.exclude = normalizedSelection.exclude.filter((value) => value === "true" || value === "false");
  }

  if (normalizedSelection.include.length === 0 && normalizedSelection.exclude.length === 0) {
    return null;
  }

  return normalizedSelection;
}

export function mergeStringSelections(
  left: Pf2eTerminalValueSelection<string>,
  right: Pf2eTerminalValueSelection<string>,
): Pf2eTerminalValueSelection<string> {
  return {
    include: [...left.include, ...right.include],
    exclude: [...left.exclude, ...right.exclude],
  };
}

export function mergeSelectionMaps(
  target: Pf2eTerminalQueryFieldSelectionMap,
  source: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = { ...target };

  for (const [field, selection] of Object.entries(source)) {
    next[field] = field in next ? mergeStringSelections(next[field]!, selection) : { ...selection };
  }

  return next;
}

export function createScopedSelectionMap(scopedFields: string[]): Pf2eTerminalQueryFieldSelectionMap {
  return Object.fromEntries(
    scopedFields.map((field) => [field, createEmptyStringSelection()]),
  ) as Pf2eTerminalQueryFieldSelectionMap;
}
