import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import { orderStringValues } from "./service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldSelectionMap,
} from "./service-types.js";
import type { Pf2eTerminalQueryPartPolicy } from "./query-parts.js";

export function createEmptyFilterPolicy<T extends number | string>(): Pf2eTerminalFilterValuePolicy<T> {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

export function createEmptyStringPolicy(): Pf2eTerminalFilterValuePolicy<string> {
  return createEmptyFilterPolicy<string>();
}

export function createEmptyNumberPolicy(): Pf2eTerminalFilterValuePolicy<number> {
  return createEmptyFilterPolicy<number>();
}

export function hasStringPolicy(policy: Pf2eTerminalFilterValuePolicy<string>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

export function hasNumberPolicy(policy: Pf2eTerminalFilterValuePolicy<number>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

export function cloneStringPolicy(
  policy: Pf2eTerminalFilterValuePolicy<string> | Pf2eTerminalQueryPartPolicy<string>,
): Pf2eTerminalFilterValuePolicy<string> {
  return {
    any: [...policy.any],
    all: [...policy.all],
    exclude: [...policy.exclude],
  };
}

export function cloneNumberPolicy(
  policy: Pf2eTerminalFilterValuePolicy<number> | Pf2eTerminalQueryPartPolicy<number>,
): Pf2eTerminalFilterValuePolicy<number> {
  return {
    any: [...policy.any],
    all: [...policy.all],
    exclude: [...policy.exclude],
  };
}

export function normalizeStringPolicy(
  policy: Partial<Pf2eTerminalFilterValuePolicy<string>> | undefined,
  ordering?: FilterValueOrdering,
): Pf2eTerminalFilterValuePolicy<string> {
  const exclude = [...new Set((policy?.exclude ?? []).map((value) => String(value).trim()).filter(Boolean))];
  const all = [...new Set((policy?.all ?? []).map((value) => String(value).trim()).filter(Boolean))].filter(
    (value) => !exclude.includes(value),
  );
  const any = [...new Set((policy?.any ?? []).map((value) => String(value).trim()).filter(Boolean))].filter(
    (value) => !exclude.includes(value) && !all.includes(value),
  );

  return {
    any: orderStringValues(any, ordering),
    all: orderStringValues(all, ordering),
    exclude: orderStringValues(exclude, ordering),
  };
}

export function normalizeNumberPolicy(
  policy: Partial<Pf2eTerminalFilterValuePolicy<number>> | undefined,
): Pf2eTerminalFilterValuePolicy<number> {
  const exclude = [...new Set((policy?.exclude ?? []).filter((value) => Number.isFinite(value)))].sort(
    (left, right) => left - right,
  );
  const all = [...new Set((policy?.all ?? []).filter((value) => Number.isFinite(value)))]
    .filter((value) => !exclude.includes(value))
    .sort((left, right) => left - right);
  const any = [...new Set((policy?.any ?? []).filter((value) => Number.isFinite(value)))]
    .filter((value) => !exclude.includes(value) && !all.includes(value))
    .sort((left, right) => left - right);

  return { any, all, exclude };
}

export function normalizeQueryFieldPolicy(
  field: Pf2eTerminalFacetField,
  policy: Partial<Pf2eTerminalFilterValuePolicy<string>> | undefined,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalFilterValuePolicy<string> | null {
  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (!["set", "enumString", "boolean"].includes(fieldSemantics.fieldType)) {
    return null;
  }

  const normalizedPolicy = normalizeStringPolicy(policy, fieldSemantics.valueOrdering);
  if (fieldSemantics.fieldType !== "set") {
    normalizedPolicy.all = [];
  }
  if (fieldSemantics.fieldType === "boolean") {
    normalizedPolicy.any = normalizedPolicy.any.filter((value) => value === "true" || value === "false");
    normalizedPolicy.exclude = normalizedPolicy.exclude.filter((value) => value === "true" || value === "false");
  }

  if (normalizedPolicy.any.length === 0 && normalizedPolicy.all.length === 0 && normalizedPolicy.exclude.length === 0) {
    return null;
  }

  return normalizedPolicy;
}

export function mergeStringPolicies(
  left: Pf2eTerminalFilterValuePolicy<string>,
  right: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalFilterValuePolicy<string> {
  return {
    any: [...left.any, ...right.any],
    all: [...left.all, ...right.all],
    exclude: [...left.exclude, ...right.exclude],
  };
}

export function mergeSelectionMaps(
  target: Pf2eTerminalQueryFieldSelectionMap,
  source: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = { ...target };

  for (const [field, policy] of Object.entries(source)) {
    next[field] = field in next ? mergeStringPolicies(next[field]!, policy) : { ...policy };
  }

  return next;
}

export function createScopedSelectionMap(scopedFields: string[]): Pf2eTerminalQueryFieldSelectionMap {
  return Object.fromEntries(
    scopedFields.map((field) => [field, createEmptyStringPolicy()]),
  ) as Pf2eTerminalQueryFieldSelectionMap;
}
