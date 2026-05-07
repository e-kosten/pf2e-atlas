import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../domain/categories.js";
import {
  buildSearchFilterDiscoveryCatalogRequest,
  type SearchFilterDiscoveryApplicability,
  type SearchFilterDiscoveryContext,
  type SearchFilterDiscoveryMode,
} from "../../domain/search-field-domains.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import type { FilterValueField, FilterValueQuery } from "../../domain/search-types.js";

export function resolveContextSearchRequest(
  mode: SearchFilterDiscoveryMode,
  context: SearchFilterDiscoveryContext,
): Readonly<SearchRequest> {
  if (mode === "catalog") {
    return buildSearchFilterDiscoveryCatalogRequest(context.applicability);
  }

  const request = context.request;
  if ((request.mode === "search" || request.mode === "lookup") && !request.filter && !request.search.query.trim()) {
    return buildSearchFilterDiscoveryCatalogRequest(context.applicability);
  }

  return request;
}

export function buildFilterValueQuery(
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
    const category = normalizeSearchCategory(firstScope.category);
    if (category) {
      query.category = category;
    }
    if (firstScope.subcategory) {
      const subcategory = normalizeSearchSubcategory(firstScope.subcategory);
      if (subcategory) {
        query.subcategory = subcategory;
      }
    }
    return query;
  }

  if (applicability.scopes.length > 1) {
    query.scopes = applicability.scopes.flatMap((scope) => {
      const category = normalizeSearchCategory(scope.category);
      if (!category) {
        return [];
      }
      const subcategory = scope.subcategory ? normalizeSearchSubcategory(scope.subcategory) : null;
      return [
        {
          category,
          ...(subcategory ? { subcategory } : {}),
        },
      ];
    });
  }

  return query;
}
