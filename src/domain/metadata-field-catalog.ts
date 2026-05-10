import { ACTOR_METRIC_DISCOVERY_NAMESPACES } from "./actor-metrics.js";
import { ITEM_METRIC_DISCOVERY_NAMESPACES } from "./item-metrics.js";
import { SEARCH_CATEGORIES } from "./categories.js";
import type { FilterValueOrdering } from "./filter-value-ordering.js";
import {
  METADATA_FIELD_KIND_OPERATORS,
  type MetadataFieldName,
  type MetadataFieldNameByType,
  type MetadataFieldType,
} from "./metadata-field-types.js";
import type { SearchFilterNode } from "./search-request-types.js";
import {
  getSearchPromotedFieldValueOrdering,
  isSearchPromotedFieldDomainKey,
} from "./search-field-domains.js";
import type { SearchCategory, SearchSubcategory } from "./search-types.js";

export interface MetadataFieldCatalogEntry {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  categories: readonly SearchCategory[];
  subcategories?: readonly SearchSubcategory[];
  discoverable?: boolean;
  notes?: string;
  valueOrdering?: FilterValueOrdering;
}

function defineMetadataFieldCatalogEntry<const Field extends MetadataFieldName>(
  entry: MetadataFieldCatalogEntry & { field: Field },
): MetadataFieldCatalogEntry & { field: Field } {
  return entry;
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

export const SEARCH_METADATA_FIELD_CATALOG = [
  defineMetadataFieldCatalogEntry({
    field: "traits",
    fieldType: "set",
    categories: ALL_CATEGORIES,
    discoverable: true,
    notes: "Cross-category trait facet. Use category or subcategory boundaries before trait predicates.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "families",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    notes: "Creature family facet derived from glossary-backed monster families plus allowlisted Pathfinder NPC Core cohort folders.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "derivedTags",
    fieldType: "set",
    categories: DERIVED_TAG_CATEGORIES,
    discoverable: true,
    notes: "Curated heuristic tags. Supported for categories with explicit derived-tag ontology coverage.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "traditions",
    fieldType: "set",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "spellKinds",
    fieldType: "set",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "damageTypes",
    fieldType: "set",
    categories: EQUIPMENT_AND_SPELL,
    discoverable: true,
    notes: "Available for spells and item records with typed damage payloads.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "languages",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "speedTypes",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "senses",
    fieldType: "set",
    categories: CREATURE_ONLY,
    discoverable: true,
    notes: "Creature sense types such as darkvision, low light vision, or scent.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "immunities",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "resistances",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "weaknesses",
    fieldType: "set",
    categories: CREATURE_AND_HAZARD,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "disableSkills",
    fieldType: "set",
    categories: ["hazard"],
    discoverable: true,
    notes: "Structured hazard disable skills parsed from disable text when PF2E markup provides a clear anchor.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "variantAxes",
    fieldType: "set",
    categories: ["equipment", "spell"],
    discoverable: true,
    notes: "Normalized variant-family axes such as rank, grade, or specialization.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "sourceCategory",
    fieldType: "enumString",
    categories: ALL_CATEGORIES,
    discoverable: true,
    notes: "Normalized source bucket such as core, rules, or adventure.",
  }),
  defineMetadataFieldCatalogEntry({
    field: "size",
    fieldType: "enumString",
    categories: CREATURE_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "usage",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "weaponGroup",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    subcategories: ["weapon", "ammo"],
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "armorGroup",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    subcategories: ["armor", "shield"],
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "itemCategory",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "baseItem",
    fieldType: "enumString",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "saveType",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "areaType",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "durationUnit",
    fieldType: "enumString",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "rarity",
    fieldType: "enumString",
    categories: ALL_CATEGORIES,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "variantFamilyKey",
    fieldType: "enumString",
    categories: ["equipment", "spell"],
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "publicationTitle",
    fieldType: "text",
    categories: ALL_CATEGORIES,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "rangeText",
    fieldType: "text",
    categories: SPELL_ONLY,
  }),
  defineMetadataFieldCatalogEntry({
    field: "durationText",
    fieldType: "text",
    categories: SPELL_ONLY,
  }),
  defineMetadataFieldCatalogEntry({
    field: "targetText",
    fieldType: "text",
    categories: SPELL_ONLY,
  }),
  defineMetadataFieldCatalogEntry({
    field: "disableText",
    fieldType: "text",
    categories: ["hazard"],
  }),
  defineMetadataFieldCatalogEntry({
    field: "variantBaseName",
    fieldType: "text",
    categories: ["equipment", "spell"],
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "variantLabel",
    fieldType: "text",
    categories: ["equipment", "spell"],
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "level",
    fieldType: "number",
    categories: ALL_CATEGORIES,
  }),
  defineMetadataFieldCatalogEntry({
    field: "priceCp",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
  }),
  defineMetadataFieldCatalogEntry({
    field: "bulkValue",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
  }),
  defineMetadataFieldCatalogEntry({
    field: "actionCost",
    fieldType: "number",
    categories: ALL_CATEGORIES,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "hands",
    fieldType: "number",
    categories: EQUIPMENT_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "rangeValue",
    fieldType: "number",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "areaValue",
    fieldType: "number",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "hasDescription",
    fieldType: "boolean",
    categories: ALL_CATEGORIES,
  }),
  defineMetadataFieldCatalogEntry({
    field: "publicationRemaster",
    fieldType: "boolean",
    categories: ALL_CATEGORIES,
  }),
  defineMetadataFieldCatalogEntry({
    field: "sustained",
    fieldType: "boolean",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "basicSave",
    fieldType: "boolean",
    categories: SPELL_ONLY,
    discoverable: true,
  }),
  defineMetadataFieldCatalogEntry({
    field: "isComplex",
    fieldType: "boolean",
    categories: ["hazard"],
    discoverable: true,
  }),
] as const;

export type SearchMetadataFieldCatalogEntry = (typeof SEARCH_METADATA_FIELD_CATALOG)[number];

const SEARCH_METADATA_FIELD_CATALOG_BY_NAME = new Map<MetadataFieldName, SearchMetadataFieldCatalogEntry>(
  SEARCH_METADATA_FIELD_CATALOG.map((entry) => [entry.field, entry]),
);

const SEARCH_METADATA_FIELD_NAMES_BY_TYPE = {
  set: new Set<MetadataFieldNameByType<"set">>(
    SEARCH_METADATA_FIELD_CATALOG.filter((entry) => entry.fieldType === "set").map(
      (entry) => entry.field as MetadataFieldNameByType<"set">,
    ),
  ),
  enumString: new Set<MetadataFieldNameByType<"enumString">>(
    SEARCH_METADATA_FIELD_CATALOG.filter((entry) => entry.fieldType === "enumString").map(
      (entry) => entry.field as MetadataFieldNameByType<"enumString">,
    ),
  ),
  text: new Set<MetadataFieldNameByType<"text">>(
    SEARCH_METADATA_FIELD_CATALOG.filter((entry) => entry.fieldType === "text").map(
      (entry) => entry.field as MetadataFieldNameByType<"text">,
    ),
  ),
  number: new Set<MetadataFieldNameByType<"number">>(
    SEARCH_METADATA_FIELD_CATALOG.filter((entry) => entry.fieldType === "number").map(
      (entry) => entry.field as MetadataFieldNameByType<"number">,
    ),
  ),
  boolean: new Set<MetadataFieldNameByType<"boolean">>(
    SEARCH_METADATA_FIELD_CATALOG.filter((entry) => entry.fieldType === "boolean").map(
      (entry) => entry.field as MetadataFieldNameByType<"boolean">,
    ),
  ),
} as const;

export function getMetadataFieldCatalogEntry(field: MetadataFieldName): SearchMetadataFieldCatalogEntry {
  const entry = SEARCH_METADATA_FIELD_CATALOG_BY_NAME.get(field);
  if (!entry) {
    throw new Error(`Unknown metadata field "${field}".`);
  }
  return entry;
}

export function isMetadataSetField(field: string): field is MetadataFieldNameByType<"set"> {
  return SEARCH_METADATA_FIELD_NAMES_BY_TYPE.set.has(field as MetadataFieldNameByType<"set">);
}

export function isMetadataEnumStringField(field: string): field is MetadataFieldNameByType<"enumString"> {
  return SEARCH_METADATA_FIELD_NAMES_BY_TYPE.enumString.has(field as MetadataFieldNameByType<"enumString">);
}

export function isMetadataTextField(field: string): field is MetadataFieldNameByType<"text"> {
  return SEARCH_METADATA_FIELD_NAMES_BY_TYPE.text.has(field as MetadataFieldNameByType<"text">);
}

export function isMetadataNumberField(field: string): field is MetadataFieldNameByType<"number"> {
  return SEARCH_METADATA_FIELD_NAMES_BY_TYPE.number.has(field as MetadataFieldNameByType<"number">);
}

export function isMetadataBooleanField(field: string): field is MetadataFieldNameByType<"boolean"> {
  return SEARCH_METADATA_FIELD_NAMES_BY_TYPE.boolean.has(field as MetadataFieldNameByType<"boolean">);
}

export interface MetadataFieldSemantics {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  operators: readonly string[];
  categories: SearchCategory[];
  subcategories?: SearchSubcategory[];
  discoverable: boolean;
  notes?: string;
  valueOrdering?: FilterValueOrdering;
}

export interface MetadataFieldTypeGroup {
  type: MetadataFieldType;
  operators: string[];
  fields: MetadataFieldName[];
}

export interface MetadataCategoryExample {
  label: string;
  filter: SearchFilterNode;
  notes?: string;
}

export interface MetadataFilterSemantics {
  fieldTypes: MetadataFieldTypeGroup[];
  metadataFields: MetadataFieldSemantics[];
  metadataFieldsByCategory: Record<SearchCategory, MetadataFieldName[]>;
  metadataFieldsByCategoryAndSubcategory: Partial<
    Record<SearchCategory, Partial<Record<SearchSubcategory, MetadataFieldName[]>>>
  >;
  examplesByCategory: Partial<Record<SearchCategory, MetadataCategoryExample[]>>;
  actorMetricDiscovery?: {
    categories: SearchCategory[];
    filterValueField: string;
    namespaces: Array<{ prefix: string; description: string }>;
    notes: string[];
  };
  itemMetricDiscovery?: {
    categories: SearchCategory[];
    filterValueField: string;
    namespaces: Array<{ prefix: string; description: string }>;
    notes: string[];
  };
  discoverableFieldLookupWorkflow: {
    semanticsFirst: string;
    filterValuesSecond: string;
  };
}

const ACTOR_METRIC_CATEGORIES: SearchCategory[] = ["creature", "hazard"];
const ITEM_METRIC_CATEGORIES: SearchCategory[] = ["equipment"];

const EXAMPLES_BY_CATEGORY: Partial<Record<SearchCategory, MetadataCategoryExample[]>> = {
  equipment: [
    {
      label: "One-handed bombs",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "weaponGroup", op: "eq", value: "bomb" } },
          { kind: "metadataPredicate", predicate: { field: "hands", op: "eq", value: 1 } },
        ],
      },
      notes: "Use equipment boundaries first, then item-native metadata for the final cut.",
    },
    {
      label: "Worn disguises",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "usage", op: "eq", value: "worn" } },
          {
            kind: "metadataPredicate",
            predicate: { field: "derivedTags", op: "includes", value: "social_infiltration" },
          },
        ],
      },
    },
  ],
  creature: [
    {
      label: "Core undead without the water trait",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "sourceCategory", op: "eq", value: "core" } },
          { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "undead" } },
          {
            kind: "not",
            child: { kind: "metadataPredicate", predicate: { field: "traits", op: "includes", value: "water" } },
          },
        ],
      },
    },
    {
      label: "Ghosts with fire resistance",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "families", op: "includes", value: "ghost" } },
          { kind: "metadataPredicate", predicate: { field: "resistances", op: "includes", value: "fire" } },
        ],
      },
    },
  ],
  spell: [
    {
      label: "Primal focus spells with cold damage",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "traditions", op: "includes", value: "primal" } },
          { kind: "metadataPredicate", predicate: { field: "spellKinds", op: "includes", value: "focus" } },
          { kind: "metadataPredicate", predicate: { field: "damageTypes", op: "includes", value: "cold" } },
        ],
      },
    },
    {
      label: "Reflex burst spells",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "saveType", op: "eq", value: "reflex" } },
          { kind: "metadataPredicate", predicate: { field: "areaType", op: "eq", value: "burst" } },
        ],
      },
    },
    {
      label: "Sustained minute-duration spells",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "sustained", op: "eq", value: true } },
          { kind: "metadataPredicate", predicate: { field: "durationUnit", op: "eq", value: "minute" } },
        ],
      },
    },
  ],
  hazard: [
    {
      label: "Complex hazards disabled with Thievery",
      filter: {
        kind: "allOf",
        children: [
          { kind: "metadataPredicate", predicate: { field: "isComplex", op: "eq", value: true } },
          { kind: "metadataPredicate", predicate: { field: "disableSkills", op: "includes", value: "thievery" } },
        ],
      },
    },
  ],
};

function uniqueFieldNames(fields: MetadataFieldName[]): MetadataFieldName[] {
  return fields.filter((field, index, values) => values.indexOf(field) === index);
}

function buildFieldTypeGroups(registry: MetadataFieldSemantics[]): MetadataFieldTypeGroup[] {
  const fieldTypes: MetadataFieldType[] = ["set", "enumString", "text", "number", "boolean"];
  return fieldTypes.map((fieldType) => {
    const entries = registry.filter((entry) => entry.fieldType === fieldType);
    return {
      type: fieldType,
      operators: [...new Set(entries.flatMap((entry) => entry.operators))],
      fields: entries.map((entry) => entry.field),
    };
  });
}

function buildFieldsByCategory(registry: MetadataFieldSemantics[]): Record<SearchCategory, MetadataFieldName[]> {
  const result = Object.fromEntries(
    SEARCH_CATEGORIES.map((category) => [category, [] as MetadataFieldName[]]),
  ) as Record<SearchCategory, MetadataFieldName[]>;

  for (const entry of registry) {
    for (const category of entry.categories) {
      result[category].push(entry.field);
    }
  }

  for (const category of SEARCH_CATEGORIES) {
    result[category] = uniqueFieldNames(result[category]);
  }

  return result;
}

function buildFieldsByCategoryAndSubcategory(
  registry: MetadataFieldSemantics[],
): Partial<Record<SearchCategory, Partial<Record<SearchSubcategory, MetadataFieldName[]>>>> {
  const result: Partial<Record<SearchCategory, Partial<Record<SearchSubcategory, MetadataFieldName[]>>>> = {};

  for (const entry of registry) {
    if (!entry.subcategories || entry.subcategories.length === 0) {
      continue;
    }

    for (const category of entry.categories) {
      const categoryBucket = result[category] ?? {};
      for (const subcategory of entry.subcategories) {
        const subcategoryBucket = categoryBucket[subcategory] ?? [];
        subcategoryBucket.push(entry.field);
        categoryBucket[subcategory] = uniqueFieldNames(subcategoryBucket);
      }
      result[category] = categoryBucket;
    }
  }

  return result;
}

export function getMetadataFieldSemantics(): MetadataFieldSemantics[] {
  return SEARCH_METADATA_FIELD_CATALOG.map((entry) => ({
    field: entry.field,
    fieldType: entry.fieldType,
    operators: [...METADATA_FIELD_KIND_OPERATORS[entry.fieldType]],
    categories: [...entry.categories],
    subcategories: entry.subcategories ? [...entry.subcategories] : undefined,
    discoverable: Boolean(entry.discoverable),
    notes: entry.notes,
    valueOrdering: isSearchPromotedFieldDomainKey(entry.field)
      ? getSearchPromotedFieldValueOrdering(entry.field)
      : entry.valueOrdering,
  }));
}

export function getMetadataFilterSemantics(): MetadataFilterSemantics {
  const metadataFields = getMetadataFieldSemantics();
  return {
    fieldTypes: buildFieldTypeGroups(metadataFields),
    metadataFields,
    metadataFieldsByCategory: buildFieldsByCategory(metadataFields),
    metadataFieldsByCategoryAndSubcategory: buildFieldsByCategoryAndSubcategory(metadataFields),
    examplesByCategory: EXAMPLES_BY_CATEGORY,
    actorMetricDiscovery: {
      categories: ACTOR_METRIC_CATEGORIES,
      filterValueField: "actorMetrics",
      namespaces: ACTOR_METRIC_DISCOVERY_NAMESPACES.map((entry) => ({ ...entry })),
      notes: [
        'Use pf2e_list_filter_values with field:"actorMetrics" to enumerate live metric keys in the current corpus.',
        "Set metricPrefix to narrow discovery to one namespace such as ability., save., skill., ac., hp., hardness., perception., or stealth.",
        "Set metric to a text or boolean metric such as save.best or skill.arcana.proficient to enumerate live values.",
      ],
    },
    itemMetricDiscovery: {
      categories: ITEM_METRIC_CATEGORIES,
      filterValueField: "itemMetrics",
      namespaces: ITEM_METRIC_DISCOVERY_NAMESPACES.map((entry) => ({ ...entry })),
      notes: [
        'Use pf2e_list_filter_values with field:"itemMetrics" to enumerate live metric keys in the current corpus.',
        "Set metricPrefix to narrow discovery to one namespace such as weapon., armor., or shield.",
        "Set metric to a text or boolean metric to enumerate live values when such metrics are available.",
      ],
    },
    discoverableFieldLookupWorkflow: {
      semanticsFirst:
        "Call pf2e_get_search_semantics first to learn which metadata fields and operators are meaningful for the target category or subcategory.",
      filterValuesSecond:
        'Call pf2e_list_filter_values only for fields marked discoverable:true or for field:"actorMetrics" or field:"itemMetrics" when you need live corpus values for a chosen metric namespace.',
    },
  };
}

export function getDiscoverableMetadataFields(): MetadataFieldName[] {
  return getMetadataFieldSemantics()
    .filter((entry) => entry.discoverable)
    .map((entry) => entry.field);
}
