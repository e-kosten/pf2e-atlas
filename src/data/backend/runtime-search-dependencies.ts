import type { DatabaseSync } from "node:sqlite";

import type { EmbeddingProvider } from "../../embeddings.js";
import { DEFAULT_RANKING_CONFIG, type RankingConfigStore } from "../../search/ranking-config.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { RuntimeSearchDependencies, SearchCandidate } from "../../search/contracts.js";
import type { SearchTraceSink } from "../../search/trace.js";
import { traceSync } from "../../search/trace.js";
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
  trace?: SearchTraceSink;
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
    trace: options.trace,
    fetchCandidateCount: (filters, dependencyOptions = {}) =>
      traceSync(
        options.trace,
        "sql.fetchCandidateCount",
        { recordKeys: dependencyOptions.recordKeys?.length ?? 0 },
        () => fetchCandidateCount(options.db, filters, dependencyOptions),
        (count) => ({ count }),
      ),
    fetchPagedCandidates: (filters, sort, offset, limit) =>
      traceSync(
        options.trace,
        "sql.fetchPagedCandidates",
        { sort, offset, limit },
        () => fetchPagedCandidates(options.db, filters, sort, offset, limit),
        (rows) => ({ rows: rows.length }),
      ).map(toSearchCandidate()),
    getAliases: (recordKey) => options.getAliases(recordKey),
    fetchCandidates: (filters, includeSearchText = false, includeEmbedding = false, dependencyOptions = {}) =>
      mapCandidates(
        traceSync(
          options.trace,
          "sql.fetchCandidates",
          {
            includeSearchText,
            includeEmbedding,
            recordKeys: dependencyOptions.recordKeys?.length ?? 0,
          },
          () => fetchCandidates(options.db, filters, includeSearchText, includeEmbedding, dependencyOptions),
          (rows) => ({ rows: rows.length }),
        ),
        includeSearchText,
      ),
    fetchLexicalRetrievalRows: (filters, ftsQuery, limit) =>
      traceSync(
        options.trace,
        "sql.fetchLexicalRetrievalRows",
        { limit, queryLength: ftsQuery.length },
        () => fetchLexicalRetrievalRows(options.db, filters, ftsQuery, limit),
        (rows) => ({ rows: rows.length }),
      ),
    fetchSemanticRetrievalRows: (filters, queryVector, limit) =>
      traceSync(
        options.trace,
        "sql.fetchSemanticRetrievalRows",
        { limit, dimensions: queryVector.length },
        () => fetchSemanticRetrievalRows(options.db, filters, queryVector, limit),
        (rows) => ({ rows: rows.length }),
      ),
  };
}
