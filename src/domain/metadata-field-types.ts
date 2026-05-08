export type MetadataFieldType = "set" | "enumString" | "text" | "number" | "boolean";

export const METADATA_FIELD_KIND_OPERATORS = {
  set: ["includes"],
  enumString: ["eq", "in", "notIn"],
  text: ["eq", "notEq", "contains", "notContains"],
  number: ["eq", "gte", "lte", "between"],
  boolean: ["eq"],
} as const;

export const METADATA_SET_FIELDS = [
  "traits",
  "families",
  "derivedTags",
  "traditions",
  "spellKinds",
  "damageTypes",
  "languages",
  "speedTypes",
  "senses",
  "immunities",
  "resistances",
  "weaknesses",
  "disableSkills",
  "variantAxes",
] as const;

export const METADATA_ENUM_STRING_FIELDS = [
  "sourceCategory",
  "size",
  "usage",
  "weaponGroup",
  "armorGroup",
  "itemCategory",
  "baseItem",
  "saveType",
  "areaType",
  "durationUnit",
  "rarity",
  "variantFamilyKey",
] as const;

export const METADATA_TEXT_STRING_FIELDS = [
  "publicationTitle",
  "rangeText",
  "durationText",
  "targetText",
  "disableText",
  "variantBaseName",
  "variantLabel",
] as const;

export const METADATA_NUMBER_FIELDS = [
  "level",
  "priceCp",
  "bulkValue",
  "actionCost",
  "hands",
  "rangeValue",
  "areaValue",
] as const;

export const METADATA_BOOLEAN_FIELDS = [
  "hasDescription",
  "publicationRemaster",
  "sustained",
  "basicSave",
  "isComplex",
] as const;

export const METADATA_FIELDS = [
  ...METADATA_SET_FIELDS,
  ...METADATA_ENUM_STRING_FIELDS,
  ...METADATA_TEXT_STRING_FIELDS,
  ...METADATA_NUMBER_FIELDS,
  ...METADATA_BOOLEAN_FIELDS,
] as const;

export type MetadataSetField = (typeof METADATA_SET_FIELDS)[number];
export type MetadataEnumStringField = (typeof METADATA_ENUM_STRING_FIELDS)[number];
export type MetadataTextStringField = (typeof METADATA_TEXT_STRING_FIELDS)[number];
export type MetadataNumberField = (typeof METADATA_NUMBER_FIELDS)[number];
export type MetadataBooleanField = (typeof METADATA_BOOLEAN_FIELDS)[number];

export type MetadataFieldName =
  | MetadataSetField
  | MetadataEnumStringField
  | MetadataTextStringField
  | MetadataNumberField
  | MetadataBooleanField;

const METADATA_FIELD_NAMES = new Set<MetadataFieldName>(METADATA_FIELDS);

export type MetadataFieldNameByTypeMap = {
  set: MetadataSetField;
  enumString: MetadataEnumStringField;
  text: MetadataTextStringField;
  number: MetadataNumberField;
  boolean: MetadataBooleanField;
};

export type MetadataFieldNameByType<FieldType extends MetadataFieldType> = MetadataFieldNameByTypeMap[FieldType];

export function isMetadataFieldName(field: string): field is MetadataFieldName {
  return METADATA_FIELD_NAMES.has(field as MetadataFieldName);
}
