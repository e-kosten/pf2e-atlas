import type { EmbeddingProvider } from "../embeddings.js";
import type {
  SearchCategory,
  SearchExplainResult,
  SearchProfile,
  SearchSort,
  SearchSubcategory,
} from "../domain/search-types.js";
import { SEARCH_VOCABULARY } from "../domain/search-types.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type { RecordKey } from "../domain/record-types.js";
import type { RankingConfig } from "./ranking-config.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "./ranking.js";
import type { MetadataAtomicPredicate } from "../domain/search-filter-metadata.js";
import type { MetricOperator, NumericMetricOperator } from "../domain/search-filter-operators.js";
import type { SearchTraceSink } from "./trace.js";
import { SEARCH_REQUEST_VOCABULARY } from "../domain/search-request-types.js";

export const SEARCH_EXECUTION_VOCABULARY = {
  PROFILE: SEARCH_VOCABULARY.PROFILE,
  SORT_KIND: SEARCH_VOCABULARY.SORT_KIND,
  MODE: SEARCH_VOCABULARY.MODE,
  FILTER_NODE_KIND: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND,
  SCOPE_SUBCATEGORY_MATCH_KIND: SEARCH_REQUEST_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND,
  FILTER_MATCH_KIND: SEARCH_REQUEST_VOCABULARY.FILTER_MATCH_KIND,
} as const;

type SearchExecutionFilterNodeKind = typeof SEARCH_EXECUTION_VOCABULARY.FILTER_NODE_KIND;
type SearchExecutionScopeSubcategoryMatchKind = typeof SEARCH_EXECUTION_VOCABULARY.SCOPE_SUBCATEGORY_MATCH_KIND;
type SearchExecutionFilterMatchKind = typeof SEARCH_EXECUTION_VOCABULARY.FILTER_MATCH_KIND;

export type SearchExecutionScopeSubcategoryMatch =
  | { kind: SearchExecutionScopeSubcategoryMatchKind["ANY"] }
  | { kind: SearchExecutionScopeSubcategoryMatchKind["EQ"]; value: SearchSubcategory }
  | { kind: SearchExecutionScopeSubcategoryMatchKind["IS_NULL"] }
  | { kind: SearchExecutionScopeSubcategoryMatchKind["IS_NOT_NULL"] };

export type SearchExecutionNumericMatch =
  | { kind: SearchExecutionFilterMatchKind["EQ"]; value: number }
  | { kind: SearchExecutionFilterMatchKind["GT"]; value: number }
  | { kind: SearchExecutionFilterMatchKind["GTE"]; value: number }
  | { kind: SearchExecutionFilterMatchKind["LT"]; value: number }
  | { kind: SearchExecutionFilterMatchKind["LTE"]; value: number }
  | { kind: SearchExecutionFilterMatchKind["BETWEEN"]; min: number; max: number };

export type SearchExecutionNullableNumericMatch =
  | SearchExecutionNumericMatch
  | { kind: SearchExecutionFilterMatchKind["IS_NULL"] | SearchExecutionFilterMatchKind["IS_NOT_NULL"] };
export type SearchExecutionNullableStringMatch =
  | { kind: SearchExecutionFilterMatchKind["EQ"]; value: string }
  | { kind: SearchExecutionFilterMatchKind["IN"]; values: string[] }
  | { kind: SearchExecutionFilterMatchKind["NOT_IN"]; values: string[] }
  | { kind: SearchExecutionFilterMatchKind["IS_NULL"] | SearchExecutionFilterMatchKind["IS_NOT_NULL"] };

export type SearchExecutionFilterNode =
  | { kind: SearchExecutionFilterNodeKind["PACK"]; value: string }
  | {
      kind: SearchExecutionFilterNodeKind["SCOPE"];
      category: SearchCategory;
      subcategory: SearchExecutionScopeSubcategoryMatch;
    }
  | { kind: SearchExecutionFilterNodeKind["LEVEL"]; match: SearchExecutionNumericMatch }
  | { kind: SearchExecutionFilterNodeKind["PRICE"]; match: SearchExecutionNumericMatch }
  | { kind: SearchExecutionFilterNodeKind["RARITY"]; match: SearchExecutionNullableStringMatch }
  | { kind: SearchExecutionFilterNodeKind["ACTION_COST"]; match: SearchExecutionNullableNumericMatch }
  | { kind: SearchExecutionFilterNodeKind["LINKS_TO"]; target: RecordKey }
  | { kind: SearchExecutionFilterNodeKind["LINKED_FROM"]; source: RecordKey }
  | { kind: SearchExecutionFilterNodeKind["METADATA_PREDICATE"]; predicate: MetadataAtomicPredicate }
  | { kind: SearchExecutionFilterNodeKind["METRIC"]; metric: string; op: MetricOperator; value: string | number | boolean }
  | { kind: SearchExecutionFilterNodeKind["METRIC_COMPARE"]; leftMetric: string; op: NumericMetricOperator; rightMetric: string }
  | { kind: SearchExecutionFilterNodeKind["ANY_OF"]; children: SearchExecutionFilterNode[] }
  | { kind: SearchExecutionFilterNodeKind["ALL_OF"]; children: SearchExecutionFilterNode[] }
  | { kind: SearchExecutionFilterNodeKind["NOT"]; child: SearchExecutionFilterNode };

export interface SearchExecutionFilters {
  searchProfile?: SearchProfile;
  sort?: SearchSort;
  sortSeed?: number;
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  excludeQuery?: string;
  filter?: SearchExecutionFilterNode;
  offset?: number;
  limit?: number;
}

export type NormalizedSearchFilters = SearchExecutionFilters;

export type SearchCandidate = {
  record: NormalizedRecord;
  searchText?: string | null;
};

export type SearchRetrievalPort = {
  fetchCandidateCount: (filters: NormalizedSearchFilters, options?: { recordKeys?: string[] }) => number;
  fetchPagedCandidates: (
    filters: NormalizedSearchFilters,
    sort: SearchSort,
    offset: number,
    limit: number,
  ) => SearchCandidate[];
  fetchCandidates: (
    filters: NormalizedSearchFilters,
    includeSearchText?: boolean,
    includeEmbedding?: boolean,
    options?: { recordKeys?: string[] },
  ) => SearchCandidate[];
  fetchLexicalRetrievalRows: (
    filters: NormalizedSearchFilters,
    lexicalQuery: string,
    limit: number,
  ) => LexicalRetrievalRow[];
  fetchSemanticRetrievalRows: (
    filters: NormalizedSearchFilters,
    queryVector: Float32Array,
    limit: number,
  ) => SemanticRetrievalRow[];
};

export type RuntimeSearchDependencies = SearchRetrievalPort & {
  embeddingProvider: EmbeddingProvider;
  rankingConfig: RankingConfig;
  rankingConfigStatus: SearchExplainResult["rankingConfig"];
  trace?: SearchTraceSink;
  getAliases: (recordKey: string) => string[];
};
