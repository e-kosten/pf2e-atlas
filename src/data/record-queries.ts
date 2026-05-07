import { DatabaseSync } from "node:sqlite";

import type { RuleReferenceEdge } from "../domain/rule-types.js";
import type { PageReferenceEdge } from "../domain/page-relations-types.js";
import type { RecordKey } from "../domain/record-types.js";
import {
  buildCandidateCountQuery,
  buildCandidateKeyQuery,
  buildCandidateQuery,
  buildPagedCandidateQuery,
  buildSharedRecordSelectFields,
  buildLexicalRetrievalQuery,
  buildSemanticRetrievalQuery,
  SQLITE_VECTOR_QUERY_K_LIMIT,
} from "./backend/search-sql.js";
import type { NormalizedSearchFilters } from "../search/contracts.js";
import { buildPlaceholders, CandidateRow, ReferenceEdgeRow, sqliteRowCount } from "./rows.js";
import type { LexicalRetrievalRow, SemanticRetrievalRow } from "../search/ranking.js";
import type { SearchSort } from "../domain/search-types.js";
import { normalizeText } from "../shared/utils.js";

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function buildFtsQuery(query: string): string | null {
  const tokens = normalizeText(query).split(" ").filter(Boolean);
  if (tokens.length === 0) {
    return null;
  }

  return tokens.map((token) => `"${token}"*`).join(" OR ");
}

function resolveSemanticRetrievalLimit(baseLimit: number, filters: NormalizedSearchFilters): number {
  const boundedBaseLimit = Math.min(SQLITE_VECTOR_QUERY_K_LIMIT, Math.max(1, baseLimit));
  return filters.filter
    ? Math.min(SQLITE_VECTOR_QUERY_K_LIMIT, Math.min(1000, Math.max(boundedBaseLimit * 2, boundedBaseLimit + 50)))
    : boundedBaseLimit;
}

function buildRecordSelect(includeRaw = false): string {
  const fields = buildSharedRecordSelectFields();

  if (includeRaw) {
    fields.push("r.raw_json AS rawJson");
  }

  return `
    SELECT
      ${fields.join(",\n      ")}
    FROM records r
    LEFT JOIN actor_records a ON a.record_key = r.record_key
    LEFT JOIN item_records i ON i.record_key = r.record_key
    LEFT JOIN spell_records s ON s.record_key = r.record_key
  `;
}

export function fetchCandidates(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  includeSearchText = false,
  includeEmbedding = false,
  options: { recordKeys?: string[] } = {},
): CandidateRow[] {
  const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding, options);
  return db.prepare(sql).all(...params) as CandidateRow[];
}

export function fetchCandidateCount(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  options: { recordKeys?: string[] } = {},
): number {
  const { sql, params } = buildCandidateCountQuery(filters, options);
  return sqliteRowCount(db.prepare(sql).get(...params) as Record<string, unknown> | undefined);
}

export function fetchCandidateRecordKeys(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  sort?: Exclude<SearchSort, "ranked" | "random">,
  options: { recordKeys?: string[] } = {},
): string[] {
  const { sql, params } = buildCandidateKeyQuery(filters, sort, options);
  return (db.prepare(sql).all(...params) as Array<{ recordKey: string }>).map((row) => row.recordKey);
}

export function fetchPagedCandidates(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  sort: SearchSort,
  offset: number,
  limit: number,
): CandidateRow[] {
  const { sql, params } = buildPagedCandidateQuery(filters, sort, offset, limit);
  return db.prepare(sql).all(...params) as CandidateRow[];
}

export function fetchLexicalRetrievalRows(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  lexicalQuery: string,
  limit: number,
): LexicalRetrievalRow[] {
  const ftsQuery = buildFtsQuery(lexicalQuery);
  if (!ftsQuery) {
    return [];
  }

  const { sql, params } = buildLexicalRetrievalQuery(filters, ftsQuery, limit);
  return db.prepare(sql).all(...params) as LexicalRetrievalRow[];
}

export function fetchSemanticRetrievalRows(
  db: DatabaseSync,
  filters: NormalizedSearchFilters,
  queryVector: Float32Array,
  limit: number,
): SemanticRetrievalRow[] {
  if (queryVector.length === 0) {
    return [];
  }

  const encodedQuery = encodeVector(queryVector);
  const { sql, params } = buildSemanticRetrievalQuery(
    filters,
    resolveSemanticRetrievalLimit(limit, filters),
  );
  return db.prepare(sql).all(encodedQuery, ...params) as SemanticRetrievalRow[];
}

export function fetchRecordRowsByKeys(db: DatabaseSync, recordKeys: string[]): CandidateRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = buildPlaceholders(recordKeys);
  return db
    .prepare(
      `
        ${buildRecordSelect()}
        WHERE r.record_key IN (${placeholders})
      `,
    )
    .all(...recordKeys) as CandidateRow[];
}

export function fetchRecordRow(db: DatabaseSync, recordKeyOrPack: string, maybeId?: string): CandidateRow | undefined {
  if (maybeId) {
    return db
      .prepare(
        `
          ${buildRecordSelect(true)}
          WHERE r.pack_name = ? AND r.id = ?
        `,
      )
      .get(recordKeyOrPack, maybeId) as CandidateRow | undefined;
  }

  return db
    .prepare(
      `
        ${buildRecordSelect(true)}
        WHERE r.record_key = ?
      `,
    )
    .get(recordKeyOrPack) as CandidateRow | undefined;
}

export function fetchReferenceEdgeRows(
  db: DatabaseSync,
  direction: RuleReferenceEdge["direction"],
  recordKeys: string[],
  { coreOnly = false }: { coreOnly?: boolean } = {},
): ReferenceEdgeRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = buildPlaceholders(recordKeys);
  const targetFilter =
    direction === "outgoing"
      ? coreOnly
        ? "AND target.source_category = 'core'"
        : ""
      : coreOnly
        ? "AND re.from_source_category = 'core'"
        : "AND re.from_source_category IN ('core', 'rules')";
  const backlinkFilter =
    direction === "backlink"
      ? "AND (re.from_record_type = 'action' OR re.from_record_type = 'feat' OR LOWER(re.from_pack_name) = 'classfeatures')"
      : "";
  const keyColumn = direction === "outgoing" ? "re.from_record_key" : "re.to_record_key";

  return db
    .prepare(
      `
        SELECT
          re.from_record_key AS fromRecordKey,
          re.to_record_key AS toRecordKey,
          re.display_text AS displayText,
          re.reference_text AS referenceText,
          re.from_pack_name AS fromPackName,
          re.from_record_type AS fromRecordType,
          re.from_document_type AS fromDocumentType,
          re.from_source_category AS fromSourceCategory
        FROM reference_edges re
        JOIN records target ON target.record_key = re.to_record_key
        WHERE ${keyColumn} IN (${placeholders})
        ${targetFilter}
        ${backlinkFilter}
      `,
    )
    .all(...recordKeys) as ReferenceEdgeRow[];
}

export function fetchPageReferenceEdgeRows(
  db: DatabaseSync,
  direction: PageReferenceEdge["direction"],
  recordKeys: readonly RecordKey[],
): ReferenceEdgeRow[] {
  if (recordKeys.length === 0) {
    return [];
  }

  const placeholders = buildPlaceholders(recordKeys);
  const keyColumn = direction === "outgoing" ? "re.from_record_key" : "re.to_record_key";

  return db
    .prepare(
      `
        SELECT
          re.from_record_key AS fromRecordKey,
          re.to_record_key AS toRecordKey,
          re.display_text AS displayText,
          re.reference_text AS referenceText,
          re.from_pack_name AS fromPackName,
          re.from_record_type AS fromRecordType,
          re.from_document_type AS fromDocumentType,
          re.from_source_category AS fromSourceCategory
        FROM reference_edges re
        WHERE ${keyColumn} IN (${placeholders})
      `,
    )
    .all(...recordKeys) as ReferenceEdgeRow[];
}
