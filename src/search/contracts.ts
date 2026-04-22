import type { EmbeddingProvider } from "../embeddings.js";
import type {
  SearchCategory,
  SearchCategoryInput,
  SearchExplainResult,
  SearchProfile,
  SearchScope,
  SearchSort,
  SearchSubcategory,
  SearchSubcategoryInput,
} from "../domain/search-types.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type { RankingConfig } from "./ranking-config.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "./ranking.js";
import type { MetadataFilterNode } from "../domain/metadata-filter-types.js";

export type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

export interface SearchExecutionFilters {
  searchProfile?: SearchProfile;
  sort?: SearchSort;
  sortSeed?: number;
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  excludeQuery?: string;
  linksTo?: string[];
  linksToMode?: "any" | "all";
  excludeLinksTo?: string[];
  pack?: string;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  metadata?: MetadataFilterNode;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
}

export type NormalizedSearchScope = {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type NormalizedSearchFilters = Omit<SearchExecutionFilters, "category" | "subcategory" | "scopes"> & {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  scopes?: NormalizedSearchScope[];
};

export type SearchCandidate = {
  record: NormalizedRecord;
  searchText?: string | null;
};

export type RuntimeSearchDependencies = {
  embeddingProvider: EmbeddingProvider;
  rankingConfig: RankingConfig;
  rankingConfigStatus: SearchExplainResult["rankingConfig"];
  fetchCandidateCount: (filters: NormalizedSearchFilters, options?: { recordKeys?: string[] }) => number;
  fetchPagedCandidates: (
    filters: NormalizedSearchFilters,
    sort: SearchSort,
    offset: number,
    limit: number,
  ) => SearchCandidate[];
  getAliases: (recordKey: string) => string[];
  fetchCandidates: (
    filters: NormalizedSearchFilters,
    includeSearchText?: boolean,
    includeEmbedding?: boolean,
    options?: { recordKeys?: string[] },
  ) => SearchCandidate[];
  fetchLexicalRetrievalRows: (
    filters: NormalizedSearchFilters,
    ftsQuery: string,
    limit: number,
  ) => LexicalRetrievalRow[];
  fetchSemanticRetrievalRows: (
    filters: NormalizedSearchFilters,
    queryVector: Float32Array,
    limit: number,
  ) => SemanticRetrievalRow[];
};
