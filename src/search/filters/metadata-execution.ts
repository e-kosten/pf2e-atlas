import type { NormalizedRecord } from "../../domain/record-types.js";
import type { MetadataFieldName, MetadataFieldNameByType, MetadataFieldType } from "../../domain/metadata-field-types.js";
import {
  getMetadataFieldCatalogEntry,
  isMetadataBooleanField,
  isMetadataEnumStringField,
  isMetadataNumberField,
  isMetadataSetField,
  isMetadataTextField,
} from "../../domain/metadata-field-catalog.js";

export type MetadataValueNormalization = "normalizedText" | "lowercaseTrim" | "derivedTag";

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

export interface MetadataExecutionSpec<Field extends MetadataFieldName = MetadataFieldName> {
  field: Field;
  recordProperty: keyof NormalizedRecord;
  valueNormalization?: MetadataValueNormalization;
  buildSqlExpression?: (context: MetadataSqlSourceContext) => string;
  buildFilterValueSource?: () => MetadataFilterValueSource;
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

export function buildFamiliesArraySql(recordAlias: string): string {
  return `COALESCE(${recordAlias}.families_json, '[]')`;
}

export const METADATA_FIELD_EXECUTION_SPECS = [
  {
    field: "traits",
    recordProperty: "traits",
    buildFilterValueSource: () => ({
      joins: ["JOIN record_traits rt ON rt.record_key = r.record_key"],
      valueExpression: "rt.trait",
    }),
  },
  {
    field: "families",
    recordProperty: "families",
    buildSqlExpression: (context) =>
      context.recordsAlias
        ? buildFamiliesArraySql(context.recordsAlias)
        : `COALESCE((SELECT meta.families_json FROM records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`,
    buildFilterValueSource: () => ({
      joins: [`JOIN json_each(${buildFamiliesArraySql("r")}) AS family`],
      valueExpression: "LOWER(family.value)",
    }),
  },
  {
    field: "derivedTags",
    recordProperty: "derivedTags",
    valueNormalization: "derivedTag",
    buildFilterValueSource: () => ({
      joins: ["JOIN record_derived_tags rdt ON rdt.record_key = r.record_key"],
      valueExpression: "rdt.tag",
    }),
  },
  {
    field: "traditions",
    recordProperty: "traditions",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "traditions_json", "spell_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(s.traditions_json, '[]')) AS tradition"],
      valueExpression: "tradition.value",
    }),
  },
  {
    field: "spellKinds",
    recordProperty: "spellKinds",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "spell_kinds_json", "spell_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(s.spell_kinds_json, '[]')) AS spell_kind"],
      valueExpression: "spell_kind.value",
    }),
  },
  {
    field: "damageTypes",
    recordProperty: "damageTypes",
    buildSqlExpression: (context) => {
      if (context.spellAlias || context.itemAlias) {
        return `COALESCE(${context.spellAlias ?? "NULL"}.damage_types_json, ${context.itemAlias ?? "NULL"}.damage_types_json, '[]')`;
      }
      return `COALESCE((SELECT meta.damage_types_json FROM spell_records meta WHERE meta.record_key = ${context.recordKeyExpr}), (SELECT meta.damage_types_json FROM item_records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`;
    },
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(s.damage_types_json, i.damage_types_json, '[]')) AS damage_type"],
      valueExpression: "damage_type.value",
    }),
  },
  {
    field: "languages",
    recordProperty: "languages",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "languages_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.languages_json, '[]')) AS language"],
      valueExpression: "language.value",
    }),
  },
  {
    field: "speedTypes",
    recordProperty: "speedTypes",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "speed_types_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.speed_types_json, '[]')) AS speed_type"],
      valueExpression: "speed_type.value",
    }),
  },
  {
    field: "senses",
    recordProperty: "senses",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "senses_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.senses_json, '[]')) AS sense"],
      valueExpression: "sense.value",
    }),
  },
  {
    field: "immunities",
    recordProperty: "immunities",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "immunities_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.immunities_json, '[]')) AS immunity"],
      valueExpression: "immunity.value",
    }),
  },
  {
    field: "resistances",
    recordProperty: "resistances",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "resistances_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.resistances_json, '[]')) AS resistance"],
      valueExpression: "resistance.value",
    }),
  },
  {
    field: "weaknesses",
    recordProperty: "weaknesses",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "weaknesses_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.weaknesses_json, '[]')) AS weakness"],
      valueExpression: "weakness.value",
    }),
  },
  {
    field: "disableSkills",
    recordProperty: "disableSkills",
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_skills_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.disable_skills_json, '[]')) AS disable_skill"],
      valueExpression: "disable_skill.value",
    }),
  },
  {
    field: "variantAxes",
    recordProperty: "variantAxes",
    buildSqlExpression: (context) => buildRecordJsonSql(context, "variant_axes_json"),
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(r.variant_axes_json, '[]')) AS variant_axis"],
      valueExpression: "variant_axis.value",
    }),
  },
  {
    field: "sourceCategory",
    recordProperty: "sourceCategory",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "source_category"),
    buildFilterValueSource: () => ({
      valueExpression: "r.source_category",
      nonEmptyClause: "AND r.source_category IS NOT NULL AND r.source_category <> ''",
    }),
  },
  {
    field: "size",
    recordProperty: "size",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "size", "actor_records"),
    buildFilterValueSource: () => ({
      valueExpression: "a.size",
      nonEmptyClause: "AND a.size IS NOT NULL AND a.size <> ''",
    }),
  },
  {
    field: "usage",
    recordProperty: "usage",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "usage_text", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.usage_text",
      nonEmptyClause: "AND i.usage_text IS NOT NULL AND i.usage_text <> ''",
    }),
  },
  {
    field: "weaponGroup",
    recordProperty: "weaponGroup",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "weapon_group", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.weapon_group",
      nonEmptyClause: "AND i.weapon_group IS NOT NULL AND i.weapon_group <> ''",
    }),
  },
  {
    field: "armorGroup",
    recordProperty: "armorGroup",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "armor_group", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.armor_group",
      nonEmptyClause: "AND i.armor_group IS NOT NULL AND i.armor_group <> ''",
    }),
  },
  {
    field: "itemCategory",
    recordProperty: "itemCategory",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "item_category", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.item_category",
      nonEmptyClause: "AND i.item_category IS NOT NULL AND i.item_category <> ''",
    }),
  },
  {
    field: "baseItem",
    recordProperty: "baseItem",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "base_item", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.base_item",
      nonEmptyClause: "AND i.base_item IS NOT NULL AND i.base_item <> ''",
    }),
  },
  {
    field: "saveType",
    recordProperty: "saveType",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "save_type", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.save_type",
      nonEmptyClause: "AND s.save_type IS NOT NULL AND s.save_type <> ''",
    }),
  },
  {
    field: "areaType",
    recordProperty: "areaType",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_type", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.area_type",
      nonEmptyClause: "AND s.area_type IS NOT NULL AND s.area_type <> ''",
    }),
  },
  {
    field: "durationUnit",
    recordProperty: "durationUnit",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_unit", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.duration_unit",
      nonEmptyClause: "AND s.duration_unit IS NOT NULL AND s.duration_unit <> ''",
    }),
  },
  {
    field: "rarity",
    recordProperty: "rarity",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "rarity"),
    buildFilterValueSource: () => ({
      valueExpression: "r.rarity",
      nonEmptyClause: "AND r.rarity IS NOT NULL AND r.rarity <> ''",
    }),
  },
  {
    field: "variantFamilyKey",
    recordProperty: "variantFamilyKey",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_family_key"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_family_key",
      nonEmptyClause: "AND r.variant_family_key IS NOT NULL AND r.variant_family_key <> ''",
    }),
  },
  {
    field: "publicationTitle",
    recordProperty: "publicationTitle",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "publication_title"),
    buildFilterValueSource: () => ({
      valueExpression: "r.publication_title",
      nonEmptyClause: "AND r.publication_title IS NOT NULL AND r.publication_title <> ''",
    }),
  },
  {
    field: "rangeText",
    recordProperty: "rangeText",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_text", "spell_records"),
  },
  {
    field: "durationText",
    recordProperty: "durationText",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_text", "spell_records"),
  },
  {
    field: "targetText",
    recordProperty: "targetText",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "target_text", "spell_records"),
  },
  {
    field: "disableText",
    recordProperty: "disableText",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_text", "actor_records"),
  },
  {
    field: "variantBaseName",
    recordProperty: "variantBaseName",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_base_name"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_base_name",
      nonEmptyClause: "AND r.variant_base_name IS NOT NULL AND r.variant_base_name <> ''",
    }),
  },
  {
    field: "variantLabel",
    recordProperty: "variantLabel",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_label"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_label",
      nonEmptyClause: "AND r.variant_label IS NOT NULL AND r.variant_label <> ''",
    }),
  },
  {
    field: "level",
    recordProperty: "level",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "level"),
  },
  {
    field: "priceCp",
    recordProperty: "priceCp",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "price_cp", "item_records"),
  },
  {
    field: "bulkValue",
    recordProperty: "bulkValue",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "bulk_value", "item_records"),
  },
  {
    field: "actionCost",
    recordProperty: "actionCost",
    buildSqlExpression: (context) => {
      if (context.spellAlias || context.itemAlias) {
        return `COALESCE(${context.spellAlias ?? "NULL"}.action_cost, ${context.itemAlias ?? "NULL"}.action_cost)`;
      }
      return `COALESCE((SELECT meta.action_cost FROM spell_records meta WHERE meta.record_key = ${context.recordKeyExpr}), (SELECT meta.action_cost FROM item_records meta WHERE meta.record_key = ${context.recordKeyExpr}))`;
    },
    buildFilterValueSource: () => ({
      valueExpression: "CAST(COALESCE(s.action_cost, i.action_cost) AS TEXT)",
      nonEmptyClause: "AND COALESCE(s.action_cost, i.action_cost) IS NOT NULL",
    }),
  },
  {
    field: "hands",
    recordProperty: "hands",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "hands", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CAST(i.hands AS TEXT)",
      nonEmptyClause: "AND i.hands IS NOT NULL",
    }),
  },
  {
    field: "rangeValue",
    recordProperty: "rangeValue",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_value", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression:
        "CASE WHEN s.range_value = CAST(s.range_value AS INTEGER) THEN CAST(CAST(s.range_value AS INTEGER) AS TEXT) ELSE CAST(s.range_value AS TEXT) END",
      nonEmptyClause: "AND s.range_value IS NOT NULL",
    }),
  },
  {
    field: "areaValue",
    recordProperty: "areaValue",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_value", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression:
        "CASE WHEN s.area_value = CAST(s.area_value AS INTEGER) THEN CAST(CAST(s.area_value AS INTEGER) AS TEXT) ELSE CAST(s.area_value AS TEXT) END",
      nonEmptyClause: "AND s.area_value IS NOT NULL",
    }),
  },
  {
    field: "hasDescription",
    recordProperty: "hasDescription",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "has_description"),
  },
  {
    field: "publicationRemaster",
    recordProperty: "publicationRemaster",
    buildSqlExpression: (context) => buildRecordScalarSql(context, "publication_remaster"),
  },
  {
    field: "sustained",
    recordProperty: "sustained",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "sustained", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE s.sustained WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND s.sustained IS NOT NULL",
    }),
  },
  {
    field: "basicSave",
    recordProperty: "basicSave",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "basic_save", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE s.basic_save WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND s.basic_save IS NOT NULL",
    }),
  },
  {
    field: "isComplex",
    recordProperty: "isComplex",
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "is_complex", "actor_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE a.is_complex WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND a.is_complex IS NOT NULL",
    }),
  },
] as const satisfies readonly MetadataExecutionSpec[];

export type MetadataExecutionSpecEntry = MetadataExecutionSpec;
export type MetadataExecutionSpecByType<FieldType extends MetadataFieldType> = MetadataExecutionSpec<
  MetadataFieldNameByType<FieldType>
>;

type MetadataRecordValueByFieldType = {
  set: string[];
  enumString: string | null;
  text: string | null;
  number: number | null;
  boolean: boolean;
};

const METADATA_FIELD_EXECUTION_SPEC_BY_NAME = new Map<MetadataFieldName, MetadataExecutionSpecEntry>(
  METADATA_FIELD_EXECUTION_SPECS.map((entry) => [entry.field, entry]),
);

export function getMetadataExecutionSpec(field: MetadataFieldName): MetadataExecutionSpecEntry {
  const spec = METADATA_FIELD_EXECUTION_SPEC_BY_NAME.get(field);
  if (!spec) {
    throw new Error(`No metadata execution spec configured for "${field}".`);
  }
  return spec;
}

function getMetadataExecutionSpecByType<FieldType extends MetadataFieldType>(
  field: MetadataFieldNameByType<FieldType>,
  fieldType: FieldType,
): MetadataExecutionSpecByType<FieldType> {
  const spec = getMetadataExecutionSpec(field);
  const catalogEntry = getMetadataFieldCatalogEntry(field);
  if (catalogEntry.fieldType !== fieldType) {
    throw new Error(`Metadata field "${field}" is not a ${fieldType} field.`);
  }
  return spec as MetadataExecutionSpecByType<FieldType>;
}

function getMetadataRecordValue<FieldType extends MetadataFieldType>(
  record: NormalizedRecord,
  spec: MetadataExecutionSpecByType<FieldType>,
): MetadataRecordValueByFieldType[FieldType] {
  return record[spec.recordProperty] as MetadataRecordValueByFieldType[FieldType];
}

export function getMetadataSetExecutionSpec(field: MetadataFieldNameByType<"set">): MetadataExecutionSpecByType<"set"> {
  return getMetadataExecutionSpecByType(field, "set");
}

export function getMetadataEnumStringExecutionSpec(
  field: MetadataFieldNameByType<"enumString">,
): MetadataExecutionSpecByType<"enumString"> {
  return getMetadataExecutionSpecByType(field, "enumString");
}

export function getMetadataTextExecutionSpec(field: MetadataFieldNameByType<"text">): MetadataExecutionSpecByType<"text"> {
  return getMetadataExecutionSpecByType(field, "text");
}

export function getMetadataNumberExecutionSpec(
  field: MetadataFieldNameByType<"number">,
): MetadataExecutionSpecByType<"number"> {
  return getMetadataExecutionSpecByType(field, "number");
}

export function getMetadataBooleanExecutionSpec(
  field: MetadataFieldNameByType<"boolean">,
): MetadataExecutionSpecByType<"boolean"> {
  return getMetadataExecutionSpecByType(field, "boolean");
}

export function getMetadataSetRecordValues(record: NormalizedRecord, field: MetadataFieldNameByType<"set">): string[] {
  return getMetadataRecordValue(record, getMetadataSetExecutionSpec(field));
}

export function getMetadataStringRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"enumString"> | MetadataFieldNameByType<"text">,
): string | null {
  return isMetadataEnumStringField(field)
    ? getMetadataRecordValue(record, getMetadataEnumStringExecutionSpec(field))
    : getMetadataRecordValue(record, getMetadataTextExecutionSpec(field));
}

export function getMetadataNumberRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"number">,
): number | null {
  return getMetadataRecordValue(record, getMetadataNumberExecutionSpec(field));
}

export function getMetadataBooleanRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"boolean">,
): boolean {
  return getMetadataRecordValue(record, getMetadataBooleanExecutionSpec(field));
}

export {
  isMetadataBooleanField,
  isMetadataEnumStringField,
  isMetadataNumberField,
  isMetadataSetField,
  isMetadataTextField,
};
