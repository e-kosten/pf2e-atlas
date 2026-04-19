import { CATEGORY_SUBCATEGORY_MAP, normalizeSearchCategory, normalizeSearchSubcategory } from "../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../domain/metadata-semantics.js";
import type { MetadataFieldName } from "../domain/metadata-field-registry.js";
import { orderFilterValues, type FilterValueOrdering } from "../domain/filter-value-ordering.js";
import type {
  LookupOptions,
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataSetField,
  NormalizedRecord,
  OntologyNodeQuery,
  SearchCategory,
  SearchCountResult,
  SearchFilters,
  SearchMode,
  SearchProfile,
  SearchResult,
  SearchSort,
  SearchSubcategory,
  SearchWindowPage,
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

export type Pf2eTerminalSearchSortOption = {
  value: Pf2eTerminalSearchSort;
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

export type Pf2eTerminalFilterValuePolicy<T extends number | string = string> = {
  any: T[];
  all: T[];
  exclude: T[];
};

export type Pf2eTerminalFacetSelection = {
  field: Pf2eTerminalFacetField;
  policy: Pf2eTerminalFilterValuePolicy<string>;
};

export type Pf2eTerminalFacetField = MetadataFieldSemantics["field"];
export type Pf2eTerminalQueryField = MetadataFieldSemantics["field"];

export type Pf2eTerminalSearchMode = "browse" | "search" | "lookup";
export type Pf2eTerminalSearchSort = SearchSort;

export type Pf2eTerminalSearchFilters = {
  category: SearchCategory | null;
  subcategory: SearchSubcategory | null;
  levelMin: number | null;
  levelMax: number | null;
  rarity: Pf2eTerminalFilterValuePolicy<string>;
  actionCost: Pf2eTerminalFilterValuePolicy<number>;
  facets: Pf2eTerminalFacetSelection[];
  metadata: MetadataFilterNode | null;
};

export type Pf2eTerminalSearchQuery = {
  mode: Pf2eTerminalSearchMode;
  limit: number;
  queryText: string;
  searchProfile: SearchProfile;
  sourceLabel: string | null;
  filters: Pf2eTerminalSearchFilters;
};

export type Pf2eTerminalQueryFieldOption = {
  value: Pf2eTerminalQueryField;
  label: string;
  description: string;
  fieldType: MetadataFieldSemantics["fieldType"];
};

export type Pf2eTerminalSearchSession = {
  windowId: string;
  query: Pf2eTerminalSearchQuery;
  results: NormalizedRecord[];
  windowOffset: number;
  resultMode: SearchMode;
  total: number;
  loadedCount: number;
  hasMore: boolean;
  nextOffset: number | null;
  searchProfile: SearchProfile | null;
  sort: Pf2eTerminalSearchSort;
  sortSeed: number | null;
};

export type Pf2eTerminalSearchService = {
  createDefaultQuery: () => Pf2eTerminalSearchQuery;
  createQueryFromOntologyQuery: (query: OntologyNodeQuery) => Pf2eTerminalSearchQuery;
  countQuery: (query: Pf2eTerminalSearchQuery) => Promise<SearchCountResult>;
  disposeSession: (session: Pf2eTerminalSearchSession) => void;
  executeQuery: (
    query: Pf2eTerminalSearchQuery,
    options?: { sort?: Pf2eTerminalSearchSort; limit?: number },
  ) => Promise<Pf2eTerminalSearchSession>;
  getActionCostOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getCategoryOptions: () => Pf2eTerminalSearchCategoryOption[];
  getFacetFieldOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetFieldOption[];
  getQueryFieldOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalQueryFieldOption[];
  getFacetValueOptions: (
    field: Pf2eTerminalFacetField,
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getProfileOptions: () => Pf2eTerminalSearchProfileOption[];
  getResultSortOptions: (mode: Pf2eTerminalSearchMode) => Pf2eTerminalSearchSortOption[];
  getRarityOptions: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalFacetValueOption[];
  getSubcategoryOptions: (category: SearchCategory | null) => Pf2eTerminalSearchSubcategoryOption[];
  getModeOptions: () => Pf2eTerminalSearchModeOption[];
  getDefaultSort: (mode: Pf2eTerminalSearchMode) => Pf2eTerminalSearchSort;
  loadMore: (
    session: Pf2eTerminalSearchSession,
    options?: { minimumLoadedCount?: number },
  ) => Promise<Pf2eTerminalSearchSession>;
  readResultWindow: (
    session: Pf2eTerminalSearchSession,
    options: { offset: number; limit: number },
  ) => Promise<Pf2eTerminalSearchSession>;
  normalizeQuery: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery;
  changeSort: (session: Pf2eTerminalSearchSession, sort: Pf2eTerminalSearchSort) => Promise<Pf2eTerminalSearchSession>;
};

type SearchServiceDependencies = {
  closeSearchWindow: (windowId: string) => void;
  countRecords: (
    filters: SearchFilters,
    options?: { mode?: "browse" | "search" | "lookup"; lexicalOnly?: boolean },
  ) => Promise<SearchCountResult>;
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
  openSearchWindow: (
    filters: SearchFilters,
    options?: { mode?: "browse" | "search" | "lookup" },
  ) => Promise<SearchWindowPage>;
  readSearchWindowPage: (windowId: string, offset: number, limit: number) => SearchWindowPage;
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

const SEARCH_SORT_OPTIONS: Record<Pf2eTerminalSearchMode, Pf2eTerminalSearchSortOption[]> = {
  browse: [
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read deterministic browse results in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read results from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read results from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle browse results into a stable random session order.",
    },
  ],
  search: [
    {
      value: "ranked",
      label: "Ranked",
      description: "Keep the current search profile's relevance order.",
    },
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read matched results in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read matched results from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read matched results from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle matched results into a stable random session order.",
    },
  ],
  lookup: [
    {
      value: "ranked",
      label: "Closest Match",
      description: "Keep the best name-match ordering for lookup results.",
    },
    {
      value: "alphabetical",
      label: "Alphabetical",
      description: "Read lookup matches in name order.",
    },
    {
      value: "levelAsc",
      label: "Level Low-High",
      description: "Read lookup matches from lowest level to highest level.",
    },
    {
      value: "levelDesc",
      label: "Level High-Low",
      description: "Read lookup matches from highest level to lowest level.",
    },
    {
      value: "random",
      label: "Random",
      description: "Shuffle lookup matches into a stable random session order.",
    },
  ],
};

const FACET_FIELD_EXCLUSIONS = new Set<Pf2eTerminalFacetField>(["rarity"]);

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

function orderStringValues(values: readonly string[], ordering?: FilterValueOrdering): string[] {
  return orderFilterValues(
    values.map((value) => ({ value, count: 0 })),
    ordering,
  ).map((entry) => entry.value);
}

function createFacetValueOptions(
  values: ReadonlyArray<{ value: string; count: number }>,
  options: {
    ordering?: FilterValueOrdering;
    labelFormatter?: (value: string) => string;
  } = {},
): Pf2eTerminalFacetValueOption[] {
  const labelFormatter = options.labelFormatter ?? formatFilterValueLabel;
  return orderFilterValues(values, options.ordering).map((entry) => ({
    value: entry.value,
    label: labelFormatter(entry.value),
    description: `${entry.count} live canonical record${entry.count === 1 ? "" : "s"}.`,
    count: entry.count,
  }));
}

function createEmptyFilterPolicy<T extends number | string>(): Pf2eTerminalFilterValuePolicy<T> {
  return {
    any: [],
    all: [],
    exclude: [],
  };
}

function createDefaultFilters(): Pf2eTerminalSearchFilters {
  return {
    category: null,
    subcategory: null,
    levelMin: null,
    levelMax: null,
    rarity: createEmptyFilterPolicy<string>(),
    actionCost: createEmptyFilterPolicy<number>(),
    facets: [],
    metadata: null,
  };
}

function createDefaultQuery(): Pf2eTerminalSearchQuery {
  return {
    mode: "browse",
    limit: 50,
    queryText: "",
    searchProfile: "balanced",
    sourceLabel: null,
    filters: createDefaultFilters(),
  };
}

function getDefaultSort(mode: Pf2eTerminalSearchMode): Pf2eTerminalSearchSort {
  return mode === "browse" ? "alphabetical" : "ranked";
}

function createSortSeed(sort: Pf2eTerminalSearchSort): number | null {
  if (sort !== "random") {
    return null;
  }

  return Math.trunc(Date.now() % 2147483647);
}

function compareFacetSelections(left: Pf2eTerminalFacetSelection, right: Pf2eTerminalFacetSelection): number {
  return left.field.localeCompare(right.field);
}

function normalizeStringPolicy(
  policy: Partial<Pf2eTerminalFilterValuePolicy<string>> | undefined,
  ordering?: FilterValueOrdering,
): Pf2eTerminalFilterValuePolicy<string> {
  const exclude = [...new Set((policy?.exclude ?? []).map((value) => String(value).trim()).filter(Boolean))];
  const all = [...new Set((policy?.all ?? []).map((value) => String(value).trim()).filter(Boolean))].filter(
    (value) => !exclude.includes(value),
  );
  const any = [...new Set((policy?.any ?? []).map((value) => String(value).trim()).filter(Boolean))].filter(
    (value) => !exclude.includes(value) && !all.includes(value),
  );

  return {
    any: orderStringValues(any, ordering),
    all: orderStringValues(all, ordering),
    exclude: orderStringValues(exclude, ordering),
  };
}

function normalizeNumberPolicy(
  policy: Partial<Pf2eTerminalFilterValuePolicy<number>> | undefined,
): Pf2eTerminalFilterValuePolicy<number> {
  const exclude = [...new Set((policy?.exclude ?? []).filter((value) => Number.isFinite(value)))].sort(
    (left, right) => left - right,
  );
  const all = [...new Set((policy?.all ?? []).filter((value) => Number.isFinite(value)))]
    .filter((value) => !exclude.includes(value))
    .sort((left, right) => left - right);
  const any = [...new Set((policy?.any ?? []).filter((value) => Number.isFinite(value)))]
    .filter((value) => !exclude.includes(value) && !all.includes(value))
    .sort((left, right) => left - right);

  return { any, all, exclude };
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

  const normalizedPolicy = normalizeStringPolicy(facet.policy, fieldSemantics.valueOrdering);
  if (fieldSemantics.fieldType !== "set") {
    normalizedPolicy.all = [];
  }
  if (fieldSemantics.fieldType === "boolean") {
    normalizedPolicy.any = normalizedPolicy.any.filter((value) => value === "true" || value === "false");
    normalizedPolicy.exclude = normalizedPolicy.exclude.filter((value) => value === "true" || value === "false");
  }

  if (normalizedPolicy.any.length === 0 && normalizedPolicy.all.length === 0 && normalizedPolicy.exclude.length === 0) {
    return null;
  }

  return {
    field: facet.field,
    policy: normalizedPolicy,
  };
}

function normalizeSearchQuery(
  query: Pf2eTerminalSearchQuery,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  const category = query.filters.category;
  const subcategory =
    category && query.filters.subcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(query.filters.subcategory)
      ? query.filters.subcategory
      : null;
  const normalizedFacets = query.filters.facets
    .map((facet) => normalizeFacetSelection(facet, fieldSemanticsByName, category, subcategory))
    .filter((facet): facet is Pf2eTerminalFacetSelection => Boolean(facet))
    .sort(compareFacetSelections);

  const levelMin = query.filters.levelMin ?? null;
  const levelMax = query.filters.levelMax ?? null;
  const normalizedLevelMin = levelMin !== null && levelMax !== null ? Math.min(levelMin, levelMax) : levelMin;
  const normalizedLevelMax = levelMin !== null && levelMax !== null ? Math.max(levelMin, levelMax) : levelMax;

  return {
    ...query,
    queryText: query.queryText.trim(),
    sourceLabel: query.sourceLabel?.trim() || null,
    filters: {
      ...query.filters,
      category,
      subcategory,
      levelMin: normalizedLevelMin,
      levelMax: normalizedLevelMax,
      rarity: (() => {
        const policy = normalizeStringPolicy(query.filters.rarity, fieldSemanticsByName.get("rarity")?.valueOrdering);
        return {
          any: policy.any,
          all: [],
          exclude: policy.exclude,
        };
      })(),
      actionCost: (() => {
        const policy = normalizeNumberPolicy(query.filters.actionCost);
        return {
          any: policy.any,
          all: [],
          exclude: policy.exclude,
        };
      })(),
      facets: normalizedFacets,
      metadata: query.filters.metadata ?? null,
    },
  };
}

function buildDiscreteFilterNodes(query: Pf2eTerminalSearchQuery): MetadataFilterNode[] {
  const nodes: MetadataFilterNode[] = [];

  if (query.filters.rarity.any.length === 1) {
    nodes.push({
      field: "rarity",
      op: "eq",
      value: query.filters.rarity.any[0]!,
    });
  } else if (query.filters.rarity.any.length > 1) {
    nodes.push({
      field: "rarity",
      op: "in",
      values: query.filters.rarity.any,
    });
  }

  if (query.filters.rarity.exclude.length > 0) {
    nodes.push({
      field: "rarity",
      op: "notIn",
      values: query.filters.rarity.exclude,
    });
  }

  if (query.filters.actionCost.any.length === 1) {
    nodes.push({
      field: "actionCost",
      op: "eq",
      value: query.filters.actionCost.any[0]!,
    });
  } else if (query.filters.actionCost.any.length > 1) {
    nodes.push({
      or: query.filters.actionCost.any.map((value) => ({
        field: "actionCost",
        op: "eq",
        value,
      })),
    });
  }

  if (query.filters.actionCost.exclude.length === 1) {
    nodes.push({
      not: {
        field: "actionCost",
        op: "eq",
        value: query.filters.actionCost.exclude[0]!,
      },
    });
  } else if (query.filters.actionCost.exclude.length > 1) {
    nodes.push({
      not: {
        or: query.filters.actionCost.exclude.map((value) => ({
          field: "actionCost",
          op: "eq",
          value,
        })),
      },
    });
  }

  return nodes;
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
    const clauses: MetadataFilterNode[] = [];
    if (facet.policy.any.length > 0) {
      clauses.push({
        field: facet.field as MetadataSetField,
        op: "includesAny",
        values: facet.policy.any,
      });
    }
    if (facet.policy.all.length > 0) {
      clauses.push({
        field: facet.field as MetadataSetField,
        op: "includesAll",
        values: facet.policy.all,
      });
    }
    if (facet.policy.exclude.length > 0) {
      clauses.push({
        field: facet.field as MetadataSetField,
        op: "excludesAny",
        values: facet.policy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: MetadataFilterNode[] = [];
    if (facet.policy.any.length === 1) {
      clauses.push({
        field: facet.field as MetadataEnumStringField,
        op: "eq",
        value: facet.policy.any[0]!,
      });
    } else if (facet.policy.any.length > 1) {
      clauses.push({
        field: facet.field as MetadataEnumStringField,
        op: "in",
        values: facet.policy.any,
      });
    }
    if (facet.policy.exclude.length > 0) {
      clauses.push({
        field: facet.field as MetadataEnumStringField,
        op: "notIn",
        values: facet.policy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: MetadataFilterNode[] = [];
    for (const value of facet.policy.any) {
      clauses.push({
        field: facet.field as MetadataBooleanField,
        op: "eq",
        value: value === "true",
      });
    }
    for (const value of facet.policy.exclude) {
      clauses.push({
        not: {
          field: facet.field as MetadataBooleanField,
          op: "eq",
          value: value === "true",
        },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  return null;
}

function buildSearchFilters(
  query: Pf2eTerminalSearchQuery,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: {
    limit?: number;
    offset?: number;
    query?: string;
    nameQuery?: string;
    searchProfile?: SearchProfile;
    sort?: SearchSort;
    sortSeed?: number | null;
  } = {},
): SearchFilters {
  const metadataClauses = [
    ...buildDiscreteFilterNodes(query),
    ...query.filters.facets
      .map((facet) => buildMetadataNodeForFacet(facet, fieldSemanticsByName))
      .filter((node): node is MetadataFilterNode => Boolean(node)),
    ...(query.filters.metadata ? [query.filters.metadata] : []),
  ];
  const metadata =
    metadataClauses.length === 0
      ? undefined
      : metadataClauses.length === 1
        ? metadataClauses[0]
        : { and: metadataClauses };
  return {
    category: query.filters.category ?? undefined,
    subcategory: query.filters.subcategory ?? undefined,
    levelMin: query.filters.levelMin ?? undefined,
    levelMax: query.filters.levelMax ?? undefined,
    rarity: undefined,
    actionCost: undefined,
    metadata,
    limit: options.limit ?? query.limit,
    offset: options.offset ?? 0,
    query: options.query,
    nameQuery: options.nameQuery,
    searchProfile: options.searchProfile,
    sort: options.sort,
    sortSeed: options.sortSeed ?? undefined,
  };
}

function getScopedMetadataFields(
  filterSemantics: ReturnType<typeof getMetadataFilterSemantics>,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): MetadataFieldSemantics["field"][] {
  if (!category) {
    return [];
  }

  const categoryFields = filterSemantics.metadataFieldsByCategory[category];
  const scopedFields = subcategory
    ? (filterSemantics.metadataFieldsByCategoryAndSubcategory[category]?.[subcategory] ?? [])
    : [];

  return [...new Set([...categoryFields, ...scopedFields])];
}

export function createPf2eTerminalSearchService(dependencies: SearchServiceDependencies): Pf2eTerminalSearchService {
  const filterSemantics = getMetadataFilterSemantics();
  const fieldSemanticsByName = new Map<Pf2eTerminalFacetField, MetadataFieldSemantics>(
    filterSemantics.metadataFields.map((entry) => [entry.field, entry]),
  );

  function getFieldValueOrdering(field: MetadataFieldName): FilterValueOrdering | undefined {
    return fieldSemanticsByName.get(field)?.valueOrdering;
  }

  function buildWindowFilters(
    query: Pf2eTerminalSearchQuery,
    options: {
      sort: Pf2eTerminalSearchSort;
      sortSeed: number | null;
      limit: number;
      offset?: number;
    },
  ): SearchFilters {
    const offset = options.offset ?? 0;
    if (query.mode === "lookup") {
      return buildSearchFilters(query, fieldSemanticsByName, {
        limit: options.limit,
        offset,
        nameQuery: query.queryText,
        sort: options.sort,
        sortSeed: options.sortSeed,
      });
    }

    return buildSearchFilters(query, fieldSemanticsByName, {
      limit: options.limit,
      offset,
      query: query.mode === "search" ? query.queryText : undefined,
      searchProfile: query.mode === "search" ? query.searchProfile : undefined,
      sort: options.sort,
      sortSeed: options.sortSeed,
    });
  }

  function createSessionFromResult(
    query: Pf2eTerminalSearchQuery,
    result: SearchWindowPage,
  ): Pf2eTerminalSearchSession {
    const sessionQuery =
      result.limit === query.limit
        ? query
        : {
            ...query,
            limit: result.limit,
          };

    return {
      windowId: result.id,
      query: sessionQuery,
      results: result.records,
      windowOffset: result.offset,
      resultMode: result.mode,
      total: result.total,
      loadedCount: result.records.length,
      hasMore: result.hasMore,
      nextOffset: result.nextOffset,
      searchProfile: result.searchProfile,
      sort: result.sort,
      sortSeed: result.sortSeed,
    };
  }

  function createQueryFromOntologyQuery(query: OntologyNodeQuery): Pf2eTerminalSearchQuery {
    const defaultQuery = createDefaultQuery();
    return normalizeSearchQuery(
      {
        ...defaultQuery,
        mode: query.kind === "lookup" ? "lookup" : query.kind === "search" ? "search" : "browse",
        limit: query.filters.limit ?? defaultQuery.limit,
        queryText: query.filters.query ?? query.filters.nameQuery ?? "",
        searchProfile: query.filters.searchProfile ?? defaultQuery.searchProfile,
        sourceLabel: query.label ?? null,
        filters: {
          ...defaultQuery.filters,
          category: normalizeSearchCategory(query.filters.category) ?? null,
          subcategory: normalizeSearchSubcategory(query.filters.subcategory) ?? null,
          levelMin: query.filters.levelMin ?? null,
          levelMax: query.filters.levelMax ?? null,
          rarity: {
            any: query.filters.rarity ? [query.filters.rarity] : [],
            all: [],
            exclude: [],
          },
          actionCost: {
            any: query.filters.actionCost === undefined ? [] : [query.filters.actionCost],
            all: [],
            exclude: [],
          },
          facets: [],
          metadata: query.filters.metadata ?? null,
        },
      },
      fieldSemanticsByName,
    );
  }

  return {
    createDefaultQuery: () => createDefaultQuery(),
    createQueryFromOntologyQuery,
    getActionCostOptions: (category, subcategory) =>
      createFacetValueOptions(
        dependencies.listFilterValues({
          field: "actionCost",
          ...(category ? { category } : {}),
          ...(subcategory ? { subcategory } : {}),
        }).values,
        {
          ordering: getFieldValueOrdering("actionCost"),
          labelFormatter: (value) => `${value} action${value === "1" ? "" : "s"}`,
        },
      ),
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
      const candidateFields = getScopedMetadataFields(filterSemantics, category, subcategory)
        .map((field) => fieldSemanticsByName.get(field))
        .filter((field): field is MetadataFieldSemantics => Boolean(field))
        .filter(
          (field) =>
            field.discoverable &&
            !FACET_FIELD_EXCLUSIONS.has(field.field) &&
            (["set", "enumString", "boolean"].includes(field.fieldType) ||
              (field.field === "actionCost" &&
                dependencies.listFilterValues({
                  field: "actionCost",
                  ...(category ? { category } : {}),
                  ...(subcategory ? { subcategory } : {}),
                }).values.length > 0)),
        );

      return candidateFields.map((field) => ({
        value: field.field,
        label: humanizeIdentifier(field.field),
        description: field.notes ?? `${field.fieldType} facet for the current browse scope.`,
        fieldType: field.fieldType,
      }));
    },
    getQueryFieldOptions: (category, subcategory) => {
      return getScopedMetadataFields(filterSemantics, category, subcategory)
        .map((field) => fieldSemanticsByName.get(field))
        .filter((field): field is MetadataFieldSemantics => Boolean(field))
        .filter((field) => field.discoverable && !["rarity"].includes(field.field))
        .map((field) => ({
          value: field.field,
          label: humanizeIdentifier(field.field),
          description: field.notes ?? `${field.fieldType} query field for the current browse scope.`,
          fieldType: field.fieldType,
        }));
    },
    getFacetValueOptions: (field, category, subcategory) =>
      createFacetValueOptions(
        dependencies.listFilterValues({
          field,
          ...(category ? { category } : {}),
          ...(subcategory ? { subcategory } : {}),
        }).values,
        {
          ordering: getFieldValueOrdering(field),
        },
      ),
    getProfileOptions: () => SEARCH_PROFILE_OPTIONS,
    getResultSortOptions: (mode) => SEARCH_SORT_OPTIONS[mode],
    getRarityOptions: (category, subcategory) =>
      createFacetValueOptions(
        dependencies.listFilterValues({
          field: "rarity",
          ...(category ? { category } : {}),
          ...(subcategory ? { subcategory } : {}),
        }).values,
        {
          ordering: getFieldValueOrdering("rarity"),
        },
      ),
    getSubcategoryOptions: (category) => {
      if (!category) {
        return [
          {
            value: null,
            label: "Any Subcategory",
            description: "Choose a category first to narrow the browse scope further.",
          },
        ];
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
    getDefaultSort: (mode) => getDefaultSort(mode),
    normalizeQuery: (query) => normalizeSearchQuery(query, fieldSemanticsByName),
    countQuery: (query) => {
      const normalizedQuery = normalizeSearchQuery(query, fieldSemanticsByName);
      if (normalizedQuery.mode === "lookup") {
        if (!normalizedQuery.queryText) {
          return Promise.resolve({
            searchProfile: null,
            mode: "structured",
            total: 0,
          });
        }
        return dependencies.countRecords(
          buildSearchFilters(normalizedQuery, fieldSemanticsByName, {
            limit: 1,
            nameQuery: normalizedQuery.queryText,
          }),
          { mode: "lookup" },
        );
      }

      if (normalizedQuery.mode === "browse" || !normalizedQuery.queryText) {
        return dependencies.countRecords(buildSearchFilters(normalizedQuery, fieldSemanticsByName, { limit: 1 }), {
          mode: "browse",
        });
      }

      return dependencies.countRecords(
        buildSearchFilters(normalizedQuery, fieldSemanticsByName, {
          limit: 1,
          query: normalizedQuery.queryText,
          searchProfile: normalizedQuery.searchProfile,
        }),
        { mode: "search", lexicalOnly: true },
      );
    },
    disposeSession: (session) => {
      dependencies.closeSearchWindow(session.windowId);
    },
    executeQuery: async (query, options = {}) => {
      const normalizedQuery = normalizeSearchQuery(query, fieldSemanticsByName);
      const sort = options.sort ?? getDefaultSort(normalizedQuery.mode);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const limit = options.limit ?? normalizedQuery.limit;
      const result = await dependencies.openSearchWindow(
        buildWindowFilters(normalizedQuery, {
          sort,
          sortSeed,
          limit,
        }),
        { mode: normalizedQuery.mode },
      );
      return createSessionFromResult(normalizedQuery, result);
    },
    loadMore: (session, options = {}) => {
      if (!session.hasMore || session.nextOffset === null) {
        return Promise.resolve(session);
      }

      const minimumLoadedCount = Math.max(
        session.loadedCount + 1,
        options.minimumLoadedCount ?? session.loadedCount + session.query.limit,
      );
      let nextSession = session;

      while (nextSession.hasMore && nextSession.nextOffset !== null && nextSession.loadedCount < minimumLoadedCount) {
        const result = dependencies.readSearchWindowPage(
          nextSession.windowId,
          nextSession.nextOffset,
          nextSession.query.limit,
        );

        nextSession = {
          ...nextSession,
          query:
            result.limit === nextSession.query.limit
              ? nextSession.query
              : {
                  ...nextSession.query,
                  limit: result.limit,
                },
          results: [...nextSession.results, ...result.records],
          total: result.total,
          loadedCount: nextSession.results.length + result.records.length,
          hasMore: result.hasMore,
          nextOffset: result.nextOffset,
          resultMode: result.mode,
          searchProfile: result.searchProfile,
        };
      }

      return Promise.resolve(nextSession);
    },
    readResultWindow: (session, options) => {
      const limit = Math.max(1, options.limit);
      const clampedOffset = Math.max(0, Math.min(options.offset, Math.max(0, session.total - limit)));
      const result = dependencies.readSearchWindowPage(session.windowId, clampedOffset, limit);

      return Promise.resolve({
        ...session,
        query:
          result.limit === session.query.limit
            ? session.query
            : {
                ...session.query,
                limit: result.limit,
              },
        results: result.records,
        windowOffset: result.offset,
        total: result.total,
        loadedCount: result.records.length,
        hasMore: result.hasMore,
        nextOffset: result.nextOffset,
        resultMode: result.mode,
        searchProfile: result.searchProfile,
      });
    },
    changeSort: async (session, sort) => {
      dependencies.closeSearchWindow(session.windowId);
      const sortSeed = sort === "random" ? createSortSeed(sort) : null;
      const result = await dependencies.openSearchWindow(
        buildWindowFilters(session.query, {
          sort,
          sortSeed,
          limit: Math.max(session.query.limit, session.loadedCount),
        }),
        { mode: session.query.mode },
      );
      return createSessionFromResult(session.query, result);
    },
  };
}
