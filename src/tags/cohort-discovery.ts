import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import { normalizeDerivedTag } from "./index.js";
import {
  type DiscoveryEvidenceTerm,
  analyzeDiscoveryEvidenceFromRecords,
} from "./evidence-analyzer.js";
import {
  type DiscoveryAnalysisRecord,
  loadDiscoveryRecords,
  resolveDiscoveryExemplars,
  type ResolvedDiscoveryExemplar,
} from "./discovery-records.js";
import { tokenizeDiscoveryText } from "./discovery-normalization.js";

const DEFAULT_CANDIDATE_LIMIT = 40;
const DEFAULT_COHORT_LIMIT = 8;

export type CohortRecommendation = "rule-led" | "hybrid" | "manual-only" | "reject";

export type DerivedTagCandidateCluster = {
  signature: string[];
  size: number;
  distinctVariantFamilies: number;
  averageSimilarity: number;
  sharedTraits: string[];
  sharedAnchors: string[];
  representativeRecords: Array<{
    recordKey: string;
    name: string;
    similarity: number;
  }>;
};

export type RuleableCohortReport = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  sourceTag: string | null;
  exemplarCount: number;
  candidateCount: number;
  resolvedExemplars: Array<{
    query: string;
    matchedBy: "recordKey" | "name" | "alias";
    recordKey: string;
    name: string;
  }>;
  anchorTerms: DiscoveryEvidenceTerm[];
  contrastRecords: Array<{
    recordKey: string;
    name: string;
    similarity: number;
  }>;
  cohorts: Array<DerivedTagCandidateCluster & {
    score: number;
    recommendation: CohortRecommendation;
  }>;
};

export type RuleableCohortOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  tag?: string;
  exemplarNames?: string[];
  exemplarRecordKeys?: string[];
  candidateLimit?: number;
  cohortLimit?: number;
  minSimilarity?: number;
};

type RankedCandidate = DiscoveryAnalysisRecord & {
  similarity: number;
  anchorOverlap: string[];
};

function variantFamilyIdentity(record: DiscoveryAnalysisRecord): string {
  return record.variantFamilyKey ?? record.recordKey;
}

function dedupeVariantFamilies<T extends DiscoveryAnalysisRecord>(records: T[]): T[] {
  const seen = new Set<string>();
  return records.filter((record) => {
    const familyId = variantFamilyIdentity(record);
    if (seen.has(familyId)) {
      return false;
    }
    seen.add(familyId);
    return true;
  });
}

function distinctVariantFamilyCount(records: DiscoveryAnalysisRecord[]): number {
  return new Set(records.map((record) => variantFamilyIdentity(record))).size;
}

function selectRepresentativeCandidates(
  bucket: RankedCandidate[],
  limit = 5,
): Array<{ recordKey: string; name: string; similarity: number }> {
  const selected: Array<{ recordKey: string; name: string; similarity: number }> = [];
  const seenFamilies = new Set<string>();
  for (const record of bucket) {
    const familyId = variantFamilyIdentity(record);
    if (seenFamilies.has(familyId)) {
      continue;
    }
    seenFamilies.add(familyId);
    selected.push({
      recordKey: record.recordKey,
      name: record.name,
      similarity: record.similarity,
    });
    if (selected.length >= limit) {
      return selected;
    }
  }

  for (const record of bucket) {
    if (selected.some((entry) => entry.recordKey === record.recordKey)) {
      continue;
    }
    selected.push({
      recordKey: record.recordKey,
      name: record.name,
      similarity: record.similarity,
    });
    if (selected.length >= limit) {
      break;
    }
  }

  return selected;
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
  if (magnitude <= 0) {
    return new Float32Array(vector.length);
  }

  const normalized = new Float32Array(vector.length);
  const divisor = Math.sqrt(magnitude);
  for (let index = 0; index < vector.length; index += 1) {
    normalized[index] = (vector[index] ?? 0) / divisor;
  }
  return normalized;
}

function cosineSimilarity(left: Float32Array, right: Float32Array): number {
  if (left.length === 0 || left.length !== right.length) {
    return 0;
  }

  let total = 0;
  for (let index = 0; index < left.length; index += 1) {
    total += (left[index] ?? 0) * (right[index] ?? 0);
  }
  return total;
}

function buildRecordFeatureSet(record: DiscoveryAnalysisRecord): Set<string> {
  const features = new Set<string>();
  for (const trait of record.traits) {
    features.add(`trait:${trait}`);
  }
  for (const token of tokenizeDiscoveryText(record.name, { filterStopwords: true })) {
    features.add(`name:${token}`);
  }
  for (const token of tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true })) {
    features.add(`text:${token}`);
  }
  for (const reference of record.references) {
    features.add(`ref-target:${reference.targetName.toLowerCase()}`);
    features.add(`ref-scope:${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`);
  }
  return features;
}

function deriveAnchorVocabulary(
  exemplars: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
): DiscoveryEvidenceTerm[] {
  const evidence = analyzeDiscoveryEvidenceFromRecords(dedupeVariantFamilies(exemplars), baseline, { limit: 10, exampleLimit: 3 });
  return [
    ...evidence.traits.slice(0, 4),
    ...evidence.nameTokens.slice(0, 4),
    ...evidence.descriptionTokens.slice(0, 4),
    ...evidence.references.slice(0, 3),
  ]
    .sort((left, right) => right.score - left.score || left.value.localeCompare(right.value))
    .filter((entry, index, all) => all.findIndex((candidate) => candidate.value === entry.value) === index)
    .slice(0, 10);
}

function rankCandidates(
  exemplars: DiscoveryAnalysisRecord[],
  candidates: DiscoveryAnalysisRecord[],
  anchors: DiscoveryEvidenceTerm[],
  options: RuleableCohortOptions,
): RankedCandidate[] {
  const centroid = normalizeVector(averageVectors(exemplars.map((record) => record.vector).filter((vector) => vector.length > 0)));
  const anchorSet = new Set(anchors.map((anchor) => anchor.value));
  const limit = Math.max(1, Math.min(options.candidateLimit ?? DEFAULT_CANDIDATE_LIMIT, 200));
  const minSimilarity = options.minSimilarity ?? Number.NEGATIVE_INFINITY;

  return candidates
    .map((record) => {
      const featureSet = buildRecordFeatureSet(record);
      const anchorOverlap = [...anchorSet].filter((anchor) =>
        featureSet.has(anchor.startsWith("target:") || anchor.startsWith("scope:") ? `ref-${anchor}` : anchor.includes(":") ? anchor : `text:${anchor}`) ||
        featureSet.has(`trait:${anchor}`) ||
        featureSet.has(`name:${anchor}`) ||
        featureSet.has(`text:${anchor}`),
      );
      return {
        ...record,
        similarity: cosineSimilarity(record.vector, centroid),
        anchorOverlap,
      };
    })
    .filter((candidate) => candidate.similarity >= minSimilarity)
    .sort((left, right) => right.similarity - left.similarity || right.anchorOverlap.length - left.anchorOverlap.length || left.name.localeCompare(right.name))
    .slice(0, limit);
}

function buildSignature(candidate: RankedCandidate): string[] {
  const preferred = uniqueSorted(candidate.anchorOverlap).slice(0, 4);
  if (preferred.length > 0) {
    return preferred;
  }

  return uniqueSorted(candidate.traits.map((trait) => `trait:${trait}`)).slice(0, 2);
}

function recommendCluster(score: number): CohortRecommendation {
  if (score >= 0.72) {
    return "rule-led";
  }
  if (score >= 0.5) {
    return "hybrid";
  }
  if (score >= 0.3) {
    return "manual-only";
  }
  return "reject";
}

function clusterCandidates(
  candidates: RankedCandidate[],
  anchors: DiscoveryEvidenceTerm[],
  options: RuleableCohortOptions,
): Array<DerivedTagCandidateCluster & { score: number; recommendation: CohortRecommendation }> {
  const anchorValues = new Set(anchors.map((anchor) => anchor.value));
  const buckets = new Map<string, RankedCandidate[]>();

  for (const candidate of candidates) {
    const signature = buildSignature(candidate);
    const key = signature.length > 0 ? signature.join("||") : "__semantic_only__";
    const bucket = buckets.get(key) ?? [];
    bucket.push(candidate);
    buckets.set(key, bucket);
  }

  const cohortLimit = Math.max(1, Math.min(options.cohortLimit ?? DEFAULT_COHORT_LIMIT, 20));
  return [...buckets.entries()]
    .map(([key, bucket]) => {
      const signature = key === "__semantic_only__" ? [] : key.split("||");
      const averageSimilarity = bucket.reduce((total, candidate) => total + candidate.similarity, 0) / Math.max(1, bucket.length);
      const distinctVariantFamilies = distinctVariantFamilyCount(bucket);
      const sharedTraits = [...new Set(bucket.flatMap((candidate) => candidate.traits))]
        .filter((trait) => bucket.every((candidate) => candidate.traits.includes(trait)));
      const sharedAnchors = signature.filter((anchor) => anchorValues.has(anchor));
      const density = sharedAnchors.length / Math.max(1, signature.length || 1);
      const sizeFactor = Math.min(1, bucket.length / 5);
      const familyDiversity = distinctVariantFamilies / Math.max(1, bucket.length);
      const similarityFactor = Math.max(0, Math.min(1, averageSimilarity));
      const traitPenalty = sharedTraits.length === 1 && sharedAnchors.every((anchor) => anchor.startsWith("trait:")) ? 0.15 : 0;
      const score = Math.max(0, Math.min(1, (sizeFactor * 0.25) + (familyDiversity * 0.25) + (similarityFactor * 0.25) + (density * 0.25) - traitPenalty));

      return {
        signature,
        size: bucket.length,
        distinctVariantFamilies,
        averageSimilarity,
        sharedTraits: uniqueSorted(sharedTraits),
        sharedAnchors: uniqueSorted(sharedAnchors),
        representativeRecords: selectRepresentativeCandidates(
          bucket
            .slice()
            .sort((left, right) => right.similarity - left.similarity || left.name.localeCompare(right.name)),
        ),
        score,
        recommendation: recommendCluster(score),
      };
    })
    .sort((left, right) =>
      right.score - left.score ||
      right.distinctVariantFamilies - left.distinctVariantFamilies ||
      right.size - left.size ||
      right.averageSimilarity - left.averageSimilarity ||
      left.signature.join(" ").localeCompare(right.signature.join(" ")))
    .slice(0, cohortLimit);
}

export function discoverRuleableCohorts(
  db: DatabaseSync,
  options: RuleableCohortOptions,
): RuleableCohortReport {
  const normalizedTag = options.tag ? normalizeDerivedTag(options.tag) : null;
  const resolvedExemplars: ResolvedDiscoveryExemplar[] = normalizedTag
    ? loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      requireTag: normalizedTag,
      includeVectors: true,
    }).map((record) => ({
      query: record.recordKey,
      matchedBy: "recordKey" as const,
      ...record,
    }))
    : resolveDiscoveryExemplars(db, {
      category: options.category,
      subcategory: options.subcategory,
      exemplarNames: options.exemplarNames,
      exemplarRecordKeys: options.exemplarRecordKeys,
    });
  if (resolvedExemplars.length === 0) {
    throw new Error("Ruleable cohort discovery requires at least one exemplar or an existing tag.");
  }

  const category = options.category ?? resolvedExemplars[0]!.category;
  const subcategory = options.subcategory ?? resolvedExemplars[0]!.subcategory;
  const exemplarKeys = resolvedExemplars.map((record) => record.recordKey);
  const baseline = loadDiscoveryRecords(db, {
    category,
    subcategory: subcategory ?? undefined,
    includeVectors: true,
  });
  const candidates = loadDiscoveryRecords(db, {
    category,
    subcategory: subcategory ?? undefined,
    excludeRecordKeys: exemplarKeys,
    excludeDerivedTag: normalizedTag ?? undefined,
    includeVectors: true,
  });
  const familyDistinctExemplars = dedupeVariantFamilies(resolvedExemplars);
  const anchors = deriveAnchorVocabulary(familyDistinctExemplars, baseline);
  const rankedCandidates = rankCandidates(familyDistinctExemplars, candidates, anchors, options);
  const cohorts = clusterCandidates(rankedCandidates, anchors, options);
  const anchorValues = new Set(anchors.map((anchor) => anchor.value));
  const contrastRecords = rankedCandidates
    .filter((candidate) => candidate.anchorOverlap.filter((anchor) => anchorValues.has(anchor)).length <= 1)
    .slice(0, 6)
    .map((candidate) => ({
      recordKey: candidate.recordKey,
      name: candidate.name,
      similarity: candidate.similarity,
    }));

  return {
    category,
    subcategory,
    sourceTag: normalizedTag,
    exemplarCount: familyDistinctExemplars.length,
    candidateCount: rankedCandidates.length,
    resolvedExemplars: familyDistinctExemplars.map((record) => ({
      query: record.query,
      matchedBy: record.matchedBy,
      recordKey: record.recordKey,
      name: record.name,
    })),
    anchorTerms: anchors,
    contrastRecords,
    cohorts,
  };
}
