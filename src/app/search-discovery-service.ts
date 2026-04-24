import type { Pf2eDataService } from "../data/service.js";
import { orderFilterValues, type FilterValueOrdering } from "../domain/filter-value-ordering.js";
import type { MetadataFieldName, MetadataFieldType } from "../domain/metadata-field-types.js";
import type { SearchFilterDiscoveryApplicability, SearchFilterDiscoveryOption, SearchFilterDiscoveryRequest, SearchFilterDiscoveryResult, SearchPromotedFieldDomainKey } from "../domain/search-field-domains.js";
import type { SearchRequestMode } from "../domain/search-request-types.js";
import type { FilterValueField, FilterValueQuery, SearchCategory, SearchCategoryInput, SearchScope, SearchSubcategory, SearchSubcategoryInput } from "../domain/search-types.js";
import { getMetadataFilterSemantics } from "../search/filters/semantics.js";

type SearchDiscoveryDataService = Pick<Pf2eDataService, "getPack" | "listFilterValues">;

export type SearchDiscoveryScope = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
};

export type SearchDiscoveryField = {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  discoverable: boolean;
  notes?: string;
  subcategories?: readonly SearchSubcategory[];
  valueOrdering?: FilterValueOrdering;
};

export type SearchDiscoveryMetricGroup = {
  metricField: "actorMetrics" | "itemMetrics";
  metadataField: "actorMetric" | "itemMetric";
  namespaces: ReadonlyArray<{ prefix: string; description: string }>;
};

export type Pf2eApplicationSearchDiscoveryService = {
  discoverFilterValues: (request: SearchFilterDiscoveryRequest) => SearchFilterDiscoveryResult;
  discoverMetricKeys: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }) => readonly SearchFilterDiscoveryOption[];
  discoverMetricValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }) => readonly SearchFilterDiscoveryOption[];
  getMetricDiscoveryGroups: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryMetricGroup[];
  getScopedMetadataFields: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryField[];
  isPromotedFieldAvailable: (
    field: SearchPromotedFieldDomainKey,
    applicability: SearchFilterDiscoveryApplicability,
  ) => boolean;
  resolvePackName: (packValue: string) => string | undefined;
};

export function createScopedSearchDiscoveryApplicability(
  mode: SearchRequestMode,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
  options: { pack?: string } = {},
): SearchFilterDiscoveryApplicability {
  return {
    mode,
    ...(options.pack ? { pack: options.pack } : {}),
    scopes: category ? [{ category, ...(subcategory ? { subcategory } : {}) }] : [],
  };
}

function buildDiscoveryCacheKey(parts: ReadonlyArray<string | null | undefined>): string {
  return parts.map((part) => part ?? "").join("|");
}

function buildFilterValueQuery(
  field: FilterValueField,
  applicability: SearchFilterDiscoveryApplicability,
  extras: { metric?: string; metricPrefix?: string } = {},
): FilterValueQuery {
  const query: FilterValueQuery = {
    field,
    ...(extras.metric ? { metric: extras.metric } : {}),
    ...(extras.metricPrefix ? { metricPrefix: extras.metricPrefix } : {}),
  };
  const [firstScope] = applicability.scopes;

  if (applicability.scopes.length === 1 && firstScope) {
    query.category = firstScope.category as SearchCategoryInput;
    if (firstScope.subcategory) {
      query.subcategory = firstScope.subcategory as SearchSubcategoryInput;
    }
    return query;
  }

  if (applicability.scopes.length > 1) {
    query.scopes = applicability.scopes.map((scope) => ({
      category: scope.category,
      ...(scope.subcategory ? { subcategory: scope.subcategory } : {}),
    })) as SearchScope[];
  }

  return query;
}

export function createPf2eApplicationSearchDiscoveryService(
  dataService: SearchDiscoveryDataService,
): Pf2eApplicationSearchDiscoveryService {
  const semantics = getMetadataFilterSemantics();
  const metadataFieldsByName = new Map<MetadataFieldName, SearchDiscoveryField>(
    semantics.metadataFields.map((field) => [
      field.field,
      {
        field: field.field,
        fieldType: field.fieldType,
        discoverable: field.discoverable,
        notes: field.notes,
        subcategories: field.subcategories,
        valueOrdering: field.valueOrdering,
      },
    ]),
  );
  const scopedFieldCache = new Map<string, readonly SearchDiscoveryField[]>();
  const metricGroupCache = new Map<string, readonly SearchDiscoveryMetricGroup[]>();
  const filterValueCache = new Map<string, SearchFilterDiscoveryResult>();
  const metricKeyCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();
  const metricValueCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();

  function getFieldValueOrdering(field: string): FilterValueOrdering | undefined {
    return metadataFieldsByName.get(field as MetadataFieldName)?.valueOrdering;
  }

  function getScopedMetadataFields(scope: SearchDiscoveryScope | null): readonly SearchDiscoveryField[] {
    if (!scope) {
      return [];
    }

    const cacheKey = buildDiscoveryCacheKey([scope.category, scope.subcategory ?? "all"]);
    const cached = scopedFieldCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const fields = (semantics.metadataFieldsByCategory[scope.category] ?? [])
      .map((field) => metadataFieldsByName.get(field))
      .filter((field): field is SearchDiscoveryField => Boolean(field))
      .filter(
        (field) => !scope.subcategory || !field.subcategories || field.subcategories.includes(scope.subcategory),
      );
    scopedFieldCache.set(cacheKey, fields);
    return fields;
  }

  function getMetricDiscoveryGroups(scope: SearchDiscoveryScope | null): readonly SearchDiscoveryMetricGroup[] {
    if (!scope) {
      return [];
    }

    const cacheKey = buildDiscoveryCacheKey([scope.category, scope.subcategory ?? "all"]);
    const cached = metricGroupCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const groups: SearchDiscoveryMetricGroup[] = [];
    if (
      semantics.actorMetricDiscovery &&
      semantics.advancedPredicates.some(
        (predicate) => predicate.name === "actorMetric" && predicate.categories.includes(scope.category),
      )
    ) {
      groups.push({
        metricField: "actorMetrics",
        metadataField: "actorMetric",
        namespaces: semantics.actorMetricDiscovery.namespaces,
      });
    }
    if (
      semantics.itemMetricDiscovery &&
      semantics.advancedPredicates.some(
        (predicate) => predicate.name === "itemMetric" && predicate.categories.includes(scope.category),
      )
    ) {
      groups.push({
        metricField: "itemMetrics",
        metadataField: "itemMetric",
        namespaces: semantics.itemMetricDiscovery.namespaces,
      });
    }

    metricGroupCache.set(cacheKey, groups);
    return groups;
  }

  function discoverFilterValues(request: SearchFilterDiscoveryRequest): SearchFilterDiscoveryResult {
    const cacheKey = buildDiscoveryCacheKey([
      request.mode,
      request.applicability.mode,
      request.applicability.pack,
      ...request.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      request.target.field,
    ]);
    const cached = filterValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(request.target.field as FilterValueField, request.applicability);
    const values = orderFilterValues(
      dataService.listFilterValues(query).values,
      getFieldValueOrdering(request.target.field),
    );
    const result: SearchFilterDiscoveryResult = {
      mode: request.mode,
      target: request.target,
      options: values.map((entry) => ({
        id: entry.value,
        value: entry.value,
        count: entry.count,
      })),
    };
    filterValueCache.set(cacheKey, result);
    return result;
  }

  function discoverMetricKeys(options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricPrefix?: string;
  }): readonly SearchFilterDiscoveryOption[] {
    const cacheKey = buildDiscoveryCacheKey([
      options.applicability.mode,
      options.applicability.pack,
      ...options.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      options.metricField,
      options.metricPrefix ?? "",
    ]);
    const cached = metricKeyCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(options.metricField, options.applicability, {
      metricPrefix: options.metricPrefix,
    });
    const result = dataService.listFilterValues(query).values.map((entry) => ({
      id: entry.value,
      value: entry.value,
      count: entry.count,
    }));
    metricKeyCache.set(cacheKey, result);
    return result;
  }

  function discoverMetricValues(options: {
    applicability: SearchFilterDiscoveryApplicability;
    metricField: "actorMetrics" | "itemMetrics";
    metricKey: string;
  }): readonly SearchFilterDiscoveryOption[] {
    const cacheKey = buildDiscoveryCacheKey([
      options.applicability.mode,
      options.applicability.pack,
      ...options.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      options.metricField,
      options.metricKey,
    ]);
    const cached = metricValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(options.metricField, options.applicability, {
      metric: options.metricKey,
    });
    const result = dataService.listFilterValues(query).values.map((entry) => ({
      id: entry.value,
      value: entry.value,
      count: entry.count,
    }));
    metricValueCache.set(cacheKey, result);
    return result;
  }

  return {
    discoverFilterValues,
    discoverMetricKeys,
    discoverMetricValues,
    getMetricDiscoveryGroups,
    getScopedMetadataFields,
    isPromotedFieldAvailable: (field, applicability) =>
      discoverFilterValues({
        mode: "catalog",
        applicability,
        target: { field },
      }).options.length > 0,
    resolvePackName: (packValue) => dataService.getPack(packValue)?.name,
  };
}
