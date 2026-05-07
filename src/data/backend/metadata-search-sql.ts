import {
  inferActorMetricValueType,
  normalizeActorMetricTextValue,
} from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import {
  isMetadataEnumStringField,
  isMetadataNumberField,
  isMetadataSetField,
  isMetadataTextField,
} from "../../domain/metadata-field-catalog.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFieldName,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../domain/metadata-field-types.js";
import type { MetadataAtomicPredicate } from "../../domain/search-filter-metadata.js";
import type { MetricOperator, NumericMetricOperator } from "../../domain/search-filter-operators.js";
import type { SqlValue } from "../sql-types.js";
import {
  normalizeMetadataAtomicPredicate,
  normalizeSearchMetricKey,
  normalizeSearchMetricValueType,
  rawMetricOperator,
} from "../../search/filters/metadata.js";

export interface MetadataSqlSourceContext {
  recordKeyExpr: string;
  recordsAlias?: string;
  actorAlias?: string;
  itemAlias?: string;
  spellAlias?: string;
}

export interface MetadataFilterValueSource {
  joins?: readonly string[];
  valueExpression: string;
  nonEmptyClause?: string;
}

function buildScalarLookupSql(recordKeyExpr: string, alias: string | undefined, column: string, table: string): string {
  return alias
    ? `${alias}.${column}`
    : `(SELECT meta.${column} FROM ${table} meta WHERE meta.record_key = ${recordKeyExpr})`;
}

function buildRecordScalarSql(context: MetadataSqlSourceContext, column: string): string {
  return context.recordsAlias
    ? `${context.recordsAlias}.${column}`
    : `(SELECT meta.${column} FROM records meta WHERE meta.record_key = ${context.recordKeyExpr})`;
}

function buildRecordJsonSql(context: MetadataSqlSourceContext, column: string): string {
  return `COALESCE(${buildRecordScalarSql(context, column)}, '[]')`;
}

function buildFamiliesArraySql(recordAlias: string): string {
  return `COALESCE(${recordAlias}.families_json, '[]')`;
}

function buildMetadataJsonArraySql(context: MetadataSqlSourceContext, field: MetadataSetField): string {
  switch (field) {
    case "families":
      return context.recordsAlias
        ? buildFamiliesArraySql(context.recordsAlias)
        : `COALESCE((SELECT meta.families_json FROM records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`;
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
    case "senses":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "senses_json", "actor_records")}, '[]')`;
    case "immunities":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "immunities_json", "actor_records")}, '[]')`;
    case "resistances":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "resistances_json", "actor_records")}, '[]')`;
    case "weaknesses":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "weaknesses_json", "actor_records")}, '[]')`;
    case "disableSkills":
      return `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_skills_json", "actor_records")}, '[]')`;
    case "variantAxes":
      return buildRecordJsonSql(context, "variant_axes_json");
    case "traits":
    case "derivedTags":
      return "[]";
  }
}

function buildMetadataScalarSqlExpression(
  context: MetadataSqlSourceContext,
  field: MetadataEnumStringField | MetadataTextStringField | MetadataNumberField | MetadataBooleanField,
): string {
  switch (field) {
    case "sourceCategory":
      return buildRecordScalarSql(context, "source_category");
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
    case "baseItem":
      return buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "base_item", "item_records");
    case "saveType":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "save_type", "spell_records");
    case "areaType":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_type", "spell_records");
    case "durationUnit":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_unit", "spell_records");
    case "rarity":
      return buildRecordScalarSql(context, "rarity");
    case "variantFamilyKey":
      return buildRecordScalarSql(context, "variant_family_key");
    case "publicationTitle":
      return buildRecordScalarSql(context, "publication_title");
    case "rangeText":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_text", "spell_records");
    case "durationText":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_text", "spell_records");
    case "targetText":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "target_text", "spell_records");
    case "disableText":
      return buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_text", "actor_records");
    case "variantBaseName":
      return buildRecordScalarSql(context, "variant_base_name");
    case "variantLabel":
      return buildRecordScalarSql(context, "variant_label");
    case "level":
      return buildRecordScalarSql(context, "level");
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
    case "areaValue":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_value", "spell_records");
    case "hasDescription":
      return buildRecordScalarSql(context, "has_description");
    case "publicationRemaster":
      return buildRecordScalarSql(context, "publication_remaster");
    case "sustained":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "sustained", "spell_records");
    case "basicSave":
      return buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "basic_save", "spell_records");
    case "isComplex":
      return buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "is_complex", "actor_records");
  }
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

function buildActorMetricPredicateClause(
  metric: string,
  op: "==" | "!=" | ">" | ">=" | "<" | "<=",
  valueType: "number" | "text" | "boolean",
  value: string | number | boolean,
  context: MetadataSqlSourceContext,
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
  context: MetadataSqlSourceContext,
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
  context: MetadataSqlSourceContext,
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
  context: MetadataSqlSourceContext,
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

export function getMetadataFilterValueSource(field: MetadataFieldName): MetadataFilterValueSource | null {
  switch (field) {
    case "traits":
      return { joins: ["JOIN record_traits rt ON rt.record_key = r.record_key"], valueExpression: "rt.trait" };
    case "families":
      return { joins: [`JOIN json_each(${buildFamiliesArraySql("r")}) AS family`], valueExpression: "LOWER(family.value)" };
    case "derivedTags":
      return { joins: ["JOIN record_derived_tags rdt ON rdt.record_key = r.record_key"], valueExpression: "rdt.tag" };
    case "traditions":
      return { joins: ["JOIN json_each(COALESCE(s.traditions_json, '[]')) AS tradition"], valueExpression: "tradition.value" };
    case "spellKinds":
      return { joins: ["JOIN json_each(COALESCE(s.spell_kinds_json, '[]')) AS spell_kind"], valueExpression: "spell_kind.value" };
    case "damageTypes":
      return { joins: ["JOIN json_each(COALESCE(s.damage_types_json, i.damage_types_json, '[]')) AS damage_type"], valueExpression: "damage_type.value" };
    case "languages":
      return { joins: ["JOIN json_each(COALESCE(a.languages_json, '[]')) AS language"], valueExpression: "language.value" };
    case "speedTypes":
      return { joins: ["JOIN json_each(COALESCE(a.speed_types_json, '[]')) AS speed_type"], valueExpression: "speed_type.value" };
    case "senses":
      return { joins: ["JOIN json_each(COALESCE(a.senses_json, '[]')) AS sense"], valueExpression: "sense.value" };
    case "immunities":
      return { joins: ["JOIN json_each(COALESCE(a.immunities_json, '[]')) AS immunity"], valueExpression: "immunity.value" };
    case "resistances":
      return { joins: ["JOIN json_each(COALESCE(a.resistances_json, '[]')) AS resistance"], valueExpression: "resistance.value" };
    case "weaknesses":
      return { joins: ["JOIN json_each(COALESCE(a.weaknesses_json, '[]')) AS weakness"], valueExpression: "weakness.value" };
    case "disableSkills":
      return { joins: ["JOIN json_each(COALESCE(a.disable_skills_json, '[]')) AS disable_skill"], valueExpression: "disable_skill.value" };
    case "variantAxes":
      return { joins: ["JOIN json_each(COALESCE(r.variant_axes_json, '[]')) AS variant_axis"], valueExpression: "variant_axis.value" };
    case "sourceCategory":
      return { valueExpression: "r.source_category", nonEmptyClause: "AND r.source_category IS NOT NULL AND r.source_category <> ''" };
    case "size":
      return { valueExpression: "a.size", nonEmptyClause: "AND a.size IS NOT NULL AND a.size <> ''" };
    case "usage":
      return { valueExpression: "i.usage_text", nonEmptyClause: "AND i.usage_text IS NOT NULL AND i.usage_text <> ''" };
    case "weaponGroup":
      return { valueExpression: "i.weapon_group", nonEmptyClause: "AND i.weapon_group IS NOT NULL AND i.weapon_group <> ''" };
    case "armorGroup":
      return { valueExpression: "i.armor_group", nonEmptyClause: "AND i.armor_group IS NOT NULL AND i.armor_group <> ''" };
    case "itemCategory":
      return { valueExpression: "i.item_category", nonEmptyClause: "AND i.item_category IS NOT NULL AND i.item_category <> ''" };
    case "baseItem":
      return { valueExpression: "i.base_item", nonEmptyClause: "AND i.base_item IS NOT NULL AND i.base_item <> ''" };
    case "saveType":
      return { valueExpression: "s.save_type", nonEmptyClause: "AND s.save_type IS NOT NULL AND s.save_type <> ''" };
    case "areaType":
      return { valueExpression: "s.area_type", nonEmptyClause: "AND s.area_type IS NOT NULL AND s.area_type <> ''" };
    case "durationUnit":
      return { valueExpression: "s.duration_unit", nonEmptyClause: "AND s.duration_unit IS NOT NULL AND s.duration_unit <> ''" };
    case "rarity":
      return { valueExpression: "r.rarity", nonEmptyClause: "AND r.rarity IS NOT NULL AND r.rarity <> ''" };
    case "variantFamilyKey":
      return { valueExpression: "r.variant_family_key", nonEmptyClause: "AND r.variant_family_key IS NOT NULL AND r.variant_family_key <> ''" };
    case "publicationTitle":
      return { valueExpression: "r.publication_title", nonEmptyClause: "AND r.publication_title IS NOT NULL AND r.publication_title <> ''" };
    case "variantBaseName":
      return { valueExpression: "r.variant_base_name", nonEmptyClause: "AND r.variant_base_name IS NOT NULL AND r.variant_base_name <> ''" };
    case "variantLabel":
      return { valueExpression: "r.variant_label", nonEmptyClause: "AND r.variant_label IS NOT NULL AND r.variant_label <> ''" };
    case "actionCost":
      return {
        valueExpression: "CAST(COALESCE(s.action_cost, i.action_cost) AS TEXT)",
        nonEmptyClause: "AND COALESCE(s.action_cost, i.action_cost) IS NOT NULL",
      };
    case "hands":
      return { valueExpression: "CAST(i.hands AS TEXT)", nonEmptyClause: "AND i.hands IS NOT NULL" };
    case "rangeValue":
      return {
        valueExpression:
          "CASE WHEN s.range_value = CAST(s.range_value AS INTEGER) THEN CAST(CAST(s.range_value AS INTEGER) AS TEXT) ELSE CAST(s.range_value AS TEXT) END",
        nonEmptyClause: "AND s.range_value IS NOT NULL",
      };
    case "areaValue":
      return {
        valueExpression:
          "CASE WHEN s.area_value = CAST(s.area_value AS INTEGER) THEN CAST(CAST(s.area_value AS INTEGER) AS TEXT) ELSE CAST(s.area_value AS TEXT) END",
        nonEmptyClause: "AND s.area_value IS NOT NULL",
      };
    case "sustained":
      return { valueExpression: "CASE s.sustained WHEN 1 THEN 'true' ELSE 'false' END", nonEmptyClause: "AND s.sustained IS NOT NULL" };
    case "basicSave":
      return { valueExpression: "CASE s.basic_save WHEN 1 THEN 'true' ELSE 'false' END", nonEmptyClause: "AND s.basic_save IS NOT NULL" };
    case "isComplex":
      return { valueExpression: "CASE a.is_complex WHEN 1 THEN 'true' ELSE 'false' END", nonEmptyClause: "AND a.is_complex IS NOT NULL" };
    case "rangeText":
    case "durationText":
    case "targetText":
    case "disableText":
    case "level":
    case "priceCp":
    case "bulkValue":
    case "hasDescription":
    case "publicationRemaster":
      return null;
    default: {
      const exhaustive: never = field;
      return exhaustive;
    }
  }
}

export function buildMetadataAtomicPredicateClause(
  predicate: MetadataAtomicPredicate,
  context: MetadataSqlSourceContext,
): { clause: string; params: SqlValue[] } {
  const normalized = normalizeMetadataAtomicPredicate(predicate) as MetadataAtomicPredicate & Record<string, unknown>;

  if (isMetadataSetField(normalized.field)) {
    if (normalized.op === "isNull") {
      const expression = buildMetadataJsonArraySql(context, normalized.field);
      return { clause: `COALESCE(json_array_length(${expression}), 0) = 0`, params: [] };
    }

    if (normalized.op === "isNotNull") {
      const expression = buildMetadataJsonArraySql(context, normalized.field);
      return { clause: `COALESCE(json_array_length(${expression}), 0) > 0`, params: [] };
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
      return { clause: `(${expression} >= ? AND ${expression} <= ?)`, params: [normalized.min as number, normalized.max as number] };
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
    return { clause: `${expression} ${operator} ?`, params: [normalized.value as SqlValue] };
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

export function buildMetricPredicateClause(
  metric: string,
  op: MetricOperator,
  value: string | number | boolean,
  context: MetadataSqlSourceContext,
): { clause: string; params: SqlValue[] } {
  const normalizedMetric = normalizeSearchMetricKey(metric);
  const metricInfo = normalizeSearchMetricValueType(normalizedMetric);
  if (!metricInfo) {
    throw new Error(`Unknown metric "${metric}".`);
  }

  const rawOp = rawMetricOperator(op);
  return metricInfo.owner === "actor"
    ? buildActorMetricPredicateClause(normalizedMetric, rawOp, metricInfo.valueType, value, context)
    : buildItemMetricPredicateClause(normalizedMetric, rawOp, metricInfo.valueType, value, context);
}

export function buildMetricCompareClause(
  leftMetric: string,
  op: NumericMetricOperator,
  rightMetric: string,
  context: MetadataSqlSourceContext,
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
