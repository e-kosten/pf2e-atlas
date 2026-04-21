import {
  ACTOR_METRIC_DISCOVERY_NAMESPACES,
  ACTOR_METRIC_NUMERIC_OPERATORS,
  ACTOR_METRIC_SCALAR_OPERATORS,
} from "../../domain/actor-metrics.js";
import { ITEM_METRIC_DISCOVERY_NAMESPACES } from "../../domain/item-metrics.js";
import { SEARCH_CATEGORIES } from "../../domain/categories.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import {
  METADATA_FIELD_KIND_OPERATORS,
  METADATA_FIELD_REGISTRY,
  type MetadataFieldName,
  type MetadataFieldType,
} from "./registry.js";
import type { MetadataFilterNode } from "./types.js";

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
  metadata: MetadataFilterNode;
  notes?: string;
}

export interface MetadataAdvancedPredicateSemantics {
  name: "actorMetric" | "actorMetricCompare" | "itemMetric" | "itemMetricCompare";
  categories: SearchCategory[];
  operators: string[];
  description: string;
  example: MetadataFilterNode;
}

export interface MetadataFilterSemantics {
  booleanGroups: Record<"and" | "or" | "not", string>;
  fieldTypes: MetadataFieldTypeGroup[];
  metadataFields: MetadataFieldSemantics[];
  metadataFieldsByCategory: Record<SearchCategory, MetadataFieldName[]>;
  metadataFieldsByCategoryAndSubcategory: Partial<
    Record<SearchCategory, Partial<Record<SearchSubcategory, MetadataFieldName[]>>>
  >;
  examplesByCategory: Partial<Record<SearchCategory, MetadataCategoryExample[]>>;
  advancedPredicates: MetadataAdvancedPredicateSemantics[];
  actorMetricDiscovery?: {
    filterValueField: string;
    namespaces: Array<{ prefix: string; description: string }>;
    notes: string[];
  };
  itemMetricDiscovery?: {
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
const EQUIPMENT_ONLY: SearchCategory[] = ["equipment"];

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
      label: "Reflex burst spells",
      metadata: {
        and: [
          { field: "saveType", op: "eq", value: "reflex" },
          { field: "areaType", op: "eq", value: "burst" },
        ],
      },
    },
    {
      label: "Sustained minute-duration spells",
      metadata: {
        and: [
          { field: "sustained", op: "eq", value: true },
          { field: "durationUnit", op: "eq", value: "minute" },
        ],
      },
    },
  ],
  hazard: [
    {
      label: "Complex hazards disabled with Thievery",
      metadata: {
        and: [
          { field: "isComplex", op: "eq", value: true },
          { field: "disableSkills", op: "includesAny", values: ["thievery"] },
        ],
      },
    },
  ],
};

const ADVANCED_PREDICATES: MetadataAdvancedPredicateSemantics[] = [
  {
    name: "actorMetric",
    categories: ACTOR_METRIC_CATEGORIES,
    operators: [...new Set([...ACTOR_METRIC_NUMERIC_OPERATORS, ...ACTOR_METRIC_SCALAR_OPERATORS])],
    description:
      "Generic keyed actor metric predicate for creature and hazard stats, saves, and other actor-shaped metrics.",
    example: {
      field: "actorMetric",
      metric: "ability.int.mod",
      op: ">=",
      value: 4,
    },
  },
  {
    name: "actorMetricCompare",
    categories: ACTOR_METRIC_CATEGORIES,
    operators: [...ACTOR_METRIC_NUMERIC_OPERATORS],
    description: "Numeric actor metric comparison between two metric keys on the same creature or hazard record.",
    example: {
      field: "actorMetricCompare",
      leftMetric: "ability.int.mod",
      op: ">",
      rightMetric: "ability.cha.mod",
    },
  },
  {
    name: "itemMetric",
    categories: EQUIPMENT_ONLY,
    operators: [...new Set([...ACTOR_METRIC_NUMERIC_OPERATORS, ...ACTOR_METRIC_SCALAR_OPERATORS])],
    description: "Generic keyed equipment metric predicate for weapon, armor, and shield stats.",
    example: {
      field: "itemMetric",
      metric: "weapon.reload",
      op: "==",
      value: 1,
    },
  },
  {
    name: "itemMetricCompare",
    categories: EQUIPMENT_ONLY,
    operators: [...ACTOR_METRIC_NUMERIC_OPERATORS],
    description: "Numeric equipment metric comparison between two metric keys on the same item record.",
    example: {
      field: "itemMetricCompare",
      leftMetric: "shield.hp",
      op: ">",
      rightMetric: "shield.bt",
    },
  },
];

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
    field: entry.field,
    fieldType: entry.fieldType,
    operators: [...METADATA_FIELD_KIND_OPERATORS[entry.fieldType]],
    categories: [...entry.categories],
    subcategories: entry.subcategories ? [...entry.subcategories] : undefined,
    discoverable: Boolean(entry.discoverable),
    notes: entry.notes,
    valueOrdering: entry.valueOrdering,
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
    advancedPredicates: ADVANCED_PREDICATES,
    actorMetricDiscovery: {
      filterValueField: "actorMetrics",
      namespaces: ACTOR_METRIC_DISCOVERY_NAMESPACES.map((entry) => ({ ...entry })),
      notes: [
        'Use pf2e_list_filter_values with field:"actorMetrics" to enumerate live metric keys in the current corpus.',
        "Set metricPrefix to narrow discovery to one namespace such as ability., save., skill., ac., hp., hardness., perception., or stealth.",
        "Set metric to a text or boolean metric such as save.best or skill.arcana.proficient to enumerate live values.",
      ],
    },
    itemMetricDiscovery: {
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
