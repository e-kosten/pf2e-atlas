import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import { DEFAULT_RANKING_CONFIG, type RankingConfigStore } from "../../search/ranking-config.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { RuntimeSearchDependencies, SearchCandidate } from "../../search/contracts.js";
import {
  fetchCandidateCount,
  fetchCandidates,
  fetchLexicalRetrievalRows,
  fetchPagedCandidates,
  fetchSemanticRetrievalRows,
} from "../record-queries.js";
import { rowToRecord } from "../rows.js";

type RuntimeSearchDependencyOptions = {
  db: DatabaseSync;
  embeddingProvider: EmbeddingProvider;
  rankingConfigStore: RankingConfigStore | null;
  decorateRecord: (record: NormalizedRecord) => NormalizedRecord;
  getAliases: (recordKey: string) => string[];
  getRankingConfigStatus: () => RuntimeSearchDependencies["rankingConfigStatus"];
};

export function createRuntimeSearchDependencies(options: RuntimeSearchDependencyOptions): RuntimeSearchDependencies {
  const toSearchCandidate =
    (searchText: string | null | undefined = null) =>
    (row: ReturnType<typeof fetchCandidates>[number]): SearchCandidate => ({
      record: options.decorateRecord(rowToRecord(row)),
      searchText,
    });

  const toSearchCandidateFromRow = (row: ReturnType<typeof fetchCandidates>[number]): SearchCandidate => ({
    record: options.decorateRecord(rowToRecord(row)),
    searchText: row.searchText,
  });

  const mapCandidates = (rows: ReturnType<typeof fetchCandidates>, includeSearchText: boolean): SearchCandidate[] =>
    rows.map(includeSearchText ? toSearchCandidateFromRow : toSearchCandidate());

  return {
    embeddingProvider: options.embeddingProvider,
    rankingConfig: options.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG,
    rankingConfigStatus: options.getRankingConfigStatus(),
    fetchCandidateCount: (filters, dependencyOptions = {}) =>
      fetchCandidateCount(options.db, filters, dependencyOptions),
    fetchPagedCandidates: (filters, sort, offset, limit) =>
      fetchPagedCandidates(options.db, filters, sort, offset, limit).map(toSearchCandidate()),
    getAliases: (recordKey) => options.getAliases(recordKey),
    fetchCandidates: (filters, includeSearchText = false, includeEmbedding = false, dependencyOptions = {}) =>
      mapCandidates(
        fetchCandidates(options.db, filters, includeSearchText, includeEmbedding, dependencyOptions),
        includeSearchText,
      ),
    fetchLexicalRetrievalRows: (filters, ftsQuery, limit) =>
      fetchLexicalRetrievalRows(options.db, filters, ftsQuery, limit),
    fetchSemanticRetrievalRows: (filters, queryVector, limit) =>
      fetchSemanticRetrievalRows(options.db, filters, queryVector, limit),
  };
}
