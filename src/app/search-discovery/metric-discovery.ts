import { getMetadataFilterSemantics } from "../../domain/metadata-field-catalog.js";
import type {
  SearchFilterDiscoveryApplicability,
  SearchFilterDiscoveryOption,
} from "../../domain/search-field-domains.js";
import type { FilterValueField } from "../../domain/search-types.js";
import { buildDiscoveryCacheKey } from "./cache-keys.js";
import { mapFilterValueDiscoveryOption } from "./options.js";
import { buildFilterValueQuery } from "./query-builders.js";
import type { SearchDiscoveryDataService, SearchDiscoveryMetricGroup, SearchDiscoveryScope } from "./types.js";

export type SearchDiscoveryMetricCatalog = {
  getMetricDiscoveryGroups: (scope: SearchDiscoveryScope | null) => readonly SearchDiscoveryMetricGroup[];
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
};

export function createSearchDiscoveryMetricCatalog(
  dataService: SearchDiscoveryDataService,
): SearchDiscoveryMetricCatalog {
  const semantics = getMetadataFilterSemantics();
  const metricGroupCache = new Map<string, readonly SearchDiscoveryMetricGroup[]>();
  const metricKeyCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();
  const metricValueCache = new Map<string, readonly SearchFilterDiscoveryOption[]>();

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
    if (semantics.actorMetricDiscovery?.categories.includes(scope.category)) {
      groups.push({
        metricField: "actorMetrics",
        metadataField: "actorMetric",
        namespaces: semantics.actorMetricDiscovery.namespaces,
      });
    }
    if (semantics.itemMetricDiscovery?.categories.includes(scope.category)) {
      groups.push({
        metricField: "itemMetrics",
        metadataField: "itemMetric",
        namespaces: semantics.itemMetricDiscovery.namespaces,
      });
    }

    metricGroupCache.set(cacheKey, groups);
    return groups;
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
    const result = (
      dataService.listMetricCatalogKeys?.(query) ?? { field: options.metricField as FilterValueField, values: [] }
    ).values.map(mapFilterValueDiscoveryOption);
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
    const result = (
      dataService.listMetricCatalogValues?.(query) ?? { field: options.metricField as FilterValueField, values: [] }
    ).values.map(mapFilterValueDiscoveryOption);
    metricValueCache.set(cacheKey, result);
    return result;
  }

  return {
    discoverMetricKeys,
    discoverMetricValues,
    getMetricDiscoveryGroups,
  };
}
