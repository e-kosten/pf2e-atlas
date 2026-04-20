import type { EmbeddingProvider } from "../embeddings.js";
import type {
  NormalizedRecord,
  SearchCategory,
  SearchExplainResult,
  SearchFilters,
  SearchSort,
  SearchSubcategory,
} from "../domain/index.js";
import type { RankingConfig } from "./ranking-config.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "./ranking.js";

export type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

export type NormalizedSearchScope = {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type NormalizedSearchFilters = Omit<SearchFilters, "category" | "subcategory" | "scopes"> & {
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
