import {
  inferActorMetricValueType,
  normalizeActorMetricKey,
  normalizeActorMetricPrefix,
} from "../../domain/actor-metrics.js";
import { inferItemMetricValueType, normalizeItemMetricKey, normalizeItemMetricPrefix } from "../../domain/item-metrics.js";
import { isMetadataFieldName } from "../../domain/metadata-field-types.js";
import { getMetadataRecordSelectClauses } from "../metadata-row-projection.js";
import { SearchSort, type FilterValueQuery } from "../../domain/search-types.js";
import { SEARCH_VOCABULARY } from "../../domain/search-types.js";
import type {
  NormalizedSearchFilters,
  SearchExecutionFilterNode,
  SearchExecutionNullableNumericMatch,
  SearchExecutionNullableStringMatch,
  SearchExecutionNumericMatch,
  SearchExecutionScopeSubcategoryMatch,
} from "../../search/contracts.js";
import type { SqlValue } from "../sql-types.js";
import {
  buildMetadataAtomicPredicateClause,
  buildMetricCompareClause,
  buildMetricPredicateClause,
  getMetadataFilterValueSource,
  type MetadataFilterValueSource,
} from "./metadata-search-sql.js";

function appendWhereClause(sql: string[], params: SqlValue[], clause: string, ...values: SqlValue[]): void {
  sql.push(clause);
  params.push(...values);
}

type SearchSqlFilterContext = {
  recordKeyExpr: string;
  categoryExpr: string;
  subcategoryExpr: string;
  packNameExpr: string;
  packLabelExpr?: string;
  levelExpr: string;
  rarityExpr: string;
  priceExpr: string;
  actionCostExpr: string;
  metadata: {
    recordKeyExpr: string;
    recordsAlias?: string;
    actorAlias?: string;
    itemAlias?: string;
    spellAlias?: string;
  };
};

const RECORDS_PRIMARY_KEY_INDEX = "sqlite_autoindex_records_1";

function recordsTableReference(alias: string, options: { recordKeys?: string[] } = {}): string {
  if (options.recordKeys && options.recordKeys.length > 0) {
    return `records ${alias} INDEXED BY ${RECORDS_PRIMARY_KEY_INDEX}`;
  }
  return `records ${alias}`;
}

function buildLinksToClause(target: string, recordKeyExpr: string): { clause: string; params: SqlValue[] } {
  return {
    clause: `EXISTS (
      SELECT 1
      FROM reference_edges re_link
      WHERE re_link.from_record_key = ${recordKeyExpr}
        AND re_link.to_record_key = ?
    )`,
    params: [target],
  };
}

function buildLinkedFromClause(source: string, recordKeyExpr: string): { clause: string; params: SqlValue[] } {
  return {
    clause: `EXISTS (
      SELECT 1
      FROM reference_edges re_link
      WHERE re_link.from_record_key = ?
        AND re_link.to_record_key = ${recordKeyExpr}
    )`,
    params: [source],
  };
}

const SCOPE_SUBCATEGORY_FILTER_BUILDERS = {
  any: (categoryExpr: string, category: string): { clause: string; params: SqlValue[] } => ({
    clause: `LOWER(${categoryExpr}) = LOWER(?)`,
    params: [category],
  }),
  isNull: (categoryExpr: string, category: string, subcategoryExpr: string): { clause: string; params: SqlValue[] } => ({
    clause: `(LOWER(${categoryExpr}) = LOWER(?) AND (${subcategoryExpr} IS NULL OR TRIM(${subcategoryExpr}) = ''))`,
    params: [category],
  }),
  isNotNull: (categoryExpr: string, category: string, subcategoryExpr: string): { clause: string; params: SqlValue[] } => ({
    clause: `(LOWER(${categoryExpr}) = LOWER(?) AND ${subcategoryExpr} IS NOT NULL AND TRIM(${subcategoryExpr}) <> '')`,
    params: [category],
  }),
  eq: (
    categoryExpr: string,
    category: string,
    subcategoryExpr: string,
    subcategoryValue: string,
  ): { clause: string; params: SqlValue[] } => ({
    clause: `(LOWER(${categoryExpr}) = LOWER(?) AND LOWER(COALESCE(${subcategoryExpr}, '')) = LOWER(?))`,
    params: [category, subcategoryValue],
  }),
} as const satisfies Record<SearchExecutionScopeSubcategoryMatch["kind"], (...args: string[]) => { clause: string; params: SqlValue[] }>;

const NUMBER_MATCH_OPERATORS = {
  eq: "=",
  gt: ">",
  gte: ">=",
  lt: "<",
  lte: "<=",
} as const satisfies Readonly<Record<Exclude<SearchExecutionNumericMatch["kind"], "between">, string>>;

function buildNumericMatchClause(
  expression: string,
  match: SearchExecutionNumericMatch,
): { clause: string; params: SqlValue[] } {
  if (match.kind === "between") {
    return {
      clause: `(${expression} >= ? AND ${expression} <= ?)`,
      params: [match.min, match.max],
    };
  }
  return {
    clause: `${expression} ${NUMBER_MATCH_OPERATORS[match.kind]} ?`,
    params: [match.value],
  };
}

function buildNullableNumericMatchClause(
  expression: string,
  match: SearchExecutionNullableNumericMatch,
): { clause: string; params: SqlValue[] } {
  if (match.kind === "isNull") {
    return { clause: `${expression} IS NULL`, params: [] };
  }
  if (match.kind === "isNotNull") {
    return { clause: `${expression} IS NOT NULL`, params: [] };
  }
  return buildNumericMatchClause(expression, match as SearchExecutionNumericMatch);
}

function buildRarityMatchClause(
  expression: string,
  match: SearchExecutionNullableStringMatch,
): { clause: string; params: SqlValue[] } {
  if (match.kind === "isNull") {
    return { clause: `${expression} IS NULL OR TRIM(${expression}) = ''`, params: [] };
  }
  if (match.kind === "isNotNull") {
    return { clause: `${expression} IS NOT NULL AND TRIM(${expression}) <> ''`, params: [] };
  }
  if (match.kind === "in" || match.kind === "notIn") {
    const values = match.values.filter((value) => value.length > 0);
    if (values.length === 0) {
      return match.kind === "in" ? { clause: "0 = 1", params: [] } : { clause: "1 = 1", params: [] };
    }
    const placeholders = values.map(() => "?").join(", ");
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${match.kind === "in" ? "IN" : "NOT IN"} (${placeholders})`,
      params: values.map((value) => value.toLowerCase()),
    };
  }
  const eqMatch = match as Extract<SearchExecutionNullableStringMatch, { kind: "eq" }>;
  return {
    clause: `LOWER(COALESCE(${expression}, '')) = LOWER(?)`,
    params: [eqMatch.value],
  };
}

function buildSearchFilterClause(
  filter: SearchExecutionFilterNode,
  context: SearchSqlFilterContext,
): { clause: string; params: SqlValue[] } {
  switch (filter.kind) {
    case "pack":
      return {
        clause: context.packLabelExpr
          ? `(LOWER(${context.packNameExpr}) = LOWER(?) OR LOWER(${context.packLabelExpr}) = LOWER(?))`
          : `LOWER(${context.packNameExpr}) = LOWER(?)`,
        params: context.packLabelExpr ? [filter.value, filter.value] : [filter.value],
      };
    case "scope": {
      switch (filter.subcategory.kind) {
        case "eq":
          return SCOPE_SUBCATEGORY_FILTER_BUILDERS.eq(
            context.categoryExpr,
            filter.category,
            context.subcategoryExpr,
            filter.subcategory.value,
          );
        case "any":
          return SCOPE_SUBCATEGORY_FILTER_BUILDERS.any(context.categoryExpr, filter.category);
        case "isNull":
          return SCOPE_SUBCATEGORY_FILTER_BUILDERS.isNull(context.categoryExpr, filter.category, context.subcategoryExpr);
        case "isNotNull":
          return SCOPE_SUBCATEGORY_FILTER_BUILDERS.isNotNull(
            context.categoryExpr,
            filter.category,
            context.subcategoryExpr,
          );
      }
    }
    case "level":
      return buildNumericMatchClause(context.levelExpr, filter.match);
    case "price":
      return buildNumericMatchClause(context.priceExpr, filter.match);
    case "rarity":
      return buildRarityMatchClause(context.rarityExpr, filter.match);
    case "actionCost":
      return buildNullableNumericMatchClause(context.actionCostExpr, filter.match);
    case "linksTo":
      return buildLinksToClause(filter.target, context.recordKeyExpr);
    case "linkedFrom":
      return buildLinkedFromClause(filter.source, context.recordKeyExpr);
    case "metadataPredicate":
      return buildMetadataAtomicPredicateClause(filter.predicate, context.metadata);
    case "metric":
      return buildMetricPredicateClause(filter.metric, filter.op, filter.value, context.metadata);
    case "metricCompare":
      return buildMetricCompareClause(filter.leftMetric, filter.op, filter.rightMetric, context.metadata);
    case "anyOf": {
      const children = filter.children.map((child) => buildSearchFilterClause(child, context));
      return {
        clause: `(${children.map((child) => child.clause).join(" OR ")})`,
        params: children.flatMap((child) => child.params),
      };
    }
    case "allOf": {
      const children = filter.children.map((child) => buildSearchFilterClause(child, context));
      return {
        clause: `(${children.map((child) => child.clause).join(" AND ")})`,
        params: children.flatMap((child) => child.params),
      };
    }
    case "not": {
      const child = buildSearchFilterClause(filter.child, context);
      return {
        clause: `(NOT ${child.clause})`,
        params: child.params,
      };
    }
  }
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

  if (filters.filter) {
    const compiled = buildSearchFilterClause(filters.filter, {
      recordKeyExpr: `${recordAlias}.record_key`,
      categoryExpr: `${recordAlias}.category`,
      subcategoryExpr: `${recordAlias}.subcategory`,
      packNameExpr: `${recordAlias}.pack_name`,
      packLabelExpr: `${recordAlias}.pack_label`,
      levelExpr: `${recordAlias}.level`,
      rarityExpr: `${recordAlias}.rarity`,
      priceExpr: `${itemAlias}.price_cp`,
      actionCostExpr: `COALESCE(${spellAlias}.action_cost, ${itemAlias}.action_cost)`,
      metadata: {
        recordKeyExpr: `${recordAlias}.record_key`,
        recordsAlias: recordAlias,
        actorAlias,
        itemAlias,
        spellAlias,
      },
    });
    appendWhereClause(sql, params, `AND ${compiled.clause}`, ...compiled.params);
  }
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
  "r.blurb_text AS blurbText",
  "r.description_snippet AS descriptionSnippet",
  "r.folder_id AS folderId",
  "r.variant_confidence AS variantConfidence",
  "r.variant_source AS variantSource",
  "r.source_path AS sourcePath",
  "r.is_unique AS isUnique",
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

export function buildSharedRecordSelectFields(includeSearchText = false, includeEmbedding = false): string[] {
  const fields = [...BASE_RECORD_SELECT_FIELDS, ...getMetadataRecordSelectClauses()];

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

  const source = getMetadataFilterValueSource(field);
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
    `FROM ${recordsTableReference("r", options)}`,
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];

  if (includeEmbedding) {
    sql.push("LEFT JOIN embeddings e ON e.record_key = r.record_key");
  }

  sql.push("WHERE 1 = 1");
  const params: SqlValue[] = [];
  applySearchFilterClauses(
    sql,
    params,
    filters,
    {
      records: "r",
      actor: "a",
      item: "i",
      spell: "s",
    },
    options,
  );

  return { sql: sql.join("\n"), params };
}

function appendCandidateOrderBy(
  sql: string[],
  sort: Exclude<SearchSort, typeof SEARCH_VOCABULARY.SORT_KIND.RANKED | typeof SEARCH_VOCABULARY.SORT_KIND.RANDOM>,
): void {
  switch (sort) {
    case SEARCH_VOCABULARY.SORT_KIND.LEVEL_ASC:
      sql.push(
        "ORDER BY CASE WHEN r.level IS NULL THEN 1 ELSE 0 END ASC, r.level ASC, r.name COLLATE NOCASE ASC, r.pack_label COLLATE NOCASE ASC, r.id ASC",
      );
      return;
    case SEARCH_VOCABULARY.SORT_KIND.LEVEL_DESC:
      sql.push(
        "ORDER BY CASE WHEN r.level IS NULL THEN 1 ELSE 0 END ASC, r.level DESC, r.name COLLATE NOCASE ASC, r.pack_label COLLATE NOCASE ASC, r.id ASC",
      );
      return;
    case SEARCH_VOCABULARY.SORT_KIND.ALPHABETICAL:
      sql.push("ORDER BY r.name COLLATE NOCASE ASC, r.pack_label COLLATE NOCASE ASC, r.id ASC");
      return;
  }
}

export function buildCandidateCountQuery(
  filters: NormalizedSearchFilters,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT COUNT(*) AS total",
    `FROM ${recordsTableReference("r", options)}`,
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    "WHERE 1 = 1",
  ];
  const params: SqlValue[] = [];
  applySearchFilterClauses(
    sql,
    params,
    filters,
    {
      records: "r",
      actor: "a",
      item: "i",
      spell: "s",
    },
    options,
  );

  return { sql: sql.join("\n"), params };
}

export function buildCandidateKeyQuery(
  filters: NormalizedSearchFilters,
  sort?: Exclude<SearchSort, typeof SEARCH_VOCABULARY.SORT_KIND.RANKED | typeof SEARCH_VOCABULARY.SORT_KIND.RANDOM>,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT r.record_key AS recordKey",
    `FROM ${recordsTableReference("r", options)}`,
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
    "WHERE 1 = 1",
  ];
  const params: SqlValue[] = [];
  applySearchFilterClauses(
    sql,
    params,
    filters,
    {
      records: "r",
      actor: "a",
      item: "i",
      spell: "s",
    },
    options,
  );

  if (sort) {
    appendCandidateOrderBy(sql, sort);
  }

  return { sql: sql.join("\n"), params };
}

export function buildPagedCandidateQuery(
  filters: NormalizedSearchFilters,
  sort: SearchSort,
  offset: number,
  limit: number,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  if (sort === SEARCH_VOCABULARY.SORT_KIND.RANKED || sort === SEARCH_VOCABULARY.SORT_KIND.RANDOM) {
    throw new Error(`Paged candidate query does not support ${sort} ordering.`);
  }

  const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding, options);
  const lines = [sql];
  appendCandidateOrderBy(lines, sort);
  lines.push("LIMIT ?");
  lines.push("OFFSET ?");
  return {
    sql: lines.join("\n"),
    params: [...params, limit, offset],
  };
}

export function buildFilterValueQuery(
  query: FilterValueQuery,
  filters: NormalizedSearchFilters,
  options: { recordKeys?: string[] } = {},
): { sql: string; params: SqlValue[] } {
  const { field } = query;
  if (field !== "actorMetrics" && field !== "itemMetrics" && (query.metricPrefix || query.metric)) {
    throw new Error("metricPrefix and metric are only supported when field is actorMetrics or itemMetrics.");
  }

  const joins = [
    `FROM ${recordsTableReference("r", options)}`,
    "LEFT JOIN actor_records a ON a.record_key = r.record_key",
    "LEFT JOIN item_records i ON i.record_key = r.record_key",
    "LEFT JOIN spell_records s ON s.record_key = r.record_key",
  ];
  const sql: string[] = [];
  const params: SqlValue[] = [];
  const postFilterClauses: string[] = [];
  const postFilterParams: SqlValue[] = [];
  let valueExpression: string;

  if (isMetadataFieldName(field)) {
    const metadataSource = applyMetadataFilterValueSource(field, joins, postFilterClauses);
    if (!metadataSource) {
      throw new Error(`No filter value source configured for metadata field "${field}".`);
    }
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
      default: {
        const exhaustive: never = field;
        throw new Error(`Unhandled filter value field "${String(exhaustive)}".`);
      }
    }
  }

  sql.push(`SELECT ${valueExpression} AS value, COUNT(*) AS count`);
  sql.push(...joins);
  sql.push("WHERE 1 = 1");
  applySearchFilterClauses(
    sql,
    params,
    filters,
    {
      records: "r",
      actor: "a",
      item: "i",
      spell: "s",
    },
    options,
  );
  sql.push(...postFilterClauses);
  params.push(...postFilterParams);
  sql.push("GROUP BY value");
  sql.push("ORDER BY count DESC, value ASC");
  return { sql: sql.join("\n"), params };
}

export function buildLexicalRetrievalQuery(
  filters: NormalizedSearchFilters,
  query: string,
  limit: number,
): { sql: string; params: SqlValue[] } {
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

export const SQLITE_VECTOR_QUERY_K_LIMIT = 4096;

export function buildSemanticRetrievalQuery(
  filters: NormalizedSearchFilters,
  limit: number,
): { sql: string; params: SqlValue[] } {
  const sql = [
    "SELECT record_key AS recordKey, distance",
    "FROM record_embeddings",
    "WHERE embedding MATCH ?",
    `AND k = ${limit}`,
  ];
  const params: SqlValue[] = [];

  if (filters.filter) {
    const compiled = buildSearchFilterClause(filters.filter, {
      recordKeyExpr: "record_embeddings.record_key",
      categoryExpr: "category",
      subcategoryExpr: "subcategory",
      packNameExpr: "pack_name",
      levelExpr: "level",
      rarityExpr: "rarity",
      priceExpr: "price_cp",
      actionCostExpr: "action_cost",
      metadata: {
        recordKeyExpr: "record_embeddings.record_key",
      },
    });
    appendWhereClause(
      sql,
      params,
      `AND ${compiled.clause}`,
      ...compiled.params.map((value) => {
        if (typeof value === "number" && Number.isInteger(value)) {
          return BigInt(value);
        }
        return value;
      }),
    );
  }

  return { sql: sql.join("\n"), params };
}
