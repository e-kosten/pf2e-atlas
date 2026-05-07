import { orderFilterValues } from "../../domain/filter-value-ordering.js";
import {
  type SearchFilterDiscoveryApplicability,
  type SearchFilterDiscoveryContext,
  type SearchFilterDiscoveryMode,
  type SearchFilterDiscoveryOption,
  type SearchFilterDiscoveryRequest,
  type SearchFilterDiscoveryResult,
} from "../../domain/search-field-domains.js";
import type { FilterValueField } from "../../domain/search-types.js";
import { buildDiscoveryCacheKey, buildDiscoveryContextCacheKey } from "./cache-keys.js";
import { mapFilterValueDiscoveryOption } from "./options.js";
import { buildFilterValueQuery, resolveContextSearchRequest } from "./query-builders.js";
import type { SearchDiscoveryDataService } from "./types.js";

export type SearchDiscoveryValueResolver = {
  discoverCatalogFilterValues: (options: {
    applicability: SearchFilterDiscoveryApplicability;
    target: { field: string };
  }) => SearchFilterDiscoveryResult;
  discoverFieldOptions: (options: {
    mode: SearchFilterDiscoveryMode;
    context: SearchFilterDiscoveryContext;
    field: FilterValueField;
    metric?: string;
    metricPrefix?: string;
  }) => Promise<readonly SearchFilterDiscoveryOption[]>;
  discoverFilterValues: (request: SearchFilterDiscoveryRequest) => Promise<SearchFilterDiscoveryResult>;
};

export function createSearchDiscoveryValueResolver(options: {
  dataService: SearchDiscoveryDataService;
  getFieldValueOrdering: (field: string) => Parameters<typeof orderFilterValues>[1];
}): SearchDiscoveryValueResolver {
  const { dataService, getFieldValueOrdering } = options;
  const discoverFilterValuesFromData = dataService.discoverFilterValues.bind(dataService);
  const listFilterValuesFromData = dataService.listFilterValues.bind(dataService);
  const catalogFilterValueCache = new Map<string, SearchFilterDiscoveryResult>();
  const asyncFilterValueCache = new Map<string, Promise<SearchFilterDiscoveryResult>>();
  const asyncDiscoveryOptionCache = new Map<string, Promise<readonly SearchFilterDiscoveryOption[]>>();

  function discoverCatalogFilterValues(input: {
    applicability: SearchFilterDiscoveryApplicability;
    target: { field: string };
  }): SearchFilterDiscoveryResult {
    const cacheKey = buildDiscoveryCacheKey([
      input.applicability.mode,
      input.applicability.pack,
      ...input.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
      input.target.field,
    ]);
    const cached = catalogFilterValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(input.target.field as FilterValueField, input.applicability);
    const catalogResult =
      input.target.field === "actorMetrics" || input.target.field === "itemMetrics"
        ? (dataService.listMetricCatalogKeys?.(query) ?? {
            field: input.target.field as FilterValueField,
            values: [],
          })
        : null;
    const values = orderFilterValues(
      (catalogResult ?? listFilterValuesFromData(query)).values,
      getFieldValueOrdering(input.target.field),
    );
    const result: SearchFilterDiscoveryResult = {
      mode: "catalog",
      target: input.target,
      options: values.map(mapFilterValueDiscoveryOption),
    };
    catalogFilterValueCache.set(cacheKey, result);
    return result;
  }

  async function discoverFieldOptions(input: {
    mode: SearchFilterDiscoveryMode;
    context: SearchFilterDiscoveryContext;
    field: FilterValueField;
    metric?: string;
    metricPrefix?: string;
  }): Promise<readonly SearchFilterDiscoveryOption[]> {
    const cacheKey = buildDiscoveryCacheKey([
      input.mode,
      buildDiscoveryContextCacheKey(input.context),
      input.field,
      input.metric ?? "",
      input.metricPrefix ?? "",
    ]);
    const cached = asyncDiscoveryOptionCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const query = buildFilterValueQuery(input.field, input.context.applicability, {
      metric: input.metric,
      metricPrefix: input.metricPrefix,
    });
    const resultPromise =
      input.mode === "catalog" && (input.field === "actorMetrics" || input.field === "itemMetrics")
        ? Promise.resolve(
            (input.metric
              ? dataService.listMetricCatalogValues?.(query)
              : dataService.listMetricCatalogKeys?.(query)) ?? {
              field: input.field,
              values: [],
            },
          )
        : discoverFilterValuesFromData(query, resolveContextSearchRequest(input.mode, input.context));
    const promise = resultPromise.then((result) =>
      orderFilterValues(result.values, getFieldValueOrdering(input.field)).map(mapFilterValueDiscoveryOption),
    );
    asyncDiscoveryOptionCache.set(cacheKey, promise);
    void promise.catch(() => {
      if (asyncDiscoveryOptionCache.get(cacheKey) === promise) {
        asyncDiscoveryOptionCache.delete(cacheKey);
      }
    });
    return promise;
  }

  async function discoverFilterValues(request: SearchFilterDiscoveryRequest): Promise<SearchFilterDiscoveryResult> {
    const cacheKey = buildDiscoveryCacheKey([
      request.mode,
      buildDiscoveryContextCacheKey(request.context),
      request.target.field,
    ]);
    const cached = asyncFilterValueCache.get(cacheKey);
    if (cached) {
      return cached;
    }

    const promise = discoverFieldOptions({
      mode: request.mode,
      context: request.context,
      field: request.target.field as FilterValueField,
    }).then(
      (discoveredOptions): SearchFilterDiscoveryResult => ({
        mode: request.mode,
        target: request.target,
        options: [...discoveredOptions],
      }),
    );
    asyncFilterValueCache.set(cacheKey, promise);
    void promise.catch(() => {
      if (asyncFilterValueCache.get(cacheKey) === promise) {
        asyncFilterValueCache.delete(cacheKey);
      }
    });
    return promise;
  }

  return {
    discoverCatalogFilterValues,
    discoverFieldOptions,
    discoverFilterValues,
  };
}
