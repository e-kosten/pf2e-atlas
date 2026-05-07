import type { DatabaseSync } from "node:sqlite";

import type { NormalizedRecord } from "../../domain/record-types.js";
import type { SearchCandidate, SearchRetrievalPort } from "../../search/contracts.js";
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

type SearchRetrievalPortOptions = {
  db: DatabaseSync;
  decorateRecord: (record: NormalizedRecord) => NormalizedRecord;
  trace?: SearchTraceSink;
};

export function createSearchRetrievalPort(options: SearchRetrievalPortOptions): SearchRetrievalPort {
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
    fetchLexicalRetrievalRows: (filters, lexicalQuery, limit) =>
      traceSync(
        options.trace,
        "sql.fetchLexicalRetrievalRows",
        { limit, queryLength: lexicalQuery.length },
        () => fetchLexicalRetrievalRows(options.db, filters, lexicalQuery, limit),
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
