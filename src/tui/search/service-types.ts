import type { MetadataFieldName } from "../../domain/metadata-field-registry.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import type { SearchVocabularyResult } from "../../data/vocabulary.js";
import type { FilterExplorerScalarClause } from "../filter-explorer/index.js";
import type {
  MetadataFilterNode,
} from "../../domain/metadata-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import type {
  LookupOptions,
  SearchCategory,
  SearchCountResult,
  SearchFilters,
  SearchMode,
  SearchProfile,
  SearchResult,
  SearchSort,
  SearchSubcategory,
  SearchWindowPage,
} from "../../domain/search-types.js";
import type { Pf2eTerminalQueryPart } from "./query-parts.js";

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

export type Pf2eTerminalFacetField = MetadataFieldSemantics["field"];
export type Pf2eTerminalMetricQueryField = "actorMetric" | "itemMetric";
export type Pf2eTerminalQueryField = MetadataFieldSemantics["field"] | Pf2eTerminalMetricQueryField;

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

export type Pf2eTerminalFilterExplorerMetricClause = {
  field: Pf2eTerminalMetricQueryField;
  metric: string;
  valueType: "number" | "text" | "boolean";
  clause: FilterExplorerScalarClause;
};

export type Pf2eTerminalFilterExplorerScalarClauseMap = Record<string, Pf2eTerminalFilterExplorerMetricClause>;

export type Pf2eTerminalFilterExplorerDraft = {
  fieldSelections: Pf2eTerminalQueryFieldSelectionMap;
  scalarClauses: Pf2eTerminalFilterExplorerScalarClauseMap;
  structuredMetadata: MetadataFilterNode | null;
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
  getAvailableRootQueryPartKinds: (
    category: SearchCategory | null,
    subcategory: SearchSubcategory | null,
  ) => Pf2eTerminalQueryPart["kind"][];
  getRootQueryParts: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryPart[];
  applyRootQueryParts: (query: Pf2eTerminalSearchQuery, parts: Pf2eTerminalQueryPart[]) => Pf2eTerminalSearchQuery;
  createFilterExplorerDraft: (
    query: Pf2eTerminalSearchQuery,
    scopedFields: readonly Pf2eTerminalQueryField[],
  ) => Pf2eTerminalFilterExplorerDraft;
  createFilterExplorerDraftFromMetadataNode: (
    node: MetadataFilterNode | null,
    scopedFields: readonly Pf2eTerminalQueryField[],
  ) => Pf2eTerminalFilterExplorerDraft;
  buildFilterExplorerMetadataNode: (draft: Pf2eTerminalFilterExplorerDraft) => MetadataFilterNode | null;
  applyFilterExplorerDraft: (
    query: Pf2eTerminalSearchQuery,
    draft: Pf2eTerminalFilterExplorerDraft,
  ) => Pf2eTerminalSearchQuery;
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

export type SearchServiceDependencies = {
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

export type LegacyFacetSelection = {
  field: Pf2eTerminalFacetField;
  policy: Pf2eTerminalFilterValuePolicy<string>;
};
