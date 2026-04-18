import type { EmbeddingProvider } from "../embeddings.js";
import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "../search-query-analysis.js";
import {
  recordMatchesFilters,
  semanticQueryLimit,
} from "./sql.js";
import {
  buildFusionConfigSummary,
  buildLexicalSignal,
  buildNormalizedRankScoreMap,
  buildRankMap,
  buildRerankAdjustments,
  compareOptionalRanks,
  computeWeightedRrfScore,
  LexicalRetrievalRow,
  packQualityScore,
  rarityPreferenceScore,
  resolveHybridFusionProfile,
  resolveSearchMode,
  resolveSearchProfile,
  scoreNameCandidate,
  SemanticRetrievalRow,
  sourcePenaltyScore,
  sourceQualityScore,
  sumRerankAdjustments,
} from "./ranking.js";
import {
  NormalizedRecord,
  SearchCountResult,
  SearchExplainResult,
  SearchFilters,
  SearchRecordExplanation,
  SearchResult,
  SearchSort,
} from "../types.js";
import { clampLimit, clampOffset, normalizeText } from "../utils.js";
import type { RankingConfig } from "./ranking-config.js";
import type { NormalizedSearchFilters } from "../data/service-types.js";
import type { CandidateRow } from "../data/rows.js";
import { rowToRecord } from "../data/rows.js";

const LOOKUP_LEXICAL_TOP_K = 100;

function nameScore(query: string, record: NormalizedRecord, aliases: string[] = []): number {
  let best = scoreNameCandidate(query, record.normalizedName);
  for (const alias of aliases) {
    best = Math.max(best, scoreNameCandidate(query, normalizeText(alias)));
  }
  return best;
}

function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return left.name.localeCompare(right.name) || left.packLabel.localeCompare(right.packLabel) || left.id.localeCompare(right.id);
}

function compareNullableLevel(left: number | null, right: number | null): number {
  if (left === null && right === null) {
    return 0;
  }
  if (left === null) {
    return 1;
  }
  if (right === null) {
    return -1;
  }
  return left - right;
}

function hashRecordSortSeed(recordKey: string, seed: number): number {
  let hash = seed | 0;
  for (let index = 0; index < recordKey.length; index += 1) {
    hash = Math.imul(hash ^ recordKey.charCodeAt(index), 16777619);
  }
  return hash >>> 0;
}

function compareRecordsForSort(
  left: NormalizedRecord,
  right: NormalizedRecord,
  sort: SearchSort,
  sortSeed: number,
): number {
  switch (sort) {
    case "levelAsc":
      return compareNullableLevel(left.level, right.level) || sortRecords(left, right);
    case "levelDesc":
      return compareNullableLevel(right.level, left.level) || sortRecords(left, right);
    case "random":
      return (
        hashRecordSortSeed(left.recordKey, sortSeed) - hashRecordSortSeed(right.recordKey, sortSeed) ||
        sortRecords(left, right)
      );
    case "alphabetical":
    case "ranked":
    default:
      return sortRecords(left, right);
  }
}

function createSearchResultPage(
  options: Omit<SearchResult, "hasMore" | "nextOffset">,
): SearchResult {
  const hasMore = options.offset + options.records.length < options.total;
  return {
    ...options,
    hasMore,
    nextOffset: hasMore ? options.offset + options.records.length : null,
  };
}

function createSearchCountResult(result: SearchResult): SearchCountResult {
  return {
    searchProfile: result.searchProfile,
    mode: result.mode,
    total: result.total,
  };
}

function buildFtsQuery(query: string): string | null {
  const tokens = normalizeText(query).split(" ").filter(Boolean);
  if (tokens.length === 0) {
    return null;
  }

  return tokens.map((token) => `"${token}"*`).join(" OR ");
}

function matchesExcludedQuery(searchText: string | null | undefined, excludedTokens: string[]): boolean {
  if (excludedTokens.length === 0) {
    return false;
  }

  const searchTokens = new Set(normalizeText(searchText ?? "").split(" ").filter(Boolean));
  return excludedTokens.some((token) => searchTokens.has(token));
}

type RuntimeSearchDependencies = {
  embeddingProvider: EmbeddingProvider;
  rankingConfig: RankingConfig;
  rankingConfigStatus: SearchExplainResult["rankingConfig"];
  decorateRecord: (record: NormalizedRecord) => NormalizedRecord;
  getAliases: (recordKey: string) => string[];
  fetchCandidates: (
    filters: NormalizedSearchFilters,
    includeSearchText?: boolean,
    includeEmbedding?: boolean,
    options?: { recordKeys?: string[] },
  ) => CandidateRow[];
  fetchLexicalRetrievalRows: (filters: NormalizedSearchFilters, ftsQuery: string, limit: number) => LexicalRetrievalRow[];
  fetchSemanticRetrievalRows: (
    filters: NormalizedSearchFilters,
    queryVector: Float32Array,
    limit: number,
  ) => SemanticRetrievalRow[];
};

export function searchStructured(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchResult {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  const sort = normalizedFilters.sort ?? "ranked";
  const sortSeed = normalizedFilters.sortSeed ?? 0;
  const candidates = deps.fetchCandidates(normalizedFilters);
  const scored = candidates
    .map((candidate) => {
      const record = deps.decorateRecord(rowToRecord(candidate));
      const packQuality = packQualityScore(record, deps.rankingConfig);
      const sourceQuality = sourceQualityScore(record, deps.rankingConfig);
      const rarityPreference = rarityPreferenceScore(record, normalizedFilters, deps.rankingConfig);
      const sourcePenalty = sourcePenaltyScore(record, normalizedFilters, deps.rankingConfig);
      const score =
        (normalizedFilters.nameQuery ? nameScore(normalizedFilters.nameQuery, record, deps.getAliases(record.recordKey)) : 0.5) +
        packQuality +
        sourceQuality +
        rarityPreference +
        sourcePenalty;

      return { record, score };
    })
    .filter(({ score }) => {
      if (normalizedFilters.nameQuery) {
        return score >= 0.2;
      }

      return true;
    })
    .sort((left, right) => {
      if (sort === "ranked") {
        return right.score - left.score || sortRecords(left.record, right.record);
      }
      return compareRecordsForSort(left.record, right.record, sort, sortSeed);
    });

  return createSearchResultPage({
    searchProfile,
    mode: "structured",
    sort,
    total: scored.length,
    offset,
    limit,
    records: scored.slice(offset, offset + limit).map(({ record }) => record),
  });
}

export function listRecords(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchResult {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const sort = normalizedFilters.sort === "ranked" || !normalizedFilters.sort ? "alphabetical" : normalizedFilters.sort;
  const sortSeed = normalizedFilters.sortSeed ?? 0;
  const records = deps.fetchCandidates(normalizedFilters).map((row) => deps.decorateRecord(rowToRecord(row)));
  records.sort((left, right) => compareRecordsForSort(left, right, sort, sortSeed));
  return createSearchResultPage({
    searchProfile: null,
    mode: "structured",
    sort,
    total: records.length,
    offset,
    limit,
    records: records.slice(offset, offset + limit),
  });
}

export async function search(
  filters: SearchFilters,
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): Promise<SearchResult> {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  const sort = normalizedFilters.sort ?? "ranked";
  const sortSeed = normalizedFilters.sortSeed ?? 0;
  const rawSemanticQuery = normalizedFilters.query?.trim() || "";
  const rawLexicalQuery = normalizedFilters.query?.trim() || normalizedFilters.nameQuery?.trim() || "";
  const rawExcludeQuery = normalizedFilters.excludeQuery?.trim() || "";
  const hybridFusion = resolveHybridFusionProfile(searchProfile, mode, deps.rankingConfig);
  const queryAnalysis = rawLexicalQuery
    ? buildSearchQueryAnalysis(rawLexicalQuery)
    : null;
  const excludeQueryAnalysis = rawExcludeQuery
    ? buildSearchQueryAnalysis(rawExcludeQuery)
    : null;
  const excludeTokens = excludeQueryAnalysis?.queryTokens ?? [];
  const literalQueryWeights = queryAnalysis
    ? buildLiteralQueryWeights(queryAnalysis)
    : null;
  const lexicalQuery = queryAnalysis?.normalizedQuery ?? rawLexicalQuery;
  const semanticVector = hybridFusion && rawSemanticQuery
    ? await deps.embeddingProvider.embed(rawSemanticQuery)
    : null;
  const lexicalRetrievalRows = lexicalQuery
    ? deps.fetchLexicalRetrievalRows(
        normalizedFilters,
        buildFtsQuery(lexicalQuery) ?? "",
        Math.max(mode === "lexical" ? LOOKUP_LEXICAL_TOP_K : (hybridFusion?.config.lexicalTopK ?? 0), (offset + limit) * 5),
      )
    : [];
  const lexicalRetrievedKeys = lexicalRetrievalRows.map((row) => row.recordKey);
  const lexicalRetrievalRanks = buildRankMap(lexicalRetrievedKeys);
  const lexicalMatches = buildNormalizedRankScoreMap(lexicalRetrievedKeys);

  const semanticRetrievalRows = semanticVector && hybridFusion
    ? deps.fetchSemanticRetrievalRows(
        normalizedFilters,
        semanticVector,
        semanticQueryLimit(Math.max(hybridFusion.config.semanticTopK, (offset + limit) * 5), normalizedFilters),
      )
    : [];
  const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
  const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

  const candidateKeys = mode === "structured"
    ? []
    : [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
  const candidateRows = mode === "structured"
    ? []
    : deps.fetchCandidates(normalizedFilters, excludeTokens.length > 0, false, { recordKeys: candidateKeys });
  const filteredCandidateRows = excludeTokens.length > 0
    ? candidateRows.filter((row) => !matchesExcludedQuery(row.searchText, excludeTokens))
    : candidateRows;
  const candidateRecords = filteredCandidateRows
    .map((row) => deps.decorateRecord(rowToRecord(row)))
    .filter((record) => recordMatchesFilters(record, normalizedFilters));
  const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

  const scored = (() => {
    if (mode === "structured") {
      const structuredRows = deps.fetchCandidates(normalizedFilters, excludeTokens.length > 0)
        .filter((row) => !excludeTokens.length || !matchesExcludedQuery(row.searchText, excludeTokens));
      return structuredRows
        .map((candidate) => {
          const record = deps.decorateRecord(rowToRecord(candidate));
          const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, deps.rankingConfig);
          const totalScore =
            (normalizedFilters.nameQuery
              ? nameScore(normalizedFilters.nameQuery, record, deps.getAliases(record.recordKey))
              : 0.5) +
            sumRerankAdjustments(rerankAdjustments);
          const explanation: SearchRecordExplanation = {
            recordKey: record.recordKey,
            name: record.name,
            totalScore,
            fusionScore: null,
            lexicalRank: null,
            semanticRank: null,
            lexicalRerankScore: null,
            matchedTraits: [],
            matchedNameTokens: [],
            rerankAdjustments,
          };

          return { record, totalScore, explanation };
        })
        .filter(({ totalScore }) => {
          if (normalizedFilters.nameQuery) {
            return totalScore >= 0.2;
          }

          return true;
        })
        .sort((left, right) => {
          if (sort === "ranked") {
            return right.totalScore - left.totalScore || sortRecords(left.record, right.record);
          }
          return compareRecordsForSort(left.record, right.record, sort, sortSeed);
        });
    }

    if (mode === "lexical") {
      return lexicalRetrievedKeys
        .map((recordKey) => candidatesByKey.get(recordKey))
        .filter((record): record is NormalizedRecord => Boolean(record))
        .map((record) => {
          const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, deps.rankingConfig);
          const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, deps.rankingConfig);
          const totalScore = lexicalSignal.lexicalScore + sumRerankAdjustments(rerankAdjustments);
          const explanation: SearchRecordExplanation = {
            recordKey: record.recordKey,
            name: record.name,
            totalScore,
            fusionScore: null,
            lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
            semanticRank: null,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            matchedTraits: lexicalSignal.matchedTraits,
            matchedNameTokens: lexicalSignal.matchedNameTokens,
            rerankAdjustments,
          };

          return {
            record,
            totalScore,
            lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            explanation,
          };
        })
        .filter(({ totalScore }) => {
          if (lexicalQuery) {
            return totalScore > 0;
          }

          return true;
        })
        .sort((left, right) => {
          if (sort === "ranked") {
            return (
              right.totalScore - left.totalScore ||
              right.lexicalRerankScore - left.lexicalRerankScore ||
              compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
              sortRecords(left.record, right.record)
            );
          }
          return compareRecordsForSort(left.record, right.record, sort, sortSeed);
        });
    }

    const fusionConfig = hybridFusion!.config;
    const rerankedLexical = lexicalRetrievedKeys
      .map((recordKey) => candidatesByKey.get(recordKey))
      .filter((record): record is NormalizedRecord => Boolean(record))
      .map((record) => ({
        record,
        lexicalSignal: buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, deps.rankingConfig),
      }))
      .filter(({ lexicalSignal }) => lexicalSignal.lexicalScore > 0)
      .sort((left, right) => {
        return (
          right.lexicalSignal.lexicalScore - left.lexicalSignal.lexicalScore ||
          compareOptionalRanks(
            semanticRetrievalRanks.get(left.record.recordKey) ?? null,
            semanticRetrievalRanks.get(right.record.recordKey) ?? null,
          ) ||
          sortRecords(left.record, right.record)
        );
      })
      .slice(0, fusionConfig.lexicalTopK);
    const rerankedLexicalRanks = buildRankMap(rerankedLexical.map(({ record }) => record.recordKey));
    const semanticRanks = buildRankMap(
      semanticRetrievedKeys
        .filter((recordKey) => candidatesByKey.has(recordKey))
        .slice(0, fusionConfig.semanticTopK),
    );

    return candidateRecords
      .filter((record) => rerankedLexicalRanks.has(record.recordKey) || semanticRanks.has(record.recordKey))
      .map((record) => {
        const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, deps.rankingConfig);
        const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, deps.rankingConfig);
        const lexicalRank = rerankedLexicalRanks.get(record.recordKey) ?? null;
        const semanticRank = semanticRanks.get(record.recordKey) ?? null;
        const fusionScore = computeWeightedRrfScore(
          lexicalRank,
          semanticRank,
          fusionConfig,
          deps.rankingConfig.hybridFusion.rrfK,
        );
        const totalScore = fusionScore + sumRerankAdjustments(rerankAdjustments);
        const explanation: SearchRecordExplanation = {
          recordKey: record.recordKey,
          name: record.name,
          totalScore,
          fusionScore,
          lexicalRank,
          semanticRank,
          lexicalRerankScore: lexicalSignal.lexicalScore,
          matchedTraits: lexicalSignal.matchedTraits,
          matchedNameTokens: lexicalSignal.matchedNameTokens,
          rerankAdjustments,
        };

        return {
          record,
          totalScore,
          fusionScore,
          lexicalRank,
          semanticRank,
          lexicalRerankScore: lexicalSignal.lexicalScore,
          explanation,
        };
      })
      .sort((left, right) => {
        if (sort === "ranked") {
          return (
            right.totalScore - left.totalScore ||
            right.fusionScore - left.fusionScore ||
            compareOptionalRanks(left.semanticRank, right.semanticRank) ||
            compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
            right.lexicalRerankScore - left.lexicalRerankScore ||
            sortRecords(left.record, right.record)
          );
        }
        return compareRecordsForSort(left.record, right.record, sort, sortSeed);
      });
  })();

  const page = scored.slice(offset, offset + limit);
  const explain: SearchExplainResult | undefined = filters.explain
    ? {
        searchProfile,
        mode,
        fusionMethod: hybridFusion ? "weightedRrf" : null,
        fusionProfile: hybridFusion?.profile ?? null,
        fusionConfig: buildFusionConfigSummary(hybridFusion?.profile ?? null, hybridFusion?.config ?? null, deps.rankingConfig),
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
        rankingConfig: deps.rankingConfigStatus,
        records: page.map(({ explanation }) => explanation),
      }
    : undefined;

  return createSearchResultPage({
    searchProfile,
    mode,
    sort,
    total: scored.length,
    offset,
    limit,
    records: page.map(({ record }) => record),
    explain,
  });
}

export function countStructuredSearch(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchCountResult {
  return createSearchCountResult(searchStructured(normalizedFilters, deps));
}

export function countSearchResults(
  filters: SearchFilters,
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): Promise<SearchCountResult> {
  return search(filters, normalizedFilters, deps).then((result) => createSearchCountResult(result));
}

export function lookup(
  name: string,
  filters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
  const results = searchStructured(filters, deps).records;

  return {
    match: results[0] ?? null,
    alternatives: results.slice(1),
  };
}
