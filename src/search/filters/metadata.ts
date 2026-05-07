import {
  inferActorMetricValueType,
  normalizeActorMetricKey,
  normalizeActorMetricTextValue,
  type ActorMetricValue,
} from "../../domain/actor-metrics.js";
import { inferItemMetricValueType, normalizeItemMetricKey } from "../../domain/item-metrics.js";
import {
  type MetadataEnumStringField,
  type MetadataSetField,
} from "../../domain/metadata-field-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { MetadataAtomicPredicate } from "../../domain/search-filter-metadata.js";
import type { MetricOperator, NumericMetricOperator } from "../../domain/search-filter-operators.js";
import { normalizeText } from "../../shared/utils.js";
import { normalizeDerivedTag } from "../../tags/runtime.js";
import {
  getMetadataBooleanRecordValue,
  getMetadataEnumStringExecutionSpec,
  getMetadataNumberRecordValue,
  getMetadataSetExecutionSpec,
  getMetadataSetRecordValues,
  getMetadataStringRecordValue,
  isMetadataEnumStringField,
  isMetadataNumberField,
  isMetadataSetField,
  isMetadataTextField,
  type MetadataExecutionSpecEntry,
  type MetadataValueNormalization,
} from "./metadata-execution.js";

function normalizeMetadataValue(field: MetadataSetField | MetadataEnumStringField, value: string): string {
  const spec = isMetadataSetField(field) ? getMetadataSetExecutionSpec(field) : getMetadataEnumStringExecutionSpec(field);
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

function inferMetadataValueNormalization(_spec: MetadataExecutionSpecEntry): MetadataValueNormalization {
  return "lowercaseTrim";
}

export function rawMetricOperator(
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

export function normalizeSearchMetricValueType(
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
    return false;
  }
  if (normalized.op === "isNotNull") {
    return true;
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
