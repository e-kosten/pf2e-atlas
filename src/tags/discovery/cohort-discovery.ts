import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../../domain/index.js";
import { uniqueSorted } from "../../shared/utils.js";
import {
  getDerivedTagExemplarRecordKeys,
  getDerivedTagLegacySeedMigrationRecordKeys,
} from "../runtime/derivation/api.js";
import { normalizeDerivedTag } from "../runtime/matcher/shared.js";
import {
  type DiscoveryEvidenceKind,
  type DiscoveryEvidenceTerm,
  analyzeDiscoveryEvidenceFromRecords,
} from "../evaluation/evidence-analyzer.js";
import {
  type DiscoveryAnalysisRecord,
  loadDiscoveryRecords,
  resolveDiscoveryExemplars,
  type ResolvedDiscoveryExemplar,
} from "./discovery-records.js";
import { summarizeDiscoverySources } from "./discovery-source-summary.js";
import { isDiscoveryNoiseToken, tokenizeDiscoveryText } from "./discovery-normalization.js";

const DEFAULT_CANDIDATE_LIMIT = 40;
const DEFAULT_COHORT_LIMIT = 8;

export type CohortRecommendation = "rule-led" | "hybrid" | "manual-only" | "reject";

export type DerivedTagCandidateCluster = {
  signature: string[];
  size: number;
  distinctVariantFamilies: number;
  averageSimilarity: number;
  sourceCount: number;
  topSources: string[];
  publicationCount: number;
  topPublications: string[];
  sourceSliceCount: number;
  topSourceSlices: string[];
  dominantSourceShare: number;
  sourceScope: "source-slice" | "publication" | "pack" | null;
  sharedTraits: string[];
  sharedAnchors: string[];
  nonNameAnchors: string[];
  reviewFlags: string[];
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
  cohorts: Array<
    DerivedTagCandidateCluster & {
      score: number;
      recommendation: CohortRecommendation;
    }
  >;
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
  anchorScore: number;
  hybridScore: number;
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

function mergeUniqueRecords<T extends DiscoveryAnalysisRecord>(records: T[]): T[] {
  const uniqueRecords = new Map<string, T>();
  for (const record of records) {
    if (!uniqueRecords.has(record.recordKey)) {
      uniqueRecords.set(record.recordKey, record);
    }
  }

  return [...uniqueRecords.values()];
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

function selectDistinctContrastCandidates(
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
    if (!isDiscoveryNoiseToken(token)) {
      features.add(`name:${token}`);
    }
  }
  for (const token of tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true })) {
    if (!isDiscoveryNoiseToken(token)) {
      features.add(`text:${token}`);
    }
  }
  for (const reference of record.references) {
    features.add(`ref-target:${reference.targetName.toLowerCase()}`);
    features.add(
      `ref-scope:${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`,
    );
  }
  return features;
}

function isNameAnchorKind(kind: DiscoveryEvidenceKind): boolean {
  return kind === "nameToken" || kind === "namePhrase";
}

function isLexicalAnchorKind(kind: DiscoveryEvidenceKind): boolean {
  return kind === "nameToken" || kind === "namePhrase" || kind === "descriptionToken" || kind === "descriptionPhrase";
}

function deriveReviewFlags(
  sharedAnchors: string[],
  sharedTraits: string[],
  anchorByValue: Map<string, DiscoveryEvidenceTerm>,
  sourceSummary: ReturnType<typeof summarizeDiscoverySources>,
  distinctVariantFamilies: number,
): string[] {
  const flags: string[] = [];
  const sharedKinds = sharedAnchors
    .map((anchor) => anchorByValue.get(anchor)?.kind)
    .filter((kind): kind is DiscoveryEvidenceKind => Boolean(kind));
  const nameAnchors = sharedAnchors.filter((anchor) =>
    isNameAnchorKind(anchorByValue.get(anchor)?.kind ?? "descriptionToken"),
  );
  const lexicalOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => isLexicalAnchorKind(kind));
  const traitOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => kind === "trait");
  const dominantNameSupport = nameAnchors.reduce(
    (maxSupport, anchor) => Math.max(maxSupport, anchorByValue.get(anchor)?.cohortSupport ?? 0),
    0,
  );
  const nameSeriesSupportFloor = Math.max(2, Math.ceil(distinctVariantFamilies * 0.6));

  if (sharedAnchors.length === 0) {
    flags.push("semantic-only");
  }
  if (lexicalOnly && sharedTraits.length === 0) {
    flags.push("lexical-only");
  }
  if (traitOnly && sharedTraits.length <= 1) {
    flags.push("trait-only");
  }
  if (nameAnchors.length >= 2 && dominantNameSupport >= nameSeriesSupportFloor) {
    flags.push("name-series");
  }
  if (
    sourceSummary.hasUsableSourceSignals &&
    sourceSummary.dominantSourceShare > 0 &&
    sourceSummary.sourceScope &&
    ((sourceSummary.sourceScope === "source-slice" && sourceSummary.sourceSliceCount === 1) ||
      (sourceSummary.sourceScope === "publication" && sourceSummary.publicationCount === 1) ||
      (sourceSummary.sourceScope === "pack" && sourceSummary.sourceCount === 1))
  ) {
    flags.push("source-local");
  } else if (sourceSummary.hasUsableSourceSignals && sourceSummary.dominantSourceShare >= 0.75) {
    flags.push("source-heavy");
  }
  if (
    sharedAnchors.length > 0 &&
    sharedAnchors.every((anchor) => isNameAnchorKind(anchorByValue.get(anchor)?.kind ?? "descriptionToken"))
  ) {
    flags.push("name-anchored");
  }

  return uniqueSorted(flags);
}

function deriveAnchorVocabulary(
  exemplars: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
): DiscoveryEvidenceTerm[] {
  const minimumSupport = exemplars.length > 1 ? 2 : 1;
  const evidence = analyzeDiscoveryEvidenceFromRecords(dedupeVariantFamilies(exemplars), baseline, {
    limit: 10,
    exampleLimit: 3,
  });
  return [
    ...evidence.traits.slice(0, 4),
    ...evidence.nameTokens.slice(0, 4),
    ...evidence.descriptionTokens.slice(0, 4),
    ...evidence.references.slice(0, 3),
  ]
    .sort((left, right) => right.score - left.score || left.value.localeCompare(right.value))
    .filter((entry, index, all) => all.findIndex((candidate) => candidate.value === entry.value) === index)
    .filter(
      (entry, _, all) =>
        !entry.value.startsWith("scope:") ||
        !all.some(
          (candidate) => candidate.value.startsWith("target:") && candidate.cohortSupport >= entry.cohortSupport,
        ),
    )
    .filter((entry) => entry.cohortSupport >= minimumSupport)
    .filter(
      (entry) =>
        entry.value.startsWith("target:") ||
        entry.value.startsWith("scope:") ||
        entry.lift >= 1.15 ||
        (entry.cohortSupport === exemplars.length && entry.cohortSupport >= 3),
    )
    .slice(0, 10);
}

function rankCandidates(
  exemplars: DiscoveryAnalysisRecord[],
  candidates: DiscoveryAnalysisRecord[],
  anchors: DiscoveryEvidenceTerm[],
  options: RuleableCohortOptions,
): RankedCandidate[] {
  const centroid = normalizeVector(
    averageVectors(exemplars.map((record) => record.vector).filter((vector) => vector.length > 0)),
  );
  const anchorSet = new Set(anchors.map((anchor) => anchor.value));
  const anchorByValue = new Map(anchors.map((anchor) => [anchor.value, anchor] as const));
  const maxAnchorScore = Math.max(
    1,
    anchors.reduce((total, anchor) => total + Math.max(0, anchor.score), 0),
  );
  const limit = Math.max(1, Math.min(options.candidateLimit ?? DEFAULT_CANDIDATE_LIMIT, 200));
  const minSimilarity = options.minSimilarity ?? Number.NEGATIVE_INFINITY;

  const rankedCandidates = candidates
    .map((record) => {
      const featureSet = buildRecordFeatureSet(record);
      const anchorOverlap = [...anchorSet].filter(
        (anchor) =>
          featureSet.has(
            anchor.startsWith("target:") || anchor.startsWith("scope:")
              ? `ref-${anchor}`
              : anchor.includes(":")
                ? anchor
                : `text:${anchor}`,
          ) ||
          featureSet.has(`trait:${anchor}`) ||
          featureSet.has(`name:${anchor}`) ||
          featureSet.has(`text:${anchor}`),
      );
      const anchorScore = anchorOverlap.reduce((total, anchor) => total + (anchorByValue.get(anchor)?.score ?? 0), 0);
      const normalizedAnchorScore = Math.max(0, Math.min(1, anchorScore / maxAnchorScore));
      const hybridScore =
        normalizedAnchorScore > 0
          ? normalizedAnchorScore * 0.55 + cosineSimilarity(record.vector, centroid) * 0.45
          : cosineSimilarity(record.vector, centroid);
      return {
        ...record,
        similarity: cosineSimilarity(record.vector, centroid),
        anchorOverlap,
        anchorScore,
        hybridScore,
      };
    })
    .filter((candidate) => candidate.similarity >= minSimilarity)
    .filter(
      (candidate, _, ranked) =>
        !anchors.length || !ranked.some((entry) => entry.anchorScore > 0) || candidate.anchorScore > 0,
    )
    .sort(
      (left, right) =>
        right.hybridScore - left.hybridScore ||
        right.anchorScore - left.anchorScore ||
        right.similarity - left.similarity ||
        right.anchorOverlap.length - left.anchorOverlap.length ||
        left.name.localeCompare(right.name),
    )
    .slice(0, limit);

  return rankedCandidates;
}

function buildSignature(candidate: RankedCandidate, anchorByValue: Map<string, DiscoveryEvidenceTerm>): string[] {
  const preferred = [
    ...new Set(
      candidate.anchorOverlap
        .slice()
        .sort(
          (left, right) =>
            (anchorByValue.get(right)?.score ?? 0) - (anchorByValue.get(left)?.score ?? 0) || left.localeCompare(right),
        ),
    ),
  ].slice(0, 4);
  if (preferred.length > 0) {
    return preferred;
  }

  return uniqueSorted(candidate.traits.map((trait) => `trait:${trait}`)).slice(0, 2);
}

function recommendCluster(
  score: number,
  sharedAnchors: string[],
  nonNameAnchors: string[],
  averageAnchorStrength: number,
  anchorByValue: Map<string, DiscoveryEvidenceTerm>,
  sharedTraits: string[],
  reviewFlags: string[],
): CohortRecommendation {
  const sharedKinds = sharedAnchors
    .map((anchor) => anchorByValue.get(anchor)?.kind)
    .filter((kind): kind is DiscoveryEvidenceKind => Boolean(kind));
  const lexicalOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => isLexicalAnchorKind(kind));
  const traitOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => kind === "trait");

  if (reviewFlags.includes("name-series") && nonNameAnchors.length < 2) {
    if (score >= 0.3) {
      return "manual-only";
    }
    return "reject";
  }
  if (reviewFlags.includes("source-local") && nonNameAnchors.length === 0) {
    if (score >= 0.3) {
      return "manual-only";
    }
    return "reject";
  }
  if (lexicalOnly && sharedTraits.length === 0) {
    if (score >= 0.3) {
      return "manual-only";
    }
    return "reject";
  }
  if (traitOnly && score >= 0.5) {
    return "hybrid";
  }

  if (
    sharedAnchors.length >= 2 &&
    nonNameAnchors.length >= 1 &&
    averageAnchorStrength >= 0.45 &&
    score >= 0.72 &&
    !reviewFlags.includes("source-local") &&
    !reviewFlags.includes("source-heavy")
  ) {
    return "rule-led";
  }
  if (sharedAnchors.length >= 1 && score >= 0.5) {
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
  const anchorByValue = new Map(anchors.map((anchor) => [anchor.value, anchor] as const));
  const buckets = new Map<string, RankedCandidate[]>();

  for (const candidate of candidates) {
    const signature = buildSignature(candidate, anchorByValue);
    const key = signature.length > 0 ? signature.join("||") : "__semantic_only__";
    const bucket = buckets.get(key) ?? [];
    bucket.push(candidate);
    buckets.set(key, bucket);
  }

  const cohortLimit = Math.max(1, Math.min(options.cohortLimit ?? DEFAULT_COHORT_LIMIT, 20));
  return [...buckets.entries()]
    .map(([key, bucket]) => {
      const signature = key === "__semantic_only__" ? [] : key.split("||");
      const averageSimilarity =
        bucket.reduce((total, candidate) => total + candidate.similarity, 0) / Math.max(1, bucket.length);
      const distinctVariantFamilies = distinctVariantFamilyCount(bucket);
      const sourceSummary = summarizeDiscoverySources(bucket);
      const sharedTraits = [...new Set(bucket.flatMap((candidate) => candidate.traits))].filter((trait) =>
        bucket.every((candidate) => candidate.traits.includes(trait)),
      );
      const sharedAnchors = signature.filter((anchor) => anchorValues.has(anchor));
      const nonNameAnchors = sharedAnchors
        .filter((anchor) => !isNameAnchorKind(anchorByValue.get(anchor)?.kind ?? "descriptionToken"))
        .slice(0, 4);
      const sharedKinds = sharedAnchors
        .map((anchor) => anchorByValue.get(anchor)?.kind)
        .filter((kind): kind is DiscoveryEvidenceKind => Boolean(kind));
      const lexicalOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => isLexicalAnchorKind(kind));
      const traitOnly = sharedKinds.length > 0 && sharedKinds.every((kind) => kind === "trait");
      const nameOnly =
        sharedKinds.length > 0 && sharedKinds.every((kind) => kind === "nameToken" || kind === "namePhrase");
      const reviewFlags = deriveReviewFlags(
        sharedAnchors,
        uniqueSorted(sharedTraits),
        anchorByValue,
        sourceSummary,
        distinctVariantFamilies,
      );
      const averageAnchorStrength =
        sharedAnchors.length === 0
          ? 0
          : sharedAnchors.reduce((total, anchor) => {
              const lift = anchorByValue.get(anchor)?.lift ?? 1;
              return total + Math.min(1, Math.log2(1 + Math.max(1, lift)) / 3);
            }, 0) / sharedAnchors.length;
      const density = sharedAnchors.length / Math.max(1, signature.length || 1);
      const sizeFactor = Math.min(1, bucket.length / 5);
      const familyDiversity = distinctVariantFamilies / Math.max(1, bucket.length);
      const similarityFactor = Math.max(0, Math.min(1, averageSimilarity));
      const traitPenalty = traitOnly && sharedTraits.length <= 1 ? 0.15 : 0;
      const lexicalOnlyPenalty = lexicalOnly && sharedTraits.length === 0 ? 0.18 : lexicalOnly ? 0.1 : 0;
      const nameOnlyPenalty = nameOnly ? 0.08 : 0;
      const semanticOnlyPenalty = sharedAnchors.length === 0 ? 0.18 : 0;
      const nameSeriesPenalty = reviewFlags.includes("name-series") ? (nonNameAnchors.length >= 2 ? 0.08 : 0.18) : 0;
      const sourcePenalty = reviewFlags.includes("source-local")
        ? 0.16
        : reviewFlags.includes("source-heavy")
          ? 0.08
          : 0;
      const score = Math.max(
        0,
        Math.min(
          1,
          sizeFactor * 0.2 +
            familyDiversity * 0.2 +
            similarityFactor * 0.2 +
            density * 0.15 +
            averageAnchorStrength * 0.25 -
            traitPenalty -
            lexicalOnlyPenalty -
            nameOnlyPenalty -
            semanticOnlyPenalty -
            nameSeriesPenalty -
            sourcePenalty,
        ),
      );

      return {
        signature,
        size: bucket.length,
        distinctVariantFamilies,
        averageSimilarity,
        sourceCount: sourceSummary.sourceCount,
        topSources: sourceSummary.topSources,
        publicationCount: sourceSummary.publicationCount,
        topPublications: sourceSummary.topPublications,
        sourceSliceCount: sourceSummary.sourceSliceCount,
        topSourceSlices: sourceSummary.topSourceSlices,
        dominantSourceShare: sourceSummary.dominantSourceShare,
        sourceScope: sourceSummary.sourceScope,
        sharedTraits: uniqueSorted(sharedTraits),
        sharedAnchors: uniqueSorted(sharedAnchors),
        nonNameAnchors,
        reviewFlags,
        representativeRecords: selectRepresentativeCandidates(
          bucket
            .slice()
            .sort((left, right) => right.similarity - left.similarity || left.name.localeCompare(right.name)),
        ),
        score,
        recommendation: recommendCluster(
          score,
          sharedAnchors,
          nonNameAnchors,
          averageAnchorStrength,
          anchorByValue,
          uniqueSorted(sharedTraits),
          reviewFlags,
        ),
      };
    })
    .sort(
      (left, right) =>
        right.score - left.score ||
        right.distinctVariantFamilies - left.distinctVariantFamilies ||
        right.size - left.size ||
        right.averageSimilarity - left.averageSimilarity ||
        left.signature.join(" ").localeCompare(right.signature.join(" ")),
    )
    .slice(0, cohortLimit);
}

export function discoverRuleableCohorts(db: DatabaseSync, options: RuleableCohortOptions): RuleableCohortReport {
  const normalizedTag = options.tag ? normalizeDerivedTag(options.tag) : null;
  const seedRecordKeys = normalizedTag
    ? uniqueSorted([
        ...getDerivedTagExemplarRecordKeys(normalizedTag, {
          category: options.category,
          subcategory: options.subcategory,
        }),
        ...getDerivedTagLegacySeedMigrationRecordKeys(normalizedTag, {
          category: options.category,
          subcategory: options.subcategory,
        }),
      ])
    : [];
  const resolvedExemplars: ResolvedDiscoveryExemplar[] = normalizedTag
    ? mergeUniqueRecords([
        ...loadDiscoveryRecords(db, {
          category: options.category,
          subcategory: options.subcategory,
          requireTag: normalizedTag,
          includeVectors: true,
        }),
        ...(seedRecordKeys.length > 0
          ? loadDiscoveryRecords(db, {
              category: options.category,
              subcategory: options.subcategory,
              recordKeys: seedRecordKeys,
              includeVectors: true,
            })
          : []),
      ]).map((record) => ({
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

  const category =
    options.category ??
    inferSingleValue(
      resolvedExemplars.map((record) => record.category),
      "category",
    );
  const subcategory = options.subcategory ?? null;
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
  const contrastRecords = selectDistinctContrastCandidates(
    rankedCandidates.filter(
      (candidate) => candidate.anchorOverlap.filter((anchor) => anchorValues.has(anchor)).length <= 1,
    ),
    6,
  );

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

function inferSingleValue<T>(values: T[], label: string): T {
  const uniqueValues = [...new Set(values)];
  if (uniqueValues.length !== 1) {
    throw new Error(
      `Exemplars span multiple ${label} values. Pass --${label} explicitly to define the candidate scope.`,
    );
  }

  return uniqueValues[0]!;
}
