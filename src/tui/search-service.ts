import { CATEGORY_SUBCATEGORY_MAP, normalizeSearchCategory, normalizeSearchSubcategory } from "../domain/categories.js";
import { getMetadataFilterSemantics, type MetadataFieldSemantics } from "../domain/metadata-semantics.js";
import type { MetadataFieldName } from "../domain/metadata-field-registry.js";
import { orderFilterValues, type FilterValueOrdering } from "../domain/filter-value-ordering.js";
import type {
  LookupOptions,
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataPredicate,
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
import {
  isMetadataQueryPart,
  metadataFilterNodeToRootQueryParts,
  rootMetadataQueryPartsToFilterNode,
  type Pf2eTerminalMetadataQueryPart,
  type Pf2eTerminalQueryPart,
  type Pf2eTerminalQueryPartKind,
  type Pf2eTerminalQueryPartPolicy,
} from "./search-query-parts.js";

export type {
  Pf2eTerminalMetadataQueryPart,
  Pf2eTerminalQueryPart,
  Pf2eTerminalQueryPartKind,
  Pf2eTerminalQueryPartPolicy,
} from "./search-query-parts.js";

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

export type Pf2eTerminalSearchStructuredPart = Pf2eTerminalQueryPart;

export type Pf2eTerminalSearchFilters = {
  category: SearchCategory | null;
  subcategory: SearchSubcategory | null;
  levelMin: number | null;
  levelMax: number | null;
  rarity: Pf2eTerminalFilterValuePolicy<string>;
  actionCost: Pf2eTerminalFilterValuePolicy<number>;
  facets: Pf2eTerminalFacetSelection[];
  metadata: MetadataFilterNode | null;
  parts: Pf2eTerminalSearchStructuredPart[];
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
  editor: Pf2eTerminalQueryFieldEditor;
};

export type Pf2eTerminalQueryFieldEditor = "policyList" | "structuredForm" | "ontologyPicker";

export type Pf2eTerminalQueryFieldSelectionMap = Record<string, Pf2eTerminalFilterValuePolicy<string>>;

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
  getAvailableRootQueryPartKinds: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalQueryPartKind[];
  getRootQueryParts: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryPart[];
  applyRootQueryParts: (query: Pf2eTerminalSearchQuery, parts: Pf2eTerminalQueryPart[]) => Pf2eTerminalSearchQuery;
  buildDiscoverableQueryFieldSelections: (
    query: Pf2eTerminalSearchQuery,
    scopedFields: string[],
  ) => Pf2eTerminalQueryFieldSelectionMap;
  applyDiscoverableQueryFieldSelections: (
    query: Pf2eTerminalSearchQuery,
    selections: Pf2eTerminalQueryFieldSelectionMap,
    scopedFields: string[],
  ) => Pf2eTerminalSearchQuery;
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
    parts: [],
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

function splitMetadataTreeIntoParts(node: MetadataFilterNode | null): Pf2eTerminalSearchStructuredPart[] {
  return metadataFilterNodeToRootQueryParts(node);
}

export function getSearchQueryCategory(query: Pf2eTerminalSearchQuery): SearchCategory | null {
  return query.filters.category;
}

export function getSearchQueryPart<TKind extends Pf2eTerminalSearchStructuredPart["kind"]>(
  query: Pf2eTerminalSearchQuery,
  kind: TKind,
): Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> | null {
  return (
    query.filters.parts.find((part): part is Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> => part.kind === kind) ??
    null
  );
}

export function getSearchQuerySubcategory(query: Pf2eTerminalSearchQuery): SearchSubcategory | null {
  return getSearchQueryPart(query, "subcategory")?.subcategory ?? query.filters.subcategory ?? null;
}

export function getSearchQueryLevelRange(query: Pf2eTerminalSearchQuery): { levelMin: number | null; levelMax: number | null } {
  const part = getSearchQueryPart(query, "levelRange");
  return {
    levelMin: part?.levelMin ?? query.filters.levelMin ?? null,
    levelMax: part?.levelMax ?? query.filters.levelMax ?? null,
  };
}

export function getSearchQueryRarityPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<string> {
  return cloneStringPolicy(getSearchQueryPart(query, "rarityPolicy")?.policy ?? query.filters.rarity);
}

export function getSearchQueryActionCostPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<number> {
  return cloneNumberPolicy(getSearchQueryPart(query, "actionCostPolicy")?.policy ?? query.filters.actionCost);
}

export function getSearchQueryMetadataTree(query: Pf2eTerminalSearchQuery): MetadataFilterNode | null {
  return rootMetadataQueryPartsToFilterNode(query.filters.parts) ?? query.filters.metadata ?? null;
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      category,
      subcategory: null,
      levelMin: null,
      levelMax: null,
      rarity: createEmptyStringPolicy(),
      actionCost: createEmptyNumberPolicy(),
      facets: [],
      metadata: null,
      parts: [],
    },
  };
}

export function setSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  part: Exclude<Pf2eTerminalSearchStructuredPart, Pf2eTerminalMetadataQueryPart>,
): Pf2eTerminalSearchQuery {
  const nextParts = query.filters.parts.filter((candidate) => candidate.kind !== part.kind && !isMetadataQueryPart(candidate));
  nextParts.push(part);
  nextParts.push(...query.filters.parts.filter(isMetadataQueryPart));
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: nextParts,
    },
  };
}

export function removeSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  kind: Exclude<Pf2eTerminalSearchStructuredPart["kind"], Pf2eTerminalMetadataQueryPart["kind"]>,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: query.filters.parts.filter((part) => part.kind !== kind),
    },
  };
}

export function setSearchQueryMetadataTree(
  query: Pf2eTerminalSearchQuery,
  node: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: [
        ...query.filters.parts.filter((part) => !isMetadataQueryPart(part)),
        ...splitMetadataTreeIntoParts(node),
      ],
    },
  };
}

function createEmptyStringPolicy(): Pf2eTerminalFilterValuePolicy<string> {
  return createEmptyFilterPolicy<string>();
}

function createEmptyNumberPolicy(): Pf2eTerminalFilterValuePolicy<number> {
  return createEmptyFilterPolicy<number>();
}

function hasStringPolicy(policy: Pf2eTerminalFilterValuePolicy<string>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function hasNumberPolicy(policy: Pf2eTerminalFilterValuePolicy<number>): boolean {
  return policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0;
}

function cloneStringPolicy(
  policy: Pf2eTerminalFilterValuePolicy<string> | Pf2eTerminalQueryPartPolicy<string>,
): Pf2eTerminalFilterValuePolicy<string> {
  return {
    any: [...policy.any],
    all: [...policy.all],
    exclude: [...policy.exclude],
  };
}

function cloneNumberPolicy(
  policy: Pf2eTerminalFilterValuePolicy<number> | Pf2eTerminalQueryPartPolicy<number>,
): Pf2eTerminalFilterValuePolicy<number> {
  return {
    any: [...policy.any],
    all: [...policy.all],
    exclude: [...policy.exclude],
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

function normalizeQueryFieldPolicy(
  field: Pf2eTerminalQueryField,
  policy: Partial<Pf2eTerminalFilterValuePolicy<string>> | undefined,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalFilterValuePolicy<string> | null {
  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (!["set", "enumString", "boolean"].includes(fieldSemantics.fieldType)) {
    return null;
  }

  const normalizedPolicy = normalizeStringPolicy(policy, fieldSemantics.valueOrdering);
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

  return normalizedPolicy;
}

function normalizeMetadataNode(node: MetadataFilterNode | null): MetadataFilterNode | null {
  if (!node) {
    return null;
  }

  if ("and" in node) {
    const children = node.and
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { and: children };
  }

  if ("or" in node) {
    const children = node.or
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { or: children };
  }

  if ("not" in node) {
    const child = normalizeMetadataNode(node.not);
    if (!child) {
      return null;
    }
    return { not: child };
  }

  return node;
}

function mergeStringPolicies(
  left: Pf2eTerminalFilterValuePolicy<string>,
  right: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalFilterValuePolicy<string> {
  return {
    any: [...left.any, ...right.any],
    all: [...left.all, ...right.all],
    exclude: [...left.exclude, ...right.exclude],
  };
}

function mergeSelectionMaps(
  target: Pf2eTerminalQueryFieldSelectionMap,
  source: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = { ...target };

  for (const [field, policy] of Object.entries(source)) {
    next[field] = field in next ? mergeStringPolicies(next[field]!, policy) : { ...policy };
  }

  return next;
}

function createScopedSelectionMap(scopedFields: string[]): Pf2eTerminalQueryFieldSelectionMap {
  return Object.fromEntries(scopedFields.map((field) => [field, createEmptyStringPolicy()])) as Pf2eTerminalQueryFieldSelectionMap;
}

function extractPolicyFromMetadataPredicate(
  node: MetadataPredicate,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): { field: Pf2eTerminalQueryField; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  const fieldSemantics = fieldSemanticsByName.get(node.field as Pf2eTerminalQueryField);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    if ("values" in node && node.op === "includesAny") {
      return { field: node.field, policy: { any: node.values.map(String), all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "includesAll") {
      return { field: node.field, policy: { any: [], all: node.values.map(String), exclude: [] } };
    }
    if ("values" in node && node.op === "excludesAny") {
      return { field: node.field, policy: { any: [], all: [], exclude: node.values.map(String) } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "enumString") {
    if ("value" in node && node.op === "eq") {
      return { field: node.field, policy: { any: [String(node.value)], all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "in") {
      return { field: node.field, policy: { any: node.values.map(String), all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "notIn") {
      return { field: node.field, policy: { any: [], all: [], exclude: node.values.map(String) } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "boolean" && "value" in node && node.op === "eq") {
    return { field: node.field, policy: { any: [String(node.value)], all: [], exclude: [] } };
  }

  return null;
}

function tryExtractScopedPolicyNode(
  node: MetadataFilterNode,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): { field: Pf2eTerminalQueryField; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  if ("and" in node) {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedPolicy = createEmptyStringPolicy();

    for (const child of node.and) {
      if ("and" in child || "or" in child || "not" in child) {
        return null;
      }
      const extracted = extractPolicyFromMetadataPredicate(child, fieldSemanticsByName);
      if (!extracted || !scopedFieldSet.has(extracted.field)) {
        return null;
      }
      if (extractedField && extracted.field !== extractedField) {
        return null;
      }
      extractedField = extracted.field;
      mergedPolicy = mergeStringPolicies(mergedPolicy, extracted.policy);
    }

    return extractedField ? { field: extractedField, policy: mergedPolicy } : null;
  }

  if ("or" in node || "not" in node) {
    return null;
  }

  const extracted = extractPolicyFromMetadataPredicate(node, fieldSemanticsByName);
  return extracted && scopedFieldSet.has(extracted.field) ? extracted : null;
}

function extractScopedQueryFieldSelections(
  node: MetadataFilterNode | null,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
} {
  if (!node) {
    return {
      metadata: null,
      selections: {},
    };
  }

  const directExtraction = tryExtractScopedPolicyNode(node, scopedFieldSet, fieldSemanticsByName);
  if (directExtraction) {
    return {
      metadata: null,
      selections: {
        [directExtraction.field]: directExtraction.policy,
      },
    };
  }

  if ("and" in node) {
    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    const children: MetadataFilterNode[] = [];

    for (const child of node.and) {
      const extracted = extractScopedQueryFieldSelections(child, scopedFieldSet, fieldSemanticsByName);
      selections = mergeSelectionMaps(selections, extracted.selections);
      if (extracted.metadata) {
        children.push(extracted.metadata);
      }
    }

    return {
      metadata: normalizeMetadataNode(children.length === 0 ? null : { and: children }),
      selections,
    };
  }

  return {
    metadata: node,
    selections: {},
  };
}

function isActionCostAvailableInScope(
  dependencies: SearchServiceDependencies,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!category) {
    return false;
  }

  return (
    dependencies.listFilterValues({
      field: "actionCost",
      category,
      ...(subcategory ? { subcategory } : {}),
    }).values.length > 0
  );
}

function buildRootQueryPartsFromLegacyFilters(
  query: Pf2eTerminalSearchQuery,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
  actionCostAvailable: boolean,
): Pf2eTerminalQueryPart[] {
  const parts: Pf2eTerminalQueryPart[] = [];

  if (subcategory) {
    parts.push({
      kind: "subcategory",
      subcategory,
    });
  }

  if (query.filters.levelMin !== null || query.filters.levelMax !== null) {
    parts.push({
      kind: "levelRange",
      levelMin: query.filters.levelMin ?? null,
      levelMax: query.filters.levelMax ?? null,
    });
  }

  const rarityPolicy = normalizeStringPolicy(query.filters.rarity, fieldSemanticsByName.get("rarity")?.valueOrdering);
  if (hasStringPolicy(rarityPolicy)) {
    parts.push({
      kind: "rarityPolicy",
      policy: {
        any: rarityPolicy.any,
        all: [],
        exclude: rarityPolicy.exclude,
      },
    });
  }

  const actionCostPolicy = normalizeNumberPolicy(query.filters.actionCost);
  if (actionCostAvailable && hasNumberPolicy(actionCostPolicy)) {
    parts.push({
      kind: "actionCostPolicy",
      policy: {
        any: actionCostPolicy.any,
        all: [],
        exclude: actionCostPolicy.exclude,
      },
    });
  }

  for (const facet of query.filters.facets) {
    const facetNode = buildMetadataNodeForFacet(facet, fieldSemanticsByName);
    if (!facetNode) {
      continue;
    }
    parts.push(...metadataFilterNodeToRootQueryParts(facetNode));
  }

  if (query.filters.metadata) {
    parts.push(...metadataFilterNodeToRootQueryParts(query.filters.metadata));
  }

  return parts;
}

function normalizeRootQueryParts(
  parts: readonly Pf2eTerminalQueryPart[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  category: SearchCategory | null,
  actionCostAvailable: boolean,
): Pf2eTerminalQueryPart[] {
  let subcategoryPart: Extract<Pf2eTerminalQueryPart, { kind: "subcategory" }> | null = null;
  let levelRangePart: Extract<Pf2eTerminalQueryPart, { kind: "levelRange" }> | null = null;
  let rarityPolicyPart: Extract<Pf2eTerminalQueryPart, { kind: "rarityPolicy" }> | null = null;
  let actionCostPolicyPart: Extract<Pf2eTerminalQueryPart, { kind: "actionCostPolicy" }> | null = null;
  const metadataParts: Pf2eTerminalQueryPart[] = [];

  for (const part of parts) {
    switch (part.kind) {
      case "subcategory": {
        if (!category) {
          continue;
        }
        const normalizedSubcategory = normalizeSearchSubcategory(part.subcategory);
        if (!normalizedSubcategory || !CATEGORY_SUBCATEGORY_MAP[category].includes(normalizedSubcategory)) {
          continue;
        }
        subcategoryPart = {
          kind: "subcategory",
          subcategory: normalizedSubcategory,
        };
        continue;
      }
      case "levelRange": {
        const levelMin = part.levelMin ?? null;
        const levelMax = part.levelMax ?? null;
        if (levelMin === null && levelMax === null) {
          continue;
        }
        levelRangePart = {
          kind: "levelRange",
          levelMin: levelMin !== null && levelMax !== null ? Math.min(levelMin, levelMax) : levelMin,
          levelMax: levelMin !== null && levelMax !== null ? Math.max(levelMin, levelMax) : levelMax,
        };
        continue;
      }
      case "rarityPolicy": {
        const normalizedPolicy = normalizeStringPolicy(
          part.policy,
          fieldSemanticsByName.get("rarity")?.valueOrdering,
        );
        if (!hasStringPolicy(normalizedPolicy)) {
          continue;
        }
        rarityPolicyPart = {
          kind: "rarityPolicy",
          policy: {
            any: normalizedPolicy.any,
            all: [],
            exclude: normalizedPolicy.exclude,
          },
        };
        continue;
      }
      case "actionCostPolicy": {
        if (!actionCostAvailable) {
          continue;
        }
        const normalizedPolicy = normalizeNumberPolicy(part.policy);
        if (!hasNumberPolicy(normalizedPolicy)) {
          continue;
        }
        actionCostPolicyPart = {
          kind: "actionCostPolicy",
          policy: {
            any: normalizedPolicy.any,
            all: [],
            exclude: normalizedPolicy.exclude,
          },
        };
        continue;
      }
      case "metadataPredicate":
      case "metadataGroup":
      case "metadataNot": {
        if (isMetadataQueryPart(part)) {
          metadataParts.push(part);
        }
      }
    }
  }

  return [
    ...(subcategoryPart ? [subcategoryPart] : []),
    ...(levelRangePart ? [levelRangePart] : []),
    ...(rarityPolicyPart ? [rarityPolicyPart] : []),
    ...(actionCostPolicyPart ? [actionCostPolicyPart] : []),
    ...metadataParts,
  ];
}

function compileRootQueryPartsToFilters(
  parts: readonly Pf2eTerminalQueryPart[],
): Pick<Pf2eTerminalSearchFilters, "subcategory" | "levelMin" | "levelMax" | "rarity" | "actionCost" | "facets" | "metadata"> {
  let subcategory: SearchSubcategory | null = null;
  let levelMin: number | null = null;
  let levelMax: number | null = null;
  let rarity = createEmptyStringPolicy();
  let actionCost = createEmptyNumberPolicy();

  for (const part of parts) {
    switch (part.kind) {
      case "subcategory":
        subcategory = part.subcategory;
        break;
      case "levelRange":
        levelMin = part.levelMin ?? null;
        levelMax = part.levelMax ?? null;
        break;
      case "rarityPolicy":
        rarity = {
          any: [...part.policy.any],
          all: [],
          exclude: [...part.policy.exclude],
        };
        break;
      case "actionCostPolicy":
        actionCost = {
          any: [...part.policy.any],
          all: [],
          exclude: [...part.policy.exclude],
        };
        break;
      case "metadataPredicate":
      case "metadataGroup":
      case "metadataNot":
        break;
    }
  }

  return {
    subcategory,
    levelMin,
    levelMax,
    rarity,
    actionCost,
    facets: [],
    metadata: rootMetadataQueryPartsToFilterNode(parts),
  };
}

function normalizeSearchQuery(
  query: Pf2eTerminalSearchQuery,
  dependencies: SearchServiceDependencies,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  const category = normalizeSearchCategory(query.filters.category) ?? null;
  const legacySubcategory =
    category && query.filters.subcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(query.filters.subcategory)
      ? query.filters.subcategory
      : null;
  const actionCostAvailable = isActionCostAvailableInScope(dependencies, category, legacySubcategory);
  const currentParts = Array.isArray(query.filters.parts) ? query.filters.parts : [];
  const nextParts = normalizeRootQueryParts(
    currentParts.length > 0
      ? currentParts
      : buildRootQueryPartsFromLegacyFilters(query, fieldSemanticsByName, category, legacySubcategory, actionCostAvailable),
    fieldSemanticsByName,
    category,
    actionCostAvailable,
  );
  const compiledFilters = compileRootQueryPartsToFilters(nextParts);

  return {
    ...query,
    queryText: query.queryText.trim(),
    sourceLabel: query.sourceLabel?.trim() || null,
    filters: {
      ...query.filters,
      category,
      subcategory: compiledFilters.subcategory,
      levelMin: compiledFilters.levelMin,
      levelMax: compiledFilters.levelMax,
      rarity: cloneStringPolicy(compiledFilters.rarity),
      actionCost: cloneNumberPolicy(compiledFilters.actionCost),
      facets: [],
      metadata: compiledFilters.metadata,
      parts: nextParts,
    },
  };
}

function buildDiscreteFilterNodes(query: Pf2eTerminalSearchQuery): MetadataFilterNode[] {
  const nodes: MetadataFilterNode[] = [];
  const rarityPolicy = getSearchQueryRarityPolicy(query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(query);

  if (rarityPolicy.any.length === 1) {
    nodes.push({
      field: "rarity",
      op: "eq",
      value: rarityPolicy.any[0]!,
    });
  } else if (rarityPolicy.any.length > 1) {
    nodes.push({
      field: "rarity",
      op: "in",
      values: rarityPolicy.any,
    });
  }

  if (rarityPolicy.exclude.length > 0) {
    nodes.push({
      field: "rarity",
      op: "notIn",
      values: rarityPolicy.exclude,
    });
  }

  if (actionCostPolicy.any.length === 1) {
    nodes.push({
      field: "actionCost",
      op: "eq",
      value: actionCostPolicy.any[0]!,
    });
  } else if (actionCostPolicy.any.length > 1) {
    nodes.push({
      or: actionCostPolicy.any.map((value) => ({
        field: "actionCost",
        op: "eq",
        value,
      })),
    });
  }

  if (actionCostPolicy.exclude.length === 1) {
    nodes.push({
      not: {
        field: "actionCost",
        op: "eq",
        value: actionCostPolicy.exclude[0]!,
      },
    });
  } else if (actionCostPolicy.exclude.length > 1) {
    nodes.push({
      not: {
        or: actionCostPolicy.exclude.map((value) => ({
          field: "actionCost",
          op: "eq",
          value,
        })),
      },
    });
  }

  return nodes;
}

function buildMetadataNodeForQueryFieldSelection(
  field: Pf2eTerminalQueryField,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const normalizedPolicy = normalizeQueryFieldPolicy(field, policy, fieldSemanticsByName);
  if (!normalizedPolicy) {
    return null;
  }

  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAny",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.all.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAll",
        values: normalizedPolicy.all,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "excludesAny",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length === 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "eq",
        value: normalizedPolicy.any[0]!,
      });
    } else if (normalizedPolicy.any.length > 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "in",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "notIn",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: MetadataFilterNode[] = [];
    for (const value of normalizedPolicy.any) {
      clauses.push({
        field: field as MetadataBooleanField,
        op: "eq",
        value: value === "true",
      });
    }
    for (const value of normalizedPolicy.exclude) {
      clauses.push({
        not: {
          field: field as MetadataBooleanField,
          op: "eq",
          value: value === "true",
        },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  return null;
}

function buildMetadataNodeForFacet(
  facet: Pf2eTerminalFacetSelection,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  return buildMetadataNodeForQueryFieldSelection(facet.field, facet.policy, fieldSemanticsByName);
}

function getQueryFieldEditor(field: MetadataFieldSemantics): Pf2eTerminalQueryFieldEditor {
  if (field.field === "derivedTags") {
    return "ontologyPicker";
  }
  if (["set", "enumString", "boolean"].includes(field.fieldType)) {
    return "policyList";
  }
  return "structuredForm";
}

function buildSearchFilters(
  query: Pf2eTerminalSearchQuery,
  _fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
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
  const metadataTree = getSearchQueryMetadataTree(query);
  const metadataClauses = [
    ...buildDiscreteFilterNodes(query),
    ...(metadataTree ? [metadataTree] : []),
  ];
  const metadata =
    metadataClauses.length === 0
      ? undefined
      : metadataClauses.length === 1
        ? metadataClauses[0]
        : { and: metadataClauses };
  return {
    category: getSearchQueryCategory(query) ?? undefined,
    subcategory: getSearchQuerySubcategory(query) ?? undefined,
    levelMin: getSearchQueryLevelRange(query).levelMin ?? undefined,
    levelMax: getSearchQueryLevelRange(query).levelMax ?? undefined,
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

  function isActionCostRelevant(category: SearchCategory | null, subcategory: SearchSubcategory | null): boolean {
    return isActionCostAvailableInScope(dependencies, category, subcategory);
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
    const category = normalizeSearchCategory(query.filters.category) ?? null;
    const normalizedSubcategory = normalizeSearchSubcategory(query.filters.subcategory) ?? null;
    const subcategory =
      category && normalizedSubcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(normalizedSubcategory)
        ? normalizedSubcategory
        : null;
    const parts: Pf2eTerminalSearchStructuredPart[] = [];
    if (subcategory) {
      parts.push({ kind: "subcategory", subcategory });
    }
    if (query.filters.levelMin !== undefined || query.filters.levelMax !== undefined) {
      parts.push({
        kind: "levelRange",
        levelMin: query.filters.levelMin ?? null,
        levelMax: query.filters.levelMax ?? null,
      });
    }
    if (query.filters.rarity) {
      parts.push({
        kind: "rarityPolicy",
        policy: {
          any: [query.filters.rarity],
          all: [],
          exclude: [],
        },
      });
    }
    if (query.filters.actionCost !== undefined) {
      parts.push({
        kind: "actionCostPolicy",
        policy: {
          any: [query.filters.actionCost],
          all: [],
          exclude: [],
        },
      });
    }
    parts.push(...splitMetadataTreeIntoParts(query.filters.metadata ?? null));
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
          category,
          subcategory,
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
          parts,
        },
      },
      dependencies,
      fieldSemanticsByName,
    );
  }

  function buildDiscoverableQueryFieldSelections(
    query: Pf2eTerminalSearchQuery,
    scopedFields: string[],
  ): Pf2eTerminalQueryFieldSelectionMap {
    const selectionMap = createScopedSelectionMap(scopedFields);
    const scopedFieldSet = new Set(scopedFields);

    const extracted = extractScopedQueryFieldSelections(getSearchQueryMetadataTree(query), scopedFieldSet, fieldSemanticsByName);
    for (const [field, policy] of Object.entries(extracted.selections)) {
      const normalizedPolicy = normalizeQueryFieldPolicy(field as Pf2eTerminalQueryField, policy, fieldSemanticsByName);
      if (!normalizedPolicy) {
        continue;
      }
      selectionMap[field] = mergeStringPolicies(selectionMap[field] ?? createEmptyStringPolicy(), normalizedPolicy);
    }

    for (const field of scopedFields) {
      const normalizedPolicy = normalizeQueryFieldPolicy(
        field as Pf2eTerminalQueryField,
        selectionMap[field],
        fieldSemanticsByName,
      );
      selectionMap[field] = normalizedPolicy ?? createEmptyStringPolicy();
    }

    return selectionMap;
  }

  function applyDiscoverableQueryFieldSelections(
    query: Pf2eTerminalSearchQuery,
    selections: Pf2eTerminalQueryFieldSelectionMap,
    scopedFields: string[],
  ): Pf2eTerminalSearchQuery {
    const scopedFieldSet = new Set(scopedFields);
    const extracted = extractScopedQueryFieldSelections(getSearchQueryMetadataTree(query), scopedFieldSet, fieldSemanticsByName);
    const metadataClauses: MetadataFilterNode[] = extracted.metadata ? [extracted.metadata] : [];

    for (const field of scopedFields) {
      const nextPolicy = normalizeQueryFieldPolicy(
        field as Pf2eTerminalQueryField,
        selections[field] ?? createEmptyStringPolicy(),
        fieldSemanticsByName,
      );
      if (!nextPolicy) {
        continue;
      }

      const metadataNode = buildMetadataNodeForQueryFieldSelection(
        field as Pf2eTerminalQueryField,
        nextPolicy,
        fieldSemanticsByName,
      );
      if (metadataNode) {
        metadataClauses.push(metadataNode);
      }
    }
    return setSearchQueryMetadataTree(
      query,
      metadataClauses.length === 0
        ? null
        : metadataClauses.length === 1
          ? metadataClauses[0]!
          : normalizeMetadataNode({ and: metadataClauses }),
    );
  }

  return {
    createDefaultQuery: () => createDefaultQuery(),
    createQueryFromOntologyQuery,
    getAvailableRootQueryPartKinds: (category, subcategory) => [
      ...(category && CATEGORY_SUBCATEGORY_MAP[category].length > 0 ? (["subcategory"] as const) : []),
      "levelRange",
      "rarityPolicy",
      ...(isActionCostRelevant(category, subcategory) ? (["actionCostPolicy"] as const) : []),
      ...(category ? (["metadataPredicate", "metadataGroup", "metadataNot"] as const) : []),
    ],
    getRootQueryParts: (query) => normalizeSearchQuery(query, dependencies, fieldSemanticsByName).filters.parts,
    applyRootQueryParts: (query, parts) =>
      normalizeSearchQuery(
        {
          ...query,
          filters: {
            ...query.filters,
            parts,
          },
        },
        dependencies,
        fieldSemanticsByName,
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
        .filter((field) => field.discoverable && !["rarity", "actionCost"].includes(field.field))
        .map((field) => ({
          value: field.field,
          label: humanizeIdentifier(field.field),
          description:
            field.notes ??
            (field.field === "derivedTags"
              ? "Derived-tag field with hierarchy-capable ontology browsing."
              : `${field.fieldType} query field for the current browse scope.`),
          fieldType: field.fieldType,
          editor: getQueryFieldEditor(field),
        }));
    },
    buildDiscoverableQueryFieldSelections,
    applyDiscoverableQueryFieldSelections,
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
    getActionCostOptions: (category, subcategory) =>
      isActionCostRelevant(category, subcategory)
        ? createFacetValueOptions(
            dependencies.listFilterValues({
              field: "actionCost",
              ...(category ? { category } : {}),
              ...(subcategory ? { subcategory } : {}),
            }).values,
            {
              ordering: getFieldValueOrdering("actionCost"),
              labelFormatter: (value) => `${value} action${value === "1" ? "" : "s"}`,
            },
          )
        : [],
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
    normalizeQuery: (query) => normalizeSearchQuery(query, dependencies, fieldSemanticsByName),
    countQuery: (query) => {
      const normalizedQuery = normalizeSearchQuery(query, dependencies, fieldSemanticsByName);
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
      const normalizedQuery = normalizeSearchQuery(query, dependencies, fieldSemanticsByName);
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
