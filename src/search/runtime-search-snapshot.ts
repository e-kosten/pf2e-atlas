import type { SearchQueryAnalysis } from "../domain/search-types.js";
import type {
  SearchCountResult,
  SearchExplainResult,
  SearchMode,
  SearchProfile,
  SearchRecordExplanation,
  SearchResult,
  SearchSort,
} from "../domain/search-types.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type { RuntimeSearchDependencies } from "./contracts.js";
import type { RankingConfig } from "./ranking-config.js";
import { buildFusionConfigSummary } from "./ranking.js";

type HybridFusionContext = {
  profile: NonNullable<SearchExplainResult["fusionProfile"]>;
  config: RankingConfig["hybridFusion"]["balanced"];
};

export type RuntimeSearchEntry = {
  record: NormalizedRecord;
  explanation: SearchRecordExplanation;
};

export type SearchWindowSnapshot = {
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  sort: SearchSort;
  records: NormalizedRecord[];
  explanations: SearchRecordExplanation[];
  explainContext?: {
    fusionMethod: SearchExplainResult["fusionMethod"];
    fusionProfile: SearchExplainResult["fusionProfile"];
    fusionConfig: ReturnType<typeof buildFusionConfigSummary>;
    lexicalQuery: string;
    semanticQuery: string;
    query: SearchExplainResult["query"];
    excludeQuery: SearchExplainResult["excludeQuery"];
  };
};

export function createSnapshot(
  searchProfile: SearchProfile | null,
  mode: SearchMode,
  sort: SearchSort,
  entries: RuntimeSearchEntry[],
  explainContext?: SearchWindowSnapshot["explainContext"],
): SearchWindowSnapshot {
  return {
    searchProfile,
    mode,
    sort,
    records: entries.map((entry) => entry.record),
    explanations: entries.map((entry) => entry.explanation),
    explainContext,
  };
}

export function createSearchResultPage(options: Omit<SearchResult, "hasMore" | "nextOffset">): SearchResult {
  const hasMore = options.offset + options.records.length < options.total;
  return {
    ...options,
    hasMore,
    nextOffset: hasMore ? options.offset + options.records.length : null,
  };
}

export function createSearchCountResult(result: SearchResult): SearchCountResult {
  return {
    searchProfile: result.searchProfile,
    mode: result.mode,
    total: result.total,
  };
}

export function sliceSnapshotToSearchResult(
  snapshot: SearchWindowSnapshot,
  offset: number,
  limit: number,
  explainRequested: boolean,
  rankingConfigStatus: SearchExplainResult["rankingConfig"],
): SearchResult {
  const records = snapshot.records.slice(offset, offset + limit);
  const explain =
    explainRequested && snapshot.explainContext
      ? {
          searchProfile: snapshot.searchProfile,
          mode: snapshot.mode,
          fusionMethod: snapshot.explainContext.fusionMethod,
          fusionProfile: snapshot.explainContext.fusionProfile,
          fusionConfig: snapshot.explainContext.fusionConfig,
          lexicalQuery: snapshot.explainContext.lexicalQuery,
          semanticQuery: snapshot.explainContext.semanticQuery,
          query: snapshot.explainContext.query,
          excludeQuery: snapshot.explainContext.excludeQuery,
          rankingConfig: rankingConfigStatus,
          records: snapshot.explanations.slice(offset, offset + limit),
        }
      : undefined;

  return createSearchResultPage({
    searchProfile: snapshot.searchProfile,
    mode: snapshot.mode,
    sort: snapshot.sort,
    total: snapshot.records.length,
    offset,
    limit,
    records,
    explain,
  });
}

export function buildExplainContext(
  lexicalQuery: string,
  rawSemanticQuery: string,
  queryAnalysis: SearchQueryAnalysis | null,
  excludeQueryAnalysis: SearchQueryAnalysis | null,
  hybridFusion: HybridFusionContext | null,
  deps: RuntimeSearchDependencies,
): NonNullable<SearchWindowSnapshot["explainContext"]> {
  return {
    fusionMethod: hybridFusion ? ("weightedRrf" as const) : null,
    fusionProfile: hybridFusion?.profile ?? null,
    fusionConfig: buildFusionConfigSummary(
      hybridFusion?.profile ?? null,
      hybridFusion?.config ?? null,
      deps.rankingConfig,
    ),
    lexicalQuery,
    semanticQuery: rawSemanticQuery,
    query: queryAnalysis
      ? {
          rawQuery: queryAnalysis.rawQuery,
          normalizedQuery: queryAnalysis.normalizedQuery,
          queryTokens: queryAnalysis.queryTokens,
        }
      : null,
    excludeQuery: excludeQueryAnalysis
      ? {
          rawQuery: excludeQueryAnalysis.rawQuery,
          normalizedQuery: excludeQueryAnalysis.normalizedQuery,
          queryTokens: excludeQueryAnalysis.queryTokens,
        }
      : null,
  };
}
