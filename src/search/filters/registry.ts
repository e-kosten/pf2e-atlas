import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import type { MetadataFieldName, MetadataFieldNameByType, MetadataFieldType } from "../../domain/metadata-field-types.js";

export type MetadataValueNormalization = "normalizedText" | "lowercaseTrim" | "derivedTag";
export type MetadataPresentation = "summary" | "detail" | "none";
export type MetadataRowValueSourceKind = "string" | "number" | "booleanNumber" | "jsonArray";

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

export interface MetadataRowValueSource {
  key: string;
  kind: MetadataRowValueSourceKind;
}

export interface MetadataFieldSpec<
  Field extends string = string,
  RecordProperty extends keyof NormalizedRecord = keyof NormalizedRecord,
  FieldType extends MetadataFieldType = MetadataFieldType,
> {
  field: Field;
  recordProperty: RecordProperty;
  fieldType: FieldType;
  categories: readonly SearchCategory[];
  subcategories?: readonly SearchSubcategory[];
  discoverable?: boolean;
  notes?: string;
  valueOrdering?: FilterValueOrdering;
  valueNormalization?: MetadataValueNormalization;
  selectClause: string;
  rowValueSource: MetadataRowValueSource;
  buildSqlExpression?: (context: MetadataSqlSourceContext) => string;
  buildFilterValueSource?: () => MetadataFilterValueSource;
  presentation?: MetadataPresentation;
  presentWhen?: (record: NormalizedRecord) => boolean;
}

const ALL_CATEGORIES = [
  "equipment",
  "feat",
  "creature",
  "hazard",
  "affliction",
  "rule",
  "spell",
  "characterCreation",
  "lore",
] as const satisfies readonly SearchCategory[];
const CREATURE_ONLY = ["creature"] as const satisfies readonly SearchCategory[];
const CREATURE_AND_HAZARD = ["creature", "hazard"] as const satisfies readonly SearchCategory[];
const EQUIPMENT_ONLY = ["equipment"] as const satisfies readonly SearchCategory[];
const SPELL_ONLY = ["spell"] as const satisfies readonly SearchCategory[];
const EQUIPMENT_AND_SPELL = ["equipment", "spell"] as const satisfies readonly SearchCategory[];
const DERIVED_TAG_CATEGORIES = [
  "equipment",
  "creature",
  "hazard",
  "affliction",
  "spell",
] as const satisfies readonly SearchCategory[];

function defineMetadataField<
  const Field extends string,
  const RecordProperty extends keyof NormalizedRecord,
  const FieldType extends MetadataFieldType,
>(spec: MetadataFieldSpec<Field, RecordProperty, FieldType>): MetadataFieldSpec<Field, RecordProperty, FieldType> {
  return spec;
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

const HAS_VARIANT_FAMILY = (record: NormalizedRecord): boolean => Boolean(record.variantFamilyKey);

export const METADATA_FIELD_REGISTRY = [
  defineMetadataField({
    field: "traits",
    recordProperty: "traits",
    fieldType: "set",
    categories: ALL_CATEGORIES,
    discoverable: true,
    notes: "Cross-category trait facet. Use category or subcategory boundaries before trait predicates.",
    selectClause: "r.traits_json AS traitsJson",
    rowValueSource: { key: "traitsJson", kind: "jsonArray" },
    buildFilterValueSource: () => ({
      joins: ["JOIN record_traits rt ON rt.record_key = r.record_key"],
      valueExpression: "rt.trait",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "families",
    recordProperty: "families",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    notes:
      "Creature family facet derived from glossary-backed monster families plus allowlisted Pathfinder NPC Core cohort folders.",
    selectClause: "r.families_json AS familiesJson",
    rowValueSource: { key: "familiesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      context.recordsAlias
        ? buildFamiliesArraySql(context.recordsAlias)
        : `COALESCE((SELECT meta.families_json FROM records meta WHERE meta.record_key = ${context.recordKeyExpr}), '[]')`,
    buildFilterValueSource: () => ({
      joins: [`JOIN json_each(${buildFamiliesArraySql("r")}) AS family`],
      valueExpression: "LOWER(family.value)",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "derivedTags",
    recordProperty: "derivedTags",
    fieldType: "set",
    categories: DERIVED_TAG_CATEGORIES,
    discoverable: true,
    notes: "Curated heuristic tags. Supported for categories with explicit derived-tag ontology coverage.",
    valueNormalization: "derivedTag",
    selectClause: "r.derived_tags_json AS derivedTagsJson",
    rowValueSource: { key: "derivedTagsJson", kind: "jsonArray" },
    buildFilterValueSource: () => ({
      joins: ["JOIN record_derived_tags rdt ON rdt.record_key = r.record_key"],
      valueExpression: "rdt.tag",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "traditions",
    recordProperty: "traditions",
    fieldType: "set",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.traditions_json AS traditionsJson",
    rowValueSource: { key: "traditionsJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "traditions_json", "spell_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(s.traditions_json, '[]')) AS tradition"],
      valueExpression: "tradition.value",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "spellKinds",
    recordProperty: "spellKinds",
    fieldType: "set",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.spell_kinds_json AS spellKindsJson",
    rowValueSource: { key: "spellKindsJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "spell_kinds_json", "spell_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(s.spell_kinds_json, '[]')) AS spell_kind"],
      valueExpression: "spell_kind.value",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "damageTypes",
    recordProperty: "damageTypes",
    fieldType: "set",
    categories: EQUIPMENT_AND_SPELL,
    discoverable: true,
    notes: "Available for spells and item records with typed damage payloads.",
    selectClause: "COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson",
    rowValueSource: { key: "damageTypesJson", kind: "jsonArray" },
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
    presentation: "detail",
  }),
  defineMetadataField({
    field: "languages",
    recordProperty: "languages",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    selectClause: "a.languages_json AS languagesJson",
    rowValueSource: { key: "languagesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "languages_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.languages_json, '[]')) AS language"],
      valueExpression: "language.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "speedTypes",
    recordProperty: "speedTypes",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    selectClause: "a.speed_types_json AS speedTypesJson",
    rowValueSource: { key: "speedTypesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "speed_types_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.speed_types_json, '[]')) AS speed_type"],
      valueExpression: "speed_type.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "senses",
    recordProperty: "senses",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    notes: "Creature sense types such as darkvision, low light vision, or scent.",
    selectClause: "a.senses_json AS sensesJson",
    rowValueSource: { key: "sensesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "senses_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.senses_json, '[]')) AS sense"],
      valueExpression: "sense.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "immunities",
    recordProperty: "immunities",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
    selectClause: "a.immunities_json AS immunitiesJson",
    rowValueSource: { key: "immunitiesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "immunities_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.immunities_json, '[]')) AS immunity"],
      valueExpression: "immunity.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "resistances",
    recordProperty: "resistances",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
    selectClause: "a.resistances_json AS resistancesJson",
    rowValueSource: { key: "resistancesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "resistances_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.resistances_json, '[]')) AS resistance"],
      valueExpression: "resistance.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "weaknesses",
    recordProperty: "weaknesses",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
    selectClause: "a.weaknesses_json AS weaknessesJson",
    rowValueSource: { key: "weaknessesJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "weaknesses_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.weaknesses_json, '[]')) AS weakness"],
      valueExpression: "weakness.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "disableSkills",
    recordProperty: "disableSkills",
    fieldType: "set",
    categories: ["hazard"],
    discoverable: true,
    notes: "Structured hazard disable skills parsed from disable text when PF2E markup provides a clear anchor.",
    selectClause: "a.disable_skills_json AS disableSkillsJson",
    rowValueSource: { key: "disableSkillsJson", kind: "jsonArray" },
    buildSqlExpression: (context) =>
      `COALESCE(${buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_skills_json", "actor_records")}, '[]')`,
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(a.disable_skills_json, '[]')) AS disable_skill"],
      valueExpression: "disable_skill.value",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "variantAxes",
    recordProperty: "variantAxes",
    fieldType: "set",
    categories: ["equipment", "spell"],
    discoverable: true,
    notes: "Normalized variant-family axes such as rank, grade, or specialization.",
    selectClause: "r.variant_axes_json AS variantAxesJson",
    rowValueSource: { key: "variantAxesJson", kind: "jsonArray" },
    buildSqlExpression: (context) => buildRecordJsonSql(context, "variant_axes_json"),
    buildFilterValueSource: () => ({
      joins: ["JOIN json_each(COALESCE(r.variant_axes_json, '[]')) AS variant_axis"],
      valueExpression: "variant_axis.value",
    }),
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  }),
  defineMetadataField({
    field: "sourceCategory",
    recordProperty: "sourceCategory",
    fieldType: "enumString",
    categories: ALL_CATEGORIES,
    discoverable: true,
    notes: "Normalized source bucket such as core, rules, or adventure.",
    selectClause: "r.source_category AS sourceCategory",
    rowValueSource: { key: "sourceCategory", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "source_category"),
    buildFilterValueSource: () => ({
      valueExpression: "r.source_category",
      nonEmptyClause: "AND r.source_category IS NOT NULL AND r.source_category <> ''",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "size",
    recordProperty: "size",
    fieldType: "enumString",
    categories: CREATURE_ONLY,
    discoverable: true,
    selectClause: "a.size AS size",
    rowValueSource: { key: "size", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "size", "actor_records"),
    buildFilterValueSource: () => ({
      valueExpression: "a.size",
      nonEmptyClause: "AND a.size IS NOT NULL AND a.size <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "usage",
    recordProperty: "usage",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
    selectClause: "i.usage_text AS usage",
    rowValueSource: { key: "usage", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "usage_text", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.usage_text",
      nonEmptyClause: "AND i.usage_text IS NOT NULL AND i.usage_text <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "weaponGroup",
    recordProperty: "weaponGroup",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    subcategories: ["weapon", "ammo"],
    discoverable: true,
    selectClause: "i.weapon_group AS weaponGroup",
    rowValueSource: { key: "weaponGroup", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "weapon_group", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.weapon_group",
      nonEmptyClause: "AND i.weapon_group IS NOT NULL AND i.weapon_group <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "armorGroup",
    recordProperty: "armorGroup",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    subcategories: ["armor", "shield"],
    discoverable: true,
    selectClause: "i.armor_group AS armorGroup",
    rowValueSource: { key: "armorGroup", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "armor_group", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.armor_group",
      nonEmptyClause: "AND i.armor_group IS NOT NULL AND i.armor_group <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "itemCategory",
    recordProperty: "itemCategory",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
    selectClause: "i.item_category AS itemCategory",
    rowValueSource: { key: "itemCategory", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "item_category", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.item_category",
      nonEmptyClause: "AND i.item_category IS NOT NULL AND i.item_category <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "baseItem",
    recordProperty: "baseItem",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
    selectClause: "i.base_item AS baseItem",
    rowValueSource: { key: "baseItem", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "base_item", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "i.base_item",
      nonEmptyClause: "AND i.base_item IS NOT NULL AND i.base_item <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "saveType",
    recordProperty: "saveType",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.save_type AS saveType",
    rowValueSource: { key: "saveType", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "save_type", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.save_type",
      nonEmptyClause: "AND s.save_type IS NOT NULL AND s.save_type <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "areaType",
    recordProperty: "areaType",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.area_type AS areaType",
    rowValueSource: { key: "areaType", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_type", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.area_type",
      nonEmptyClause: "AND s.area_type IS NOT NULL AND s.area_type <> ''",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "durationUnit",
    recordProperty: "durationUnit",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.duration_unit AS durationUnit",
    rowValueSource: { key: "durationUnit", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_unit", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "s.duration_unit",
      nonEmptyClause: "AND s.duration_unit IS NOT NULL AND s.duration_unit <> ''",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "rarity",
    recordProperty: "rarity",
    fieldType: "enumString",
    categories: ALL_CATEGORIES,
    discoverable: true,
    valueOrdering: {
      kind: "canonical",
      order: ["common", "uncommon", "rare", "unique"],
    },
    selectClause: "r.rarity AS rarity",
    rowValueSource: { key: "rarity", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "rarity"),
    buildFilterValueSource: () => ({
      valueExpression: "r.rarity",
      nonEmptyClause: "AND r.rarity IS NOT NULL AND r.rarity <> ''",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "variantFamilyKey",
    recordProperty: "variantFamilyKey",
    fieldType: "enumString",
    categories: ["equipment", "spell"],
    discoverable: true,
    selectClause: "r.variant_family_key AS variantFamilyKey",
    rowValueSource: { key: "variantFamilyKey", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_family_key"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_family_key",
      nonEmptyClause: "AND r.variant_family_key IS NOT NULL AND r.variant_family_key <> ''",
    }),
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  }),
  defineMetadataField({
    field: "publicationTitle",
    recordProperty: "publicationTitle",
    fieldType: "text",
    categories: ALL_CATEGORIES,
    discoverable: true,
    selectClause: "r.publication_title AS publicationTitle",
    rowValueSource: { key: "publicationTitle", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "publication_title"),
    buildFilterValueSource: () => ({
      valueExpression: "r.publication_title",
      nonEmptyClause: "AND r.publication_title IS NOT NULL AND r.publication_title <> ''",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "rangeText",
    recordProperty: "rangeText",
    fieldType: "text",
    categories: SPELL_ONLY,
    selectClause: "s.range_text AS rangeText",
    rowValueSource: { key: "rangeText", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_text", "spell_records"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "durationText",
    recordProperty: "durationText",
    fieldType: "text",
    categories: SPELL_ONLY,
    selectClause: "s.duration_text AS durationText",
    rowValueSource: { key: "durationText", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "duration_text", "spell_records"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "targetText",
    recordProperty: "targetText",
    fieldType: "text",
    categories: SPELL_ONLY,
    selectClause: "s.target_text AS targetText",
    rowValueSource: { key: "targetText", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "target_text", "spell_records"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "disableText",
    recordProperty: "disableText",
    fieldType: "text",
    categories: ["hazard"],
    selectClause: "a.disable_text AS disableText",
    rowValueSource: { key: "disableText", kind: "string" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "disable_text", "actor_records"),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "variantBaseName",
    recordProperty: "variantBaseName",
    fieldType: "text",
    categories: ["equipment", "spell"],
    discoverable: true,
    selectClause: "r.variant_base_name AS variantBaseName",
    rowValueSource: { key: "variantBaseName", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_base_name"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_base_name",
      nonEmptyClause: "AND r.variant_base_name IS NOT NULL AND r.variant_base_name <> ''",
    }),
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  }),
  defineMetadataField({
    field: "variantLabel",
    recordProperty: "variantLabel",
    fieldType: "text",
    categories: ["equipment", "spell"],
    discoverable: true,
    selectClause: "r.variant_label AS variantLabel",
    rowValueSource: { key: "variantLabel", kind: "string" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "variant_label"),
    buildFilterValueSource: () => ({
      valueExpression: "r.variant_label",
      nonEmptyClause: "AND r.variant_label IS NOT NULL AND r.variant_label <> ''",
    }),
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  }),
  defineMetadataField({
    field: "level",
    recordProperty: "level",
    fieldType: "number",
    categories: ALL_CATEGORIES,
    selectClause: "r.level AS level",
    rowValueSource: { key: "level", kind: "number" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "level"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "priceCp",
    recordProperty: "priceCp",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
    selectClause: "i.price_cp AS priceCp",
    rowValueSource: { key: "priceCp", kind: "number" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "price_cp", "item_records"),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "bulkValue",
    recordProperty: "bulkValue",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
    selectClause: "i.bulk_value AS bulkValue",
    rowValueSource: { key: "bulkValue", kind: "number" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "bulk_value", "item_records"),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "actionCost",
    recordProperty: "actionCost",
    fieldType: "number",
    categories: ALL_CATEGORIES,
    discoverable: true,
    valueOrdering: { kind: "numericAsc" },
    selectClause: "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    rowValueSource: { key: "actionCost", kind: "number" },
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
    presentation: "detail",
  }),
  defineMetadataField({
    field: "hands",
    recordProperty: "hands",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
    selectClause: "i.hands AS hands",
    rowValueSource: { key: "hands", kind: "number" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.itemAlias, "hands", "item_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CAST(i.hands AS TEXT)",
      nonEmptyClause: "AND i.hands IS NOT NULL",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "rangeValue",
    recordProperty: "rangeValue",
    fieldType: "number",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.range_value AS rangeValue",
    rowValueSource: { key: "rangeValue", kind: "number" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "range_value", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression:
        "CASE WHEN s.range_value = CAST(s.range_value AS INTEGER) THEN CAST(CAST(s.range_value AS INTEGER) AS TEXT) ELSE CAST(s.range_value AS TEXT) END",
      nonEmptyClause: "AND s.range_value IS NOT NULL",
    }),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "areaValue",
    recordProperty: "areaValue",
    fieldType: "number",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.area_value AS areaValue",
    rowValueSource: { key: "areaValue", kind: "number" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "area_value", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression:
        "CASE WHEN s.area_value = CAST(s.area_value AS INTEGER) THEN CAST(CAST(s.area_value AS INTEGER) AS TEXT) ELSE CAST(s.area_value AS TEXT) END",
      nonEmptyClause: "AND s.area_value IS NOT NULL",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "isUnique",
    recordProperty: "isUnique",
    fieldType: "boolean",
    categories: ALL_CATEGORIES,
    selectClause: "r.is_unique AS isUnique",
    rowValueSource: { key: "isUnique", kind: "booleanNumber" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "is_unique"),
    presentation: "detail",
  }),
  defineMetadataField({
    field: "hasDescription",
    recordProperty: "hasDescription",
    fieldType: "boolean",
    categories: ALL_CATEGORIES,
    selectClause: "r.has_description AS hasDescription",
    rowValueSource: { key: "hasDescription", kind: "booleanNumber" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "has_description"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "publicationRemaster",
    recordProperty: "publicationRemaster",
    fieldType: "boolean",
    categories: ALL_CATEGORIES,
    selectClause: "r.publication_remaster AS publicationRemaster",
    rowValueSource: { key: "publicationRemaster", kind: "booleanNumber" },
    buildSqlExpression: (context) => buildRecordScalarSql(context, "publication_remaster"),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "sustained",
    recordProperty: "sustained",
    fieldType: "boolean",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.sustained AS sustained",
    rowValueSource: { key: "sustained", kind: "booleanNumber" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "sustained", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE s.sustained WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND s.sustained IS NOT NULL",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "basicSave",
    recordProperty: "basicSave",
    fieldType: "boolean",
    categories: SPELL_ONLY,
    discoverable: true,
    selectClause: "s.basic_save AS basicSave",
    rowValueSource: { key: "basicSave", kind: "booleanNumber" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.spellAlias, "basic_save", "spell_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE s.basic_save WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND s.basic_save IS NOT NULL",
    }),
    presentation: "summary",
  }),
  defineMetadataField({
    field: "isComplex",
    recordProperty: "isComplex",
    fieldType: "boolean",
    categories: ["hazard"],
    discoverable: true,
    selectClause: "a.is_complex AS isComplex",
    rowValueSource: { key: "isComplex", kind: "booleanNumber" },
    buildSqlExpression: (context) =>
      buildScalarLookupSql(context.recordKeyExpr, context.actorAlias, "is_complex", "actor_records"),
    buildFilterValueSource: () => ({
      valueExpression: "CASE a.is_complex WHEN 1 THEN 'true' ELSE 'false' END",
      nonEmptyClause: "AND a.is_complex IS NOT NULL",
    }),
    presentation: "detail",
  }),
] as const;

export type MetadataFieldSpecEntry = (typeof METADATA_FIELD_REGISTRY)[number];
export type MetadataFieldSpecByType<FieldType extends MetadataFieldType> = Extract<
  MetadataFieldSpecEntry,
  { fieldType: FieldType }
>;

type MetadataRecordValueByFieldType = {
  set: string[];
  enumString: string | null;
  text: string | null;
  number: number | null;
  boolean: boolean;
};

export const METADATA_FIELD_SPEC_BY_NAME = new Map<MetadataFieldName, MetadataFieldSpecEntry>(
  METADATA_FIELD_REGISTRY.map((entry) => [entry.field, entry]),
);
const METADATA_FIELD_NAMES_BY_TYPE = {
  set: new Set<MetadataFieldNameByType<"set">>(
    METADATA_FIELD_REGISTRY.filter((entry): entry is MetadataFieldSpecByType<"set"> => entry.fieldType === "set").map(
      (entry) => entry.field,
    ),
  ),
  enumString: new Set<MetadataFieldNameByType<"enumString">>(
    METADATA_FIELD_REGISTRY.filter(
      (entry): entry is MetadataFieldSpecByType<"enumString"> => entry.fieldType === "enumString",
    ).map((entry) => entry.field),
  ),
  text: new Set<MetadataFieldNameByType<"text">>(
    METADATA_FIELD_REGISTRY.filter((entry): entry is MetadataFieldSpecByType<"text"> => entry.fieldType === "text").map(
      (entry) => entry.field,
    ),
  ),
  number: new Set<MetadataFieldNameByType<"number">>(
    METADATA_FIELD_REGISTRY.filter(
      (entry): entry is MetadataFieldSpecByType<"number"> => entry.fieldType === "number",
    ).map((entry) => entry.field),
  ),
  boolean: new Set<MetadataFieldNameByType<"boolean">>(
    METADATA_FIELD_REGISTRY.filter(
      (entry): entry is MetadataFieldSpecByType<"boolean"> => entry.fieldType === "boolean",
    ).map((entry) => entry.field),
  ),
} as const;

export function getMetadataFieldSpec(field: MetadataFieldName): MetadataFieldSpecEntry {
  const spec = METADATA_FIELD_SPEC_BY_NAME.get(field);
  if (!spec) {
    throw new Error(`Unknown metadata field "${field}".`);
  }
  return spec;
}

export function isMetadataFieldName(field: string): field is MetadataFieldName {
  return METADATA_FIELD_SPEC_BY_NAME.has(field as MetadataFieldName);
}

function getMetadataFieldSpecByType<FieldType extends MetadataFieldType>(
  field: MetadataFieldNameByType<FieldType>,
  fieldType: FieldType,
): MetadataFieldSpecByType<FieldType> {
  const spec = getMetadataFieldSpec(field);
  if (spec.fieldType !== fieldType) {
    throw new Error(`Metadata field "${field}" is not a ${fieldType} field.`);
  }
  return spec as MetadataFieldSpecByType<FieldType>;
}

function getMetadataRecordValue<FieldType extends MetadataFieldType>(
  record: NormalizedRecord,
  spec: MetadataFieldSpecByType<FieldType>,
): MetadataRecordValueByFieldType[FieldType] {
  return record[spec.recordProperty] as MetadataRecordValueByFieldType[FieldType];
}

export function isMetadataSetField(field: string): field is MetadataFieldNameByType<"set"> {
  return METADATA_FIELD_NAMES_BY_TYPE.set.has(field as MetadataFieldNameByType<"set">);
}

export function isMetadataEnumStringField(field: string): field is MetadataFieldNameByType<"enumString"> {
  return METADATA_FIELD_NAMES_BY_TYPE.enumString.has(field as MetadataFieldNameByType<"enumString">);
}

export function isMetadataTextField(field: string): field is MetadataFieldNameByType<"text"> {
  return METADATA_FIELD_NAMES_BY_TYPE.text.has(field as MetadataFieldNameByType<"text">);
}

export function isMetadataNumberField(field: string): field is MetadataFieldNameByType<"number"> {
  return METADATA_FIELD_NAMES_BY_TYPE.number.has(field as MetadataFieldNameByType<"number">);
}

export function isMetadataBooleanField(field: string): field is MetadataFieldNameByType<"boolean"> {
  return METADATA_FIELD_NAMES_BY_TYPE.boolean.has(field as MetadataFieldNameByType<"boolean">);
}

export function getMetadataSetFieldSpec(field: MetadataFieldNameByType<"set">): MetadataFieldSpecByType<"set"> {
  return getMetadataFieldSpecByType(field, "set");
}

export function getMetadataEnumStringFieldSpec(
  field: MetadataFieldNameByType<"enumString">,
): MetadataFieldSpecByType<"enumString"> {
  return getMetadataFieldSpecByType(field, "enumString");
}

export function getMetadataTextFieldSpec(field: MetadataFieldNameByType<"text">): MetadataFieldSpecByType<"text"> {
  return getMetadataFieldSpecByType(field, "text");
}

export function getMetadataNumberFieldSpec(
  field: MetadataFieldNameByType<"number">,
): MetadataFieldSpecByType<"number"> {
  return getMetadataFieldSpecByType(field, "number");
}

export function getMetadataBooleanFieldSpec(
  field: MetadataFieldNameByType<"boolean">,
): MetadataFieldSpecByType<"boolean"> {
  return getMetadataFieldSpecByType(field, "boolean");
}

export function getMetadataSetRecordValues(record: NormalizedRecord, field: MetadataFieldNameByType<"set">): string[] {
  return getMetadataRecordValue(record, getMetadataSetFieldSpec(field));
}

export function getMetadataStringRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"enumString"> | MetadataFieldNameByType<"text">,
): string | null {
  return isMetadataEnumStringField(field)
    ? getMetadataRecordValue(record, getMetadataEnumStringFieldSpec(field))
    : getMetadataRecordValue(record, getMetadataTextFieldSpec(field));
}

export function getMetadataNumberRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"number">,
): number | null {
  return getMetadataRecordValue(record, getMetadataNumberFieldSpec(field));
}

export function getMetadataBooleanRecordValue(
  record: NormalizedRecord,
  field: MetadataFieldNameByType<"boolean">,
): boolean {
  return getMetadataRecordValue(record, getMetadataBooleanFieldSpec(field));
}

export function getMetadataRecordSelectClauses(): string[] {
  return METADATA_FIELD_REGISTRY.map((entry) => entry.selectClause);
}

export function getMetadataFieldSpecsByPresentation(
  presentation: Exclude<MetadataPresentation, "none">,
): MetadataFieldSpecEntry[] {
  return METADATA_FIELD_REGISTRY.filter((entry) => entry.presentation === presentation);
}
