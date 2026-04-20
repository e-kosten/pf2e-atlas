import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "./query-analysis.js";
import { recordMatchesFilters, semanticQueryLimit } from "./sql.js";
import {
  buildFusionConfigSummary,
  buildLexicalSignal,
  buildNormalizedRankScoreMap,
  buildRankMap,
  buildRerankAdjustments,
  compareOptionalRanks,
  computeWeightedRrfScore,
  packQualityScore,
  rarityPreferenceScore,
  resolveHybridFusionProfile,
  resolveSearchMode,
  resolveSearchProfile,
  scoreNameCandidate,
  sourcePenaltyScore,
  sourceQualityScore,
  sumRerankAdjustments,
} from "./ranking.js";
import {
  NormalizedRecord,
  SearchCountResult,
  SearchExplainResult,
  SearchFilters,
  SearchMode,
  SearchProfile,
  SearchRecordExplanation,
  SearchResult,
  SearchSort,
} from "../types.js";
import { clampLimit, clampOffset, normalizeText } from "../utils.js";
import type { NormalizedSearchFilters, RuntimeSearchDependencies } from "./contracts.js";

const LOOKUP_LEXICAL_TOP_K = 100;

function nameScore(query: string, record: NormalizedRecord, aliases: string[] = []): number {
  let best = scoreNameCandidate(query, record.normalizedName);
  for (const alias of aliases) {
    best = Math.max(best, scoreNameCandidate(query, normalizeText(alias)));
  }
  return best;
}

function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return (
    left.name.localeCompare(right.name) ||
    left.packLabel.localeCompare(right.packLabel) ||
    left.id.localeCompare(right.id)
  );
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

function createSearchResultPage(options: Omit<SearchResult, "hasMore" | "nextOffset">): SearchResult {
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

  const searchTokens = new Set(
    normalizeText(searchText ?? "")
      .split(" ")
      .filter(Boolean),
  );
  return excludedTokens.some((token) => searchTokens.has(token));
}

type SearchWindowSnapshot = {
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

function createSnapshot(
  searchProfile: SearchProfile | null,
  mode: SearchMode,
  sort: SearchSort,
  entries: Array<{ record: NormalizedRecord; explanation: SearchRecordExplanation }>,
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

function sliceSnapshotToSearchResult(
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

export function buildStructuredSearchSnapshot(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchWindowSnapshot {
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  const sort = normalizedFilters.sort ?? "ranked";
  const sortSeed = normalizedFilters.sortSeed ?? 0;
  const entries = deps
    .fetchCandidates(normalizedFilters)
    .map(({ record }) => {
      const packQuality = packQualityScore(record, deps.rankingConfig);
      const sourceQuality = sourceQualityScore(record, deps.rankingConfig);
      const rarityPreference = rarityPreferenceScore(record, normalizedFilters, deps.rankingConfig);
      const sourcePenalty = sourcePenaltyScore(record, normalizedFilters, deps.rankingConfig);
      const totalScore =
        (normalizedFilters.nameQuery
          ? nameScore(normalizedFilters.nameQuery, record, deps.getAliases(record.recordKey))
          : 0.5) +
        packQuality +
        sourceQuality +
        rarityPreference +
        sourcePenalty;
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
        rerankAdjustments: {
          packQuality,
          sourceQuality,
          rarityPreference,
          sourcePenalty,
        },
      };

      return { record, totalScore, explanation };
    })
    .filter(({ totalScore }) => !normalizedFilters.nameQuery || totalScore >= 0.2)
    .sort((left, right) => {
      if (sort === "ranked") {
        return right.totalScore - left.totalScore || sortRecords(left.record, right.record);
      }
      return compareRecordsForSort(left.record, right.record, sort, sortSeed);
    })
    .map(({ record, explanation }) => ({ record, explanation }));

  return createSnapshot(searchProfile, "structured", sort, entries);
}

export async function buildSearchWindowSnapshot(
  filters: SearchFilters,
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): Promise<SearchWindowSnapshot> {
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  if (mode === "structured") {
    return buildStructuredSearchSnapshot(normalizedFilters, deps);
  }
  const sort = normalizedFilters.sort ?? "ranked";
  const sortSeed = normalizedFilters.sortSeed ?? 0;
  const rawSemanticQuery = normalizedFilters.query?.trim() || "";
  const rawLexicalQuery = normalizedFilters.query?.trim() || normalizedFilters.nameQuery?.trim() || "";
  const rawExcludeQuery = normalizedFilters.excludeQuery?.trim() || "";
  const hybridFusion = resolveHybridFusionProfile(searchProfile, mode, deps.rankingConfig);
  const queryAnalysis = rawLexicalQuery ? buildSearchQueryAnalysis(rawLexicalQuery) : null;
  const excludeQueryAnalysis = rawExcludeQuery ? buildSearchQueryAnalysis(rawExcludeQuery) : null;
  const excludeTokens = excludeQueryAnalysis?.queryTokens ?? [];
  const literalQueryWeights = queryAnalysis ? buildLiteralQueryWeights(queryAnalysis) : null;
  const lexicalQuery = queryAnalysis?.normalizedQuery ?? rawLexicalQuery;
  const semanticVector = hybridFusion && rawSemanticQuery ? await deps.embeddingProvider.embed(rawSemanticQuery) : null;
  const candidateCount = Math.max(1, deps.fetchCandidateCount(normalizedFilters));
  const lexicalRetrievalRows = lexicalQuery
    ? deps.fetchLexicalRetrievalRows(
        normalizedFilters,
        buildFtsQuery(lexicalQuery) ?? "",
        Math.max(mode === "lexical" ? LOOKUP_LEXICAL_TOP_K : (hybridFusion?.config.lexicalTopK ?? 0), candidateCount),
      )
    : [];
  const lexicalRetrievedKeys = lexicalRetrievalRows.map((row) => row.recordKey);
  const lexicalRetrievalRanks = buildRankMap(lexicalRetrievedKeys);
  const lexicalMatches = buildNormalizedRankScoreMap(lexicalRetrievedKeys);

  const semanticRetrievalRows =
    semanticVector && hybridFusion
      ? deps.fetchSemanticRetrievalRows(
          normalizedFilters,
          semanticVector,
          semanticQueryLimit(Math.max(hybridFusion.config.semanticTopK, candidateCount), normalizedFilters),
        )
      : [];
  const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
  const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

  const candidateKeys = [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
  const candidateRows = deps.fetchCandidates(normalizedFilters, excludeTokens.length > 0, false, {
    recordKeys: candidateKeys,
  });
  const filteredCandidateRows =
    excludeTokens.length > 0
      ? candidateRows.filter((candidate) => !matchesExcludedQuery(candidate.searchText, excludeTokens))
      : candidateRows;
  const candidateRecords = filteredCandidateRows
    .map(({ record }) => record)
    .filter((record) => recordMatchesFilters(record, normalizedFilters));
  const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

  const entries = (() => {
    if (mode === "lexical") {
      return lexicalRetrievedKeys
        .map((recordKey) => candidatesByKey.get(recordKey))
        .filter((record): record is NormalizedRecord => Boolean(record))
        .map((record) => {
          const lexicalSignal = buildLexicalSignal(
            record,
            lexicalQuery,
            literalQueryWeights,
            lexicalMatches,
            deps.rankingConfig,
          );
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
        .filter(({ totalScore }) => !lexicalQuery || totalScore > 0)
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
        })
        .map(({ record, explanation }) => ({ record, explanation }));
    }

    const fusionConfig = hybridFusion!.config;
    const rerankedLexical = lexicalRetrievedKeys
      .map((recordKey) => candidatesByKey.get(recordKey))
      .filter((record): record is NormalizedRecord => Boolean(record))
      .map((record) => ({
        record,
        lexicalSignal: buildLexicalSignal(
          record,
          lexicalQuery,
          literalQueryWeights,
          lexicalMatches,
          deps.rankingConfig,
        ),
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
      semanticRetrievedKeys.filter((recordKey) => candidatesByKey.has(recordKey)).slice(0, fusionConfig.semanticTopK),
    );

    return candidateRecords
      .filter((record) => rerankedLexicalRanks.has(record.recordKey) || semanticRanks.has(record.recordKey))
      .map((record) => {
        const lexicalSignal = buildLexicalSignal(
          record,
          lexicalQuery,
          literalQueryWeights,
          lexicalMatches,
          deps.rankingConfig,
        );
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
      })
      .map(({ record, explanation }) => ({ record, explanation }));
  })();

  const explainContext = {
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

  return createSnapshot(searchProfile, mode, sort, entries, explainContext);
}

export function searchStructured(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchResult {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const snapshot = buildStructuredSearchSnapshot(normalizedFilters, deps);
  return sliceSnapshotToSearchResult(snapshot, offset, limit, false, deps.rankingConfigStatus);
}

export function listRecords(normalizedFilters: NormalizedSearchFilters, deps: RuntimeSearchDependencies): SearchResult {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const sort = normalizedFilters.sort === "ranked" || !normalizedFilters.sort ? "alphabetical" : normalizedFilters.sort;
  if (sort === "random") {
    const sortSeed = normalizedFilters.sortSeed ?? 0;
    const records = deps.fetchCandidates(normalizedFilters).map(({ record }) => record);
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
  const total = deps.fetchCandidateCount(normalizedFilters);
  const records = deps
    .fetchPagedCandidates(normalizedFilters, sort, offset, limit)
    .map(({ record }) => record);
  return createSearchResultPage({
    searchProfile: null,
    mode: "structured",
    sort,
    total,
    offset,
    limit,
    records,
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
  const queryAnalysis = rawLexicalQuery ? buildSearchQueryAnalysis(rawLexicalQuery) : null;
  const excludeQueryAnalysis = rawExcludeQuery ? buildSearchQueryAnalysis(rawExcludeQuery) : null;
  const excludeTokens = excludeQueryAnalysis?.queryTokens ?? [];
  const literalQueryWeights = queryAnalysis ? buildLiteralQueryWeights(queryAnalysis) : null;
  const lexicalQuery = queryAnalysis?.normalizedQuery ?? rawLexicalQuery;
  const semanticVector = hybridFusion && rawSemanticQuery ? await deps.embeddingProvider.embed(rawSemanticQuery) : null;
  const lexicalRetrievalRows = lexicalQuery
    ? deps.fetchLexicalRetrievalRows(
        normalizedFilters,
        buildFtsQuery(lexicalQuery) ?? "",
        Math.max(
          mode === "lexical" ? LOOKUP_LEXICAL_TOP_K : (hybridFusion?.config.lexicalTopK ?? 0),
          (offset + limit) * 5,
        ),
      )
    : [];
  const lexicalRetrievedKeys = lexicalRetrievalRows.map((row) => row.recordKey);
  const lexicalRetrievalRanks = buildRankMap(lexicalRetrievedKeys);
  const lexicalMatches = buildNormalizedRankScoreMap(lexicalRetrievedKeys);

  const semanticRetrievalRows =
    semanticVector && hybridFusion
      ? deps.fetchSemanticRetrievalRows(
          normalizedFilters,
          semanticVector,
          semanticQueryLimit(Math.max(hybridFusion.config.semanticTopK, (offset + limit) * 5), normalizedFilters),
        )
      : [];
  const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
  const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

  const candidateKeys = mode === "structured" ? [] : [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
  const candidateRows =
    mode === "structured"
      ? []
      : deps.fetchCandidates(normalizedFilters, excludeTokens.length > 0, false, { recordKeys: candidateKeys });
  const filteredCandidateRows =
    excludeTokens.length > 0
      ? candidateRows.filter((candidate) => !matchesExcludedQuery(candidate.searchText, excludeTokens))
      : candidateRows;
  const candidateRecords = filteredCandidateRows
    .map(({ record }) => record)
    .filter((record) => recordMatchesFilters(record, normalizedFilters));
  const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

  const scored = (() => {
    if (mode === "structured") {
      const structuredRows = deps
        .fetchCandidates(normalizedFilters, excludeTokens.length > 0)
        .filter((candidate) => !excludeTokens.length || !matchesExcludedQuery(candidate.searchText, excludeTokens));
      return structuredRows
        .map(({ record }) => {
          const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, deps.rankingConfig);
          const totalScore =
            (normalizedFilters.nameQuery
              ? nameScore(normalizedFilters.nameQuery, record, deps.getAliases(record.recordKey))
              : 0.5) + sumRerankAdjustments(rerankAdjustments);
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
        .filter(({ totalScore }) => !normalizedFilters.nameQuery || totalScore >= 0.2)
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
          const lexicalSignal = buildLexicalSignal(
            record,
            lexicalQuery,
            literalQueryWeights,
            lexicalMatches,
            deps.rankingConfig,
          );
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
        .filter(({ totalScore }) => !lexicalQuery || totalScore > 0)
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
        lexicalSignal: buildLexicalSignal(
          record,
          lexicalQuery,
          literalQueryWeights,
          lexicalMatches,
          deps.rankingConfig,
        ),
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
      semanticRetrievedKeys.filter((recordKey) => candidatesByKey.has(recordKey)).slice(0, fusionConfig.semanticTopK),
    );

    return candidateRecords
      .filter((record) => rerankedLexicalRanks.has(record.recordKey) || semanticRanks.has(record.recordKey))
      .map((record) => {
        const lexicalSignal = buildLexicalSignal(
          record,
          lexicalQuery,
          literalQueryWeights,
          lexicalMatches,
          deps.rankingConfig,
        );
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
  if (!normalizedFilters.nameQuery?.trim()) {
    return {
      searchProfile: resolveSearchProfile(normalizedFilters, "search", resolveSearchMode(normalizedFilters, "search")),
      mode: resolveSearchMode(normalizedFilters, "search"),
      total: deps.fetchCandidateCount(normalizedFilters),
    };
  }
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
