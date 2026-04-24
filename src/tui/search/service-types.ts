import type { MetadataFieldName } from "../../domain/metadata-field-types.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { Pf2eApplicationSearchDiscoveryService } from "../../app/search-discovery-service.js";
import type { SearchCategorySummaryResult, SearchVocabularyResult } from "../../data/vocabulary.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import type { MetadataFilterNode } from "../../domain/metadata-filter-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import type {
  LookupOptions,
  LookupResult,
  SearchCategory,
  SearchCountResult,
  SearchMode,
  SearchProfile,
  SearchResult,
  SearchSubcategory,
  SearchWindowPage,
} from "../../domain/search-types.js";
import type {
  FilterExplorerComposeDraft,
  FilterExplorerSelectionMap,
} from "../filter-explorer/types.js";

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
export type Pf2eTerminalBrowseSort = "alphabetical" | "levelAsc" | "levelDesc" | "random";
export type Pf2eTerminalSearchModeSort = "ranked";
export type Pf2eTerminalLookupSort =
  | "alphabeticalTiered"
  | "alphabeticalGlobal"
  | "levelAscTiered"
  | "levelAscGlobal"
  | "levelDescTiered"
  | "levelDescGlobal";
export type Pf2eTerminalSearchSort = Pf2eTerminalBrowseSort | Pf2eTerminalSearchModeSort | Pf2eTerminalLookupSort;
export type Pf2eTerminalLookupMatchType = LookupResult["matchType"];

export type Pf2eTerminalSearchQuery = SearchRequest;

export type Pf2eTerminalQueryFieldOption = {
  value: Pf2eTerminalQueryField;
  label: string;
  description: string;
  fieldType: MetadataFieldSemantics["fieldType"];
  editor: Pf2eTerminalQueryFieldEditor;
};

export type Pf2eTerminalQueryFieldEditor = "policyList" | "structuredForm" | "sharedExplorer";

export type Pf2eTerminalQueryFieldSelectionMap = FilterExplorerSelectionMap;

export type Pf2eTerminalFilterExplorerDraft = FilterExplorerComposeDraft;

export type Pf2eTerminalPreparedFilterExplorerDraft = {
  draft: Pf2eTerminalFilterExplorerDraft;
  preservedMetadata: MetadataFilterNode | null;
  scopedFields: readonly Pf2eTerminalQueryField[];
};

export type Pf2eTerminalPreparedFilterExplorerContext = Omit<Pf2eTerminalPreparedFilterExplorerDraft, "draft">;

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
  createDefaultQuery: (mode?: Pf2eTerminalSearchMode) => Pf2eTerminalSearchQuery;
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
  prepareFilterExplorerDraft: (
    query: Pf2eTerminalSearchQuery,
    scopedFields: readonly Pf2eTerminalQueryField[],
  ) => Pf2eTerminalPreparedFilterExplorerDraft;
  prepareFilterExplorerDraftFromMetadataNode: (
    node: MetadataFilterNode | null,
    scopedFields: readonly Pf2eTerminalQueryField[],
  ) => Pf2eTerminalPreparedFilterExplorerDraft;
  buildFilterExplorerMetadataNode: (
    draft: Pf2eTerminalFilterExplorerDraft,
    options?: { preservedMetadata?: MetadataFilterNode | null },
  ) => MetadataFilterNode | null;
  applyFilterExplorerDraft: (
    query: Pf2eTerminalSearchQuery,
    draft: Pf2eTerminalFilterExplorerDraft,
    options?: {
      preservedMetadata?: MetadataFilterNode | null;
      scopedFields?: readonly Pf2eTerminalQueryField[];
    },
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
    request: SearchRequest,
    options?: { lexicalOnly?: boolean },
  ) => Promise<SearchCountResult>;
  discovery: Pf2eApplicationSearchDiscoveryService;
  getSearchCategorySummary?: () => SearchCategorySummaryResult;
  getSearchVocabulary: () => SearchVocabularyResult;
  lookup: (
    name: string,
    options?: LookupOptions,
  ) => { match: NormalizedRecord | null; alternatives: NormalizedRecord[]; matchType: Pf2eTerminalLookupMatchType };
  listRecords: (request: SearchRequest) => SearchResult;
  openSearchWindow: (request: SearchRequest) => Promise<SearchWindowPage>;
  readSearchWindowPage: (windowId: string, offset: number, limit: number) => SearchWindowPage;
  search: (request: SearchRequest) => Promise<SearchResult>;
};
