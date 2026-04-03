import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../types.js";
import { tokenize } from "../search/ranking.js";
import { normalizeText } from "../utils.js";
import { normalizeDerivedTag } from "./index.js";

const DEFAULT_CANDIDATE_LIMIT = 25;
const DEFAULT_EXEMPLAR_LIMIT = 8;
const DEFAULT_CONTRAST_LIMIT = 6;
const DEFAULT_COMMON_TRAIT_LIMIT = 8;
const DEFAULT_SHARED_TOKEN_LIMIT = 8;
const DEFAULT_SHARED_PHRASE_LIMIT = 6;
const DEFAULT_CANDIDATE_EVIDENCE_LIMIT = 12;
const SIMILARITY_BUCKETS = [0.9, 0.85, 0.8];

const STOPWORDS = new Set([
  "about",
  "after",
  "also",
  "among",
  "and",
  "another",
  "around",
  "because",
  "become",
  "becomes",
  "before",
  "being",
  "between",
  "beyond",
  "both",
  "can",
  "creature",
  "creatures",
  "during",
  "each",
  "from",
  "gain",
  "gains",
  "have",
  "help",
  "helps",
  "into",
  "just",
  "like",
  "make",
  "makes",
  "many",
  "more",
  "most",
  "often",
  "other",
  "over",
  "such",
  "than",
  "that",
  "their",
  "them",
  "they",
  "this",
  "those",
  "through",
  "toward",
  "under",
  "until",
  "when",
  "where",
  "which",
  "while",
  "with",
  "within",
  "without",
]);

export type SemanticDiscoveryRecord = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  traits: string[];
  derivedTags: string[];
  descriptionText: string | null;
  vector: Float32Array;
};

export type ResolvedSemanticExemplar = {
  query: string;
  matchedBy: "recordKey" | "name" | "alias";
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  traits: string[];
};

export type SemanticDiscoveryCandidate = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  similarity: number;
  traits: string[];
  sharedTraits: string[];
  derivedTags: string[];
  descriptionText: string | null;
};

export type SemanticDiscoveryEvidenceTerm = {
  value: string;
  support: number;
  exemplarSupport: number;
  candidateSupport: number;
};

export type SemanticDiscoverySimilarityBucket = {
  minSimilarity: number;
  count: number;
};

export type SemanticDiscoveryResult = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  exemplarCount: number;
  candidateCount: number;
  matchedCandidateCount: number;
  commonTraits: string[];
  sharedTokens: SemanticDiscoveryEvidenceTerm[];
  sharedPhrases: SemanticDiscoveryEvidenceTerm[];
  similarityBuckets: SemanticDiscoverySimilarityBucket[];
  resolvedExemplars: ResolvedSemanticExemplar[];
  exemplars: Array<{
    name: string;
    recordKey: string;
    level: number | null;
    traits: string[];
    similarityToCentroid: number;
  }>;
  candidates: SemanticDiscoveryCandidate[];
  contrastRecords: SemanticDiscoveryCandidate[];
};

export type SemanticDiscoveryOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  exemplarNames?: string[];
  exemplarRecordKeys?: string[];
  limit?: number;
  exemplarLimit?: number;
  contrastLimit?: number;
  commonTraitLimit?: number;
  sharedTokenLimit?: number;
  sharedPhraseLimit?: number;
  candidateEvidenceLimit?: number;
  minSimilarity?: number;
  excludeDerivedTag?: string;
};

type LoadedDiscoveryRow = {
  query?: string;
  recordKey: string;
  name: string;
  category: string;
  subcategory: string | null;
  level: number | bigint | null;
  traitsJson: string;
  derivedTagsJson: string;
  descriptionText: string | null;
  vectorBlob: Uint8Array;
  matchedBy?: string | null;
};

type DiscoveryScope = {
  category: SearchCategory;
  subcategory?: SearchSubcategory;
};

export function discoverSemanticCandidates(
  db: DatabaseSync,
  options: SemanticDiscoveryOptions,
): SemanticDiscoveryResult {
  const resolvedByKey = (options.exemplarRecordKeys ?? []).map((recordKey) =>
    resolveExemplarByRecordKey(db, recordKey, options.category, options.subcategory)
  );
  const resolvedByName = (options.exemplarNames ?? []).map((name) =>
    resolveExemplarByName(db, name, options.category, options.subcategory)
  );
  const resolvedExemplars = [...resolvedByKey, ...resolvedByName];
  if (resolvedExemplars.length === 0) {
    throw new Error("Provide at least one exemplar via --name or --record-key.");
  }

  const scope = resolveDiscoveryScope(options, resolvedExemplars);
  const exemplarRecordKeys = new Set(resolvedExemplars.map((record) => record.recordKey));
  const candidateRecords = loadCandidateRecords(db, scope, {
    excludeRecordKeys: exemplarRecordKeys,
    excludeDerivedTag: options.excludeDerivedTag,
  });

  return rankSemanticDiscoveryCandidates(
    resolvedExemplars.map(toSemanticDiscoveryRecord),
    candidateRecords,
    {
      ...options,
      category: scope.category,
      subcategory: scope.subcategory,
    },
    resolvedExemplars.map(toResolvedSemanticExemplar),
  );
}

export function rankSemanticDiscoveryCandidates(
  exemplars: SemanticDiscoveryRecord[],
  candidates: SemanticDiscoveryRecord[],
  options: SemanticDiscoveryOptions,
  resolvedExemplars: ResolvedSemanticExemplar[] = exemplars.map((record) => ({
    query: record.recordKey,
    matchedBy: "recordKey",
    recordKey: record.recordKey,
    name: record.name,
    category: record.category,
    subcategory: record.subcategory,
    level: record.level,
    traits: [...record.traits],
  })),
): SemanticDiscoveryResult {
  if (exemplars.length === 0) {
    throw new Error("Cannot run semantic discovery without at least one exemplar.");
  }

  const vectorDimensions = exemplars.find((record) => record.vector.length > 0)?.vector.length ?? 0;
  if (vectorDimensions === 0) {
    throw new Error("Semantic discovery requires at least one exemplar with a valid embedding vector.");
  }

  const validExemplars = exemplars.filter((record) => record.vector.length === vectorDimensions);
  const validCandidates = candidates.filter((record) => record.vector.length === vectorDimensions);
  if (validExemplars.length === 0) {
    throw new Error("No exemplars had a usable embedding vector.");
  }

  const centroid = normalizeVector(averageVectors(validExemplars.map((record) => record.vector)));
  const limit = clampPositiveInteger(options.limit, DEFAULT_CANDIDATE_LIMIT, 200);
  const exemplarLimit = clampPositiveInteger(options.exemplarLimit, DEFAULT_EXEMPLAR_LIMIT, 25);
  const contrastLimit = clampPositiveInteger(options.contrastLimit, DEFAULT_CONTRAST_LIMIT, 25);
  const commonTraitLimit = clampPositiveInteger(options.commonTraitLimit, DEFAULT_COMMON_TRAIT_LIMIT, 25);
  const sharedTokenLimit = clampPositiveInteger(options.sharedTokenLimit, DEFAULT_SHARED_TOKEN_LIMIT, 25);
  const sharedPhraseLimit = clampPositiveInteger(options.sharedPhraseLimit, DEFAULT_SHARED_PHRASE_LIMIT, 25);
  const candidateEvidenceLimit = clampPositiveInteger(options.candidateEvidenceLimit, DEFAULT_CANDIDATE_EVIDENCE_LIMIT, 50);
  const minSimilarity = options.minSimilarity ?? Number.NEGATIVE_INFINITY;
  const commonTraits = collectCommonTraits(validExemplars, commonTraitLimit);

  const rankedCandidates = validCandidates
    .map((candidate) => ({
      recordKey: candidate.recordKey,
      name: candidate.name,
      category: candidate.category,
      subcategory: candidate.subcategory,
      level: candidate.level,
      similarity: cosineSimilarity(candidate.vector, centroid),
      traits: [...candidate.traits],
      sharedTraits: commonTraits.filter((trait) => candidate.traits.includes(trait)),
      derivedTags: [...candidate.derivedTags],
      descriptionText: candidate.descriptionText,
    }))
    .sort((left, right) => right.similarity - left.similarity || left.name.localeCompare(right.name));

  const matchedCandidates = rankedCandidates.filter((candidate) => candidate.similarity >= minSimilarity);
  const topCandidates = matchedCandidates.slice(0, limit);
  const evidencePool = matchedCandidates.slice(0, candidateEvidenceLimit);
  const { tokens, phrases } = collectSharedEvidence(validExemplars, evidencePool, {
    sharedTokenLimit,
    sharedPhraseLimit,
  });
  const similarityBuckets = SIMILARITY_BUCKETS.map((bucket) => ({
    minSimilarity: bucket,
    count: rankedCandidates.filter((candidate) => candidate.similarity >= bucket).length,
  }));
  const candidateKeys = new Set(topCandidates.map((candidate) => candidate.recordKey));
  const contrastMinSimilarity = Number.isFinite(minSimilarity)
    ? Math.max(0, minSimilarity - 0.05)
    : 0.6;
  const contrastRecords = pickContrastRecords(rankedCandidates, candidateKeys, commonTraits, contrastMinSimilarity, contrastLimit);
  const representativeExemplars = validExemplars
    .map((record) => ({
      name: record.name,
      recordKey: record.recordKey,
      level: record.level,
      traits: [...record.traits],
      similarityToCentroid: cosineSimilarity(record.vector, centroid),
    }))
    .sort((left, right) => right.similarityToCentroid - left.similarityToCentroid || left.name.localeCompare(right.name))
    .slice(0, exemplarLimit);
  const category = options.category ?? validExemplars[0]!.category;

  return {
    category,
    subcategory: options.subcategory ?? null,
    exemplarCount: validExemplars.length,
    candidateCount: rankedCandidates.length,
    matchedCandidateCount: matchedCandidates.length,
    commonTraits,
    sharedTokens: tokens,
    sharedPhrases: phrases,
    similarityBuckets,
    resolvedExemplars,
    exemplars: representativeExemplars,
    candidates: topCandidates,
    contrastRecords,
  };
}

function resolveDiscoveryScope(
  options: SemanticDiscoveryOptions,
  exemplars: LoadedDiscoveryRow[],
): DiscoveryScope {
  const category = options.category ?? inferSingleValue(exemplars.map((record) => record.category as SearchCategory), "category");
  const subcategory = options.subcategory;

  return {
    category,
    subcategory,
  };
}

function inferSingleValue<T>(values: T[], label: string): T {
  const uniqueValues = [...new Set(values)];
  if (uniqueValues.length !== 1) {
    throw new Error(`Exemplars span multiple ${label} values. Pass --${label} explicitly to define the candidate scope.`);
  }

  return uniqueValues[0]!;
}

function resolveExemplarByRecordKey(
  db: DatabaseSync,
  recordKey: string,
  category?: SearchCategory,
  subcategory?: SearchSubcategory,
): LoadedDiscoveryRow {
  const sql = [
    "SELECT",
    "  r.record_key AS recordKey,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.derived_tags_json AS derivedTagsJson,",
    "  r.description_text AS descriptionText,",
    "  e.vector_blob AS vectorBlob,",
    "  'recordKey' AS matchedBy",
    "FROM records r",
    "JOIN embeddings e ON e.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
    "AND r.record_key = ?",
  ];
  const params: Array<string> = [recordKey];
  if (category) {
    sql.push("AND r.category = ?");
    params.push(category);
  }
  if (subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(subcategory);
  }

  const row = db.prepare(sql.join("\n")).get(...params) as LoadedDiscoveryRow | undefined;
  if (!row) {
    throw new Error(`Could not resolve exemplar record key "${recordKey}" within the requested scope.`);
  }
  row.query = recordKey;

  if (decodeVector(row.vectorBlob).length === 0) {
    throw new Error(`Exemplar "${row.name}" (${recordKey}) does not have a usable embedding vector.`);
  }

  return row;
}

function resolveExemplarByName(
  db: DatabaseSync,
  name: string,
  category?: SearchCategory,
  subcategory?: SearchSubcategory,
): LoadedDiscoveryRow {
  const normalizedName = normalizeText(name);
  if (!normalizedName) {
    throw new Error("Exemplar names must contain at least one alphanumeric token.");
  }

  const sql = [
    "SELECT DISTINCT",
    "  r.record_key AS recordKey,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.derived_tags_json AS derivedTagsJson,",
    "  r.description_text AS descriptionText,",
    "  e.vector_blob AS vectorBlob,",
    "  CASE WHEN r.normalized_name = ? THEN 'name' ELSE 'alias' END AS matchedBy",
    "FROM records r",
    "JOIN embeddings e ON e.record_key = r.record_key",
    "LEFT JOIN record_aliases ra",
    "  ON ra.canonical_record_key = r.record_key",
    "  AND ra.normalized_alias = ?",
    "WHERE r.is_search_canonical = 1",
    "AND (r.normalized_name = ? OR ra.normalized_alias = ?)",
  ];
  const params: Array<string> = [normalizedName, normalizedName, normalizedName, normalizedName];
  if (category) {
    sql.push("AND r.category = ?");
    params.push(category);
  }
  if (subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(subcategory);
  }
  sql.push("ORDER BY matchedBy ASC, r.name ASC, r.record_key ASC");

  const rows = db.prepare(sql.join("\n")).all(...params) as LoadedDiscoveryRow[];
  const exactRows = dedupeLoadedRows(rows.filter((row) => row.matchedBy === "name"));
  if (exactRows.length === 1) {
    exactRows[0]!.query = name;
    ensureUsableEmbedding(exactRows[0]!, name);
    return exactRows[0]!;
  }

  const dedupedRows = dedupeLoadedRows(rows);
  if (dedupedRows.length === 1) {
    dedupedRows[0]!.query = name;
    ensureUsableEmbedding(dedupedRows[0]!, name);
    return dedupedRows[0]!;
  }

  if (dedupedRows.length === 0) {
    throw new Error(`Could not resolve exemplar name "${name}" within the requested scope.`);
  }

  const choices = dedupedRows
    .slice(0, 5)
    .map((row) => `${row.name} (${row.recordKey})`)
    .join(", ");
  throw new Error(`Exemplar name "${name}" is ambiguous. Use --record-key or narrow the scope. Matches: ${choices}.`);
}

function ensureUsableEmbedding(row: LoadedDiscoveryRow, query: string): void {
  if (decodeVector(row.vectorBlob).length === 0) {
    throw new Error(`Resolved exemplar "${query}" to "${row.name}", but it does not have a usable embedding vector.`);
  }
}

function dedupeLoadedRows(rows: LoadedDiscoveryRow[]): LoadedDiscoveryRow[] {
  const unique = new Map<string, LoadedDiscoveryRow>();
  for (const row of rows) {
    if (!unique.has(row.recordKey)) {
      unique.set(row.recordKey, row);
    }
  }

  return [...unique.values()];
}

function toSemanticDiscoveryRecord(row: LoadedDiscoveryRow): SemanticDiscoveryRecord {
  return {
    recordKey: row.recordKey,
    name: row.name,
    category: row.category as SearchCategory,
    subcategory: (row.subcategory ?? null) as SearchSubcategory | null,
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: JSON.parse(row.traitsJson) as string[],
    derivedTags: JSON.parse(row.derivedTagsJson) as string[],
    descriptionText: row.descriptionText,
    vector: decodeVector(row.vectorBlob),
  };
}

function toResolvedSemanticExemplar(row: LoadedDiscoveryRow): ResolvedSemanticExemplar {
  return {
    query: row.query ?? (row.matchedBy === "recordKey" ? row.recordKey : row.name),
    matchedBy: (row.matchedBy as ResolvedSemanticExemplar["matchedBy"] | undefined) ?? "recordKey",
    recordKey: row.recordKey,
    name: row.name,
    category: row.category as SearchCategory,
    subcategory: (row.subcategory ?? null) as SearchSubcategory | null,
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: JSON.parse(row.traitsJson) as string[],
  };
}

function loadCandidateRecords(
  db: DatabaseSync,
  scope: DiscoveryScope,
  options: {
    excludeRecordKeys: Set<string>;
    excludeDerivedTag?: string;
  },
): SemanticDiscoveryRecord[] {
  const sql = [
    "SELECT",
    "  r.record_key AS recordKey,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.derived_tags_json AS derivedTagsJson,",
    "  r.description_text AS descriptionText,",
    "  e.vector_blob AS vectorBlob",
    "FROM records r",
    "JOIN embeddings e ON e.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
    "AND r.category = ?",
  ];
  const params: Array<string> = [scope.category];

  if (scope.subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(scope.subcategory);
  }

  if (options.excludeDerivedTag) {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(normalizeDerivedTag(options.excludeDerivedTag));
  }

  if (options.excludeRecordKeys.size > 0) {
    const placeholders = [...options.excludeRecordKeys].map(() => "?").join(", ");
    sql.push(`AND r.record_key NOT IN (${placeholders})`);
    params.push(...options.excludeRecordKeys);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as LoadedDiscoveryRow[];
  return rows.map(toSemanticDiscoveryRecord);
}

function clampPositiveInteger(value: number | undefined, fallback: number, max: number): number {
  if (value === undefined || Number.isNaN(value)) {
    return fallback;
  }

  return Math.max(1, Math.min(max, Math.trunc(value)));
}

function collectCommonTraits(records: SemanticDiscoveryRecord[], limit: number): string[] {
  const counts = new Map<string, number>();
  for (const record of records) {
    for (const trait of new Set(record.traits)) {
      counts.set(trait, (counts.get(trait) ?? 0) + 1);
    }
  }

  const minimumSupport = Math.min(records.length, Math.max(2, Math.ceil(records.length * 0.25)));
  return [...counts.entries()]
    .filter(([, count]) => count >= minimumSupport)
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
    .slice(0, limit)
    .map(([trait]) => trait);
}

function collectSharedEvidence(
  exemplars: SemanticDiscoveryRecord[],
  candidates: SemanticDiscoveryCandidate[],
  options: {
    sharedTokenLimit: number;
    sharedPhraseLimit: number;
  },
): { tokens: SemanticDiscoveryEvidenceTerm[]; phrases: SemanticDiscoveryEvidenceTerm[] } {
  const exemplarTexts = exemplars.map((record) => record.descriptionText ?? "");
  const candidateTexts = candidates.map((record) => record.descriptionText ?? "");
  const tokens = scoreEvidenceTerms(exemplarTexts, candidateTexts, {
    extractor: extractTokenSet,
    limit: options.sharedTokenLimit,
  });
  const phrases = scoreEvidenceTerms(exemplarTexts, candidateTexts, {
    extractor: extractPhraseSet,
    limit: options.sharedPhraseLimit,
  });

  return { tokens, phrases };
}

function scoreEvidenceTerms(
  exemplarTexts: string[],
  candidateTexts: string[],
  options: {
    extractor: (text: string) => Set<string>;
    limit: number;
  },
): SemanticDiscoveryEvidenceTerm[] {
  const exemplarCounts = collectEvidenceCounts(exemplarTexts, options.extractor);
  const candidateCounts = collectEvidenceCounts(candidateTexts, options.extractor);
  const exemplarMinimum = exemplarTexts.length === 0 ? Number.POSITIVE_INFINITY : Math.min(exemplarTexts.length, Math.max(2, Math.ceil(exemplarTexts.length * 0.5)));
  const candidateMinimum = candidateTexts.length === 0 ? Number.POSITIVE_INFINITY : Math.min(candidateTexts.length, Math.max(2, Math.ceil(candidateTexts.length * 0.3)));
  const values = new Set([...exemplarCounts.keys(), ...candidateCounts.keys()]);

  return [...values]
    .map((value) => {
      const exemplarSupport = exemplarCounts.get(value) ?? 0;
      const candidateSupport = candidateCounts.get(value) ?? 0;
      return {
        value,
        support: exemplarSupport + candidateSupport,
        exemplarSupport,
        candidateSupport,
      };
    })
    .filter((entry) => entry.exemplarSupport >= exemplarMinimum || entry.candidateSupport >= candidateMinimum)
    .sort((left, right) =>
      right.support - left.support ||
      right.exemplarSupport - left.exemplarSupport ||
      right.candidateSupport - left.candidateSupport ||
      left.value.localeCompare(right.value)
    )
    .slice(0, options.limit);
}

function collectEvidenceCounts(
  texts: string[],
  extractor: (text: string) => Set<string>,
): Map<string, number> {
  const counts = new Map<string, number>();
  for (const text of texts) {
    for (const value of extractor(text)) {
      counts.set(value, (counts.get(value) ?? 0) + 1);
    }
  }

  return counts;
}

function extractTokenSet(text: string): Set<string> {
  return new Set(tokenize(text).filter((token) => isUsefulToken(token)));
}

function extractPhraseSet(text: string): Set<string> {
  const tokens = tokenize(text);
  const phrases = new Set<string>();
  for (let size = 2; size <= 3; size += 1) {
    for (let index = 0; index <= tokens.length - size; index += 1) {
      const slice = tokens.slice(index, index + size);
      if (slice.some((token) => !token)) {
        continue;
      }
      if (STOPWORDS.has(slice[0]!) || STOPWORDS.has(slice[slice.length - 1]!)) {
        continue;
      }
      if (slice.filter((token) => isUsefulToken(token)).length < Math.ceil(size / 2)) {
        continue;
      }

      phrases.add(slice.join(" "));
    }
  }

  return phrases;
}

function isUsefulToken(token: string): boolean {
  return token.length >= 4 && !STOPWORDS.has(token) && !/^\d+$/.test(token);
}

function pickContrastRecords(
  rankedCandidates: SemanticDiscoveryCandidate[],
  topCandidateKeys: Set<string>,
  commonTraits: string[],
  minSimilarity: number,
  limit: number,
): SemanticDiscoveryCandidate[] {
  const overlapThreshold = commonTraits.length === 0 ? 0 : Math.max(0, Math.floor(commonTraits.length / 2));
  const primary = rankedCandidates
    .filter((candidate) => !topCandidateKeys.has(candidate.recordKey))
    .filter((candidate) => candidate.similarity >= minSimilarity)
    .filter((candidate) => candidate.sharedTraits.length <= overlapThreshold)
    .slice(0, limit);

  if (primary.length >= limit) {
    return primary;
  }

  const selected = new Set(primary.map((candidate) => candidate.recordKey));
  const fallback = rankedCandidates
    .filter((candidate) => !topCandidateKeys.has(candidate.recordKey))
    .filter((candidate) => !selected.has(candidate.recordKey))
    .slice(0, limit - primary.length);
  return [...primary, ...fallback];
}

function decodeVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

function averageVectors(vectors: Float32Array[]): Float32Array {
  if (vectors.length === 0) {
    return new Float32Array(0);
  }

  const dimensions = vectors[0]?.length ?? 0;
  const total = new Float32Array(dimensions);
  let contributing = 0;
  for (const vector of vectors) {
    if (vector.length !== dimensions) {
      continue;
    }

    contributing += 1;
    for (let index = 0; index < dimensions; index += 1) {
      total[index] = (total[index] ?? 0) + vector[index]!;
    }
  }

  if (contributing === 0) {
    return new Float32Array(0);
  }

  for (let index = 0; index < dimensions; index += 1) {
    total[index] = (total[index] ?? 0) / contributing;
  }

  return total;
}

function normalizeVector(vector: Float32Array): Float32Array {
  let magnitude = 0;
  for (const value of vector) {
    magnitude += value * value;
  }

  if (magnitude === 0) {
    return vector;
  }

  const scale = 1 / Math.sqrt(magnitude);
  for (let index = 0; index < vector.length; index += 1) {
    vector[index] = (vector[index] ?? 0) * scale;
  }

  return vector;
}

function cosineSimilarity(left: Float32Array, right: Float32Array): number {
  if (left.length === 0 || right.length === 0 || left.length !== right.length) {
    return 0;
  }

  let total = 0;
  for (let index = 0; index < left.length; index += 1) {
    total += left[index]! * right[index]!;
  }

  return total;
}
