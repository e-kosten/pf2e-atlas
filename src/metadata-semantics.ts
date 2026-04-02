import { SEARCH_CATEGORIES } from "./categories.js";
import {
  FILTER_VALUE_FIELDS,
  METADATA_BOOLEAN_FIELDS,
  METADATA_ENUM_STRING_FIELDS,
  METADATA_NUMBER_FIELDS,
  METADATA_SET_FIELDS,
  METADATA_TEXT_STRING_FIELDS,
  MetadataBooleanField,
  MetadataBooleanOperator,
  MetadataEnumStringField,
  MetadataEnumStringOperator,
  MetadataFilterNode,
  MetadataNumberField,
  MetadataNumberOperator,
  MetadataSetField,
  MetadataSetOperator,
  MetadataTextStringField,
  MetadataTextStringOperator,
  SearchCategory,
  SearchSubcategory,
} from "./types.js";

type MetadataFieldName =
  | MetadataSetField
  | MetadataEnumStringField
  | MetadataTextStringField
  | MetadataNumberField
  | MetadataBooleanField;

type MetadataFieldType = "set" | "enumString" | "text" | "number" | "boolean";

type MetadataFieldOperators = readonly (
  | MetadataSetOperator
  | MetadataEnumStringOperator
  | MetadataTextStringOperator
  | MetadataNumberOperator
  | MetadataBooleanOperator
)[];

export interface MetadataFieldSemantics {
  field: MetadataFieldName;
  fieldType: MetadataFieldType;
  operators: MetadataFieldOperators;
  categories: SearchCategory[];
  subcategories?: SearchSubcategory[];
  discoverable: boolean;
  notes?: string;
}

export interface MetadataFieldTypeGroup {
  type: MetadataFieldType;
  operators: string[];
  fields: MetadataFieldName[];
}

export interface MetadataCategoryExample {
  label: string;
  metadata: MetadataFilterNode;
  notes?: string;
}

export interface MetadataFilterSemantics {
  booleanGroups: Record<"and" | "or" | "not", string>;
  fieldTypes: MetadataFieldTypeGroup[];
  metadataFields: MetadataFieldSemantics[];
  metadataFieldsByCategory: Record<SearchCategory, MetadataFieldName[]>;
  metadataFieldsByCategoryAndSubcategory: Partial<Record<SearchCategory, Partial<Record<SearchSubcategory, MetadataFieldName[]>>>>;
  examplesByCategory: Partial<Record<SearchCategory, MetadataCategoryExample[]>>;
  discoverableFieldLookupWorkflow: {
    semanticsFirst: string;
    filterValuesSecond: string;
  };
}

const SET_OPERATORS = ["includesAny", "includesAll", "excludesAny"] as const;
const ENUM_STRING_OPERATORS = ["eq", "in", "notIn"] as const;
const TEXT_OPERATORS = ["contains", "notContains"] as const;
const NUMBER_OPERATORS = ["eq", "gte", "lte", "between"] as const;
const BOOLEAN_OPERATORS = ["eq"] as const;

const ALL_CATEGORIES = [...SEARCH_CATEGORIES];
const CREATURE_ONLY: SearchCategory[] = ["creature"];
const EQUIPMENT_ONLY: SearchCategory[] = ["equipment"];
const SPELL_ONLY: SearchCategory[] = ["spell"];
const EQUIPMENT_AND_SPELL: SearchCategory[] = ["equipment", "spell"];
const DERIVED_TAG_CATEGORIES: SearchCategory[] = ["equipment", "creature"];

const METADATA_FIELD_NAME_SET = new Set<MetadataFieldName>([
  ...METADATA_SET_FIELDS,
  ...METADATA_ENUM_STRING_FIELDS,
  ...METADATA_TEXT_STRING_FIELDS,
  ...METADATA_NUMBER_FIELDS,
  ...METADATA_BOOLEAN_FIELDS,
]);

const DISCOVERABLE_METADATA_FIELDS = new Set<MetadataFieldName>(
  FILTER_VALUE_FIELDS.filter((field) => METADATA_FIELD_NAME_SET.has(field as MetadataFieldName)) as MetadataFieldName[],
);

const METADATA_FIELD_REGISTRY: MetadataFieldSemantics[] = [
  {
    field: "traits",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("traits"),
    notes: "Cross-category trait facet. Use category or subcategory boundaries before trait predicates.",
  },
  {
    field: "families",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("families"),
    notes: "Creature family facet derived from PF2E family glossary references.",
  },
  {
    field: "derivedTags",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: DERIVED_TAG_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("derivedTags"),
    notes: "Curated heuristic tags. Currently supported for equipment and creature records.",
  },
  {
    field: "traditions",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: SPELL_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("traditions"),
  },
  {
    field: "spellKinds",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: SPELL_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("spellKinds"),
  },
  {
    field: "damageTypes",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: EQUIPMENT_AND_SPELL,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("damageTypes"),
    notes: "Available for spells and item records with typed damage payloads.",
  },
  {
    field: "languages",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("languages"),
  },
  {
    field: "speedTypes",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("speedTypes"),
  },
  {
    field: "immunities",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("immunities"),
  },
  {
    field: "resistances",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("resistances"),
  },
  {
    field: "weaknesses",
    fieldType: "set",
    operators: SET_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("weaknesses"),
  },
  {
    field: "sourceCategory",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("sourceCategory"),
    notes: "Normalized source bucket such as core, rules, or adventure.",
  },
  {
    field: "size",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: CREATURE_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("size"),
  },
  {
    field: "usage",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: EQUIPMENT_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("usage"),
  },
  {
    field: "weaponGroup",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: EQUIPMENT_ONLY,
    subcategories: ["weapon"],
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("weaponGroup"),
  },
  {
    field: "armorGroup",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: EQUIPMENT_ONLY,
    subcategories: ["armor"],
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("armorGroup"),
  },
  {
    field: "itemCategory",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: EQUIPMENT_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("itemCategory"),
    notes: "Item-native family/category metadata. Pair with subcategory boundaries for better precision.",
  },
  {
    field: "rarity",
    fieldType: "enumString",
    operators: ENUM_STRING_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("rarity"),
    notes: "Also available as a top-level filter for the common common/uncommon/rare/unique boundary.",
  },
  {
    field: "publicationTitle",
    fieldType: "text",
    operators: TEXT_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("publicationTitle"),
  },
  {
    field: "level",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("level"),
    notes: "Also available via top-level levelMin and levelMax.",
  },
  {
    field: "priceCp",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: EQUIPMENT_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("priceCp"),
    notes: "Also available via top-level priceMin and priceMax.",
  },
  {
    field: "bulkValue",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: EQUIPMENT_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("bulkValue"),
  },
  {
    field: "actionCost",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: ["equipment", "spell", "rule"],
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("actionCost"),
    notes: "Also available as a top-level filter.",
  },
  {
    field: "hands",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: EQUIPMENT_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("hands"),
  },
  {
    field: "rangeValue",
    fieldType: "number",
    operators: NUMBER_OPERATORS,
    categories: SPELL_ONLY,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("rangeValue"),
  },
  {
    field: "isUnique",
    fieldType: "boolean",
    operators: BOOLEAN_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("isUnique"),
  },
  {
    field: "hasDescription",
    fieldType: "boolean",
    operators: BOOLEAN_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("hasDescription"),
  },
  {
    field: "publicationRemaster",
    fieldType: "boolean",
    operators: BOOLEAN_OPERATORS,
    categories: ALL_CATEGORIES,
    discoverable: DISCOVERABLE_METADATA_FIELDS.has("publicationRemaster"),
  },
];

const BOOLEAN_GROUPS = {
  and: "Requires every child predicate or group to match. Must contain at least 2 child nodes.",
  or: "Requires at least one child predicate or group to match. Must contain at least 2 child nodes.",
  not: "Negates exactly one child predicate or group.",
} as const;

const EXAMPLES_BY_CATEGORY: Partial<Record<SearchCategory, MetadataCategoryExample[]>> = {
  equipment: [
    {
      label: "One-handed bombs",
      metadata: {
        and: [
          { field: "weaponGroup", op: "eq", value: "bomb" },
          { field: "hands", op: "eq", value: 1 },
        ],
      },
      notes: "Use equipment boundaries first, then item-native metadata for the final cut.",
    },
    {
      label: "Worn disguises",
      metadata: {
        and: [
          { field: "usage", op: "eq", value: "worn" },
          { field: "derivedTags", op: "includesAny", values: ["social_infiltration"] },
        ],
      },
    },
  ],
  creature: [
    {
      label: "Core undead without the water trait",
      metadata: {
        and: [
          { field: "sourceCategory", op: "eq", value: "core" },
          { field: "traits", op: "includesAny", values: ["undead"] },
          { field: "traits", op: "excludesAny", values: ["water"] },
        ],
      },
    },
    {
      label: "Ghosts with fire resistance",
      metadata: {
        and: [
          { field: "families", op: "includesAny", values: ["ghost"] },
          { field: "resistances", op: "includesAny", values: ["fire"] },
        ],
      },
    },
  ],
  spell: [
    {
      label: "Primal focus spells with cold damage",
      metadata: {
        and: [
          { field: "traditions", op: "includesAny", values: ["primal"] },
          { field: "spellKinds", op: "includesAny", values: ["focus"] },
          { field: "damageTypes", op: "includesAny", values: ["cold"] },
        ],
      },
    },
    {
      label: "Short-range remastered spells",
      metadata: {
        and: [
          { field: "rangeValue", op: "lte", value: 60 },
          { field: "publicationRemaster", op: "eq", value: true },
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
  return METADATA_FIELD_REGISTRY.map((entry) => ({
    ...entry,
    categories: [...entry.categories],
    subcategories: entry.subcategories ? [...entry.subcategories] : undefined,
    operators: [...entry.operators],
  }));
}

export function getMetadataFilterSemantics(): MetadataFilterSemantics {
  const metadataFields = getMetadataFieldSemantics();
  return {
    booleanGroups: BOOLEAN_GROUPS,
    fieldTypes: buildFieldTypeGroups(metadataFields),
    metadataFields,
    metadataFieldsByCategory: buildFieldsByCategory(metadataFields),
    metadataFieldsByCategoryAndSubcategory: buildFieldsByCategoryAndSubcategory(metadataFields),
    examplesByCategory: EXAMPLES_BY_CATEGORY,
    discoverableFieldLookupWorkflow: {
      semanticsFirst: "Call pf2e_get_search_semantics first to learn which metadata fields and operators are meaningful for the target category or subcategory.",
      filterValuesSecond: "Call pf2e_list_filter_values only for fields marked discoverable:true when you need live corpus values for a chosen field.",
    },
  };
}

export function getDiscoverableMetadataFields(): MetadataFieldName[] {
  return getMetadataFieldSemantics()
    .filter((entry) => entry.discoverable)
    .map((entry) => entry.field);
}
