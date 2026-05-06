import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type { FilterValueOrdering } from "../../domain/filter-value-ordering.js";
import type { Pf2eApplicationSearchDiscoveryService } from "../../app/search-discovery-service.js";
import type {
  SearchCategorySummaryResult,
  SearchSemanticsBootstrapSummaryResult,
  SearchVocabularyResult,
} from "../../data/vocabulary.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import type { MetadataFilterNode } from "./metadata-filter-draft.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../domain/search-field-domains.js";
import type {
  LookupOptions,
  LookupResult,
  SearchCategory,
  SearchCountResult,
  SearchMode,
  SearchProfile,
  SearchResultRecord,
  SearchResult,
  SearchSubcategory,
  SearchWindowPage,
} from "../../domain/search-types.js";
import type {
  FilterExplorerComposeDraft,
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
  valueOrdering?: FilterValueOrdering;
};

export type Pf2eTerminalFacetValueOption = {
  value: string;
  label: string;
  description: string;
  count: number;
};

export type Pf2eTerminalValueSelection<T extends number | string = string> = {
  include: T[];
  exclude: T[];
};

export type Pf2eTerminalFacetField = MetadataFieldSemantics["field"];
export type Pf2eTerminalMetricQueryField = "actorMetric" | "itemMetric";
export type Pf2eTerminalSyntheticQueryField = "pack";
export type Pf2eTerminalQueryField =
  | MetadataFieldSemantics["field"]
  | Pf2eTerminalMetricQueryField
  | Pf2eTerminalSyntheticQueryField;

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
export type Pf2eTerminalSearchResultRecord = SearchResultRecord;

export type Pf2eTerminalSearchQuery = SearchRequest;

export type Pf2eTerminalQueryFieldOption = {
  value: Pf2eTerminalQueryField;
  label: string;
  description: string;
  fieldType: MetadataFieldSemantics["fieldType"];
  editor: Pf2eTerminalQueryFieldEditor;
  valueOrdering?: FilterValueOrdering;
};

export type Pf2eTerminalQueryFieldEditor = "structuredForm" | "sharedExplorer";

export type Pf2eTerminalQueryFieldSelectionMap = Record<string, Pf2eTerminalValueSelection<string>>;

export type Pf2eTerminalFilterExplorerDraft = FilterExplorerComposeDraft;

export type Pf2eTerminalPreparedFilterExplorerDraft = {
  draft: Pf2eTerminalFilterExplorerDraft;
  preservedMetadata: MetadataFilterNode | null;
  scopedFields: readonly Pf2eTerminalQueryField[];
};

export type Pf2eTerminalPreparedFilterExplorerContext = Omit<Pf2eTerminalPreparedFilterExplorerDraft, "draft">;
export type Pf2eTerminalFilterExplorerInsertionResult =
  | { kind: "replace"; node: MetadataFilterNode | null }
  | { kind: "insert"; nodes: MetadataFilterNode[] };

export type Pf2eTerminalSearchSession = {
  windowId: string;
  query: Pf2eTerminalSearchQuery;
  results: Pf2eTerminalSearchResultRecord[];
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
  loadMetricKeyOptions: (
    query: Pf2eTerminalSearchQuery,
    field: Pf2eTerminalMetricQueryField,
    discoveryMode: SearchFilterDiscoveryMode,
    options?: { numericOnly?: boolean },
  ) => Promise<Pf2eTerminalFacetValueOption[]>;
  getPackLabel: (packValue: string) => string;
  loadPackOptions: (
    query: Pf2eTerminalSearchQuery,
    discoveryMode: SearchFilterDiscoveryMode,
  ) => Promise<Pf2eTerminalFacetValueOption[]>;
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
  buildFilterExplorerInsertionResult: (
    draft: Pf2eTerminalFilterExplorerDraft,
    options?: { preservedMetadata?: MetadataFilterNode | null; preferReplace?: boolean },
  ) => Pf2eTerminalFilterExplorerInsertionResult;
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
  getPack?: (packValue: string) => { name: string; label?: string } | undefined;
  getSearchCategorySummary?: () => SearchCategorySummaryResult;
  getSearchSemanticsBootstrapSummary?: () => SearchSemanticsBootstrapSummaryResult;
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
