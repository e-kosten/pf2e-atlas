export const METADATA_SET_FIELDS = [
  "traits",
  "families",
  "derivedTags",
  "traditions",
  "spellKinds",
  "damageTypes",
  "languages",
  "speedTypes",
  "immunities",
  "resistances",
  "weaknesses",
] as const;

export type MetadataSetField = (typeof METADATA_SET_FIELDS)[number];

export const METADATA_ENUM_STRING_FIELDS = [
  "sourceCategory",
  "size",
  "usage",
  "weaponGroup",
  "armorGroup",
  "itemCategory",
  "rarity",
] as const;

export type MetadataEnumStringField = (typeof METADATA_ENUM_STRING_FIELDS)[number];

export const METADATA_TEXT_STRING_FIELDS = [
  "publicationTitle",
] as const;

export type MetadataTextStringField = (typeof METADATA_TEXT_STRING_FIELDS)[number];

export const METADATA_NUMBER_FIELDS = [
  "level",
  "priceCp",
  "bulkValue",
  "actionCost",
  "hands",
  "rangeValue",
] as const;

export type MetadataNumberField = (typeof METADATA_NUMBER_FIELDS)[number];

export const METADATA_BOOLEAN_FIELDS = [
  "isUnique",
  "hasDescription",
  "publicationRemaster",
] as const;

export type MetadataBooleanField = (typeof METADATA_BOOLEAN_FIELDS)[number];

export type MetadataSetOperator = "includesAny" | "includesAll" | "excludesAny";
export type MetadataEnumStringOperator = "eq" | "in" | "notIn";
export type MetadataTextStringOperator = "contains" | "notContains";
export type MetadataNumberOperator = "eq" | "gte" | "lte" | "between";
export type MetadataBooleanOperator = "eq";

export type MetadataSetPredicate = {
  field: MetadataSetField;
  op: MetadataSetOperator;
  values: string[];
};

export type MetadataEnumStringPredicate =
  | {
    field: MetadataEnumStringField;
    op: "eq";
    value: string;
  }
  | {
    field: MetadataEnumStringField;
    op: "in" | "notIn";
    values: string[];
  };

export type MetadataTextStringPredicate = {
  field: MetadataTextStringField;
  op: MetadataTextStringOperator;
  value: string;
};

export type MetadataNumberPredicate =
  | {
    field: MetadataNumberField;
    op: "eq" | "gte" | "lte";
    value: number;
  }
  | {
    field: MetadataNumberField;
    op: "between";
    min: number;
    max: number;
  };

export type MetadataBooleanPredicate = {
  field: MetadataBooleanField;
  op: MetadataBooleanOperator;
  value: boolean;
};

export type MetadataPredicate =
  | MetadataSetPredicate
  | MetadataEnumStringPredicate
  | MetadataTextStringPredicate
  | MetadataNumberPredicate
  | MetadataBooleanPredicate;

export type MetadataFilterNode =
  | MetadataPredicate
  | { and: MetadataFilterNode[] }
  | { or: MetadataFilterNode[] }
  | { not: MetadataFilterNode };
