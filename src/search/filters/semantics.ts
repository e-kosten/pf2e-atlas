import {
  ACTOR_METRIC_DISCOVERY_NAMESPACES,
} from "../../domain/actor-metrics.js";
import { ITEM_METRIC_DISCOVERY_NAMESPACES } from "../../domain/item-metrics.js";
import { SEARCH_CATEGORIES } from "../../domain/categories.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import { METADATA_FIELD_KIND_OPERATORS, type MetadataFieldName, type MetadataFieldType } from "../../domain/metadata-field-types.js";
import type { SearchFilterNode } from "../../domain/search-request-types.js";
import {
  getSearchPromotedFieldValueOrdering,
  isSearchPromotedFieldDomainKey,
} from "../../domain/search-field-domains.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import { METADATA_FIELD_REGISTRY } from "./registry.js";

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
const EQUIPMENT_ONLY: SearchCategory[] = ["equipment"];

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
  return METADATA_FIELD_REGISTRY.map((entry) => ({
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
      categories: EQUIPMENT_ONLY,
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
