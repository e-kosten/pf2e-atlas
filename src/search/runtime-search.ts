import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "./query-analysis.js";
import { recordMatchesFilters } from "./filters/scope.js";
import {
  buildLexicalSignal,
  buildNormalizedRankScoreMap,
  buildRankMap,
  buildRerankAdjustments,
  compareOptionalRanks,
  computeWeightedRrfScore,
  resolveHybridFusionProfile,
  resolveSearchMode,
  resolveSearchProfile,
  sumRerankAdjustments,
} from "./ranking.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type {
  SearchCountResult,
  SearchRecordExplanation,
  SearchResult,
} from "../domain/search-types.js";
import type { NormalizedSearchFilters, RuntimeSearchDependencies, SearchExecutionFilters } from "./contracts.js";
import { clampLimit, clampOffset } from "./primitives.js";
import { compareRecordsForSort, sortRecords } from "./runtime-search-sorting.js";
import {
  buildExplainContext,
  createSearchCountResult,
  createSearchResultPage,
  createSnapshot,
  sliceSnapshotToSearchResult,
} from "./runtime-search-snapshot.js";
import { buildStructuredSearchEntries, matchesExcludedQuery } from "./runtime-search-structured.js";
import type { SearchWindowSnapshot } from "./runtime-search-snapshot.js";
import { traceAsync, traceSync } from "./trace.js";
import { SEARCH_EXECUTION_VOCABULARY } from "./contracts.js";

const LOOKUP_LEXICAL_TOP_K = 100;
const SORT_KIND_RANKED = SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANKED;

export function buildStructuredSearchSnapshot(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): SearchWindowSnapshot {
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  const sort = normalizedFilters.sort ?? SORT_KIND_RANKED;
  const entries = buildStructuredSearchEntries(normalizedFilters, deps);

  return createSnapshot(searchProfile, SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED, sort, entries);
}

export async function buildSearchWindowSnapshot(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): Promise<SearchWindowSnapshot> {
  const mode = resolveSearchMode(normalizedFilters, "search");
  const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
  const sort = normalizedFilters.sort ?? SORT_KIND_RANKED;
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
  const explainContext = buildExplainContext(
    lexicalQuery,
    rawSemanticQuery,
    queryAnalysis,
    excludeQueryAnalysis,
    hybridFusion,
    deps,
  );

  if (mode === SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED) {
    return createSnapshot(
      searchProfile,
      mode,
      sort,
      buildStructuredSearchEntries(normalizedFilters, deps, excludeTokens),
      explainContext,
    );
  }
  const semanticVector =
    hybridFusion && rawSemanticQuery
      ? await traceAsync(
          deps.trace,
          "search.embedQuery",
          { queryLength: rawSemanticQuery.length },
          () => deps.embeddingProvider.embed(rawSemanticQuery),
          (vector) => ({ dimensions: vector.length }),
        )
      : null;
  const candidateCount = Math.max(1, deps.fetchCandidateCount(normalizedFilters));
  const lexicalRetrievalRows = lexicalQuery
    ? deps.fetchLexicalRetrievalRows(
        normalizedFilters,
        lexicalQuery,
        Math.max(
          mode === SEARCH_EXECUTION_VOCABULARY.MODE.LEXICAL
            ? LOOKUP_LEXICAL_TOP_K
            : (hybridFusion?.config.lexicalTopK ?? 0),
          candidateCount,
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
        hybridFusion.config.semanticTopK,
      )
      : [];
  const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
  const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

  const candidateKeys = [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
  const candidateRows = deps.fetchCandidates(normalizedFilters, excludeTokens.length > 0, false, {
    recordKeys: candidateKeys,
  });
  const filteredCandidateRows = traceSync(
    deps.trace,
    "search.filterExcludedCandidates",
    { rows: candidateRows.length, excludeTokens: excludeTokens.length },
    () =>
      excludeTokens.length > 0
        ? candidateRows.filter((candidate) => !matchesExcludedQuery(candidate.searchText, excludeTokens))
        : candidateRows,
    (rows) => ({ rows: rows.length }),
  );
  const candidateRecords = traceSync(
    deps.trace,
    "search.filterCandidateRecords",
    { rows: filteredCandidateRows.length },
    () =>
      filteredCandidateRows
        .map(({ record }) => record)
        .filter((record) => recordMatchesFilters(record, normalizedFilters)),
    (records) => ({ records: records.length }),
  );
  const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

  const entries = traceSync(
    deps.trace,
    "search.rankWindowEntries",
    {
      mode,
      candidates: candidateRecords.length,
      lexicalRows: lexicalRetrievalRows.length,
      semanticRows: semanticRetrievalRows.length,
    },
    () => {
    if (mode === SEARCH_EXECUTION_VOCABULARY.MODE.LEXICAL) {
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
    if (sort === SORT_KIND_RANKED) {
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
        if (sort === SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANKED) {
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
    },
    (rankedEntries) => ({ entries: rankedEntries.length }),
  );

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
  const sort =
    normalizedFilters.sort === SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANKED || !normalizedFilters.sort
      ? SEARCH_EXECUTION_VOCABULARY.SORT_KIND.ALPHABETICAL
      : normalizedFilters.sort;
  if (sort === SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANDOM) {
    const sortSeed = normalizedFilters.sortSeed ?? 0;
    const records = deps.fetchCandidates(normalizedFilters).map(({ record }) => record);
    records.sort((left, right) => compareRecordsForSort(left, right, sort, sortSeed));
    return createSearchResultPage({
      searchProfile: null,
      mode: SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED,
      sort,
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    });
  }
  const total = deps.fetchCandidateCount(normalizedFilters);
  const records = deps.fetchPagedCandidates(normalizedFilters, sort, offset, limit).map(({ record }) => record);
  return createSearchResultPage({
    searchProfile: null,
    mode: SEARCH_EXECUTION_VOCABULARY.MODE.STRUCTURED,
    sort,
    total,
    offset,
    limit,
    records,
  });
}

export async function search(
  filters: SearchExecutionFilters,
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
): Promise<SearchResult> {
  const limit = clampLimit(normalizedFilters.limit);
  const offset = clampOffset(normalizedFilters.offset);
  const snapshot = await buildSearchWindowSnapshot(normalizedFilters, deps);
  return sliceSnapshotToSearchResult(snapshot, offset, limit, Boolean(filters.explain), deps.rankingConfigStatus);
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
  filters: SearchExecutionFilters,
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
