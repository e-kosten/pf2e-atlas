import {
  inferActorMetricValueType,
  normalizeActorMetricKey,
  normalizeActorMetricPrefix,
} from "../domain/actor-metrics.js";
import {
  inferItemMetricValueType,
  normalizeItemMetricKey,
  normalizeItemMetricPrefix,
} from "../domain/item-metrics.js";
import {
  getMetadataFieldSpec,
  getMetadataRecordSelectClauses,
  isMetadataFieldName,
  type MetadataFilterValueSource,
} from "../domain/metadata-field-registry.js";
import {
  FilterValueQuery,
  NormalizedRecord,
  SearchCategory,
  SearchScope,
  SearchSubcategory,
} from "../types.js";
import {
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import type { NormalizedSearchFilters, NormalizedSearchScope, SqlValue } from "../data/service-types.js";
import { appendMetadataFilterClauses, recordMatchesMetadataFilter } from "./metadata-filters.js";
import { normalizeText, uniqueSorted } from "../utils.js";

function appendWhereClause(sql: string[], params: SqlValue[], clause: string, ...values: SqlValue[]): void {
  sql.push(clause);
  params.push(...values);
}

export function normalizeSearchScope(scope: SearchScope): NormalizedSearchScope {
  const category = normalizeSearchCategory(scope.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(scope.category)));
  }

  const subcategories = scope.subcategories?.map((subcategory) => {
    const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
    if (!canonicalSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(subcategory)));
    }
    return canonicalSubcategory;
  });

  const uniqueSubcategories = subcategories
    ? uniqueSorted(subcategories) as SearchSubcategory[]
    : undefined;

  return {
    category,
    subcategories: uniqueSubcategories && uniqueSubcategories.length > 0 ? uniqueSubcategories : undefined,
  };
}

export function resolveEffectiveCategory(filters: Pick<NormalizedSearchFilters, "category" | "subcategory" | "scopes">): SearchCategory | null {
  if (filters.scopes && filters.scopes.length > 0) {
    return null;
  }

  const inferredCategoryFromSubcategory = !filters.category && filters.subcategory
    ? getCategoryForSubcategory(filters.subcategory)
    : null;
  return filters.category ?? inferredCategoryFromSubcategory;
}

function appendScopedCategoryClauses(
  sql: string[],
  params: SqlValue[],
  scopes: NormalizedSearchScope[],
  renderTerm: (category: SearchCategory, subcategories: SearchSubcategory[] | undefined) => { clause: string; values: SqlValue[] },
): void {
  const renderedScopes = scopes.map((scope) => renderTerm(scope.category, scope.subcategories));
  appendWhereClause(
    sql,
    params,
    `AND (${renderedScopes.map((entry) => entry.clause).join(" OR ")})`,
    ...renderedScopes.flatMap((entry) => entry.values),
  );
}

function applySearchFilterClauses(
  sql: string[],
  params: SqlValue[],
  filters: NormalizedSearchFilters,
  aliases: {
    records: string;
    actor: string;
    item: string;
    spell: string;
  },
  options: {
    recordKeys?: string[];
  } = {},
): void {
  const recordAlias = aliases.records;
  const actorAlias = aliases.actor;
  const itemAlias = aliases.item;
  const spellAlias = aliases.spell;

  appendWhereClause(sql, params, `AND ${recordAlias}.is_search_canonical = 1`);

  if (options.recordKeys && options.recordKeys.length > 0) {
    const placeholders = options.recordKeys.map(() => "?").join(", ");
    appendWhereClause(sql, params, `AND ${recordAlias}.record_key IN (${placeholders})`, ...options.recordKeys);
  }

  if (filters.pack) {
    appendWhereClause(
      sql,
      params,
      `AND (LOWER(${recordAlias}.pack_name) = LOWER(?) OR LOWER(${recordAlias}.pack_label) = LOWER(?))`,
      filters.pack,
      filters.pack,
    );
  }

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: `LOWER(${recordAlias}.category) = LOWER(?)`,
          values: [category],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(LOWER(${recordAlias}.category) = LOWER(?) AND LOWER(COALESCE(${recordAlias}.subcategory, '')) IN (${placeholders}))`,
        values: [category, ...subcategories.map((subcategory) => normalizeText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, `AND LOWER(${recordAlias}.category) = LOWER(?)`, effectiveCategory);
    }

    if (filters.subcategory) {
      appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.subcategory, '')) = LOWER(?)`, filters.subcategory);
    }
  }

  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level >= ?`, filters.levelMin);
  }

  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, `AND ${recordAlias}.level <= ?`, filters.levelMax);
  }

  if (filters.rarity) {
    appendWhereClause(sql, params, `AND LOWER(COALESCE(${recordAlias}.rarity, '')) = LOWER(?)`, filters.rarity);
  }

  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp >= ?`, filters.priceMin);
  }

  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, `AND ${itemAlias}.price_cp <= ?`, filters.priceMax);
  }

  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, `AND COALESCE(${spellAlias}.action_cost, ${itemAlias}.action_cost) = ?`, filters.actionCost);
  }

  appendMetadataFilterClauses(sql, params, filters.metadata, {
    recordKeyExpr: `${recordAlias}.record_key`,
    recordsAlias: recordAlias,
    actorAlias,
    itemAlias,
    spellAlias,
  }, appendWhereClause);
}

const BASE_RECORD_SELECT_FIELDS = [
  "r.record_key AS recordKey",
  "r.id AS id",
  "r.name AS name",
  "r.normalized_name AS normalizedName",
  "r.record_type AS type",
  "r.category AS category",
  "r.subcategory AS subcategory",
  "r.pack_name AS packName",
  "r.pack_label AS packLabel",
  "r.document_type AS documentType",
  "r.description_text AS descriptionText",
  "r.description_snippet AS descriptionSnippet",
  "r.folder_id AS folderId",
  "r.variant_confidence AS variantConfidence",
  "r.variant_source AS variantSource",
  "r.source_path AS sourcePath",
  "r.is_search_canonical AS isSearchCanonical",
  `COALESCE((
    SELECT json_group_array(json_object(
      'metricKey', am.metric_key,
      'valueType', am.value_type,
      'numberValue', am.number_value,
      'textValue', am.text_value,
      'boolValue', am.bool_value
    ))
    FROM actor_metrics am
    WHERE am.record_key = r.record_key
  ), '[]') AS actorMetricsJson`,
  `COALESCE((
    SELECT json_group_array(json_object(
      'metricKey', im.metric_key,
      'valueType', im.value_type,
      'numberValue', im.number_value,
      'textValue', im.text_value,
      'boolValue', im.bool_value
    ))
    FROM item_metrics im
    WHERE im.record_key = r.record_key
  ), '[]') AS itemMetricsJson`,
] as const;

export function buildSharedRecordSelectFields(
  includeSearchText = false,
  includeEmbedding = false,
): string[] {
  const fields = [
    ...BASE_RECORD_SELECT_FIELDS,
    ...getMetadataRecordSelectClauses(),
  ];

  if (includeSearchText) {
    fields.push("r.search_text AS searchText");
  }

  if (includeEmbedding) {
    fields.push("e.vector_blob AS embeddingBlob");
  }

  return fields;
}

function applyMetadataFilterValueSource(
  field: FilterValueQuery["field"],
  joins: string[],
  postFilterClauses: string[],
): MetadataFilterValueSource | null {
  if (!isMetadataFieldName(field)) {
    return null;
  }

  const source = getMetadataFieldSpec(field).buildFilterValueSource?.() ?? null;
  if (!source) {
    return null;
  }

  joins.push(...(source.joins ?? []));
  if (source.nonEmptyClause) {
    postFilterClauses.push(source.nonEmptyClause);
  }
  return source;
}

export function buildCandidateQuery(
  filters: NormalizedSearchFilters,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  const fields = buildSharedRecordSelectFields(includeSearchText, includeEmbedding);

  const sql = [
    `SELECT ${fields.join(", ")}`,
    "FROM records r",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];

  if (includeEmbedding) {
    sql.push("LEFT JOIN embeddings e ON e.record_key = r.record_key");
  }

  sql.push("WHERE 1 = 1");
  const params: SqlValue[] = [];
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  }, options);

  return { sql: sql.join("\n"), params };
}

export function buildFilterValueQuery(query: FilterValueQuery, filters: NormalizedSearchFilters): { sql: string; params: SqlValue[] } {
  const { field } = query;
  if (field !== "actorMetrics" && field !== "itemMetrics" && (query.metricPrefix || query.metric)) {
    throw new Error("metricPrefix and metric are only supported when field is actorMetrics or itemMetrics.");
  }

  const joins = [
    "FROM records r",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];
  const sql: string[] = [];
  const params: SqlValue[] = [];
  const postFilterClauses: string[] = [];
  const postFilterParams: SqlValue[] = [];
  let valueExpression = "";

  const metadataSource = applyMetadataFilterValueSource(field, joins, postFilterClauses);
  if (metadataSource) {
    valueExpression = metadataSource.valueExpression;
  } else {
    switch (field) {
      case "actorMetrics":
      joins.push("JOIN actor_metrics am ON am.record_key = r.record_key");
      if (query.metric) {
        const normalizedMetric = normalizeActorMetricKey(query.metric);
        const metricType = inferActorMetricValueType(normalizedMetric);
        if (!metricType) {
          throw new Error(`Unknown actor metric "${query.metric}".`);
        }

        postFilterParams.push(normalizedMetric);
        postFilterClauses.push("AND am.metric_key = ?");
        if (metricType === "text") {
          valueExpression = "am.text_value";
          postFilterClauses.push("AND am.value_type = 'text' AND am.text_value IS NOT NULL AND am.text_value <> ''");
        } else if (metricType === "boolean") {
          valueExpression = "CASE am.bool_value WHEN 1 THEN 'true' ELSE 'false' END";
          postFilterClauses.push("AND am.value_type = 'boolean' AND am.bool_value IS NOT NULL");
        } else {
          throw new Error("actorMetrics value listing only supports text and boolean metrics.");
        }
      } else {
        valueExpression = "am.metric_key";
        if (query.metricPrefix) {
          postFilterParams.push(`${normalizeActorMetricPrefix(query.metricPrefix)}%`);
          postFilterClauses.push("AND am.metric_key LIKE ?");
        }
      }
      break;
      case "itemMetrics":
      joins.push("JOIN item_metrics im ON im.record_key = r.record_key");
      if (query.metric) {
        const normalizedMetric = normalizeItemMetricKey(query.metric);
        const metricType = inferItemMetricValueType(normalizedMetric);
        if (!metricType) {
          throw new Error(`Unknown item metric "${query.metric}".`);
        }

        postFilterParams.push(normalizedMetric);
        postFilterClauses.push("AND im.metric_key = ?");
        if (metricType === "text") {
          valueExpression = "im.text_value";
          postFilterClauses.push("AND im.value_type = 'text' AND im.text_value IS NOT NULL AND im.text_value <> ''");
        } else if (metricType === "boolean") {
          valueExpression = "CASE im.bool_value WHEN 1 THEN 'true' ELSE 'false' END";
          postFilterClauses.push("AND im.value_type = 'boolean' AND im.bool_value IS NOT NULL");
        } else {
          throw new Error("itemMetrics value listing only supports text and boolean metrics.");
        }
      } else {
        valueExpression = "im.metric_key";
        if (query.metricPrefix) {
          postFilterParams.push(`${normalizeItemMetricPrefix(query.metricPrefix)}%`);
          postFilterClauses.push("AND im.metric_key LIKE ?");
        }
      }
      break;
      case "sources":
        valueExpression = "r.source_category";
        break;
      case "categories":
        valueExpression = "r.category";
        break;
      case "subcategories":
        valueExpression = "r.subcategory";
        postFilterClauses.push("AND r.subcategory IS NOT NULL AND r.subcategory <> ''");
        break;
      case "packs":
        valueExpression = "r.pack_label";
        postFilterClauses.push("AND r.pack_label IS NOT NULL AND r.pack_label <> ''");
        break;
    }
  }

  sql.push(`SELECT ${valueExpression} AS value, COUNT(*) AS count`);
  sql.push(...joins);
  sql.push("WHERE 1 = 1");
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push(...postFilterClauses);
  params.push(...postFilterParams);
  sql.push("GROUP BY value");
  sql.push("ORDER BY count DESC, value ASC");
  return { sql: sql.join("\n"), params };
}

export function buildLexicalRetrievalQuery(filters: NormalizedSearchFilters, query: string, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT r.record_key AS recordKey, bm25(records_fts, 8.0, 1.5) AS rank",
    "FROM records_fts",
    "JOIN records r ON r.record_key = records_fts.record_key",
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    "WHERE records_fts MATCH ?",
  ];
  const params: SqlValue[] = [query];
  applySearchFilterClauses(sql, params, filters, {
    records: "r",
    actor: "a",
    item: "i",
    spell: "s",
  });
  sql.push("ORDER BY rank");
  sql.push("LIMIT ?");
  params.push(limit);
  return { sql: sql.join("\n"), params };
}

export function semanticQueryLimit(baseLimit: number, filters: NormalizedSearchFilters): number {
  return filters.metadata ? Math.min(1000, Math.max(baseLimit * 2, baseLimit + 50)) : baseLimit;
}

function normalizeVectorText(value: string | null | undefined): string {
  return normalizeText(value ?? "") || "";
}

export function buildSemanticRetrievalQuery(filters: NormalizedSearchFilters, limit: number): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT record_key AS recordKey, distance",
    "FROM record_embeddings",
    "WHERE embedding MATCH ?",
    `AND k = ${limit}`,
  ];
  const params: SqlValue[] = [];

  if (filters.scopes && filters.scopes.length > 0) {
    appendScopedCategoryClauses(sql, params, filters.scopes, (category, subcategories) => {
      if (!subcategories || subcategories.length === 0) {
        return {
          clause: "category = ?",
          values: [normalizeVectorText(category)],
        };
      }

      const placeholders = subcategories.map(() => "?").join(", ");
      return {
        clause: `(category = ? AND subcategory IN (${placeholders}))`,
        values: [normalizeVectorText(category), ...subcategories.map((subcategory) => normalizeVectorText(subcategory))],
      };
    });
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory) {
      appendWhereClause(sql, params, "AND category = ?", normalizeVectorText(effectiveCategory));
    }
    if (filters.subcategory) {
      appendWhereClause(sql, params, "AND subcategory = ?", normalizeVectorText(filters.subcategory));
    }
  }
  if (filters.pack) {
    appendWhereClause(sql, params, "AND pack_name = ?", normalizeVectorText(filters.pack));
  }
  if (filters.levelMin !== undefined) {
    appendWhereClause(sql, params, "AND level >= ?", BigInt(filters.levelMin));
  }
  if (filters.levelMax !== undefined) {
    appendWhereClause(sql, params, "AND level <= ?", BigInt(filters.levelMax));
  }
  if (filters.rarity) {
    appendWhereClause(sql, params, "AND rarity = ?", normalizeVectorText(filters.rarity));
  }
  if (filters.priceMin !== undefined) {
    appendWhereClause(sql, params, "AND price_cp >= ?", BigInt(filters.priceMin));
  }
  if (filters.priceMax !== undefined) {
    appendWhereClause(sql, params, "AND price_cp <= ?", BigInt(filters.priceMax));
  }
  if (filters.actionCost !== undefined) {
    appendWhereClause(sql, params, "AND action_cost = ?", BigInt(filters.actionCost));
  }

  appendMetadataFilterClauses(sql, params, filters.metadata, {
    recordKeyExpr: "record_embeddings.record_key",
  }, appendWhereClause);

  return { sql: sql.join("\n"), params };
}

function recordMatchesScope(record: NormalizedRecord, scope: NormalizedSearchScope): boolean {
  if (record.category !== scope.category) {
    return false;
  }

  if (!scope.subcategories || scope.subcategories.length === 0) {
    return true;
  }

  return record.subcategory !== null && scope.subcategories.includes(record.subcategory);
}

export function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (filters.pack) {
    const normalizedPack = normalizeText(filters.pack);
    if (normalizeText(record.packName) !== normalizedPack && normalizeText(record.packLabel) !== normalizedPack) {
      return false;
    }
  }

  if (filters.scopes && filters.scopes.length > 0) {
    if (!filters.scopes.some((scope) => recordMatchesScope(record, scope))) {
      return false;
    }
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory && record.category !== effectiveCategory) {
      return false;
    }
    if (filters.subcategory && record.subcategory !== filters.subcategory) {
      return false;
    }
  }
  if (filters.levelMin !== undefined && (record.level === null || record.level < filters.levelMin)) {
    return false;
  }
  if (filters.levelMax !== undefined && (record.level === null || record.level > filters.levelMax)) {
    return false;
  }
  if (filters.rarity && normalizeText(record.rarity ?? "") !== normalizeText(filters.rarity)) {
    return false;
  }
  if (filters.priceMin !== undefined && (record.priceCp === null || record.priceCp < filters.priceMin)) {
    return false;
  }
  if (filters.priceMax !== undefined && (record.priceCp === null || record.priceCp > filters.priceMax)) {
    return false;
  }
  if (filters.actionCost !== undefined && record.actionCost !== filters.actionCost) {
    return false;
  }
  if (filters.metadata && !recordMatchesMetadataFilter(record, filters.metadata)) {
    return false;
  }

  return true;
}
