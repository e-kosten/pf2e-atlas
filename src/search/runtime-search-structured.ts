import type { SearchRecordExplanation } from "../domain/search-types.js";
import { normalizeText } from "../shared/utils.js";
import type { NormalizedSearchFilters, RuntimeSearchDependencies } from "./contracts.js";
import { buildRerankAdjustments, sumRerankAdjustments } from "./ranking.js";
import { compareRecordsForSort, nameScore, sortRecords } from "./runtime-search-sorting.js";
import type { RuntimeSearchEntry } from "./runtime-search-snapshot.js";
import { SEARCH_EXECUTION_VOCABULARY } from "./contracts.js";

export function matchesExcludedQuery(searchText: string | null | undefined, excludedTokens: string[]): boolean {
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

export function buildStructuredSearchEntries(
  normalizedFilters: NormalizedSearchFilters,
  deps: RuntimeSearchDependencies,
  excludeTokens: string[] = [],
): RuntimeSearchEntry[] {
  const sort = normalizedFilters.sort ?? SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANKED;
  const sortSeed = normalizedFilters.sortSeed ?? 0;
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
      if (sort === SEARCH_EXECUTION_VOCABULARY.SORT_KIND.RANKED) {
        return right.totalScore - left.totalScore || sortRecords(left.record, right.record);
      }
      return compareRecordsForSort(left.record, right.record, sort, sortSeed);
    })
    .map(({ record, explanation }) => ({ record, explanation }));
}
