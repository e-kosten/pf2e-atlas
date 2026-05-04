import type { MetadataFieldName, MetadataFieldType } from "./metadata-field-types.js";

const METADATA_FIELD_LABELS = {
  actionCost: "Action Cost",
  areaType: "Area Type",
  areaValue: "Area Value",
  armorGroup: "Armor Group",
  baseItem: "Base Item",
  basicSave: "Basic Save",
  bulkValue: "Bulk",
  damageTypes: "Damage Types",
  derivedTags: "Derived Tags",
  disableSkills: "Disable Skills",
  disableText: "Disable Text",
  durationText: "Duration",
  durationUnit: "Duration Unit",
  hands: "Hands",
  hasDescription: "Has Description",
  isComplex: "Complex Hazard",
  itemCategory: "Item Category",
  priceCp: "Price",
  publicationRemaster: "Publication Remaster",
  publicationTitle: "Publication Title",
  rangeText: "Range",
  rangeValue: "Range Value",
  saveType: "Save Type",
  sourceCategory: "Source Category",
  speedTypes: "Speed Types",
  spellKinds: "Spell Kinds",
  sustained: "Sustained",
  targetText: "Target",
  usage: "Usage",
  variantAxes: "Variant Axes",
  variantBaseName: "Variant Base Name",
  variantFamilyKey: "Variant Family",
  variantLabel: "Variant Label",
  weaponGroup: "Weapon Group",
} satisfies Partial<Record<MetadataFieldName, string>>;
const METADATA_FIELD_LABELS_BY_VALUE: Partial<Record<string, string>> = METADATA_FIELD_LABELS;

const METADATA_FIELD_TYPE_LABELS: Record<MetadataFieldType, string> = {
  set: "String Set",
  enumString: "Enumerated String",
  text: "Free Text",
  number: "Number",
  boolean: "Boolean",
};
const METADATA_FIELD_TYPE_LABELS_BY_VALUE: Partial<Record<string, string>> = METADATA_FIELD_TYPE_LABELS;

export function humanizeOntologySearchIdentifier(value: string): string {
  return value
    .trim()
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .split(/[_\-.:/]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => segment.trim())
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function formatMetadataFieldLabel(field: string): string {
  return METADATA_FIELD_LABELS_BY_VALUE[field] ?? humanizeOntologySearchIdentifier(field);
}

export function formatMetadataFieldTypeLabel(fieldType: string): string {
  return METADATA_FIELD_TYPE_LABELS_BY_VALUE[fieldType] ?? humanizeOntologySearchIdentifier(fieldType);
}

export function describeMetadataFieldType(fieldType: string): string {
  return formatMetadataFieldTypeLabel(fieldType).toLowerCase();
}

export function formatOntologySearchVocabularyLabel(value: string): string {
  return value in METADATA_FIELD_TYPE_LABELS
    ? formatMetadataFieldTypeLabel(value)
    : formatMetadataFieldLabel(value);
}
