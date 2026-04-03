import {
  METADATA_BOOLEAN_FIELDS,
  METADATA_ENUM_STRING_FIELDS,
  METADATA_NUMBER_FIELDS,
  METADATA_SET_FIELDS,
  METADATA_TEXT_STRING_FIELDS,
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

export type MetadataSqlContext = {
  recordKeyExpr: string;
  recordsAlias?: string;
  actorAlias?: string;
  itemAlias?: string;
  spellAlias?: string;
};

const METADATA_SET_FIELD_NAMES = new Set<string>(METADATA_SET_FIELDS);
const METADATA_ENUM_STRING_FIELD_NAMES = new Set<string>(METADATA_ENUM_STRING_FIELDS);
const METADATA_TEXT_STRING_FIELD_NAMES = new Set<string>(METADATA_TEXT_STRING_FIELDS);
const METADATA_NUMBER_FIELD_NAMES = new Set<string>(METADATA_NUMBER_FIELDS);
const METADATA_BOOLEAN_FIELD_NAMES = new Set<string>(METADATA_BOOLEAN_FIELDS);

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function normalizeMetadataValue(field: MetadataSetField | MetadataEnumStringField, value: string): string {
  if (field === "derivedTags") {
    return normalizeDerivedTag(value);
  }

  return normalizeText(value);
}

function normalizeMetadataTextMatchValue(value: string): string {
  return value.trim().toLowerCase();
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

  if (METADATA_SET_FIELD_NAMES.has(field)) {
    if (!["includesAny", "includesAll", "excludesAny"].includes(op)) {
      throw new Error(`Unsupported metadata operator "${op}" for set field "${field}".`);
    }

    if (!Array.isArray(raw.values) || raw.values.length === 0 || !raw.values.every((value): value is string => typeof value === "string")) {
      throw new Error(`metadata predicate "${field}" requires a non-empty string values array.`);
    }

    return {
      field: field as MetadataSetField,
      op: op as "includesAny" | "includesAll" | "excludesAny",
      values: normalizeMetadataValues(field as MetadataSetField, raw.values),
    };
  }

  if (METADATA_ENUM_STRING_FIELD_NAMES.has(field)) {
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
      if (!Array.isArray(raw.values) || raw.values.length === 0 || !raw.values.every((value): value is string => typeof value === "string")) {
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

  if (METADATA_TEXT_STRING_FIELD_NAMES.has(field)) {
    if (!["contains", "notContains"].includes(op) || typeof raw.value !== "string") {
      throw new Error(`metadata predicate "${field}" requires op "contains" or "notContains" with a string value.`);
    }

    return {
      field: field as MetadataTextStringField,
      op: op as "contains" | "notContains",
      value: normalizeMetadataTextMatchValue(raw.value),
    };
  }

  if (METADATA_NUMBER_FIELD_NAMES.has(field)) {
    if (op === "between") {
      if (typeof raw.min !== "number" || !Number.isFinite(raw.min) || typeof raw.max !== "number" || !Number.isFinite(raw.max)) {
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

  if (METADATA_BOOLEAN_FIELD_NAMES.has(field)) {
    if (op !== "eq" || typeof raw.value !== "boolean") {
      throw new Error(`metadata predicate "${field}" requires op "eq" with a boolean value.`);
    }

    return {
      field: field as MetadataBooleanField,
      op,
      value: raw.value,
    };
  }

  throw new Error(`Unknown metadata field "${field}".`);
}

function buildScalarLookupSql(
  recordKeyExpr: string,
  alias: string | undefined,
  column: string,
  table: string,
): string {
  return alias ? `${alias}.${column}` : `(SELECT meta.${column} FROM ${table} meta WHERE meta.record_key = ${recordKeyExpr})`;
}

export function buildFamiliesArraySql(recordAlias: string): string {
  return `COALESCE(${recordAlias}.families_json, '[]')`;
}

function buildMetadataJsonArraySql(context: MetadataSqlContext, field: MetadataSetField): string {
  switch (field) {
    case "families":
      return context.recordsAlias ? buildFamiliesArraySql(context.recordsAlias) : `COALESCE((SELECT meta.families_json FROM records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`;
    case "traditions":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "traditions_json", "spell_records")}, '[]')`;
    case "spellKinds":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "spell_kinds_json", "spell_records")}, '[]')`;
    case "damageTypes":
      if (context.spellAlias || context.itemAlias) {
        return `COALESCE(${context.spellAlias ?? "NULL"}.damage_types_json, ${context.itemAlias ?? "NULL"}.damage_types_json, '[]')`;
      }
      return `COALESCE((SELECT meta.damage_types_json FROM spell_records meta WHERE meta.record_key = ${context.recordKeyExpr}), (SELECT meta.damage_types_json FROM item_records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`;
    case "languages":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "languages_json", "actor_records")}, '[]')`;
    case "speedTypes":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "speed_types_json", "actor_records")}, '[]')`;
    case "immunities":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "immunities_json", "actor_records")}, '[]')`;
    case "resistances":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "resistances_json", "actor_records")}, '[]')`;
    case "weaknesses":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "weaknesses_json", "actor_records")}, '[]')`;
    default:
      return "[]";
  }
}

function buildMetadataScalarSqlExpression(
  context: MetadataSqlContext,
  field: MetadataEnumStringField | MetadataTextStringField | MetadataNumberField | MetadataBooleanField,
): string {
  switch (field) {
    case "sourceCategory":
      return context.recordsAlias ? `${context.recordsAlias}.source_category` : `(SELECT meta.source_category FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "publicationTitle":
      return context.recordsAlias ? `${context.recordsAlias}.publication_title` : `(SELECT meta.publication_title FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "size":
      return buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "size", "actor_records");
    case "usage":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "usage_text", "item_records");
    case "weaponGroup":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "weapon_group", "item_records");
    case "armorGroup":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "armor_group", "item_records");
    case "itemCategory":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "item_category", "item_records");
    case "rarity":
      return context.recordsAlias ? `${context.recordsAlias}.rarity` : `(SELECT meta.rarity FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "level":
      return context.recordsAlias ? `${context.recordsAlias}.level` : `(SELECT meta.level FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "priceCp":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "price_cp", "item_records");
    case "bulkValue":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "bulk_value", "item_records");
    case "actionCost":
      if (context.spellAlias || context.itemAlias) {
        return `COALESCE(${context.spellAlias ?? "NULL"}.action_cost, ${context.itemAlias ?? "NULL"}.action_cost)`;
      }
      return `COALESCE((SELECT meta.action_cost FROM spell_records meta WHERE meta.record_key = ${context.recordKeyExpr}), (SELECT meta.action_cost FROM item_records meta WHERE meta.record_key = ${context.recordKeyExpr}))`;
    case "hands":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "hands", "item_records");
    case "rangeValue":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_value", "spell_records");
    case "isUnique":
      return context.recordsAlias ? `${context.recordsAlias}.is_unique` : `(SELECT meta.is_unique FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "hasDescription":
      return context.recordsAlias ? `${context.recordsAlias}.has_description` : `(SELECT meta.has_description FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
    case "publicationRemaster":
      return context.recordsAlias ? `${context.recordsAlias}.publication_remaster` : `(SELECT meta.publication_remaster FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
  }
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
  if (METADATA_SET_FIELD_NAMES.has(predicate.field)) {
    const setPredicate = predicate as Extract<MetadataPredicate, { field: MetadataSetField }>;
    return buildMetadataSetPredicateClause(setPredicate.field, setPredicate.op, setPredicate.values, context);
  }

  if (METADATA_ENUM_STRING_FIELD_NAMES.has(predicate.field)) {
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

  if (METADATA_TEXT_STRING_FIELD_NAMES.has(predicate.field)) {
    const textPredicate = predicate as Extract<MetadataPredicate, { field: MetadataTextStringField }>;
    const expression = buildMetadataScalarSqlExpression(context, textPredicate.field);
    return {
      clause: `LOWER(COALESCE(${expression}, '')) ${textPredicate.op === "notContains" ? "NOT " : ""}LIKE ?`,
      params: [`%${textPredicate.value}%`],
    };
  }

  if (METADATA_NUMBER_FIELD_NAMES.has(predicate.field)) {
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
  switch (field) {
    case "traits":
      return record.traits;
    case "families":
      return record.families;
    case "derivedTags":
      return record.derivedTags;
    case "traditions":
      return record.traditions;
    case "spellKinds":
      return record.spellKinds;
    case "damageTypes":
      return record.damageTypes;
    case "languages":
      return record.languages;
    case "speedTypes":
      return record.speedTypes;
    case "immunities":
      return record.immunities;
    case "resistances":
      return record.resistances;
    case "weaknesses":
      return record.weaknesses;
  }
}

function getRecordStringValue(record: NormalizedRecord, field: MetadataEnumStringField | MetadataTextStringField): string | null {
  switch (field) {
    case "sourceCategory":
      return record.sourceCategory;
    case "publicationTitle":
      return record.publicationTitle;
    case "size":
      return record.size;
    case "usage":
      return record.usage;
    case "weaponGroup":
      return record.weaponGroup;
    case "armorGroup":
      return record.armorGroup;
    case "itemCategory":
      return record.itemCategory;
    case "rarity":
      return record.rarity;
  }
}

function getRecordNumberValue(record: NormalizedRecord, field: MetadataNumberField): number | null {
  switch (field) {
    case "level":
      return record.level;
    case "priceCp":
      return record.priceCp;
    case "bulkValue":
      return record.bulkValue;
    case "actionCost":
      return record.actionCost;
    case "hands":
      return record.hands;
    case "rangeValue":
      return record.rangeValue;
  }
}

function getRecordBooleanValue(record: NormalizedRecord, field: MetadataBooleanField): boolean {
  switch (field) {
    case "isUnique":
      return record.isUnique;
    case "hasDescription":
      return record.hasDescription;
    case "publicationRemaster":
      return record.publicationRemaster;
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

  if (METADATA_SET_FIELD_NAMES.has(node.field)) {
    const setPredicate = node as Extract<MetadataPredicate, { field: MetadataSetField }>;
    const normalizedValues = new Set(getRecordSetValues(record, setPredicate.field).map((value) => normalizeMetadataValue(setPredicate.field, value)).filter(Boolean));
    if (setPredicate.op === "includesAll") {
      return setPredicate.values.every((value) => normalizedValues.has(value));
    }
    if (setPredicate.op === "includesAny") {
      return setPredicate.values.some((value) => normalizedValues.has(value));
    }
    return !setPredicate.values.some((value) => normalizedValues.has(value));
  }

  if (METADATA_ENUM_STRING_FIELD_NAMES.has(node.field)) {
    const stringPredicate = node as Extract<MetadataPredicate, { field: MetadataEnumStringField }>;
    const normalizedValue = normalizeMetadataValue(stringPredicate.field, getRecordStringValue(record, stringPredicate.field) ?? "");
    if (stringPredicate.op === "eq") {
      return normalizedValue === stringPredicate.value;
    }
    if (stringPredicate.op === "in") {
      return stringPredicate.values.includes(normalizedValue);
    }
    return !stringPredicate.values.includes(normalizedValue);
  }

  if (METADATA_TEXT_STRING_FIELD_NAMES.has(node.field)) {
    const textPredicate = node as Extract<MetadataPredicate, { field: MetadataTextStringField }>;
    const normalizedValue = normalizeMetadataTextMatchValue(getRecordStringValue(record, textPredicate.field) ?? "");
    return textPredicate.op === "contains"
      ? normalizedValue.includes(textPredicate.value)
      : !normalizedValue.includes(textPredicate.value);
  }

  if (METADATA_NUMBER_FIELD_NAMES.has(node.field)) {
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
