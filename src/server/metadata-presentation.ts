import type { NormalizedRecord } from "../domain/record-types.js";
import type { MetadataFieldName } from "../domain/metadata-field-types.js";

export type MetadataPresentation = "summary" | "detail" | "none";

export interface MetadataPresentationSpec<Field extends MetadataFieldName = MetadataFieldName> {
  field: Field;
  recordProperty: keyof NormalizedRecord;
  presentation: Exclude<MetadataPresentation, "none">;
  presentWhen?: (record: NormalizedRecord) => boolean;
}

const HAS_VARIANT_FAMILY = (record: NormalizedRecord): boolean => Boolean(record.variantFamilyKey);

const METADATA_PRESENTATION_SPECS = [
  {
    field: "traits",
    recordProperty: "traits",
    presentation: "summary",
  },
  {
    field: "families",
    recordProperty: "families",
    presentation: "summary",
  },
  {
    field: "derivedTags",
    recordProperty: "derivedTags",
    presentation: "summary",
  },
  {
    field: "traditions",
    recordProperty: "traditions",
    presentation: "summary",
  },
  {
    field: "spellKinds",
    recordProperty: "spellKinds",
    presentation: "summary",
  },
  {
    field: "damageTypes",
    recordProperty: "damageTypes",
    presentation: "detail",
  },
  {
    field: "languages",
    recordProperty: "languages",
    presentation: "detail",
  },
  {
    field: "speedTypes",
    recordProperty: "speedTypes",
    presentation: "detail",
  },
  {
    field: "senses",
    recordProperty: "senses",
    presentation: "detail",
  },
  {
    field: "immunities",
    recordProperty: "immunities",
    presentation: "detail",
  },
  {
    field: "resistances",
    recordProperty: "resistances",
    presentation: "detail",
  },
  {
    field: "weaknesses",
    recordProperty: "weaknesses",
    presentation: "detail",
  },
  {
    field: "disableSkills",
    recordProperty: "disableSkills",
    presentation: "detail",
  },
  {
    field: "variantAxes",
    recordProperty: "variantAxes",
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  },
  {
    field: "sourceCategory",
    recordProperty: "sourceCategory",
    presentation: "summary",
  },
  {
    field: "size",
    recordProperty: "size",
    presentation: "detail",
  },
  {
    field: "usage",
    recordProperty: "usage",
    presentation: "detail",
  },
  {
    field: "weaponGroup",
    recordProperty: "weaponGroup",
    presentation: "detail",
  },
  {
    field: "armorGroup",
    recordProperty: "armorGroup",
    presentation: "detail",
  },
  {
    field: "itemCategory",
    recordProperty: "itemCategory",
    presentation: "detail",
  },
  {
    field: "baseItem",
    recordProperty: "baseItem",
    presentation: "detail",
  },
  {
    field: "saveType",
    recordProperty: "saveType",
    presentation: "detail",
  },
  {
    field: "areaType",
    recordProperty: "areaType",
    presentation: "detail",
  },
  {
    field: "durationUnit",
    recordProperty: "durationUnit",
    presentation: "summary",
  },
  {
    field: "rarity",
    recordProperty: "rarity",
    presentation: "summary",
  },
  {
    field: "variantFamilyKey",
    recordProperty: "variantFamilyKey",
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  },
  {
    field: "publicationTitle",
    recordProperty: "publicationTitle",
    presentation: "summary",
  },
  {
    field: "rangeText",
    recordProperty: "rangeText",
    presentation: "summary",
  },
  {
    field: "durationText",
    recordProperty: "durationText",
    presentation: "summary",
  },
  {
    field: "targetText",
    recordProperty: "targetText",
    presentation: "summary",
  },
  {
    field: "disableText",
    recordProperty: "disableText",
    presentation: "detail",
  },
  {
    field: "variantBaseName",
    recordProperty: "variantBaseName",
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  },
  {
    field: "variantLabel",
    recordProperty: "variantLabel",
    presentation: "summary",
    presentWhen: HAS_VARIANT_FAMILY,
  },
  {
    field: "level",
    recordProperty: "level",
    presentation: "summary",
  },
  {
    field: "priceCp",
    recordProperty: "priceCp",
    presentation: "detail",
  },
  {
    field: "bulkValue",
    recordProperty: "bulkValue",
    presentation: "detail",
  },
  {
    field: "actionCost",
    recordProperty: "actionCost",
    presentation: "detail",
  },
  {
    field: "hands",
    recordProperty: "hands",
    presentation: "detail",
  },
  {
    field: "rangeValue",
    recordProperty: "rangeValue",
    presentation: "detail",
  },
  {
    field: "areaValue",
    recordProperty: "areaValue",
    presentation: "summary",
  },
  {
    field: "hasDescription",
    recordProperty: "hasDescription",
    presentation: "summary",
  },
  {
    field: "publicationRemaster",
    recordProperty: "publicationRemaster",
    presentation: "summary",
  },
  {
    field: "sustained",
    recordProperty: "sustained",
    presentation: "summary",
  },
  {
    field: "basicSave",
    recordProperty: "basicSave",
    presentation: "summary",
  },
  {
    field: "isComplex",
    recordProperty: "isComplex",
    presentation: "detail",
  },
] as const satisfies readonly MetadataPresentationSpec[];

export function getMetadataFieldPresentationSpecs(
  presentation: Exclude<MetadataPresentation, "none">,
): MetadataPresentationSpec[] {
  return METADATA_PRESENTATION_SPECS.filter((entry) => entry.presentation === presentation);
}
