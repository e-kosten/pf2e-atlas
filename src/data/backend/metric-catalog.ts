import type { DatabaseSync } from "node:sqlite";

import { normalizeActorMetricKey, normalizeActorMetricPrefix } from "../../domain/actor-metrics.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../domain/categories.js";
import { normalizeItemMetricKey, normalizeItemMetricPrefix } from "../../domain/item-metrics.js";
import type {
  FilterValueQuery,
  FilterValueResult,
  SearchCategory,
  SearchSubcategory,
} from "../../domain/search-types.js";

const CATEGORY_WIDE_SUBCATEGORY = "*";
type MetricField = "actorMetrics" | "itemMetrics";

type MetricCatalogScope = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
};

function isMetricField(field: FilterValueQuery["field"]): field is MetricField {
  return field === "actorMetrics" || field === "itemMetrics";
}

function normalizeMetricPrefix(field: MetricField, prefix: string | undefined): string | null {
  if (!prefix) {
    return null;
  }
  return field === "actorMetrics" ? normalizeActorMetricPrefix(prefix) : normalizeItemMetricPrefix(prefix);
}

function normalizeMetricKey(field: MetricField, metric: string): string {
  return field === "actorMetrics" ? normalizeActorMetricKey(metric) : normalizeItemMetricKey(metric);
}

function resolveCatalogScopes(query: FilterValueQuery): MetricCatalogScope[] {
  if (query.scopes && query.scopes.length > 0) {
    return query.scopes.flatMap((scope): MetricCatalogScope[] => {
      const category = normalizeSearchCategory(scope.category);
      if (!category) {
        return [];
      }
      const subcategories = scope.subcategories ?? [];
      if (subcategories.length === 0) {
        return [{ category, subcategory: null }];
      }
      return subcategories
        .map((subcategory) => normalizeSearchSubcategory(subcategory))
        .filter((subcategory): subcategory is SearchSubcategory => Boolean(subcategory))
        .map((subcategory) => ({ category, subcategory }));
    });
  }

  const category = query.category ? normalizeSearchCategory(query.category) : null;
  if (!category) {
    return [];
  }
  const subcategory = query.subcategory ? normalizeSearchSubcategory(query.subcategory) : null;
  if (query.subcategory && !subcategory) {
    return [];
  }
  return [{ category, subcategory }];
}

function subcategoryCatalogValue(subcategory: SearchSubcategory | null): string {
  return subcategory ?? CATEGORY_WIDE_SUBCATEGORY;
}

export class Pf2eMetricCatalogBackendService {
  constructor(private readonly db: DatabaseSync) {}

  listMetricKeys(query: FilterValueQuery): FilterValueResult | null {
    if (!isMetricField(query.field) || query.metric) {
      return null;
    }

    const scopes = resolveCatalogScopes(query);
    if (scopes.length === 0) {
      return { field: query.field, values: [] };
    }

    const prefix = normalizeMetricPrefix(query.field, query.metricPrefix);
    const scopeClauses = scopes.map(() => "(category = ? AND subcategory = ?)").join(" OR ");
    const params = scopes.flatMap((scope) => [scope.category, subcategoryCatalogValue(scope.subcategory)]);
    if (prefix !== null) {
      params.push(prefix);
    }

    const rows = this.db
      .prepare(
        `
        SELECT metric_key AS value, SUM(catalog_count) AS count
        FROM metric_key_catalog
        WHERE metric_field = ?
          AND (${scopeClauses})
          ${prefix !== null ? "AND namespace_prefix = ?" : ""}
        GROUP BY metric_key
        ORDER BY count DESC, value ASC
      `,
      )
      .all(query.field, ...params) as Array<{ value: string; count: number }>;

    return {
      field: query.field,
      values: rows.map((row) => ({ value: row.value, count: Number(row.count) })),
    };
  }

  listMetricValues(query: FilterValueQuery): FilterValueResult | null {
    if (!isMetricField(query.field) || !query.metric) {
      return null;
    }

    const scopes = resolveCatalogScopes(query);
    if (scopes.length === 0) {
      return { field: query.field, values: [] };
    }

    const metric = normalizeMetricKey(query.field, query.metric);
    const scopeClauses = scopes.map(() => "(category = ? AND subcategory = ?)").join(" OR ");
    const params = scopes.flatMap((scope) => [scope.category, subcategoryCatalogValue(scope.subcategory)]);

    const rows = this.db
      .prepare(
        `
        SELECT value, SUM(catalog_count) AS count
        FROM metric_value_catalog
        WHERE metric_field = ?
          AND metric_key = ?
          AND (${scopeClauses})
        GROUP BY value
        ORDER BY count DESC, value ASC
      `,
      )
      .all(query.field, metric, ...params) as Array<{ value: string; count: number }>;

    return {
      field: query.field,
      values: rows.map((row) => ({ value: row.value, count: Number(row.count) })),
    };
  }
}
