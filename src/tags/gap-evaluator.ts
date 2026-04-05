import { DatabaseSync } from "node:sqlite";

import { normalizeDerivedTag } from "./index.js";
import { SearchCategory, SearchSubcategory } from "../types.js";
import { type DiscoveryEvidenceTerm, analyzeDiscoveryEvidenceFromRecords } from "./evidence-analyzer.js";
import { tokenizeDiscoveryText } from "./discovery-normalization.js";

export type DerivedTagGapRecord = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  traits: string[];
  descriptionText: string | null;
  vector: Float32Array;
};

export type DerivedTagGapCandidate = {
  recordKey: string;
  name: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  level: number | null;
  similarity: number;
  traits: string[];
  sharedTraits: string[];
  descriptionText: string | null;
};

export type DerivedTagGapEvaluation = {
  tag: string;
  candidateCategory: SearchCategory | null;
  candidateSubcategory: SearchSubcategory | null;
  exemplarCategory: SearchCategory | null;
  exemplarSubcategory: SearchSubcategory | null;
  exemplarCount: number;
  candidateCount: number;
  commonTraits: string[];
  discriminativeTokens: DiscoveryEvidenceTerm[];
  discriminativePhrases: DiscoveryEvidenceTerm[];
  exemplars: Array<{
    name: string;
    recordKey: string;
    level: number | null;
    traits: string[];
    similarityToCentroid: number;
  }>;
  candidates: DerivedTagGapCandidate[];
  contrastRecords: DerivedTagGapCandidate[];
  candidateCohorts: Array<{
    signature: string[];
    size: number;
    sharedTraits: string[];
    representativeNames: string[];
  }>;
};

export type DerivedTagGapEvaluationOptions = {
  tag: string;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  exemplarCategory?: SearchCategory;
  exemplarSubcategory?: SearchSubcategory;
  limit?: number;
  exemplarLimit?: number;
  commonTraitLimit?: number;
  minSimilarity?: number;
};

type DerivedTagGapScope = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
};

type LoadedGapRow = {
  recordKey: string;
  name: string;
  category: string;
  subcategory: string | null;
  level: number | bigint | null;
  traitsJson: string;
  descriptionText: string | null;
  vectorBlob: Uint8Array;
};

export function evaluateDerivedTagGaps(
  db: DatabaseSync,
  options: DerivedTagGapEvaluationOptions,
): DerivedTagGapEvaluation {
  const normalizedTag = normalizeDerivedTag(options.tag);
  const candidateScope = getCandidateScope(options);
  const exemplarScope = getExemplarScope(options, candidateScope);
  const exemplars = loadGapRecords(db, { ...exemplarScope, tag: normalizedTag, mode: "tagged" });
  if (exemplars.length === 0) {
    throw new Error(`No canonical records with derived tag "${normalizedTag}" matched exemplar scope "${renderScope(exemplarScope)}".`);
  }

  const candidates = loadGapRecords(db, { ...candidateScope, tag: normalizedTag, mode: "untagged" });
  return rankDerivedTagGapCandidates(exemplars, candidates, {
    ...options,
    tag: normalizedTag,
  });
}

export function rankDerivedTagGapCandidates(
  exemplars: DerivedTagGapRecord[],
  candidates: DerivedTagGapRecord[],
  options: DerivedTagGapEvaluationOptions,
): DerivedTagGapEvaluation {
  if (exemplars.length === 0) {
    throw new Error("Cannot evaluate derived-tag gaps without at least one tagged exemplar.");
  }

  const normalizedTag = normalizeDerivedTag(options.tag);
  const centroid = normalizeVector(averageVectors(exemplars.map((record) => record.vector)));
  const limit = Math.max(1, Math.min(options.limit ?? 25, 200));
  const exemplarLimit = Math.max(1, Math.min(options.exemplarLimit ?? 8, 25));
  const commonTraitLimit = Math.max(1, Math.min(options.commonTraitLimit ?? 8, 25));
  const minSimilarity = options.minSimilarity ?? Number.NEGATIVE_INFINITY;
  const commonTraits = collectCommonTraits(exemplars, commonTraitLimit);
  const evidence = analyzeDiscoveryEvidenceFromRecords(
    exemplars.map(toAnalysisRecord),
    [...exemplars, ...candidates].map(toAnalysisRecord),
    { limit: 8, exampleLimit: 3 },
  );
  const discriminativeTokens = evidence.descriptionTokens.slice(0, 6);
  const discriminativePhrases = evidence.descriptionPhrases.slice(0, 6);

  const rankedCandidates = candidates
    .map((candidate) => ({
      recordKey: candidate.recordKey,
      name: candidate.name,
      category: candidate.category,
      subcategory: candidate.subcategory,
      level: candidate.level,
      similarity: cosineSimilarity(candidate.vector, centroid),
      traits: candidate.traits,
      sharedTraits: commonTraits.filter((trait) => candidate.traits.includes(trait)),
      descriptionText: candidate.descriptionText,
    }))
    .filter((candidate) => candidate.similarity >= minSimilarity)
    .sort((left, right) => right.similarity - left.similarity || left.name.localeCompare(right.name))
    .slice(0, limit);
  const contrastRecords = rankedCandidates
    .filter((candidate) => countAnchorOverlap(candidate.descriptionText, discriminativeTokens) === 0)
    .slice(0, Math.min(6, rankedCandidates.length));
  const candidateCohorts = buildCandidateCohorts(rankedCandidates, discriminativeTokens, discriminativePhrases);

  const representativeExemplars = exemplars
    .map((record) => ({
      name: record.name,
      recordKey: record.recordKey,
      level: record.level,
      traits: record.traits,
      similarityToCentroid: cosineSimilarity(record.vector, centroid),
    }))
    .sort((left, right) => right.similarityToCentroid - left.similarityToCentroid || left.name.localeCompare(right.name))
    .slice(0, exemplarLimit);

  return {
    tag: normalizedTag,
    candidateCategory: options.category ?? null,
    candidateSubcategory: options.subcategory ?? null,
    exemplarCategory: options.exemplarCategory ?? options.category ?? null,
    exemplarSubcategory: options.exemplarSubcategory ?? options.subcategory ?? null,
    exemplarCount: exemplars.length,
    candidateCount: candidates.length,
    commonTraits,
    discriminativeTokens,
    discriminativePhrases,
    exemplars: representativeExemplars,
    candidates: rankedCandidates,
    contrastRecords,
    candidateCohorts,
  };
}

function getCandidateScope(options: DerivedTagGapEvaluationOptions): DerivedTagGapScope {
  return {
    category: options.category,
    subcategory: options.subcategory,
  };
}

function getExemplarScope(
  options: DerivedTagGapEvaluationOptions,
  candidateScope: DerivedTagGapScope,
): DerivedTagGapScope {
  return {
    category: options.exemplarCategory ?? candidateScope.category,
    subcategory: options.exemplarSubcategory ?? candidateScope.subcategory,
  };
}

function renderScope(scope: DerivedTagGapScope): string {
  if (scope.category && scope.subcategory) {
    return `${scope.category}/${scope.subcategory}`;
  }
  if (scope.category) {
    return scope.category;
  }
  if (scope.subcategory) {
    return `*/${scope.subcategory}`;
  }
  return "all canonical records";
}

function loadGapRecords(
  db: DatabaseSync,
  options: DerivedTagGapEvaluationOptions & { tag: string; mode: "tagged" | "untagged" },
): DerivedTagGapRecord[] {
  const sql = [
    "SELECT",
    "  r.record_key AS recordKey,",
    "  r.name AS name,",
    "  r.category AS category,",
    "  r.subcategory AS subcategory,",
    "  r.level AS level,",
    "  r.traits_json AS traitsJson,",
    "  r.description_text AS descriptionText,",
    "  e.vector_blob AS vectorBlob",
    "FROM records r",
    "JOIN embeddings e ON e.record_key = r.record_key",
    "WHERE r.is_search_canonical = 1",
  ];
  const params: Array<string | number> = [];

  if (options.mode === "tagged") {
    sql.push("AND EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(options.tag);
  } else {
    sql.push("AND NOT EXISTS (SELECT 1 FROM record_derived_tags d WHERE d.record_key = r.record_key AND d.tag = ?)");
    params.push(options.tag);
  }

  if (options.category) {
    sql.push("AND r.category = ?");
    params.push(options.category);
  }
  if (options.subcategory) {
    sql.push("AND r.subcategory = ?");
    params.push(options.subcategory);
  }

  const rows = db.prepare(sql.join("\n")).all(...params) as LoadedGapRow[];
  return rows.map((row) => ({
    recordKey: row.recordKey,
    name: row.name,
    category: row.category as SearchCategory,
    subcategory: (row.subcategory ?? null) as SearchSubcategory | null,
    level: typeof row.level === "bigint" ? Number(row.level) : row.level,
    traits: JSON.parse(row.traitsJson) as string[],
    descriptionText: row.descriptionText,
    vector: decodeVector(row.vectorBlob),
  }));
}

function decodeVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

function toAnalysisRecord(record: DerivedTagGapRecord) {
  return {
    recordKey: record.recordKey,
    name: record.name,
    category: record.category,
    subcategory: record.subcategory,
    level: record.level,
    traits: [...record.traits],
    derivedTags: [],
    descriptionText: record.descriptionText,
    vector: record.vector,
    references: [],
  };
}

function countAnchorOverlap(
  text: string | null,
  anchors: DiscoveryEvidenceTerm[],
): number {
  const tokens = new Set(tokenizeDiscoveryText(text ?? "", { filterStopwords: true }));
  return anchors.filter((anchor) =>
    anchor.value.split(" ").every((token) => tokens.has(token))
  ).length;
}

function buildCandidateCohorts(
  candidates: DerivedTagGapCandidate[],
  tokenAnchors: DiscoveryEvidenceTerm[],
  phraseAnchors: DiscoveryEvidenceTerm[],
): DerivedTagGapEvaluation["candidateCohorts"] {
  const buckets = new Map<string, DerivedTagGapCandidate[]>();
  const signaturesByKey = new Map<string, string[]>();
  const anchors = [...tokenAnchors.map((term) => term.value), ...phraseAnchors.map((term) => term.value)];

  for (const candidate of candidates) {
    const tokens = new Set(tokenizeDiscoveryText(candidate.descriptionText ?? "", { filterStopwords: true }));
    const signature = anchors
      .filter((anchor) => anchor.split(" ").every((token) => tokens.has(token)))
      .slice(0, 4);
    const key = signature.length > 0 ? signature.join("||") : "__semantic_only__";
    const bucket = buckets.get(key) ?? [];
    bucket.push(candidate);
    buckets.set(key, bucket);
    signaturesByKey.set(key, signature);
  }

  return [...buckets.entries()]
    .map(([key, bucket]) => ({
      signature: signaturesByKey.get(key) ?? [],
      size: bucket.length,
      sharedTraits: [...new Set(bucket.flatMap((candidate) => candidate.sharedTraits))]
        .filter((trait) => bucket.every((candidate) => candidate.sharedTraits.includes(trait)))
        .sort((left, right) => left.localeCompare(right)),
      representativeNames: bucket.slice(0, 4).map((candidate) => candidate.name),
    }))
    .sort((left, right) => right.size - left.size || left.signature.join(" ").localeCompare(right.signature.join(" ")))
    .slice(0, 6);
}

function averageVectors(vectors: Float32Array[]): Float32Array {
  if (vectors.length === 0) {
    return new Float32Array(0);
  }

  const dimensions = vectors[0]?.length ?? 0;
  const total = new Float32Array(dimensions);
  for (const vector of vectors) {
    if (vector.length !== dimensions) {
      continue;
    }

    for (let index = 0; index < dimensions; index += 1) {
      total[index] = (total[index] ?? 0) + vector[index]!;
    }
  }

  for (let index = 0; index < dimensions; index += 1) {
    total[index] = (total[index] ?? 0) / vectors.length;
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

function collectCommonTraits(records: DerivedTagGapRecord[], limit: number): string[] {
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
