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

export interface MetadataExecutionSpec<Field extends MetadataFieldName = MetadataFieldName> {
  field: Field;
  recordProperty: keyof NormalizedRecord;
  valueNormalization?: MetadataValueNormalization;
}

export const METADATA_FIELD_EXECUTION_SPECS = [
  { field: "traits", recordProperty: "traits" },
  { field: "families", recordProperty: "families" },
  { field: "derivedTags", recordProperty: "derivedTags", valueNormalization: "derivedTag" },
  { field: "traditions", recordProperty: "traditions" },
  { field: "spellKinds", recordProperty: "spellKinds" },
  { field: "damageTypes", recordProperty: "damageTypes" },
  { field: "languages", recordProperty: "languages" },
  { field: "speedTypes", recordProperty: "speedTypes" },
  { field: "senses", recordProperty: "senses" },
  { field: "immunities", recordProperty: "immunities" },
  { field: "resistances", recordProperty: "resistances" },
  { field: "weaknesses", recordProperty: "weaknesses" },
  { field: "disableSkills", recordProperty: "disableSkills" },
  { field: "variantAxes", recordProperty: "variantAxes" },
  { field: "sourceCategory", recordProperty: "sourceCategory" },
  { field: "size", recordProperty: "size" },
  { field: "usage", recordProperty: "usage" },
  { field: "weaponGroup", recordProperty: "weaponGroup" },
  { field: "armorGroup", recordProperty: "armorGroup" },
  { field: "itemCategory", recordProperty: "itemCategory" },
  { field: "baseItem", recordProperty: "baseItem" },
  { field: "saveType", recordProperty: "saveType" },
  { field: "areaType", recordProperty: "areaType" },
  { field: "durationUnit", recordProperty: "durationUnit" },
  { field: "rarity", recordProperty: "rarity" },
  { field: "variantFamilyKey", recordProperty: "variantFamilyKey" },
  { field: "publicationTitle", recordProperty: "publicationTitle" },
  { field: "rangeText", recordProperty: "rangeText" },
  { field: "durationText", recordProperty: "durationText" },
  { field: "targetText", recordProperty: "targetText" },
  { field: "disableText", recordProperty: "disableText" },
  { field: "variantBaseName", recordProperty: "variantBaseName" },
  { field: "variantLabel", recordProperty: "variantLabel" },
  { field: "level", recordProperty: "level" },
  { field: "priceCp", recordProperty: "priceCp" },
  { field: "bulkValue", recordProperty: "bulkValue" },
  { field: "actionCost", recordProperty: "actionCost" },
  { field: "hands", recordProperty: "hands" },
  { field: "rangeValue", recordProperty: "rangeValue" },
  { field: "areaValue", recordProperty: "areaValue" },
  { field: "hasDescription", recordProperty: "hasDescription" },
  { field: "publicationRemaster", recordProperty: "publicationRemaster" },
  { field: "sustained", recordProperty: "sustained" },
  { field: "basicSave", recordProperty: "basicSave" },
  { field: "isComplex", recordProperty: "isComplex" },
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
