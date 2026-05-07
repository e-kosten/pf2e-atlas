import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../domain/categories.js";
import { createSearchFilterDiscoveryContext } from "../../domain/search-field-domains.js";
import type { SearchFilterDiscoveryMode, SearchFilterDiscoveryOption } from "../../domain/search-field-domains.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import type { FilterValueField, SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { createScopedSearchDiscoveryApplicability } from "./applicability.js";
import { normalizeDiscoveryTargetField } from "./options.js";
import type { SearchDiscoveryMetricCatalog } from "./metric-discovery.js";
import type { SearchDiscoveryValueResolver } from "./value-discovery.js";
import type { SearchDiscoveryField, SearchDiscoveryScope, SearchSemanticsDiscoveryReader } from "./types.js";

export function createCatalogSearchSemanticsReader(options: {
  metricCatalog: SearchDiscoveryMetricCatalog;
  valueResolver: Pick<SearchDiscoveryValueResolver, "discoverCatalogFilterValues">;
}): SearchSemanticsDiscoveryReader {
  const { metricCatalog, valueResolver } = options;
  return {
    scope: null,
    mode: "catalog",
    discoverFieldValues: ({ category, subcategory, field }) =>
      valueResolver.discoverCatalogFilterValues({
        applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
        target: { field },
      }).options,
    discoverFieldValuesAsync: ({ category, subcategory, field }) =>
      Promise.resolve(
        valueResolver.discoverCatalogFilterValues({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          target: { field },
        }).options,
      ),
    discoverMetricKeys: ({ category, subcategory, metricField, metricPrefix }) =>
      Promise.resolve(
        metricCatalog.discoverMetricKeys({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          metricField,
          metricPrefix,
        }),
      ),
    discoverMetricValues: ({ category, subcategory, metricField, metricKey }) =>
      Promise.resolve(
        metricCatalog.discoverMetricValues({
          applicability: createScopedSearchDiscoveryApplicability("browse", category, subcategory),
          metricField,
          metricKey,
        }),
      ),
  };
}

export async function prepareSearchSemanticsReader(options: {
  request: Readonly<SearchRequest>;
  mode: SearchFilterDiscoveryMode;
  targetFields?: readonly string[];
  discoverFieldOptions: SearchDiscoveryValueResolver["discoverFieldOptions"];
  getScopedMetadataFields: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryField[];
}): Promise<SearchSemanticsDiscoveryReader> {
  const context = createSearchFilterDiscoveryContext(options.request);
  const [scopeEntry] = context.applicability.scopes;
  const category = scopeEntry ? normalizeSearchCategory(scopeEntry.category) : null;
  const subcategory =
    scopeEntry?.subcategory === undefined || scopeEntry.subcategory === null
      ? null
      : normalizeSearchSubcategory(scopeEntry.subcategory);
  const scope =
    scopeEntry && category && (scopeEntry.subcategory === undefined || scopeEntry.subcategory === null || subcategory)
      ? ({ category, subcategory } satisfies SearchDiscoveryScope)
      : null;
  if (!scope) {
    return {
      scope: null,
      mode: options.mode,
      discoverFieldValues: () => [],
      discoverFieldValuesAsync: () => Promise.resolve([]),
      discoverMetricKeys: () => Promise.resolve([]),
      discoverMetricValues: () => Promise.resolve([]),
    };
  }
  const preparedScope = scope;

  const targetFields = options.targetFields
    ? new Set(options.targetFields.map((field) => normalizeDiscoveryTargetField(field)))
    : null;
  const fieldValuesByField = new Map<string, readonly SearchFilterDiscoveryOption[]>();
  const fieldValuePromisesByField = new Map<string, Promise<readonly SearchFilterDiscoveryOption[]>>();
  const eagerlyDiscoveredField = targetFields
    ? targetFields.size === 1
      ? targetFields.values().next().value
      : null
    : "derivedTags";
  const eagerFields = options
    .getScopedMetadataFields(scope)
    .filter((field) => field.discoverable)
    .filter((field) => field.field === eagerlyDiscoveredField);

  await Promise.all(
    eagerFields.map(async (field) => {
      const discoveredOptions = await options.discoverFieldOptions({
        mode: options.mode,
        context,
        field: field.field as FilterValueField,
      });
      fieldValuesByField.set(field.field, discoveredOptions);
    }),
  );

  function matchesScope(category: SearchCategory, subcategory: SearchSubcategory | null): boolean {
    return category === preparedScope.category && (subcategory ?? null) === preparedScope.subcategory;
  }

  function includesTargetField(field: string): boolean {
    return !targetFields || targetFields.has(field);
  }

  function loadFieldValues(field: string): Promise<readonly SearchFilterDiscoveryOption[]> {
    const cached = fieldValuesByField.get(field);
    if (cached) {
      return Promise.resolve(cached);
    }
    const existing = fieldValuePromisesByField.get(field);
    if (existing) {
      return existing;
    }
    const promise = options
      .discoverFieldOptions({
        mode: options.mode,
        context,
        field: field as FilterValueField,
      })
      .then((discoveredOptions) => {
        fieldValuesByField.set(field, discoveredOptions);
        fieldValuePromisesByField.delete(field);
        return discoveredOptions;
      });
    fieldValuePromisesByField.set(field, promise);
    void promise.catch(() => {
      if (fieldValuePromisesByField.get(field) === promise) {
        fieldValuePromisesByField.delete(field);
      }
    });
    return promise;
  }

  return {
    scope: preparedScope,
    mode: options.mode,
    discoverFieldValues: ({ category, subcategory, field }) =>
      matchesScope(category, subcategory) && includesTargetField(field) ? (fieldValuesByField.get(field) ?? []) : [],
    discoverFieldValuesAsync: ({ category, subcategory, field }) =>
      matchesScope(category, subcategory) && includesTargetField(field) ? loadFieldValues(field) : Promise.resolve([]),
    discoverMetricKeys: ({ category, subcategory, metricField, metricPrefix }) =>
      matchesScope(category, subcategory) && includesTargetField(metricField)
        ? options.discoverFieldOptions({
            mode: options.mode,
            context,
            field: metricField,
            metricPrefix,
          })
        : Promise.resolve([]),
    discoverMetricValues: ({ category, subcategory, metricField, metricKey }) =>
      matchesScope(category, subcategory) && includesTargetField(metricField)
        ? options.discoverFieldOptions({
            mode: options.mode,
            context,
            field: metricField,
            metric: metricKey,
          })
        : Promise.resolve([]),
  };
}
