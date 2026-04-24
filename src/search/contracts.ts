import type { EmbeddingProvider } from "../embeddings.js";
import type {
  SearchCategory,
  SearchExplainResult,
  SearchProfile,
  SearchSort,
  SearchSubcategory,
} from "../domain/search-types.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type { RankingConfig } from "./ranking-config.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "./ranking.js";
import type { MetadataAtomicPredicate } from "../domain/search-filter-metadata.js";
import type { MetricOperator, NumericMetricOperator } from "../domain/search-filter-operators.js";

export type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

export type SearchExecutionScopeSubcategoryMatch =
  | { kind: "any" }
  | { kind: "eq"; value: SearchSubcategory }
  | { kind: "isNull" }
  | { kind: "isNotNull" };

export type SearchExecutionNumericMatch =
  | { kind: "eq"; value: number }
  | { kind: "gte"; value: number }
  | { kind: "lte"; value: number }
  | { kind: "between"; min: number; max: number };

export type SearchExecutionNullableNumericMatch = SearchExecutionNumericMatch | { kind: "isNull" | "isNotNull" };
export type SearchExecutionNullableStringMatch =
  | { kind: "eq"; value: string }
  | { kind: "isNull" | "isNotNull" };

export type SearchExecutionFilterNode =
  | { kind: "pack"; value: string }
  | {
      kind: "scope";
      category: SearchCategory;
      subcategory: SearchExecutionScopeSubcategoryMatch;
    }
  | { kind: "level"; match: SearchExecutionNumericMatch }
  | { kind: "price"; match: SearchExecutionNumericMatch }
  | { kind: "rarity"; match: SearchExecutionNullableStringMatch }
  | { kind: "actionCost"; match: SearchExecutionNullableNumericMatch }
  | { kind: "linksTo"; target: string }
  | { kind: "metadataPredicate"; predicate: MetadataAtomicPredicate }
  | { kind: "metric"; metric: string; op: MetricOperator; value: string | number | boolean }
  | { kind: "metricCompare"; leftMetric: string; op: NumericMetricOperator; rightMetric: string }
  | { kind: "anyOf"; children: SearchExecutionFilterNode[] }
  | { kind: "allOf"; children: SearchExecutionFilterNode[] }
  | { kind: "not"; child: SearchExecutionFilterNode };

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
