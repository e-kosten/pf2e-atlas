import {
  CATEGORY_SUBCATEGORY_MAP,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import {
  getMetadataFilterSemantics,
  type MetadataFieldSemantics,
} from "../domain/metadata-semantics.js";
import type { MetadataFieldName } from "../domain/metadata-field-registry.js";
import type {
  LookupOptions,
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataSetField,
  NormalizedRecord,
  OntologyNodeQuery,
  SearchCategory,
  SearchFilters,
  SearchMode,
  SearchProfile,
  SearchResult,
  SearchSubcategory,
} from "../types.js";
import type { SearchVocabularyResult } from "../data/vocabulary.js";

export type Pf2eTerminalSearchCategoryOption = {
  value: SearchCategory | null;
  label: string;
  description: string;
};

export type Pf2eTerminalSearchSubcategoryOption = {
  value: SearchSubcategory | null;
  label: string;
  description: string;
};

export type Pf2eTerminalSearchProfileOption = {
  value: SearchProfile;
  label: string;
  description: string;
};

export type Pf2eTerminalSearchModeOption = {
  value: Pf2eTerminalSearchMode;
  label: string;
  description: string;
};

export type Pf2eTerminalFacetFieldOption = {
  value: Pf2eTerminalFacetField;
  label: string;
  description: string;
  fieldType: MetadataFieldSemantics["fieldType"];
};

export type Pf2eTerminalFacetValueOption = {
  value: string;
  label: string;
  description: string;
  count: number;
};

export type Pf2eTerminalFacetSelection = {
  field: Pf2eTerminalFacetField;
  values: string[];
};

export type Pf2eTerminalFacetField = MetadataFieldSemantics["field"];

export type Pf2eTerminalSearchMode = "browse" | "search" | "lookup";

export type Pf2eTerminalSearchFilters = {
  category: SearchCategory | null;
  subcategory: SearchSubcategory | null;
  levelMin: number | null;
  levelMax: number | null;
  rarity: string | null;
  actionCost: number | null;
  facets: Pf2eTerminalFacetSelection[];
};

export type Pf2eTerminalSearchRequest = {
  mode: Pf2eTerminalSearchMode;
  limit: number;
  queryText: string;
  searchProfile: SearchProfile;
  sourceLabel: string | null;
  filters: Pf2eTerminalSearchFilters;
};

export type Pf2eTerminalSearchSession = {
  request: Pf2eTerminalSearchRequest;
  results: NormalizedRecord[];
  resultMode: SearchMode;
  total: number;
  searchProfile: SearchProfile | null;
};

export type Pf2eTerminalSearchService = {
  createDefaultRequest: () => Pf2eTerminalSearchRequest;
  createRequestFromOntologyQuery: (query: OntologyNodeQuery) => Pf2eTerminalSearchRequest;
  getActionCostOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getCategoryOptions: () => Pf2eTerminalSearchCategoryOption[];
  getFacetFieldOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetFieldOption[];
  getFacetValueOptions: (
    field: Pf2eTerminalFacetField,
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getProfileOptions: () => Pf2eTerminalSearchProfileOption[];
  getRarityOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getSubcategoryOptions: (category: SearchCategory | null) => Pf2eTerminalSearchSubcategoryOption[];
  getModeOptions: () => Pf2eTerminalSearchModeOption[];
  normalizeRequest: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest;
  runQuery: (request: Pf2eTerminalSearchRequest) => Promise<Pf2eTerminalSearchSession>;
};

type SearchServiceDependencies = {
  getSearchVocabulary: () => SearchVocabularyResult;
  listFilterValues: (query: {
    field: MetadataFieldName | "actionCost" | "rarity";
    category?: SearchCategory;
    subcategory?: SearchSubcategory;
  }) => { values: Array<{ value: string; count: number }> };
  lookup: (
    name: string,
    options?: LookupOptions,
  ) => { match: NormalizedRecord | null; alternatives: NormalizedRecord[] };
  listRecords: (filters: SearchFilters) => SearchResult;
  search: (filters: SearchFilters) => Promise<SearchResult>;
};

const SEARCH_PROFILE_OPTIONS: Pf2eTerminalSearchProfileOption[] = [
  {
    value: "balanced",
    label: "Balanced",
    description: "Default hybrid retrieval for concise themed searches.",
  },
  {
    value: "lexical",
    label: "Lexical",
    description: "Exact-wording heavy retrieval for names and precise PF2E terms.",
  },
  {
    value: "concept",
    label: "Concept",
    description: "Semantic-forward retrieval for broader exploratory concept searches.",
  },
];

const SEARCH_MODE_OPTIONS: Pf2eTerminalSearchModeOption[] = [
  {
    value: "browse",
    label: "Browse",
    description: "Deterministic listing over structured filters with no ranking required.",
  },
  {
    value: "search",
    label: "Search",
    description: "Ranked lexical or semantic retrieval using the current search profile.",
  },
  {
    value: "lookup",
    label: "Lookup",
    description: "Exact or near-exact name lookup within the current category boundaries.",
  },
];

const FACET_FIELD_EXCLUSIONS = new Set<Pf2eTerminalFacetField>(["rarity", "actionCost"]);

function humanizeIdentifier(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

function formatCategoryLabel(category: SearchCategory): string {
  return humanizeIdentifier(category);
}

function formatSubcategoryLabel(subcategory: SearchSubcategory): string {
  return humanizeIdentifier(subcategory);
}

function formatFilterValueLabel(value: string): string {
  if (value === "true") {
    return "True";
  }
  if (value === "false") {
    return "False";
  }
  return humanizeIdentifier(value);
}

function createDefaultFilters(): Pf2eTerminalSearchFilters {
  return {
    category: null,
    subcategory: null,
    levelMin: null,
    levelMax: null,
    rarity: null,
    actionCost: null,
    facets: [],
  };
}

function createDefaultRequest(): Pf2eTerminalSearchRequest {
  return {
    mode: "browse",
    limit: 20,
    queryText: "",
    searchProfile: "balanced",
    sourceLabel: null,
    filters: createDefaultFilters(),
  };
}

function compareFacetSelections(
  left: Pf2eTerminalFacetSelection,
  right: Pf2eTerminalFacetSelection,
): number {
  return left.field.localeCompare(right.field);
}

function normalizeFacetSelection(
  facet: Pf2eTerminalFacetSelection,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): Pf2eTerminalFacetSelection | null {
  const fieldSemantics = fieldSemanticsByName.get(facet.field);
  if (!fieldSemantics || FACET_FIELD_EXCLUSIONS.has(facet.field)) {
    return null;
  }

  if (!fieldSemantics.discoverable) {
    return null;
  }

  if (!["set", "enumString", "boolean"].includes(fieldSemantics.fieldType)) {
    return null;
  }

  if (category && !fieldSemantics.categories.includes(category)) {
    return null;
  }

  if (subcategory && fieldSemantics.subcategories && !fieldSemantics.subcategories.includes(subcategory)) {
    return null;
  }

  const normalizedValues = [...new Set(facet.values.map((value) => String(value).trim()).filter(Boolean))]
    .sort((left, right) => left.localeCompare(right));
  if (normalizedValues.length === 0) {
    return null;
  }

  if (fieldSemantics.fieldType === "boolean") {
    const booleanValue = normalizedValues.find((value) => value === "true" || value === "false");
    return booleanValue ? { field: facet.field, values: [booleanValue] } : null;
  }

  return {
    field: facet.field,
    values: normalizedValues,
  };
}

function normalizeRequest(
  request: Pf2eTerminalSearchRequest,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchRequest {
  const category = request.filters.category;
  const subcategory = category && request.filters.subcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(request.filters.subcategory)
    ? request.filters.subcategory
    : null;
  const normalizedFacets = request.filters.facets
    .map((facet) => normalizeFacetSelection(facet, fieldSemanticsByName, category, subcategory))
    .filter((facet): facet is Pf2eTerminalFacetSelection => Boolean(facet))
    .sort(compareFacetSelections);

  const levelMin = request.filters.levelMin ?? null;
  const levelMax = request.filters.levelMax ?? null;
  const normalizedLevelMin = levelMin !== null && levelMax !== null ? Math.min(levelMin, levelMax) : levelMin;
  const normalizedLevelMax = levelMin !== null && levelMax !== null ? Math.max(levelMin, levelMax) : levelMax;

  return {
    ...request,
    queryText: request.queryText.trim(),
    sourceLabel: request.sourceLabel?.trim() || null,
    filters: {
      ...request.filters,
      category,
      subcategory,
      levelMin: normalizedLevelMin,
      levelMax: normalizedLevelMax,
      rarity: request.filters.rarity?.trim() || null,
      actionCost: request.filters.actionCost ?? null,
      facets: normalizedFacets,
    },
  };
}

function buildMetadataNodeForFacet(
  facet: Pf2eTerminalFacetSelection,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const fieldSemantics = fieldSemanticsByName.get(facet.field);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    return {
      field: facet.field as MetadataSetField,
      op: "includesAny",
      values: facet.values,
    };
  }

  if (fieldSemantics.fieldType === "enumString") {
    if (facet.values.length === 1) {
      return {
        field: facet.field as MetadataEnumStringField,
        op: "eq",
        value: facet.values[0]!,
      };
    }
    return {
      field: facet.field as MetadataEnumStringField,
      op: "in",
      values: facet.values,
    };
  }

  if (fieldSemantics.fieldType === "boolean") {
    return {
      field: facet.field as MetadataBooleanField,
      op: "eq",
      value: facet.values[0] === "true",
    };
  }

  return null;
}

function buildMetadataFilter(
  facets: Pf2eTerminalFacetSelection[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | undefined {
  const clauses = facets
    .map((facet) => buildMetadataNodeForFacet(facet, fieldSemanticsByName))
    .filter((node): node is MetadataFilterNode => Boolean(node));
  if (clauses.length === 0) {
    return undefined;
  }
  if (clauses.length === 1) {
    return clauses[0];
  }
  return { and: clauses };
}

function buildSearchFilters(
  request: Pf2eTerminalSearchRequest,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): SearchFilters {
  const metadata = buildMetadataFilter(request.filters.facets, fieldSemanticsByName);
  return {
    category: request.filters.category ?? undefined,
    subcategory: request.filters.subcategory ?? undefined,
    levelMin: request.filters.levelMin ?? undefined,
    levelMax: request.filters.levelMax ?? undefined,
    rarity: request.filters.rarity ?? undefined,
    actionCost: request.filters.actionCost ?? undefined,
    metadata,
    limit: request.limit,
  };
}

function createFacetSelectionsFromMetadata(
  metadata: MetadataFilterNode | undefined,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalFacetSelection[] {
  if (!metadata) {
    return [];
  }

  const facets = new Map<Pf2eTerminalFacetField, Set<string>>();

  const collect = (node: MetadataFilterNode): boolean => {
    if ("and" in node) {
      return node.and.every((child) => collect(child));
    }
    if ("or" in node || "not" in node) {
      return false;
    }

    const field = node.field as Pf2eTerminalFacetField;
    const fieldSemantics = fieldSemanticsByName.get(field);
    if (!fieldSemantics) {
      return false;
    }

    if (fieldSemantics.fieldType === "set" && node.op === "includesAny") {
      const bucket = facets.get(field) ?? new Set<string>();
      for (const value of node.values) {
        bucket.add(String(value));
      }
      facets.set(field, bucket);
      return true;
    }

    if (fieldSemantics.fieldType === "enumString") {
      const bucket = facets.get(field) ?? new Set<string>();
      if (node.op === "eq") {
        bucket.add(String(node.value));
        facets.set(field, bucket);
        return true;
      }
      if (node.op === "in") {
        for (const value of node.values) {
          bucket.add(String(value));
        }
        facets.set(field, bucket);
        return true;
      }
    }

    if (fieldSemantics.fieldType === "boolean" && node.op === "eq") {
      facets.set(field, new Set([String(node.value)]));
      return true;
    }

    return false;
  };

  if (!collect(metadata)) {
    return [];
  }

  return [...facets.entries()]
    .map(([field, values]) => ({
      field,
      values: [...values].sort((left, right) => left.localeCompare(right)),
    }))
    .sort(compareFacetSelections);
}

export function createPf2eTerminalSearchService(
  dependencies: SearchServiceDependencies,
): Pf2eTerminalSearchService {
  const filterSemantics = getMetadataFilterSemantics();
  const fieldSemanticsByName = new Map<Pf2eTerminalFacetField, MetadataFieldSemantics>(
    filterSemantics.metadataFields.map((entry) => [entry.field, entry]),
  );

  return {
    createDefaultRequest: () => createDefaultRequest(),
    createRequestFromOntologyQuery: (query) => {
      const request = createDefaultRequest();
      const facets = createFacetSelectionsFromMetadata(query.filters.metadata, fieldSemanticsByName);
      return normalizeRequest({
        ...request,
        mode: query.kind === "lookup" ? "lookup" : query.kind === "search" ? "search" : "browse",
        limit: query.filters.limit ?? request.limit,
        queryText: query.filters.query ?? query.filters.nameQuery ?? "",
        searchProfile: query.filters.searchProfile ?? request.searchProfile,
        sourceLabel: query.label ?? null,
        filters: {
          ...request.filters,
          category: normalizeSearchCategory(query.filters.category) ?? null,
          subcategory: normalizeSearchSubcategory(query.filters.subcategory) ?? null,
          levelMin: query.filters.levelMin ?? null,
          levelMax: query.filters.levelMax ?? null,
          rarity: query.filters.rarity ?? null,
          actionCost: query.filters.actionCost ?? null,
          facets,
        },
      }, fieldSemanticsByName);
    },
    getActionCostOptions: (category, subcategory) =>
      dependencies.listFilterValues({
        field: "actionCost",
        ...(category ? { category } : {}),
        ...(subcategory ? { subcategory } : {}),
      }).values.map((entry) => ({
        value: entry.value,
        label: `${entry.value} action${entry.value === "1" ? "" : "s"}`,
        description: `${entry.count} live canonical record${entry.count === 1 ? "" : "s"}.`,
        count: entry.count,
      })),
    getCategoryOptions: () => {
      const vocabulary = dependencies.getSearchVocabulary();
      return [
        {
          value: null,
          label: "Any Category",
          description: "Search or browse across the full indexed PF2E corpus.",
        },
        ...vocabulary.categories.map((category: SearchVocabularyResult["categories"][number]) => ({
          value: category.value,
          label: formatCategoryLabel(category.value),
          description: `${category.count} indexed canonical record${category.count === 1 ? "" : "s"}.`,
        })),
      ];
    },
    getFacetFieldOptions: (category, subcategory) => {
      const categoryFields = category ? filterSemantics.metadataFieldsByCategory[category] ?? [] : [];
      const scopedFields = subcategory && category
        ? filterSemantics.metadataFieldsByCategoryAndSubcategory[category]?.[subcategory] ?? []
        : [];
      const candidateFields = (subcategory && scopedFields.length > 0 ? scopedFields : categoryFields)
        .map((field) => fieldSemanticsByName.get(field))
        .filter((field): field is MetadataFieldSemantics => Boolean(field))
        .filter((field) =>
          field.discoverable &&
          !FACET_FIELD_EXCLUSIONS.has(field.field) &&
          ["set", "enumString", "boolean"].includes(field.fieldType),
        );

      return candidateFields.map((field) => ({
        value: field.field,
        label: humanizeIdentifier(field.field),
        description: field.notes ?? `${field.fieldType} facet for the current browse scope.`,
        fieldType: field.fieldType,
      }));
    },
    getFacetValueOptions: (field, category, subcategory) =>
      dependencies.listFilterValues({
        field,
        ...(category ? { category } : {}),
        ...(subcategory ? { subcategory } : {}),
      }).values.map((entry) => ({
        value: entry.value,
        label: formatFilterValueLabel(entry.value),
        description: `${entry.count} live canonical record${entry.count === 1 ? "" : "s"}.`,
        count: entry.count,
      })),
    getProfileOptions: () => SEARCH_PROFILE_OPTIONS,
    getRarityOptions: (category, subcategory) =>
      dependencies.listFilterValues({
        field: "rarity",
        ...(category ? { category } : {}),
        ...(subcategory ? { subcategory } : {}),
      }).values.map((entry) => ({
        value: entry.value,
        label: formatFilterValueLabel(entry.value),
        description: `${entry.count} live canonical record${entry.count === 1 ? "" : "s"}.`,
        count: entry.count,
      })),
    getSubcategoryOptions: (category) => {
      if (!category) {
        return [{
          value: null,
          label: "Any Subcategory",
          description: "Choose a category first to narrow the browse scope further.",
        }];
      }

      return [
        {
          value: null,
          label: "Any Subcategory",
          description: `Browse every ${formatCategoryLabel(category)} record in the current category.`,
        },
        ...CATEGORY_SUBCATEGORY_MAP[category].map((subcategory) => ({
          value: subcategory,
          label: formatSubcategoryLabel(subcategory),
          description: `Restrict the workspace to ${formatSubcategoryLabel(subcategory)} records.`,
        })),
      ];
    },
    getModeOptions: () => SEARCH_MODE_OPTIONS,
    normalizeRequest: (request) => normalizeRequest(request, fieldSemanticsByName),
    runQuery: async (request) => {
      const normalizedRequest = normalizeRequest(request, fieldSemanticsByName);
      if (normalizedRequest.mode === "lookup") {
        const lookupOptions: LookupOptions | undefined = normalizedRequest.filters.category || normalizedRequest.filters.subcategory
          ? {
            ...(normalizedRequest.filters.category ? { category: normalizedRequest.filters.category } : {}),
            ...(normalizedRequest.filters.subcategory ? { subcategory: normalizedRequest.filters.subcategory } : {}),
          }
          : undefined;
        const lookup = dependencies.lookup(normalizedRequest.queryText, lookupOptions);
        return {
          request: normalizedRequest,
          results: lookup.match ? [lookup.match, ...lookup.alternatives] : [],
          resultMode: "lexical",
          total: lookup.match ? 1 + lookup.alternatives.length : 0,
          searchProfile: null,
        };
      }

      const filters = buildSearchFilters(normalizedRequest, fieldSemanticsByName);
      const result = normalizedRequest.mode === "browse"
        ? dependencies.listRecords(filters)
        : await dependencies.search({
          ...filters,
          query: normalizedRequest.queryText,
          searchProfile: normalizedRequest.searchProfile,
        });
      return {
        request: normalizedRequest,
        results: result.records,
        resultMode: result.mode,
        total: result.total,
        searchProfile: result.searchProfile,
      };
    },
  };
}
