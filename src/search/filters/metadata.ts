import {
  inferActorMetricValueType,
  normalizeActorMetricKey,
  normalizeActorMetricTextValue,
  type ActorMetricValue,
} from "../../domain/actor-metrics.js";
import {
  inferItemMetricValueType,
  normalizeItemMetricKey,
} from "../../domain/item-metrics.js";
import type {
  MetadataAtomicPredicate,
} from "../../domain/search-filter-metadata.js";
import type {
  MetricOperator,
  NumericMetricOperator,
} from "../../domain/search-filter-operators.js";
import {
  type MetadataBooleanField,
  type MetadataEnumStringField,
  type MetadataNumberField,
  type MetadataSetField,
  type MetadataTextStringField,
} from "../../domain/metadata-field-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import { normalizeDerivedTag } from "../../tags/runtime.js";
import { normalizeText } from "../../shared/utils.js";
import type { SqlValue } from "../contracts.js";
import {
  getMetadataBooleanFieldSpec,
  getMetadataBooleanRecordValue,
  getMetadataEnumStringFieldSpec,
  getMetadataNumberFieldSpec,
  getMetadataNumberRecordValue,
  getMetadataSetFieldSpec,
  getMetadataSetRecordValues,
  getMetadataStringRecordValue,
  getMetadataTextFieldSpec,
  isMetadataBooleanField,
  isMetadataEnumStringField,
  isMetadataNumberField,
  isMetadataSetField,
  isMetadataTextField,
  type MetadataFieldSpecEntry,
  type MetadataSqlSourceContext,
  type MetadataValueNormalization,
} from "./registry.js";

export type MetadataSqlContext = {} & MetadataSqlSourceContext;

function normalizeMetadataValue(field: MetadataSetField | MetadataEnumStringField, value: string): string {
  const spec = isMetadataSetField(field) ? getMetadataSetFieldSpec(field) : getMetadataEnumStringFieldSpec(field);
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
  return spec.fieldType === "text" ? "normalizedText" : "lowercaseTrim";
}

function buildMetadataJsonArraySql(context: MetadataSqlContext, field: MetadataSetField): string {
  const spec = getMetadataSetFieldSpec(field);
  return spec.buildSqlExpression ? spec.buildSqlExpression(context) : "[]";
}

function buildMetadataScalarSqlExpression(
  context: MetadataSqlContext,
  field: MetadataEnumStringField | MetadataTextStringField | MetadataNumberField | MetadataBooleanField,
): string {
  const spec = isMetadataEnumStringField(field)
    ? getMetadataEnumStringFieldSpec(field)
    : isMetadataTextField(field)
      ? getMetadataTextFieldSpec(field)
      : isMetadataNumberField(field)
        ? getMetadataNumberFieldSpec(field)
        : getMetadataBooleanFieldSpec(field);
  if (!spec.buildSqlExpression) {
    throw new Error(`Metadata field "${field}" does not provide a SQL expression.`);
  }
  return spec.buildSqlExpression(context);
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

function rawMetricOperator(
  op: MetricOperator | NumericMetricOperator,
): "==" | "!=" | ">" | ">=" | "<" | "<=" {
  switch (op) {
    case "eq":
      return "==";
    case "notEq":
      return "!=";
    case "gt":
      return ">";
    case "gte":
      return ">=";
    case "lt":
      return "<";
    case "lte":
      return "<=";
  }
}

function normalizeSearchMetricValueType(
  metric: string,
): { owner: "actor"; valueType: "number" | "text" | "boolean" } | { owner: "item"; valueType: "number" | "text" | "boolean" } | null {
  const actorMetric = inferActorMetricValueType(metric);
  if (actorMetric) {
    return { owner: "actor", valueType: actorMetric };
  }

  const itemMetric = inferItemMetricValueType(metric);
  if (itemMetric) {
    return { owner: "item", valueType: itemMetric };
  }

  return null;
}

function buildActorMetricPredicateClause(
  metric: string,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  valueType: "number" | "text" | "boolean",
  value: string | number | boolean,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const sqlOperator = metricSqlOperator(op);

  if (valueType === "number") {
    if (typeof value !== "number" || !Number.isFinite(value)) {
      throw new Error(`Actor metric "${metric}" requires a finite numeric value.`);
    }
    return {
      clause: `EXISTS (
        SELECT 1
        FROM actor_metrics actor_metric
        WHERE actor_metric.record_key = ${context.recordKeyExpr}
          AND actor_metric.metric_key = ?
          AND actor_metric.value_type = 'number'
          AND actor_metric.number_value ${sqlOperator} ?
      )`,
      params: [metric, value],
    };
  }

  if (valueType === "text") {
    if (typeof value !== "string") {
      throw new Error(`Actor metric "${metric}" requires a string value.`);
    }
    return {
      clause: `EXISTS (
        SELECT 1
        FROM actor_metrics actor_metric
        WHERE actor_metric.record_key = ${context.recordKeyExpr}
          AND actor_metric.metric_key = ?
          AND actor_metric.value_type = 'text'
          AND LOWER(COALESCE(actor_metric.text_value, '')) ${sqlOperator} ?
      )`,
      params: [metric, normalizeActorMetricTextValue(value)],
    };
  }

  if (typeof value !== "boolean") {
    throw new Error(`Actor metric "${metric}" requires a boolean value.`);
  }

  return {
    clause: `EXISTS (
      SELECT 1
      FROM actor_metrics actor_metric
      WHERE actor_metric.record_key = ${context.recordKeyExpr}
        AND actor_metric.metric_key = ?
        AND actor_metric.value_type = 'boolean'
        AND COALESCE(actor_metric.bool_value, 0) ${sqlOperator} ?
    )`,
    params: [metric, value ? 1 : 0],
  };
}

function buildItemMetricPredicateClause(
  metric: string,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  valueType: "number" | "text" | "boolean",
  value: string | number | boolean,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const sqlOperator = metricSqlOperator(op);

  if (valueType === "number") {
    if (typeof value !== "number" || !Number.isFinite(value)) {
      throw new Error(`Item metric "${metric}" requires a finite numeric value.`);
    }
    return {
      clause: `EXISTS (
        SELECT 1
        FROM item_metrics item_metric
        WHERE item_metric.record_key = ${context.recordKeyExpr}
          AND item_metric.metric_key = ?
          AND item_metric.value_type = 'number'
          AND item_metric.number_value ${sqlOperator} ?
      )`,
      params: [metric, value],
    };
  }

  if (valueType === "text") {
    if (typeof value !== "string") {
      throw new Error(`Item metric "${metric}" requires a string value.`);
    }
    return {
      clause: `EXISTS (
        SELECT 1
        FROM item_metrics item_metric
        WHERE item_metric.record_key = ${context.recordKeyExpr}
          AND item_metric.metric_key = ?
          AND item_metric.value_type = 'text'
          AND LOWER(COALESCE(item_metric.text_value, '')) ${sqlOperator} ?
      )`,
      params: [metric, normalizeActorMetricTextValue(value)],
    };
  }

  if (typeof value !== "boolean") {
    throw new Error(`Item metric "${metric}" requires a boolean value.`);
  }

  return {
    clause: `EXISTS (
      SELECT 1
      FROM item_metrics item_metric
      WHERE item_metric.record_key = ${context.recordKeyExpr}
        AND item_metric.metric_key = ?
        AND item_metric.value_type = 'boolean'
        AND COALESCE(item_metric.bool_value, 0) ${sqlOperator} ?
    )`,
    params: [metric, value ? 1 : 0],
  };
}

function buildActorMetricCompareClause(
  leftMetric: string,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  rightMetric: string,
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
        AND left_metric.number_value ${metricSqlOperator(op)} right_metric.number_value
    )`,
    params: [leftMetric, rightMetric],
  };
}

function buildItemMetricCompareClause(
  leftMetric: string,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  rightMetric: string,
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
        AND left_metric.number_value ${metricSqlOperator(op)} right_metric.number_value
    )`,
    params: [leftMetric, rightMetric],
  };
}

function compareMetricValues(
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

export function normalizeMetadataAtomicPredicate(predicate: MetadataAtomicPredicate): MetadataAtomicPredicate {
  if (isMetadataSetField(predicate.field)) {
    if (predicate.op === "isNull" || predicate.op === "isNotNull") {
      return predicate;
    }
    if (typeof predicate.value !== "string" || predicate.value.trim().length === 0) {
      throw new Error(`metadata predicate "${predicate.field}" requires a string value.`);
    }
    return {
      field: predicate.field,
      op: predicate.op,
      value: normalizeMetadataValue(predicate.field, predicate.value),
    } as MetadataAtomicPredicate;
  }

  if (isMetadataEnumStringField(predicate.field)) {
    if (predicate.op === "isNull" || predicate.op === "isNotNull") {
      return predicate;
    }
    if (typeof predicate.value !== "string" || predicate.value.trim().length === 0) {
      throw new Error(`metadata predicate "${predicate.field}" requires a string value.`);
    }
    return {
      field: predicate.field,
      op: predicate.op,
      value: normalizeMetadataValue(predicate.field, predicate.value),
    } as MetadataAtomicPredicate;
  }

  if (isMetadataTextField(predicate.field)) {
    if (predicate.op === "isNull" || predicate.op === "isNotNull") {
      return predicate;
    }
    if (typeof predicate.value !== "string") {
      throw new Error(`metadata predicate "${predicate.field}" requires a string value.`);
    }
    return {
      field: predicate.field,
      op: predicate.op,
      value: normalizeMetadataTextMatchValue(predicate.value),
    } as MetadataAtomicPredicate;
  }

  if (isMetadataNumberField(predicate.field)) {
    if (predicate.op === "isNull" || predicate.op === "isNotNull") {
      return predicate;
    }
    if (predicate.op === "between") {
      if (!Number.isFinite(predicate.min) || !Number.isFinite(predicate.max)) {
        throw new Error(`metadata predicate "${predicate.field}" requires finite min/max values.`);
      }
      if ((predicate.min as number) > (predicate.max as number)) {
        throw new Error(`metadata predicate "${predicate.field}" requires min <= max.`);
      }
      return predicate;
    }
    if (!Number.isFinite(predicate.value)) {
      throw new Error(`metadata predicate "${predicate.field}" requires a finite numeric value.`);
    }
    return predicate;
  }

  if (predicate.op === "isNull" || predicate.op === "isNotNull") {
    return predicate;
  }

  if (typeof predicate.value !== "boolean") {
    throw new Error(`metadata predicate "${predicate.field}" requires a boolean value.`);
  }

  return predicate;
}

export function buildMetadataAtomicPredicateClause(
  predicate: MetadataAtomicPredicate,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const normalized = normalizeMetadataAtomicPredicate(predicate) as MetadataAtomicPredicate & Record<string, unknown>;

  if (isMetadataSetField(normalized.field)) {
    if (normalized.op === "isNull") {
      const expression = buildMetadataJsonArraySql(context, normalized.field);
      return {
        clause: `COALESCE(json_array_length(${expression}), 0) = 0`,
        params: [],
      };
    }

    if (normalized.op === "isNotNull") {
      const expression = buildMetadataJsonArraySql(context, normalized.field);
      return {
        clause: `COALESCE(json_array_length(${expression}), 0) > 0`,
        params: [],
      };
    }

    const value = normalized.value as string;
    if (normalized.field === "traits") {
      return {
        clause: `EXISTS (
          SELECT 1
          FROM record_traits value_set
          WHERE value_set.record_key = ${context.recordKeyExpr}
            AND value_set.trait = ?
        )`,
        params: [value],
      };
    }

    if (normalized.field === "derivedTags") {
      return {
        clause: `EXISTS (
          SELECT 1
          FROM record_derived_tags value_set
          WHERE value_set.record_key = ${context.recordKeyExpr}
            AND value_set.tag = ?
        )`,
        params: [value],
      };
    }

    return {
      clause: `EXISTS (
        SELECT 1
        FROM json_each(${buildMetadataJsonArraySql(context, normalized.field)}) AS value_set
        WHERE LOWER(value_set.value) = ?
      )`,
      params: [value],
    };
  }

  const expression = buildMetadataScalarSqlExpression(context, normalized.field as never);

  if (isMetadataEnumStringField(normalized.field)) {
    if (normalized.op === "isNull") {
      return { clause: `${expression} IS NULL OR TRIM(${expression}) = ''`, params: [] };
    }
    if (normalized.op === "isNotNull") {
      return { clause: `${expression} IS NOT NULL AND TRIM(${expression}) <> ''`, params: [] };
    }
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${normalized.op === "notEq" ? "<>" : "="} ?`,
      params: [normalized.value as SqlValue],
    };
  }

  if (isMetadataTextField(normalized.field)) {
    if (normalized.op === "isNull") {
      return { clause: `${expression} IS NULL OR TRIM(${expression}) = ''`, params: [] };
    }
    if (normalized.op === "isNotNull") {
      return { clause: `${expression} IS NOT NULL AND TRIM(${expression}) <> ''`, params: [] };
    }
    if (normalized.op === "contains" || normalized.op === "notContains") {
      return {
        clause: `LOWER(COALESCE(${expression}, '')) ${normalized.op === "notContains" ? "NOT " : ""}LIKE ?`,
        params: [`%${normalized.value as string}%`],
      };
    }
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${normalized.op === "notEq" ? "<>" : "="} ?`,
      params: [normalized.value as SqlValue],
    };
  }

  if (isMetadataNumberField(normalized.field)) {
    if (normalized.op === "isNull") {
      return { clause: `${expression} IS NULL`, params: [] };
    }
    if (normalized.op === "isNotNull") {
      return { clause: `${expression} IS NOT NULL`, params: [] };
    }
    if (normalized.op === "between") {
      return {
        clause: `(${expression} >= ? AND ${expression} <= ?)`,
        params: [normalized.min as number, normalized.max as number],
      };
    }
    const operator =
      normalized.op === "eq"
        ? "="
        : normalized.op === "notEq"
          ? "<>"
          : normalized.op === "gt"
            ? ">"
            : normalized.op === "gte"
              ? ">="
              : normalized.op === "lt"
                ? "<"
                : "<=";
    return {
      clause: `${expression} ${operator} ?`,
      params: [normalized.value as SqlValue],
    };
  }

  if (normalized.op === "isNull") {
    return { clause: `${expression} IS NULL`, params: [] };
  }
  if (normalized.op === "isNotNull") {
    return { clause: `${expression} IS NOT NULL`, params: [] };
  }

  return {
    clause: `COALESCE(${expression}, 0) ${normalized.op === "notEq" ? "<>" : "="} ?`,
    params: [normalized.value ? 1 : 0],
  };
}

export function recordMatchesMetadataAtomicPredicate(record: NormalizedRecord, predicate: MetadataAtomicPredicate): boolean {
  const normalized = normalizeMetadataAtomicPredicate(predicate) as MetadataAtomicPredicate & Record<string, unknown>;

  if (isMetadataSetField(normalized.field)) {
    const normalizedValues = new Set(
      getMetadataSetRecordValues(record, normalized.field)
        .map((value) => normalizeMetadataValue(normalized.field as MetadataSetField, value))
        .filter(Boolean),
    );
    if (normalized.op === "isNull") {
      return normalizedValues.size === 0;
    }
    if (normalized.op === "isNotNull") {
      return normalizedValues.size > 0;
    }
    return normalizedValues.has(normalized.value as string);
  }

  if (isMetadataEnumStringField(normalized.field)) {
    const value = getMetadataStringRecordValue(record, normalized.field);
    const normalizedValue = value === null ? null : normalizeMetadataValue(normalized.field, value);
    if (normalized.op === "isNull") {
      return normalizedValue === null || normalizedValue === "";
    }
    if (normalized.op === "isNotNull") {
      return normalizedValue !== null && normalizedValue !== "";
    }
    return normalized.op === "eq" ? normalizedValue === normalized.value : normalizedValue !== normalized.value;
  }

  if (isMetadataTextField(normalized.field)) {
    const value = getMetadataStringRecordValue(record, normalized.field);
    const normalizedValue = value === null ? null : normalizeMetadataTextMatchValue(value);
    if (normalized.op === "isNull") {
      return normalizedValue === null || normalizedValue === "";
    }
    if (normalized.op === "isNotNull") {
      return normalizedValue !== null && normalizedValue !== "";
    }
    if (normalized.op === "contains") {
      return Boolean(normalizedValue?.includes(normalized.value as string));
    }
    if (normalized.op === "notContains") {
      return !normalizedValue?.includes(normalized.value as string);
    }
    return normalized.op === "eq" ? normalizedValue === normalized.value : normalizedValue !== normalized.value;
  }

  if (isMetadataNumberField(normalized.field)) {
    const value = getMetadataNumberRecordValue(record, normalized.field);
    if (normalized.op === "isNull") {
      return value === null;
    }
    if (normalized.op === "isNotNull") {
      return value !== null;
    }
    if (value === null) {
      return false;
    }
    if (normalized.op === "between") {
      return value >= (normalized.min as number) && value <= (normalized.max as number);
    }
    if (normalized.op === "eq") {
      return value === (normalized.value as number);
    }
    if (normalized.op === "notEq") {
      return value !== (normalized.value as number);
    }
    if (normalized.op === "gt") {
      return value > (normalized.value as number);
    }
    if (normalized.op === "gte") {
      return value >= (normalized.value as number);
    }
    if (normalized.op === "lt") {
      return value < (normalized.value as number);
    }
    return value <= (normalized.value as number);
  }

  const value = getMetadataBooleanRecordValue(record, normalized.field);
  if (normalized.op === "isNull") {
    return value === null;
  }
  if (normalized.op === "isNotNull") {
    return value !== null;
  }
  return normalized.op === "eq" ? value === normalized.value : value !== normalized.value;
}

export function normalizeSearchMetricKey(metric: string): string {
  const actorMetric = normalizeActorMetricKey(metric);
  if (actorMetric && inferActorMetricValueType(actorMetric)) {
    return actorMetric;
  }

  const itemMetric = normalizeItemMetricKey(metric);
  if (itemMetric && inferItemMetricValueType(itemMetric)) {
    return itemMetric;
  }

  throw new Error(`Unknown metric "${metric}".`);
}

export function buildMetricPredicateClause(
  metric: string,
  op: MetricOperator,
  value: string | number | boolean,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const normalizedMetric = normalizeSearchMetricKey(metric);
  const rawOp = rawMetricOperator(op);
  const metricInfo = normalizeSearchMetricValueType(normalizedMetric);
  if (!metricInfo) {
    throw new Error(`Unknown metric "${metric}".`);
  }

  return metricInfo.owner === "actor"
    ? buildActorMetricPredicateClause(normalizedMetric, rawOp, metricInfo.valueType, value, context)
    : buildItemMetricPredicateClause(normalizedMetric, rawOp, metricInfo.valueType, value, context);
}

export function buildMetricCompareClause(
  leftMetric: string,
  op: NumericMetricOperator,
  rightMetric: string,
  context: MetadataSqlContext,
): { clause: string; params: SqlValue[] } {
  const normalizedLeftMetric = normalizeSearchMetricKey(leftMetric);
  const normalizedRightMetric = normalizeSearchMetricKey(rightMetric);
  const rawOp = rawMetricOperator(op);

  const actorLeft = inferActorMetricValueType(normalizedLeftMetric) === "number";
  const actorRight = inferActorMetricValueType(normalizedRightMetric) === "number";
  if (actorLeft && actorRight) {
    return buildActorMetricCompareClause(normalizedLeftMetric, rawOp, normalizedRightMetric, context);
  }

  const itemLeft = inferItemMetricValueType(normalizedLeftMetric) === "number";
  const itemRight = inferItemMetricValueType(normalizedRightMetric) === "number";
  if (itemLeft && itemRight) {
    return buildItemMetricCompareClause(normalizedLeftMetric, rawOp, normalizedRightMetric, context);
  }

  throw new Error(`Unknown numeric metric comparison "${leftMetric}" ${op} "${rightMetric}".`);
}

export function recordMatchesMetricPredicate(
  record: NormalizedRecord,
  metric: string,
  op: MetricOperator,
  value: string | number | boolean,
): boolean {
  const normalizedMetric = normalizeSearchMetricKey(metric);
  const rawOp = rawMetricOperator(op);

  const actorMetricType = inferActorMetricValueType(normalizedMetric);
  const actorValue = actorMetricType ? record.actorMetrics[normalizedMetric] : undefined;
  if (actorValue !== undefined) {
    const normalizedValue =
      typeof value === "string" && actorMetricType === "text" ? normalizeActorMetricTextValue(value) : value;
    return compareMetricValues(actorValue, rawOp, normalizedValue as ActorMetricValue);
  }

  const itemMetricType = inferItemMetricValueType(normalizedMetric);
  const itemValue = itemMetricType ? record.itemMetrics[normalizedMetric] : undefined;
  if (itemValue !== undefined) {
    const normalizedValue =
      typeof value === "string" && itemMetricType === "text" ? normalizeActorMetricTextValue(value) : value;
    return compareMetricValues(itemValue, rawOp, normalizedValue as ActorMetricValue);
  }

  return false;
}

export function recordMatchesMetricComparePredicate(
  record: NormalizedRecord,
  leftMetric: string,
  op: NumericMetricOperator,
  rightMetric: string,
): boolean {
  const normalizedLeftMetric = normalizeSearchMetricKey(leftMetric);
  const normalizedRightMetric = normalizeSearchMetricKey(rightMetric);
  const rawOp = rawMetricOperator(op);

  const actorLeftValue = record.actorMetrics[normalizedLeftMetric];
  const actorRightValue = record.actorMetrics[normalizedRightMetric];
  if (typeof actorLeftValue === "number" && typeof actorRightValue === "number") {
    return compareMetricValues(actorLeftValue, rawOp, actorRightValue);
  }

  const itemLeftValue = record.itemMetrics[normalizedLeftMetric];
  const itemRightValue = record.itemMetrics[normalizedRightMetric];
  if (typeof itemLeftValue === "number" && typeof itemRightValue === "number") {
    return compareMetricValues(itemLeftValue, rawOp, itemRightValue);
  }

  return false;
}
