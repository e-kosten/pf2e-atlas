import {
  inferActorMetricValueType,
  isActorMetricNumericOperator,
  isActorMetricScalarOperator,
  normalizeActorMetricKey,
  normalizeActorMetricTextValue,
  type ActorMetricValue,
} from "../domain/actor-metrics.js";
import { inferItemMetricValueType, normalizeItemMetricKey } from "../domain/item-metrics.js";
import {
  getMetadataFieldSpec,
  isMetadataFieldName,
  type MetadataFieldName,
  type MetadataFieldSpecEntry,
  type MetadataSqlSourceContext,
  type MetadataValueNormalization,
} from "../domain/metadata-field-registry.js";
import {
  METADATA_BOOLEAN_FIELDS,
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataNumberField,
  MetadataPredicate,
  MetadataSetField,
  MetadataTextStringField,
  NormalizedRecord,
} from "../types.js";
import { normalizeDerivedTag } from "../tags/index.js";
import { normalizeText } from "../utils.js";
import type { SqlValue } from "../data/service-types.js";

export type MetadataSqlContext = {} & MetadataSqlSourceContext;

const ACTOR_METRIC_FIELD_NAMES = new Set<string>(["actorMetric", "actorMetricCompare"]);
const ITEM_METRIC_FIELD_NAMES = new Set<string>(["itemMetric", "itemMetricCompare"]);

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function normalizeMetadataValue(field: MetadataSetField | MetadataEnumStringField, value: string): string {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  const normalization = spec.valueNormalization ?? inferMetadataValueNormalization(spec);

  switch (normalization) {
    case "derivedTag":
      return normalizeDerivedTag(value);
    case "lowercaseTrim":
      return value.trim().toLowerCase();
    case "normalizedText":
      return normalizeText(value);
  }
}

function normalizeMetadataTextMatchValue(value: string): string {
  return value.trim().toLowerCase();
}

function inferMetadataValueNormalization(spec: MetadataFieldSpecEntry): MetadataValueNormalization {
  return spec.fieldType === "enumString" ? "lowercaseTrim" : "normalizedText";
}

function normalizeActorMetricName(metric: unknown, label: string): string {
  if (typeof metric !== "string") {
    throw new Error(`${label} must be a string.`);
  }

  const normalized = normalizeActorMetricKey(metric);
  if (!normalized) {
    throw new Error(`${label} must not be empty.`);
  }

  const metricType = inferActorMetricValueType(normalized);
  if (!metricType) {
    throw new Error(`Unknown actor metric "${metric}".`);
  }

  return normalized;
}

function normalizeItemMetricName(metric: unknown, label: string): string {
  if (typeof metric !== "string") {
    throw new Error(`${label} must be a string.`);
  }

  const normalized = normalizeItemMetricKey(metric);
  if (!normalized) {
    throw new Error(`${label} must not be empty.`);
  }

  const metricType = inferItemMetricValueType(normalized);
  if (!metricType) {
    throw new Error(`Unknown item metric "${metric}".`);
  }

  return normalized;
}

function metricSqlOperator(op: "==" | "!=" | ">" | ">=" | "<" | "<="): "=" | "<>" | ">" | ">=" | "<" | "<=" {
  switch (op) {
    case "==":
      return "=";
    case "!=":
      return "<>";
    case ">":
      return ">";
    case ">=":
      return ">=";
    case "<":
      return "<";
    case "<=":
      return "<=";
  }
}

function normalizeMetadataValues(field: MetadataSetField | MetadataEnumStringField, values: string[]): string[] {
  return [...new Set(values.map((value) => normalizeMetadataValue(field, value)).filter(Boolean))];
}

export function normalizeMetadataFilterNode(node: MetadataFilterNode): MetadataFilterNode {
  if (!isPlainObject(node)) {
    throw new Error("metadata must be an object predicate or boolean group.");
  }

  const raw = node as Record<string, unknown>;

  if ("and" in raw) {
    if (!Array.isArray(raw.and) || raw.and.length < 2) {
      throw new Error("metadata.and must contain at least 2 child nodes.");
    }

    return { and: raw.and.map((child) => normalizeMetadataFilterNode(child as MetadataFilterNode)) };
  }

  if ("or" in raw) {
    if (!Array.isArray(raw.or) || raw.or.length < 2) {
      throw new Error("metadata.or must contain at least 2 child nodes.");
    }

    return { or: raw.or.map((child) => normalizeMetadataFilterNode(child as MetadataFilterNode)) };
  }

  if ("not" in raw) {
    return { not: normalizeMetadataFilterNode(raw.not as MetadataFilterNode) };
  }

  const field = typeof raw.field === "string" ? raw.field : null;
  const op = typeof raw.op === "string" ? raw.op : null;
  if (!field || !op) {
    throw new Error("metadata predicates must include field and op.");
  }

  if (field === "actorMetric") {
    const metric = normalizeActorMetricName(raw.metric, "actorMetric.metric");
    const metricType = inferActorMetricValueType(metric);
    if (!metricType) {
      throw new Error(`Unknown actor metric "${raw.metric}".`);
    }

    if (metricType === "number") {
      if (!isActorMetricNumericOperator(op) || typeof raw.value !== "number" || !Number.isFinite(raw.value)) {
        throw new Error(`actor metric "${metric}" requires a finite numeric value with one of ==, !=, >, >=, <, <=.`);
      }

      return {
        field: "actorMetric",
        metric,
        op,
        value: raw.value,
      };
    }

    if (!isActorMetricScalarOperator(op)) {
      throw new Error(`actor metric "${metric}" only supports == and !=.`);
    }

    if (metricType === "text") {
      if (typeof raw.value !== "string") {
        throw new Error(`actor metric "${metric}" requires a string value.`);
      }

      return {
        field: "actorMetric",
        metric,
        op,
        value: normalizeActorMetricTextValue(raw.value),
      };
    }

    if (typeof raw.value !== "boolean") {
      throw new Error(`actor metric "${metric}" requires a boolean value.`);
    }

    return {
      field: "actorMetric",
      metric,
      op,
      value: raw.value,
    };
  }

  if (field === "actorMetricCompare") {
    const leftMetric = normalizeActorMetricName(raw.leftMetric, "actorMetricCompare.leftMetric");
    const rightMetric = normalizeActorMetricName(raw.rightMetric, "actorMetricCompare.rightMetric");
    if (inferActorMetricValueType(leftMetric) !== "number" || inferActorMetricValueType(rightMetric) !== "number") {
      throw new Error("actorMetricCompare only supports numeric actor metrics.");
    }
    if (!isActorMetricNumericOperator(op)) {
      throw new Error("actorMetricCompare requires one of ==, !=, >, >=, <, <=.");
    }

    return {
      field: "actorMetricCompare",
      leftMetric,
      op,
      rightMetric,
    };
  }

  if (field === "itemMetric") {
    const metric = normalizeItemMetricName(raw.metric, "itemMetric.metric");
    const metricType = inferItemMetricValueType(metric);
    if (!metricType) {
      throw new Error(`Unknown item metric "${raw.metric}".`);
    }

    if (metricType === "number") {
      if (!isActorMetricNumericOperator(op) || typeof raw.value !== "number" || !Number.isFinite(raw.value)) {
        throw new Error(`item metric "${metric}" requires a finite numeric value with one of ==, !=, >, >=, <, <=.`);
      }

      return {
        field: "itemMetric",
        metric,
        op,
        value: raw.value,
      };
    }

    if (!isActorMetricScalarOperator(op)) {
      throw new Error(`item metric "${metric}" only supports == and !=.`);
    }

    if (metricType === "text") {
      if (typeof raw.value !== "string") {
        throw new Error(`item metric "${metric}" requires a string value.`);
      }

      return {
        field: "itemMetric",
        metric,
        op,
        value: normalizeActorMetricTextValue(raw.value),
      };
    }

    if (typeof raw.value !== "boolean") {
      throw new Error(`item metric "${metric}" requires a boolean value.`);
    }

    return {
      field: "itemMetric",
      metric,
      op,
      value: raw.value,
    };
  }

  if (field === "itemMetricCompare") {
    const leftMetric = normalizeItemMetricName(raw.leftMetric, "itemMetricCompare.leftMetric");
    const rightMetric = normalizeItemMetricName(raw.rightMetric, "itemMetricCompare.rightMetric");
    if (inferItemMetricValueType(leftMetric) !== "number" || inferItemMetricValueType(rightMetric) !== "number") {
      throw new Error("itemMetricCompare only supports numeric item metrics.");
    }
    if (!isActorMetricNumericOperator(op)) {
      throw new Error("itemMetricCompare requires one of ==, !=, >, >=, <, <=.");
    }

    return {
      field: "itemMetricCompare",
      leftMetric,
      op,
      rightMetric,
    };
  }

  if (!isMetadataFieldName(field)) {
    throw new Error(`Unknown metadata field "${field}".`);
  }

  const spec = getMetadataFieldSpec(field);

  if (spec.fieldType === "set") {
    if (!["includesAny", "includesAll", "excludesAny"].includes(op)) {
      throw new Error(`Unsupported metadata operator "${op}" for set field "${field}".`);
    }

    if (
      !Array.isArray(raw.values) ||
      raw.values.length === 0 ||
      !raw.values.every((value): value is string => typeof value === "string")
    ) {
      throw new Error(`metadata predicate "${field}" requires a non-empty string values array.`);
    }

    return {
      field: field as MetadataSetField,
      op: op as "includesAny" | "includesAll" | "excludesAny",
      values: normalizeMetadataValues(field as MetadataSetField, raw.values),
    };
  }

  if (spec.fieldType === "enumString") {
    if (op === "eq") {
      if (typeof raw.value !== "string") {
        throw new Error(`metadata predicate "${field}" with op "eq" requires a string value.`);
      }

      return {
        field: field as MetadataEnumStringField,
        op,
        value: normalizeMetadataValue(field as MetadataEnumStringField, raw.value),
      };
    }

    if (op === "in" || op === "notIn") {
      if (
        !Array.isArray(raw.values) ||
        raw.values.length === 0 ||
        !raw.values.every((value): value is string => typeof value === "string")
      ) {
        throw new Error(`metadata predicate "${field}" with op "${op}" requires a non-empty string values array.`);
      }

      return {
        field: field as MetadataEnumStringField,
        op,
        values: normalizeMetadataValues(field as MetadataEnumStringField, raw.values),
      };
    }

    throw new Error(`Unsupported metadata operator "${op}" for string field "${field}".`);
  }

  if (spec.fieldType === "text") {
    if (!["contains", "notContains"].includes(op) || typeof raw.value !== "string") {
      throw new Error(`metadata predicate "${field}" requires op "contains" or "notContains" with a string value.`);
    }

    return {
      field: field as MetadataTextStringField,
      op: op as "contains" | "notContains",
      value: normalizeMetadataTextMatchValue(raw.value),
    };
  }

  if (spec.fieldType === "number") {
    if (op === "between") {
      if (
        typeof raw.min !== "number" ||
        !Number.isFinite(raw.min) ||
        typeof raw.max !== "number" ||
        !Number.isFinite(raw.max)
      ) {
        throw new Error(`metadata predicate "${field}" with op "between" requires finite min and max numbers.`);
      }
      if (raw.min > raw.max) {
        throw new Error(`metadata predicate "${field}" with op "between" requires min <= max.`);
      }

      return {
        field: field as MetadataNumberField,
        op,
        min: raw.min,
        max: raw.max,
      };
    }

    if (!["eq", "gte", "lte"].includes(op) || typeof raw.value !== "number" || !Number.isFinite(raw.value)) {
      throw new Error(`metadata predicate "${field}" requires op "eq", "gte", or "lte" with a finite numeric value.`);
    }

    return {
      field: field as MetadataNumberField,
      op: op as "eq" | "gte" | "lte",
      value: raw.value,
    };
  }

  if (op !== "eq" || typeof raw.value !== "boolean") {
    throw new Error(`metadata predicate "${field}" requires op "eq" with a boolean value.`);
  }

  return {
    field: field as MetadataBooleanField,
    op,
    value: raw.value,
  };
}

function buildActorMetricPredicateClause(
  predicate: Extract<MetadataPredicate, { field: "actorMetric" }>,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const metricType = inferActorMetricValueType(predicate.metric);
  if (!metricType) {
    throw new Error(`Unknown actor metric "${predicate.metric}".`);
  }

  const sqlOperator = metricSqlOperator(predicate.op);
  if (metricType === "number") {
    const numericValue = predicate.value as number;
    return {
      clause: `EXISTS (
        SELECT 1
        FROM actor_metrics actor_metric
        WHERE actor_metric.record_key = ${context.recordKeyExpr}
          AND actor_metric.metric_key = ?
          AND actor_metric.value_type = 'number'
          AND actor_metric.number_value ${sqlOperator} ?
      )`,
      params: [predicate.metric, numericValue],
    };
  }

  if (metricType === "text") {
    const textValue = predicate.value as string;
    return {
      clause: `EXISTS (
        SELECT 1
        FROM actor_metrics actor_metric
        WHERE actor_metric.record_key = ${context.recordKeyExpr}
          AND actor_metric.metric_key = ?
          AND actor_metric.value_type = 'text'
          AND LOWER(COALESCE(actor_metric.text_value, '')) ${sqlOperator} ?
      )`,
      params: [predicate.metric, textValue],
    };
  }

  const booleanValue = predicate.value as boolean;
  return {
    clause: `EXISTS (
      SELECT 1
      FROM actor_metrics actor_metric
      WHERE actor_metric.record_key = ${context.recordKeyExpr}
        AND actor_metric.metric_key = ?
        AND actor_metric.value_type = 'boolean'
        AND COALESCE(actor_metric.bool_value, 0) ${sqlOperator} ?
    )`,
    params: [predicate.metric, booleanValue ? 1 : 0],
  };
}

function buildActorMetricCompareClause(
  predicate: Extract<MetadataPredicate, { field: "actorMetricCompare" }>,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  return {
    clause: `EXISTS (
      SELECT 1
      FROM actor_metrics left_metric
      JOIN actor_metrics right_metric
        ON right_metric.record_key = left_metric.record_key
      WHERE left_metric.record_key = ${context.recordKeyExpr}
        AND left_metric.metric_key = ?
        AND left_metric.value_type = 'number'
        AND right_metric.metric_key = ?
        AND right_metric.value_type = 'number'
        AND left_metric.number_value ${metricSqlOperator(predicate.op)} right_metric.number_value
    )`,
    params: [predicate.leftMetric, predicate.rightMetric],
  };
}

function buildItemMetricPredicateClause(
  predicate: Extract<MetadataPredicate, { field: "itemMetric" }>,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const metricType = inferItemMetricValueType(predicate.metric);
  if (!metricType) {
    throw new Error(`Unknown item metric "${predicate.metric}".`);
  }

  const sqlOperator = metricSqlOperator(predicate.op);
  if (metricType === "number") {
    const numericValue = predicate.value as number;
    return {
      clause: `EXISTS (
        SELECT 1
        FROM item_metrics item_metric
        WHERE item_metric.record_key = ${context.recordKeyExpr}
          AND item_metric.metric_key = ?
          AND item_metric.value_type = 'number'
          AND item_metric.number_value ${sqlOperator} ?
      )`,
      params: [predicate.metric, numericValue],
    };
  }

  if (metricType === "text") {
    const textValue = predicate.value as string;
    return {
      clause: `EXISTS (
        SELECT 1
        FROM item_metrics item_metric
        WHERE item_metric.record_key = ${context.recordKeyExpr}
          AND item_metric.metric_key = ?
          AND item_metric.value_type = 'text'
          AND LOWER(COALESCE(item_metric.text_value, '')) ${sqlOperator} ?
      )`,
      params: [predicate.metric, textValue],
    };
  }

  const booleanValue = predicate.value as boolean;
  return {
    clause: `EXISTS (
      SELECT 1
      FROM item_metrics item_metric
      WHERE item_metric.record_key = ${context.recordKeyExpr}
        AND item_metric.metric_key = ?
        AND item_metric.value_type = 'boolean'
        AND COALESCE(item_metric.bool_value, 0) ${sqlOperator} ?
    )`,
    params: [predicate.metric, booleanValue ? 1 : 0],
  };
}

function buildItemMetricCompareClause(
  predicate: Extract<MetadataPredicate, { field: "itemMetricCompare" }>,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  return {
    clause: `EXISTS (
      SELECT 1
      FROM item_metrics left_metric
      JOIN item_metrics right_metric
        ON right_metric.record_key = left_metric.record_key
      WHERE left_metric.record_key = ${context.recordKeyExpr}
        AND left_metric.metric_key = ?
        AND left_metric.value_type = 'number'
        AND right_metric.metric_key = ?
        AND right_metric.value_type = 'number'
        AND left_metric.number_value ${metricSqlOperator(predicate.op)} right_metric.number_value
    )`,
    params: [predicate.leftMetric, predicate.rightMetric],
  };
}

function buildScalarLookupSql(recordKeyExpr: string, alias: string | undefined, column: string, table: string): string {
  return alias
    ? `${alias}.${column}`
    : `(SELECT meta.${column} FROM ${table} meta WHERE meta.record_key = ${recordKeyExpr})`;
}

function buildMetadataJsonArraySql(context: MetadataSqlContext, field: MetadataSetField): string {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  return spec.buildSqlExpression ? spec.buildSqlExpression(context) : "[]";
}

function buildMetadataScalarSqlExpression(
  context: MetadataSqlContext,
  field: MetadataEnumStringField | MetadataTextStringField | MetadataNumberField | MetadataBooleanField,
): string {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  if (!spec.buildSqlExpression) {
    throw new Error(`Metadata field "${field}" does not provide a SQL expression.`);
  }
  return spec.buildSqlExpression(context);
}

function buildMetadataSetPredicateClause(
  field: MetadataSetField,
  op: "includesAny" | "includesAll" | "excludesAny",
  values: string[],
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const params: SqlValue[] = [];
  const buildMembershipClause = (operator: "EXISTS" | "NOT EXISTS", valueSql: string, localParams: SqlValue[]) => {
    if (field === "traits") {
      return {
        clause: `${operator} (SELECT 1 FROM record_traits value_set WHERE value_set.record_key = ${context.recordKeyExpr} AND value_set.trait ${valueSql})`,
        params: localParams,
      };
    }

    if (field === "derivedTags") {
      return {
        clause: `${operator} (SELECT 1 FROM record_derived_tags value_set WHERE value_set.record_key = ${context.recordKeyExpr} AND value_set.tag ${valueSql})`,
        params: localParams,
      };
    }

    return {
      clause: `${operator} (SELECT 1 FROM json_each(${buildMetadataJsonArraySql(context, field)}) AS value_set WHERE LOWER(value_set.value) ${valueSql})`,
      params: localParams,
    };
  };

  if (op === "includesAll") {
    const clauses = values.map((value) => {
      const built = buildMembershipClause("EXISTS", "= ?", [value]);
      params.push(...built.params);
      return built.clause;
    });

    return {
      clause: `(${clauses.join(" AND ")})`,
      params,
    };
  }

  const placeholders = values.map(() => "?").join(", ");
  return buildMembershipClause(op === "excludesAny" ? "NOT EXISTS" : "EXISTS", `IN (${placeholders})`, values);
}

function buildMetadataPredicateClause(
  predicate: MetadataPredicate,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  if (predicate.field === "actorMetric") {
    return buildActorMetricPredicateClause(predicate, context);
  }

  if (predicate.field === "actorMetricCompare") {
    return buildActorMetricCompareClause(predicate, context);
  }

  if (predicate.field === "itemMetric") {
    return buildItemMetricPredicateClause(predicate, context);
  }

  if (predicate.field === "itemMetricCompare") {
    return buildItemMetricCompareClause(predicate, context);
  }

  const spec = getMetadataFieldSpec(predicate.field as MetadataFieldName);
  if (spec.fieldType === "set") {
    const setPredicate = predicate as Extract<MetadataPredicate, { field: MetadataSetField }>;
    return buildMetadataSetPredicateClause(setPredicate.field, setPredicate.op, setPredicate.values, context);
  }

  if (spec.fieldType === "enumString") {
    const stringPredicate = predicate as Extract<MetadataPredicate, { field: MetadataEnumStringField }>;
    const expression = buildMetadataScalarSqlExpression(context, stringPredicate.field);
    if (stringPredicate.op === "eq") {
      return {
        clause: `LOWER(COALESCE(${expression}, '')) = ?`,
        params: [stringPredicate.value],
      };
    }

    const placeholders = stringPredicate.values.map(() => "?").join(", ");
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${stringPredicate.op === "notIn" ? "NOT " : ""}IN (${placeholders})`,
      params: stringPredicate.values,
    };
  }

  if (spec.fieldType === "text") {
    const textPredicate = predicate as Extract<MetadataPredicate, { field: MetadataTextStringField }>;
    const expression = buildMetadataScalarSqlExpression(context, textPredicate.field);
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${textPredicate.op === "notContains" ? "NOT " : ""}LIKE ?`,
      params: [`%${textPredicate.value}%`],
    };
  }

  if (spec.fieldType === "number") {
    const numberPredicate = predicate as Extract<MetadataPredicate, { field: MetadataNumberField }>;
    const expression = buildMetadataScalarSqlExpression(context, numberPredicate.field);
    if (numberPredicate.op === "between") {
      return {
        clause: `(${expression} >= ? AND ${expression} <= ?)`,
        params: [numberPredicate.min, numberPredicate.max],
      };
    }

    const operator = numberPredicate.op === "eq" ? "=" : numberPredicate.op === "gte" ? ">=" : "<=";
    return {
      clause: `${expression} ${operator} ?`,
      params: [numberPredicate.value],
    };
  }

  const booleanPredicate = predicate as Extract<MetadataPredicate, { field: MetadataBooleanField }>;
  const expression = buildMetadataScalarSqlExpression(context, booleanPredicate.field);
  return {
    clause: `COALESCE(${expression}, 0) = ?`,
    params: [booleanPredicate.value ? 1 : 0],
  };
}

function buildMetadataFilterClause(
  node: MetadataFilterNode,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  if ("and" in node) {
    const children = node.and.map((child) => buildMetadataFilterClause(child, context));
    return {
      clause: `(${children.map((child) => child.clause).join(" AND ")})`,
      params: children.flatMap((child) => child.params),
    };
  }

  if ("or" in node) {
    const children = node.or.map((child) => buildMetadataFilterClause(child, context));
    return {
      clause: `(${children.map((child) => child.clause).join(" OR ")})`,
      params: children.flatMap((child) => child.params),
    };
  }

  if ("not" in node) {
    const child = buildMetadataFilterClause(node.not, context);
    return {
      clause: `(NOT ${child.clause})`,
      params: child.params,
    };
  }

  return buildMetadataPredicateClause(node, context);
}

export function appendMetadataFilterClauses(
  sql: string[],
  params: SqlValue[],
  metadata: MetadataFilterNode | undefined,
  context: MetadataSqlContext,
  appendClause: (sql: string[], params: SqlValue[], clause: string, ...values: SqlValue[]) => void,
): void {
  if (!metadata) {
    return;
  }

  const compiled = buildMetadataFilterClause(metadata, context);
  appendClause(sql, params, `AND ${compiled.clause}`, ...compiled.params);
}

function getRecordSetValues(record: NormalizedRecord, field: MetadataSetField): string[] {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  return record[spec.recordProperty] as string[];
}

function getRecordStringValue(
  record: NormalizedRecord,
  field: MetadataEnumStringField | MetadataTextStringField,
): string | null {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  return record[spec.recordProperty] as string | null;
}

function getRecordNumberValue(record: NormalizedRecord, field: MetadataNumberField): number | null {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  return record[spec.recordProperty] as number | null;
}

function getRecordBooleanValue(record: NormalizedRecord, field: MetadataBooleanField): boolean {
  const spec = getMetadataFieldSpec(field as MetadataFieldName);
  return record[spec.recordProperty] as boolean;
}

function compareActorMetricValues(
  left: ActorMetricValue,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  right: ActorMetricValue,
): boolean {
  switch (op) {
    case "==":
      return left === right;
    case "!=":
      return left !== right;
    case ">":
      return (left as number) > (right as number);
    case ">=":
      return (left as number) >= (right as number);
    case "<":
      return (left as number) < (right as number);
    case "<=":
      return (left as number) <= (right as number);
  }
}

export function recordMatchesMetadataFilter(record: NormalizedRecord, node: MetadataFilterNode): boolean {
  if ("and" in node) {
    return node.and.every((child) => recordMatchesMetadataFilter(record, child));
  }

  if ("or" in node) {
    return node.or.some((child) => recordMatchesMetadataFilter(record, child));
  }

  if ("not" in node) {
    return !recordMatchesMetadataFilter(record, node.not);
  }

  if (node.field === "actorMetric") {
    const metricType = inferActorMetricValueType(node.metric);
    const recordValue = record.actorMetrics[node.metric];
    if (!metricType || recordValue === undefined) {
      return false;
    }

    if (metricType === "text" && typeof node.value === "string") {
      return compareActorMetricValues(recordValue, node.op, normalizeActorMetricTextValue(node.value));
    }

    return compareActorMetricValues(recordValue, node.op, node.value);
  }

  if (node.field === "actorMetricCompare") {
    const leftValue = record.actorMetrics[node.leftMetric];
    const rightValue = record.actorMetrics[node.rightMetric];
    if (typeof leftValue !== "number" || typeof rightValue !== "number") {
      return false;
    }

    return compareActorMetricValues(leftValue, node.op, rightValue);
  }

  if (node.field === "itemMetric") {
    const metricType = inferItemMetricValueType(node.metric);
    const recordValue = record.itemMetrics[node.metric];
    if (!metricType || recordValue === undefined) {
      return false;
    }

    if (metricType === "text" && typeof node.value === "string") {
      return compareActorMetricValues(recordValue, node.op, normalizeActorMetricTextValue(node.value));
    }

    return compareActorMetricValues(recordValue, node.op, node.value);
  }

  if (node.field === "itemMetricCompare") {
    const leftValue = record.itemMetrics[node.leftMetric];
    const rightValue = record.itemMetrics[node.rightMetric];
    if (typeof leftValue !== "number" || typeof rightValue !== "number") {
      return false;
    }

    return compareActorMetricValues(leftValue, node.op, rightValue);
  }

  const spec = getMetadataFieldSpec(node.field as MetadataFieldName);

  if (spec.fieldType === "set") {
    const setPredicate = node as Extract<MetadataPredicate, { field: MetadataSetField }>;
    const normalizedValues = new Set(
      getRecordSetValues(record, setPredicate.field)
        .map((value) => normalizeMetadataValue(setPredicate.field, value))
        .filter(Boolean),
    );
    if (setPredicate.op === "includesAll") {
      return setPredicate.values.every((value) => normalizedValues.has(value));
    }
    if (setPredicate.op === "includesAny") {
      return setPredicate.values.some((value) => normalizedValues.has(value));
    }
    return !setPredicate.values.some((value) => normalizedValues.has(value));
  }

  if (spec.fieldType === "enumString") {
    const stringPredicate = node as Extract<MetadataPredicate, { field: MetadataEnumStringField }>;
    const normalizedValue = normalizeMetadataValue(
      stringPredicate.field,
      getRecordStringValue(record, stringPredicate.field) ?? "",
    );
    if (stringPredicate.op === "eq") {
      return normalizedValue === stringPredicate.value;
    }
    if (stringPredicate.op === "in") {
      return stringPredicate.values.includes(normalizedValue);
    }
    return !stringPredicate.values.includes(normalizedValue);
  }

  if (spec.fieldType === "text") {
    const textPredicate = node as Extract<MetadataPredicate, { field: MetadataTextStringField }>;
    const normalizedValue = normalizeMetadataTextMatchValue(getRecordStringValue(record, textPredicate.field) ?? "");
    return textPredicate.op === "contains"
      ? normalizedValue.includes(textPredicate.value)
      : !normalizedValue.includes(textPredicate.value);
  }

  if (spec.fieldType === "number") {
    const numberPredicate = node as Extract<MetadataPredicate, { field: MetadataNumberField }>;
    const numericValue = getRecordNumberValue(record, numberPredicate.field);
    if (numericValue === null) {
      return false;
    }
    if (numberPredicate.op === "between") {
      return numericValue >= numberPredicate.min && numericValue <= numberPredicate.max;
    }
    if (numberPredicate.op === "eq") {
      return numericValue === numberPredicate.value;
    }
    if (numberPredicate.op === "gte") {
      return numericValue >= numberPredicate.value;
    }
    return numericValue <= numberPredicate.value;
  }

  const booleanPredicate = node as Extract<MetadataPredicate, { field: MetadataBooleanField }>;
  return getRecordBooleanValue(record, booleanPredicate.field) === booleanPredicate.value;
}
