import {
  inferActorMetricValueType,
  normalizeActorMetricKey,
  normalizeActorMetricTextValue,
  type ActorMetricValue,
} from "../domain/actor-metrics.js";
import { inferItemMetricValueType, normalizeItemMetricKey } from "../domain/item-metrics.js";
import {
  getMetadataBooleanFieldSpec,
  getMetadataBooleanRecordValue,
  getMetadataEnumStringFieldSpec,
  METADATA_FIELD_SPEC_BY_NAME,
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
} from "../domain/metadata-field-registry.js";
import {
  ACTOR_METRIC_COMPARE_PREDICATE_SPEC,
  ACTOR_METRIC_PREDICATE_SPEC,
  ITEM_METRIC_COMPARE_PREDICATE_SPEC,
  ITEM_METRIC_PREDICATE_SPEC,
  getMetadataFieldPredicateVariant,
  getMetricValuePredicateOperatorKind,
  type MetadataMetricComparePredicateSpec,
  type MetadataMetricValuePredicateSpec,
} from "../domain/metadata-predicate-spec.js";
import {
  MetadataBooleanField,
  MetadataBooleanPredicate,
  MetadataEnumStringField,
  MetadataEnumStringPredicate,
  MetadataFilterNode,
  MetadataNumberField,
  MetadataNumberPredicate,
  MetadataPredicate,
  MetadataSetPredicate,
  MetadataSetField,
  MetadataTextStringField,
  MetadataTextStringPredicate,
  NormalizedRecord,
} from "../types.js";
import { normalizeDerivedTag } from "../tags/index.js";
import { normalizeText } from "../utils.js";
import type { SqlValue } from "./contracts.js";

export type MetadataSqlContext = {} & MetadataSqlSourceContext;

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

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

function normalizeMetricValuePredicate(
  raw: Record<string, unknown>,
  op: string,
  spec: MetadataMetricValuePredicateSpec,
  normalizeMetricName: (metric: unknown, label: string) => string,
  inferMetricValueType: (metric: string) => "number" | "text" | "boolean" | null,
  metricLabel: "actor" | "item",
): Extract<MetadataPredicate, { field: typeof spec.field }> {
  const metric = normalizeMetricName(raw[spec.metricKey], `${spec.field}.${spec.metricKey}`);
  const metricType = inferMetricValueType(metric);
  if (!metricType) {
    throw new Error(`Unknown ${metricLabel} metric "${String(raw[spec.metricKey])}".`);
  }

  const operatorKind = getMetricValuePredicateOperatorKind(spec, op);
  if (metricType === "number") {
    if (operatorKind !== "numeric" || typeof raw.value !== "number" || !Number.isFinite(raw.value)) {
      throw new Error(`${metricLabel} metric "${metric}" requires a finite numeric value with one of ==, !=, >, >=, <, <=.`);
    }

    return {
      field: spec.field,
      [spec.metricKey]: metric,
      op,
      value: raw.value,
    } as Extract<MetadataPredicate, { field: typeof spec.field }>;
  }

  if (operatorKind !== "scalar") {
    throw new Error(`${metricLabel} metric "${metric}" only supports == and !=.`);
  }

  if (metricType === "text") {
    if (typeof raw.value !== "string") {
      throw new Error(`${metricLabel} metric "${metric}" requires a string value.`);
    }

    return {
      field: spec.field,
      [spec.metricKey]: metric,
      op,
      value: normalizeActorMetricTextValue(raw.value),
    } as Extract<MetadataPredicate, { field: typeof spec.field }>;
  }

  if (typeof raw.value !== "boolean") {
    throw new Error(`${metricLabel} metric "${metric}" requires a boolean value.`);
  }

  return {
    field: spec.field,
    [spec.metricKey]: metric,
    op,
    value: raw.value,
  } as Extract<MetadataPredicate, { field: typeof spec.field }>;
}

function normalizeMetricComparePredicate(
  raw: Record<string, unknown>,
  op: string,
  spec: MetadataMetricComparePredicateSpec,
  normalizeMetricName: (metric: unknown, label: string) => string,
  inferMetricValueType: (metric: string) => "number" | "text" | "boolean" | null,
  metricLabel: "actorMetricCompare" | "itemMetricCompare",
): Extract<MetadataPredicate, { field: typeof spec.field }> {
  const leftMetric = normalizeMetricName(raw[spec.leftMetricKey], `${spec.field}.${spec.leftMetricKey}`);
  const rightMetric = normalizeMetricName(raw[spec.rightMetricKey], `${spec.field}.${spec.rightMetricKey}`);
  if (inferMetricValueType(leftMetric) !== "number" || inferMetricValueType(rightMetric) !== "number") {
    throw new Error(`${metricLabel} only supports numeric ${metricLabel === "actorMetricCompare" ? "actor" : "item"} metrics.`);
  }
  if (!spec.operators.includes(op)) {
    throw new Error(`${metricLabel} requires one of ==, !=, >, >=, <, <=.`);
  }

  return {
    field: spec.field,
    [spec.leftMetricKey]: leftMetric,
    op,
    [spec.rightMetricKey]: rightMetric,
  } as Extract<MetadataPredicate, { field: typeof spec.field }>;
}

function normalizeMetadataValues(field: MetadataSetField | MetadataEnumStringField, values: string[]): string[] {
  return [...new Set(values.map((value) => normalizeMetadataValue(field, value)).filter(Boolean))];
}

function isMetadataSetPredicate(
  predicate: MetadataPredicate,
): predicate is Extract<MetadataPredicate, { field: MetadataSetField }> {
  return isMetadataSetField(predicate.field);
}

function isMetadataEnumStringPredicate(
  predicate: MetadataPredicate,
): predicate is Extract<MetadataPredicate, { field: MetadataEnumStringField }> {
  return isMetadataEnumStringField(predicate.field);
}

function isMetadataTextPredicate(
  predicate: MetadataPredicate,
): predicate is Extract<MetadataPredicate, { field: MetadataTextStringField }> {
  return isMetadataTextField(predicate.field);
}

function isMetadataNumberPredicate(
  predicate: MetadataPredicate,
): predicate is Extract<MetadataPredicate, { field: MetadataNumberField }> {
  return isMetadataNumberField(predicate.field);
}

function isMetadataBooleanPredicate(
  predicate: MetadataPredicate,
): predicate is Extract<MetadataPredicate, { field: MetadataBooleanField }> {
  return isMetadataBooleanField(predicate.field);
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

  if (field === ACTOR_METRIC_PREDICATE_SPEC.field) {
    return normalizeMetricValuePredicate(
      raw,
      op,
      ACTOR_METRIC_PREDICATE_SPEC,
      normalizeActorMetricName,
      inferActorMetricValueType,
      "actor",
    );
  }

  if (field === ACTOR_METRIC_COMPARE_PREDICATE_SPEC.field) {
    return normalizeMetricComparePredicate(
      raw,
      op,
      ACTOR_METRIC_COMPARE_PREDICATE_SPEC,
      normalizeActorMetricName,
      inferActorMetricValueType,
      "actorMetricCompare",
    );
  }

  if (field === ITEM_METRIC_PREDICATE_SPEC.field) {
    return normalizeMetricValuePredicate(
      raw,
      op,
      ITEM_METRIC_PREDICATE_SPEC,
      normalizeItemMetricName,
      inferItemMetricValueType,
      "item",
    );
  }

  if (field === ITEM_METRIC_COMPARE_PREDICATE_SPEC.field) {
    return normalizeMetricComparePredicate(
      raw,
      op,
      ITEM_METRIC_COMPARE_PREDICATE_SPEC,
      normalizeItemMetricName,
      inferItemMetricValueType,
      "itemMetricCompare",
    );
  }

  const metadataFieldSpec = METADATA_FIELD_SPEC_BY_NAME.get(field as MetadataSetField);
  if (!metadataFieldSpec) {
    throw new Error(`Unknown metadata field "${field}".`);
  }

  const predicateVariant = getMetadataFieldPredicateVariant(metadataFieldSpec.fieldType, op);

  if (metadataFieldSpec.fieldType === "set") {
    if (!predicateVariant) {
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
      op: predicateVariant.op,
      values: normalizeMetadataValues(field as MetadataSetField, raw.values),
    } as MetadataSetPredicate;
  }

  if (metadataFieldSpec.fieldType === "enumString") {
    if (!predicateVariant) {
      throw new Error(`Unsupported metadata operator "${op}" for string field "${field}".`);
    }

    if (predicateVariant.payload === "string") {
      if (typeof raw.value !== "string") {
        throw new Error(`metadata predicate "${field}" with op "eq" requires a string value.`);
      }

      return {
        field: field as MetadataEnumStringField,
        op: predicateVariant.op,
        value: normalizeMetadataValue(field as MetadataEnumStringField, raw.value),
      } as MetadataEnumStringPredicate;
    }

    if (predicateVariant.payload === "stringArray") {
      if (
        !Array.isArray(raw.values) ||
        raw.values.length === 0 ||
        !raw.values.every((value): value is string => typeof value === "string")
      ) {
        throw new Error(`metadata predicate "${field}" with op "${op}" requires a non-empty string values array.`);
      }

      return {
        field: field as MetadataEnumStringField,
        op: predicateVariant.op,
        values: normalizeMetadataValues(field as MetadataEnumStringField, raw.values),
      } as MetadataEnumStringPredicate;
    }

    throw new Error(`Unsupported metadata operator "${op}" for string field "${field}".`);
  }

  if (metadataFieldSpec.fieldType === "text") {
    if (!predicateVariant || predicateVariant.payload !== "string" || typeof raw.value !== "string") {
      throw new Error(
        `metadata predicate "${field}" requires op "eq", "notEq", "contains", or "notContains" with a string value.`,
      );
    }

    return {
      field: field as MetadataTextStringField,
      op: predicateVariant.op,
      value: normalizeMetadataTextMatchValue(raw.value),
    } as MetadataTextStringPredicate;
  }

  if (metadataFieldSpec.fieldType === "number") {
    if (predicateVariant?.payload === "numberRange") {
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
        op: predicateVariant.op,
        min: raw.min,
        max: raw.max,
      } as MetadataNumberPredicate;
    }

    if (predicateVariant?.payload !== "number" || typeof raw.value !== "number" || !Number.isFinite(raw.value)) {
      throw new Error(`metadata predicate "${field}" requires op "eq", "gte", or "lte" with a finite numeric value.`);
    }

    return {
      field: field as MetadataNumberField,
      op: predicateVariant.op,
      value: raw.value,
    } as MetadataNumberPredicate;
  }

  if (metadataFieldSpec.fieldType !== "boolean" || predicateVariant?.payload !== "boolean" || typeof raw.value !== "boolean") {
    throw new Error(`metadata predicate "${field}" requires op "eq" with a boolean value.`);
  }

  return {
    field: field as MetadataBooleanField,
    op: predicateVariant.op,
    value: raw.value,
  } as MetadataBooleanPredicate;
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

  if (isMetadataSetPredicate(predicate)) {
    return buildMetadataSetPredicateClause(predicate.field, predicate.op, predicate.values, context);
  }

  if (isMetadataEnumStringPredicate(predicate)) {
    const expression = buildMetadataScalarSqlExpression(context, predicate.field);
    if (predicate.op === "eq") {
      return {
        clause: `LOWER(COALESCE(${expression}, '')) = ?`,
        params: [predicate.value],
      };
    }

    const placeholders = predicate.values.map(() => "?").join(", ");
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${predicate.op === "notIn" ? "NOT " : ""}IN (${placeholders})`,
      params: predicate.values,
    };
  }

  if (isMetadataTextPredicate(predicate)) {
    const expression = buildMetadataScalarSqlExpression(context, predicate.field);
    if (predicate.op === "eq" || predicate.op === "notEq") {
      return {
        clause: `LOWER(COALESCE(${expression}, '')) ${predicate.op === "notEq" ? "<>" : "="} ?`,
        params: [predicate.value],
      };
    }
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${predicate.op === "notContains" ? "NOT " : ""}LIKE ?`,
      params: [`%${predicate.value}%`],
    };
  }

  if (isMetadataNumberPredicate(predicate)) {
    const expression = buildMetadataScalarSqlExpression(context, predicate.field);
    if (predicate.op === "between") {
      return {
        clause: `(${expression} >= ? AND ${expression} <= ?)`,
        params: [predicate.min, predicate.max],
      };
    }

    const operator = predicate.op === "eq" ? "=" : predicate.op === "gte" ? ">=" : "<=";
    return {
      clause: `${expression} ${operator} ?`,
      params: [predicate.value],
    };
  }

  if (!isMetadataBooleanPredicate(predicate)) {
    throw new Error("Unknown metadata field.");
  }

  const expression = buildMetadataScalarSqlExpression(context, predicate.field);
  return {
    clause: `COALESCE(${expression}, 0) = ?`,
    params: [predicate.value ? 1 : 0],
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

  if (isMetadataSetPredicate(node)) {
    const normalizedValues = new Set(
      getMetadataSetRecordValues(record, node.field)
        .map((value) => normalizeMetadataValue(node.field, value))
        .filter(Boolean),
    );
    if (node.op === "includesAll") {
      return node.values.every((value) => normalizedValues.has(value));
    }
    if (node.op === "includesAny") {
      return node.values.some((value) => normalizedValues.has(value));
    }
    return !node.values.some((value) => normalizedValues.has(value));
  }

  if (isMetadataEnumStringPredicate(node)) {
    const normalizedValue = normalizeMetadataValue(node.field, getMetadataStringRecordValue(record, node.field) ?? "");
    if (node.op === "eq") {
      return normalizedValue === node.value;
    }
    if (node.op === "in") {
      return node.values.includes(normalizedValue);
    }
    return !node.values.includes(normalizedValue);
  }

  if (isMetadataTextPredicate(node)) {
    const normalizedValue = normalizeMetadataTextMatchValue(getMetadataStringRecordValue(record, node.field) ?? "");
    if (node.op === "eq") {
      return normalizedValue === node.value;
    }
    if (node.op === "notEq") {
      return normalizedValue !== node.value;
    }
    return node.op === "contains" ? normalizedValue.includes(node.value) : !normalizedValue.includes(node.value);
  }

  if (isMetadataNumberPredicate(node)) {
    const numericValue = getMetadataNumberRecordValue(record, node.field);
    if (numericValue === null) {
      return false;
    }
    if (node.op === "between") {
      return numericValue >= node.min && numericValue <= node.max;
    }
    if (node.op === "eq") {
      return numericValue === node.value;
    }
    if (node.op === "gte") {
      return numericValue >= node.value;
    }
    return numericValue <= node.value;
  }

  if (!isMetadataBooleanPredicate(node)) {
    throw new Error("Unknown metadata field.");
  }

  return getMetadataBooleanRecordValue(record, node.field) === node.value;
}
