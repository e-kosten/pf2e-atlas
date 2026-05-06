import type { NormalizedRecord } from "../domain/record-types.js";
import type { MetadataFieldName } from "../domain/metadata-field-types.js";

export type MetadataRowValueSourceKind = "string" | "number" | "booleanNumber" | "jsonArray";

export interface MetadataRowValueSource {
  key: string;
  kind: MetadataRowValueSourceKind;
}

export interface MetadataRowProjection<Field extends MetadataFieldName = MetadataFieldName> {
  field: Field;
  recordProperty: keyof NormalizedRecord;
  selectClause: string;
  rowValueSource: MetadataRowValueSource;
}

export const METADATA_ROW_PROJECTIONS = [
  {
    field: "traits",
    recordProperty: "traits",
    selectClause: "r.traits_json AS traitsJson",
    rowValueSource: { key: "traitsJson", kind: "jsonArray" },
  },
  {
    field: "families",
    recordProperty: "families",
    selectClause: "r.families_json AS familiesJson",
    rowValueSource: { key: "familiesJson", kind: "jsonArray" },
  },
  {
    field: "derivedTags",
    recordProperty: "derivedTags",
    selectClause: "r.derived_tags_json AS derivedTagsJson",
    rowValueSource: { key: "derivedTagsJson", kind: "jsonArray" },
  },
  {
    field: "traditions",
    recordProperty: "traditions",
    selectClause: "s.traditions_json AS traditionsJson",
    rowValueSource: { key: "traditionsJson", kind: "jsonArray" },
  },
  {
    field: "spellKinds",
    recordProperty: "spellKinds",
    selectClause: "s.spell_kinds_json AS spellKindsJson",
    rowValueSource: { key: "spellKindsJson", kind: "jsonArray" },
  },
  {
    field: "damageTypes",
    recordProperty: "damageTypes",
    selectClause: "COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson",
    rowValueSource: { key: "damageTypesJson", kind: "jsonArray" },
  },
  {
    field: "languages",
    recordProperty: "languages",
    selectClause: "a.languages_json AS languagesJson",
    rowValueSource: { key: "languagesJson", kind: "jsonArray" },
  },
  {
    field: "speedTypes",
    recordProperty: "speedTypes",
    selectClause: "a.speed_types_json AS speedTypesJson",
    rowValueSource: { key: "speedTypesJson", kind: "jsonArray" },
  },
  {
    field: "senses",
    recordProperty: "senses",
    selectClause: "a.senses_json AS sensesJson",
    rowValueSource: { key: "sensesJson", kind: "jsonArray" },
  },
  {
    field: "immunities",
    recordProperty: "immunities",
    selectClause: "a.immunities_json AS immunitiesJson",
    rowValueSource: { key: "immunitiesJson", kind: "jsonArray" },
  },
  {
    field: "resistances",
    recordProperty: "resistances",
    selectClause: "a.resistances_json AS resistancesJson",
    rowValueSource: { key: "resistancesJson", kind: "jsonArray" },
  },
  {
    field: "weaknesses",
    recordProperty: "weaknesses",
    selectClause: "a.weaknesses_json AS weaknessesJson",
    rowValueSource: { key: "weaknessesJson", kind: "jsonArray" },
  },
  {
    field: "disableSkills",
    recordProperty: "disableSkills",
    selectClause: "a.disable_skills_json AS disableSkillsJson",
    rowValueSource: { key: "disableSkillsJson", kind: "jsonArray" },
  },
  {
    field: "variantAxes",
    recordProperty: "variantAxes",
    selectClause: "r.variant_axes_json AS variantAxesJson",
    rowValueSource: { key: "variantAxesJson", kind: "jsonArray" },
  },
  {
    field: "sourceCategory",
    recordProperty: "sourceCategory",
    selectClause: "r.source_category AS sourceCategory",
    rowValueSource: { key: "sourceCategory", kind: "string" },
  },
  {
    field: "size",
    recordProperty: "size",
    selectClause: "a.size AS size",
    rowValueSource: { key: "size", kind: "string" },
  },
  {
    field: "usage",
    recordProperty: "usage",
    selectClause: "i.usage_text AS usage",
    rowValueSource: { key: "usage", kind: "string" },
  },
  {
    field: "weaponGroup",
    recordProperty: "weaponGroup",
    selectClause: "i.weapon_group AS weaponGroup",
    rowValueSource: { key: "weaponGroup", kind: "string" },
  },
  {
    field: "armorGroup",
    recordProperty: "armorGroup",
    selectClause: "i.armor_group AS armorGroup",
    rowValueSource: { key: "armorGroup", kind: "string" },
  },
  {
    field: "itemCategory",
    recordProperty: "itemCategory",
    selectClause: "i.item_category AS itemCategory",
    rowValueSource: { key: "itemCategory", kind: "string" },
  },
  {
    field: "baseItem",
    recordProperty: "baseItem",
    selectClause: "i.base_item AS baseItem",
    rowValueSource: { key: "baseItem", kind: "string" },
  },
  {
    field: "saveType",
    recordProperty: "saveType",
    selectClause: "s.save_type AS saveType",
    rowValueSource: { key: "saveType", kind: "string" },
  },
  {
    field: "areaType",
    recordProperty: "areaType",
    selectClause: "s.area_type AS areaType",
    rowValueSource: { key: "areaType", kind: "string" },
  },
  {
    field: "durationUnit",
    recordProperty: "durationUnit",
    selectClause: "s.duration_unit AS durationUnit",
    rowValueSource: { key: "durationUnit", kind: "string" },
  },
  {
    field: "rarity",
    recordProperty: "rarity",
    selectClause: "r.rarity AS rarity",
    rowValueSource: { key: "rarity", kind: "string" },
  },
  {
    field: "variantFamilyKey",
    recordProperty: "variantFamilyKey",
    selectClause: "r.variant_family_key AS variantFamilyKey",
    rowValueSource: { key: "variantFamilyKey", kind: "string" },
  },
  {
    field: "publicationTitle",
    recordProperty: "publicationTitle",
    selectClause: "r.publication_title AS publicationTitle",
    rowValueSource: { key: "publicationTitle", kind: "string" },
  },
  {
    field: "rangeText",
    recordProperty: "rangeText",
    selectClause: "s.range_text AS rangeText",
    rowValueSource: { key: "rangeText", kind: "string" },
  },
  {
    field: "durationText",
    recordProperty: "durationText",
    selectClause: "s.duration_text AS durationText",
    rowValueSource: { key: "durationText", kind: "string" },
  },
  {
    field: "targetText",
    recordProperty: "targetText",
    selectClause: "s.target_text AS targetText",
    rowValueSource: { key: "targetText", kind: "string" },
  },
  {
    field: "disableText",
    recordProperty: "disableText",
    selectClause: "a.disable_text AS disableText",
    rowValueSource: { key: "disableText", kind: "string" },
  },
  {
    field: "variantBaseName",
    recordProperty: "variantBaseName",
    selectClause: "r.variant_base_name AS variantBaseName",
    rowValueSource: { key: "variantBaseName", kind: "string" },
  },
  {
    field: "variantLabel",
    recordProperty: "variantLabel",
    selectClause: "r.variant_label AS variantLabel",
    rowValueSource: { key: "variantLabel", kind: "string" },
  },
  {
    field: "level",
    recordProperty: "level",
    selectClause: "r.level AS level",
    rowValueSource: { key: "level", kind: "number" },
  },
  {
    field: "priceCp",
    recordProperty: "priceCp",
    selectClause: "i.price_cp AS priceCp",
    rowValueSource: { key: "priceCp", kind: "number" },
  },
  {
    field: "bulkValue",
    recordProperty: "bulkValue",
    selectClause: "i.bulk_value AS bulkValue",
    rowValueSource: { key: "bulkValue", kind: "number" },
  },
  {
    field: "actionCost",
    recordProperty: "actionCost",
    selectClause: "COALESCE(s.action_cost, i.action_cost) AS actionCost",
    rowValueSource: { key: "actionCost", kind: "number" },
  },
  {
    field: "hands",
    recordProperty: "hands",
    selectClause: "i.hands AS hands",
    rowValueSource: { key: "hands", kind: "number" },
  },
  {
    field: "rangeValue",
    recordProperty: "rangeValue",
    selectClause: "s.range_value AS rangeValue",
    rowValueSource: { key: "rangeValue", kind: "number" },
  },
  {
    field: "areaValue",
    recordProperty: "areaValue",
    selectClause: "s.area_value AS areaValue",
    rowValueSource: { key: "areaValue", kind: "number" },
  },
  {
    field: "hasDescription",
    recordProperty: "hasDescription",
    selectClause: "r.has_description AS hasDescription",
    rowValueSource: { key: "hasDescription", kind: "booleanNumber" },
  },
  {
    field: "publicationRemaster",
    recordProperty: "publicationRemaster",
    selectClause: "r.publication_remaster AS publicationRemaster",
    rowValueSource: { key: "publicationRemaster", kind: "booleanNumber" },
  },
  {
    field: "sustained",
    recordProperty: "sustained",
    selectClause: "s.sustained AS sustained",
    rowValueSource: { key: "sustained", kind: "booleanNumber" },
  },
  {
    field: "basicSave",
    recordProperty: "basicSave",
    selectClause: "s.basic_save AS basicSave",
    rowValueSource: { key: "basicSave", kind: "booleanNumber" },
  },
  {
    field: "isComplex",
    recordProperty: "isComplex",
    selectClause: "a.is_complex AS isComplex",
    rowValueSource: { key: "isComplex", kind: "booleanNumber" },
  },
] as const satisfies readonly MetadataRowProjection[];

export function getMetadataRecordSelectClauses(): string[] {
  return METADATA_ROW_PROJECTIONS.map((entry) => entry.selectClause);
}
