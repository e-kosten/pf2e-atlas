import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import {
  extractDiscoveryNgrams,
  normalizeDiscoveryFeature,
  tokenizeDiscoveryText,
} from "./discovery-normalization.js";
import {
  type DiscoveryAnalysisRecord,
  loadDiscoveryRecords,
} from "./discovery-records.js";
import type { CohortRecommendation } from "./cohort-discovery.js";

const DEFAULT_COHORT_LIMIT = 8;
const DEFAULT_ANCHOR_LIMIT = 16;
const DEFAULT_MIN_FEATURE_SUPPORT = 2;
const DEFAULT_MIN_FEATURE_LIFT = 1.2;
const MAX_TEXT_FEATURE_FRACTION = 0.04;
const MAX_STRUCTURED_FEATURE_FRACTION = 0.2;
const MAX_INFORMATIVE_FEATURES_PER_RECORD = 12;
const MAX_NEIGHBORS_PER_RECORD = 80;

export type UntaggedCohortOptions = {
  category: SearchCategory;
  subcategory?: SearchSubcategory;
  cohortLimit?: number;
  anchorLimit?: number;
  minFeatureSupport?: number;
  minFeatureLift?: number;
};

export type UntaggedCohortAnchor = {
  value: string;
  support: number;
  baselineSupport: number;
  lift: number;
  score: number;
};

export type UntaggedCohortCluster = {
  signature: string[];
  size: number;
  averageSimilarity: number;
  sharedTraits: string[];
  anchorSupport: number;
  anchorLift: number;
  score: number;
  recommendation: CohortRecommendation;
  representativeRecords: Array<{
    recordKey: string;
    name: string;
    similarity: number;
  }>;
  contrastRecords: Array<{
    recordKey: string;
    name: string;
    similarity: number;
  }>;
};

export type UntaggedCohortReport = {
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  untaggedRecordCount: number;
  baselineRecordCount: number;
  anchorTerms: UntaggedCohortAnchor[];
  cohorts: UntaggedCohortCluster[];
};

type DiscoveryFeatureKind =
  | "name"
  | "name_phrase"
  | "text"
  | "text_phrase"
  | "trait"
  | "target"
  | "scope";

type DiscoveryFeature = {
  key: string;
  display: string;
  kind: DiscoveryFeatureKind;
};

type RankedFeature = UntaggedCohortAnchor & {
  key: string;
  kind: DiscoveryFeatureKind;
};

type RecordNode = {
  record: DiscoveryAnalysisRecord;
  informativeFeatureKeys: Set<string>;
  allFeatureKeys: Set<string>;
};

function featureTokenCount(value: string): number {
  return value.split(" ").filter(Boolean).length;
}

function featureSpecificityWeight(kind: DiscoveryFeatureKind, value: string): number {
  if (kind === "trait" || kind === "target" || kind === "scope") {
    return 1.15;
  }
  if (kind === "name" || kind === "text") {
    return 1;
  }

  const tokenCount = featureTokenCount(value);
  return tokenCount <= 2 ? 0.85 : 0.65;
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

function featureKey(kind: DiscoveryFeatureKind, value: string): string {
  return `${kind}:${value}`;
}

function featureDisplay(kind: DiscoveryFeatureKind, value: string): string {
  if (kind === "trait") {
    return `trait:${value}`;
  }
  if (kind === "target") {
    return `target:${value}`;
  }
  if (kind === "scope") {
    return `scope:${value}`;
  }

  return value;
}

function collectRecordFeatures(record: DiscoveryAnalysisRecord): DiscoveryFeature[] {
  const features: DiscoveryFeature[] = [];
  const push = (kind: DiscoveryFeatureKind, value: string): void => {
    if (!value) {
      return;
    }
    features.push({
      key: featureKey(kind, value),
      display: featureDisplay(kind, value),
      kind,
    });
  };

  for (const token of tokenizeDiscoveryText(record.name, { filterStopwords: true })) {
    push("name", token);
  }
  for (const phrase of extractDiscoveryNgrams(record.name, 2, { filterStopwords: true })) {
    push("name_phrase", phrase.normalized);
  }
  for (const phrase of extractDiscoveryNgrams(record.name, 3, { filterStopwords: true })) {
    push("name_phrase", phrase.normalized);
  }
  for (const token of tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true })) {
    push("text", token);
  }
  for (const phrase of extractDiscoveryNgrams(record.descriptionText ?? "", 2, { filterStopwords: true })) {
    push("text_phrase", phrase.normalized);
  }
  for (const phrase of extractDiscoveryNgrams(record.descriptionText ?? "", 3, { filterStopwords: true })) {
    push("text_phrase", phrase.normalized);
  }
  for (const trait of record.traits) {
    push("trait", normalizeDiscoveryFeature(trait));
  }
  for (const reference of record.references) {
    push("target", normalizeDiscoveryFeature(reference.targetName));
    push("scope", `${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`);
  }

  return dedupeFeatures(features);
}

function dedupeFeatures(features: DiscoveryFeature[]): DiscoveryFeature[] {
  const byKey = new Map<string, DiscoveryFeature>();
  for (const feature of features) {
    byKey.set(feature.key, feature);
  }
  return [...byKey.values()];
}

function collectFeatureSupport(
  records: DiscoveryAnalysisRecord[],
): { counts: Map<string, number>; byKey: Map<string, DiscoveryFeature>; featuresByRecordKey: Map<string, DiscoveryFeature[]> } {
  const counts = new Map<string, number>();
  const byKey = new Map<string, DiscoveryFeature>();
  const featuresByRecordKey = new Map<string, DiscoveryFeature[]>();

  for (const record of records) {
    const features = collectRecordFeatures(record);
    featuresByRecordKey.set(record.recordKey, features);
    for (const feature of features) {
      byKey.set(feature.key, feature);
      counts.set(feature.key, (counts.get(feature.key) ?? 0) + 1);
    }
  }

  return { counts, byKey, featuresByRecordKey };
}

function isPlaceholderOnlyFeature(feature: DiscoveryFeature): boolean {
  return feature.display === "{{number}}" || feature.display === "{{range}}" || feature.display === "{{dice}}";
}

function isEligibleSimilarityFeature(
  feature: DiscoveryFeature,
  support: number,
  totalRecords: number,
  minSupport: number,
): boolean {
  if (support < minSupport) {
    return false;
  }
  if (isPlaceholderOnlyFeature(feature)) {
    return false;
  }

  const maxFraction = feature.kind === "trait" || feature.kind === "target" || feature.kind === "scope"
    ? MAX_STRUCTURED_FEATURE_FRACTION
    : MAX_TEXT_FEATURE_FRACTION;
  if (support > Math.max(3, Math.floor(totalRecords * maxFraction))) {
    return false;
  }

  return true;
}

function buildRecordNodes(
  untagged: DiscoveryAnalysisRecord[],
  featureSupport: ReturnType<typeof collectFeatureSupport>,
  minSupport: number,
): RecordNode[] {
  const totalRecords = untagged.length;

  return untagged.map((record) => {
    const features = featureSupport.featuresByRecordKey.get(record.recordKey) ?? [];
    const informative = features
      .filter((feature) => isEligibleSimilarityFeature(feature, featureSupport.counts.get(feature.key) ?? 0, totalRecords, minSupport))
      .sort((left, right) =>
        (featureSupport.counts.get(left.key) ?? Number.POSITIVE_INFINITY) - (featureSupport.counts.get(right.key) ?? Number.POSITIVE_INFINITY) ||
        left.display.localeCompare(right.display))
      .slice(0, MAX_INFORMATIVE_FEATURES_PER_RECORD)
      .map((feature) => feature.key);

    return {
      record,
      informativeFeatureKeys: new Set(informative),
      allFeatureKeys: new Set(features.map((feature) => feature.key)),
    };
  });
}

function buildNeighborGraph(
  nodes: RecordNode[],
): Map<string, Set<string>> {
  const buckets = new Map<string, string[]>();
  const nodeByKey = new Map(nodes.map((node) => [node.record.recordKey, node] as const));

  for (const node of nodes) {
    for (const featureKey of node.informativeFeatureKeys) {
      const bucket = buckets.get(featureKey) ?? [];
      bucket.push(node.record.recordKey);
      buckets.set(featureKey, bucket);
    }
  }

  const graph = new Map<string, Set<string>>();
  for (const node of nodes) {
    const candidateWeights = new Map<string, number>();
    for (const featureKey of node.informativeFeatureKeys) {
      for (const candidateKey of buckets.get(featureKey) ?? []) {
        if (candidateKey === node.record.recordKey) {
          continue;
        }
        candidateWeights.set(candidateKey, (candidateWeights.get(candidateKey) ?? 0) + 1);
      }
    }

    const candidateKeys = [...candidateWeights.entries()]
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
      .slice(0, MAX_NEIGHBORS_PER_RECORD)
      .map(([candidateKey]) => candidateKey);

    for (const candidateKey of candidateKeys) {
      const candidate = nodeByKey.get(candidateKey);
      if (!candidate) {
        continue;
      }

      const sharedFeatures = [...node.informativeFeatureKeys].filter((key) => candidate.informativeFeatureKeys.has(key)).length;
      if (sharedFeatures === 0) {
        continue;
      }

      const union = new Set([...node.informativeFeatureKeys, ...candidate.informativeFeatureKeys]).size;
      const featureJaccard = union > 0 ? sharedFeatures / union : 0;
      const cosine = cosineSimilarity(node.record.vector, candidate.record.vector);
      const hybrid = (Math.max(0, cosine) * 0.45) + (featureJaccard * 0.4) + (Math.min(1, sharedFeatures / 4) * 0.15);
      const shouldLink = hybrid >= 0.36 || (sharedFeatures >= 2 && hybrid >= 0.26);
      if (!shouldLink) {
        continue;
      }

      const leftBucket = graph.get(node.record.recordKey) ?? new Set<string>();
      leftBucket.add(candidateKey);
      graph.set(node.record.recordKey, leftBucket);

      const rightBucket = graph.get(candidateKey) ?? new Set<string>();
      rightBucket.add(node.record.recordKey);
      graph.set(candidateKey, rightBucket);
    }
  }

  return graph;
}

function collectComponents(
  nodes: RecordNode[],
  graph: Map<string, Set<string>>,
): string[][] {
  const seen = new Set<string>();
  const components: string[][] = [];

  for (const node of nodes) {
    const start = node.record.recordKey;
    if (seen.has(start)) {
      continue;
    }

    const stack = [start];
    const component: string[] = [];
    seen.add(start);
    while (stack.length > 0) {
      const current = stack.pop()!;
      component.push(current);
      for (const neighbor of graph.get(current) ?? []) {
        if (seen.has(neighbor)) {
          continue;
        }
        seen.add(neighbor);
        stack.push(neighbor);
      }
    }

    if (component.length >= 2) {
      components.push(uniqueSorted(component));
    }
  }

  return components;
}

function rankClusterAnchors(
  memberKeys: string[],
  featureSupport: ReturnType<typeof collectFeatureSupport>,
  baselineSupport: ReturnType<typeof collectFeatureSupport>,
  options: UntaggedCohortOptions,
): RankedFeature[] {
  const supportCounts = new Map<string, number>();
  for (const recordKey of memberKeys) {
    for (const feature of featureSupport.featuresByRecordKey.get(recordKey) ?? []) {
      supportCounts.set(feature.key, (supportCounts.get(feature.key) ?? 0) + 1);
    }
  }

  const minLift = Math.max(1, options.minFeatureLift ?? DEFAULT_MIN_FEATURE_LIFT);
  return [...supportCounts.entries()]
    .map(([key, support]) => {
      const feature = featureSupport.byKey.get(key);
      if (!feature) {
        return null;
      }
      if (isPlaceholderOnlyFeature(feature)) {
        return null;
      }

      const baselineCount = baselineSupport.counts.get(key) ?? 0;
      const clusterRatio = (support + 1) / Math.max(1, memberKeys.length + 2);
      const baselineRatio = (baselineCount + 1) / Math.max(1, baselineSupport.featuresByRecordKey.size + 2);
      const lift = baselineRatio > 0 ? clusterRatio / baselineRatio : support;
      const score = support * Math.max(1, lift) * featureSpecificityWeight(feature.kind, feature.display);

      return {
        key,
        kind: feature.kind,
        value: feature.display,
        support,
        baselineSupport: baselineCount,
        lift,
        score,
      } satisfies RankedFeature;
    })
    .filter((entry): entry is RankedFeature => Boolean(entry))
    .filter((entry) => entry.support >= Math.max(2, Math.ceil(memberKeys.length * 0.45)) && entry.lift >= minLift)
    .sort((left, right) =>
      right.score - left.score ||
      right.support - left.support ||
      right.lift - left.lift ||
      featureTokenCount(left.value) - featureTokenCount(right.value) ||
      left.value.localeCompare(right.value))
    .slice(0, 6);
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

function jaccard(left: Set<string>, right: Set<string>): number {
  const intersection = [...left].filter((value) => right.has(value)).length;
  const union = new Set([...left, ...right]).size;
  return union === 0 ? 0 : intersection / union;
}

function buildCandidateCohorts(
  untagged: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
  options: UntaggedCohortOptions,
): UntaggedCohortCluster[] {
  const featureSupport = collectFeatureSupport(untagged);
  const baselineSupport = collectFeatureSupport(baseline);
  const minSupport = Math.max(1, options.minFeatureSupport ?? DEFAULT_MIN_FEATURE_SUPPORT);
  const nodes = buildRecordNodes(untagged, featureSupport, minSupport);
  const graph = buildNeighborGraph(nodes);
  const components = collectComponents(nodes, graph);
  const recordsByKey = new Map(untagged.map((record) => [record.recordKey, record] as const));
  const allFeatureKeysByRecordKey = new Map(nodes.map((node) => [node.record.recordKey, node.allFeatureKeys] as const));
  const cohortLimit = Math.max(1, Math.min(options.cohortLimit ?? DEFAULT_COHORT_LIMIT, 20));
  return components
    .map((memberKeys) => {
      const members = memberKeys.map((recordKey) => recordsByKey.get(recordKey)).filter((record): record is DiscoveryAnalysisRecord => Boolean(record));
      const clusterAnchors = rankClusterAnchors(memberKeys, featureSupport, baselineSupport, options);
      const centroid = normalizeVector(averageVectors(members.map((record) => record.vector).filter((vector) => vector.length > 0)));
      const memberSimilarities = members.map((record) => ({
        record,
        similarity: cosineSimilarity(record.vector, centroid),
      }));
      const averageSimilarity = memberSimilarities.reduce((total, entry) => total + entry.similarity, 0) / Math.max(1, memberSimilarities.length);
      const sharedTraits = [...new Set(members.flatMap((member) => member.traits))]
        .filter((trait) => members.every((member) => member.traits.includes(trait)));
      const nonMembers = untagged.filter((record) => !memberKeys.includes(record.recordKey));
      const signatureKeys = clusterAnchors.map((anchor) => anchor.key);
      const signatureDisplay = clusterAnchors.map((anchor) => anchor.value);
      const signatureKeySet = new Set(signatureKeys);
      const contrastRecords = nonMembers
        .map((record) => {
          const similarity = cosineSimilarity(record.vector, centroid);
          const featureSet = allFeatureKeysByRecordKey.get(record.recordKey) ?? new Set<string>();
          const overlap = signatureKeys.filter((key) => featureSet.has(key)).length;
          return { record, similarity, overlap };
        })
        .filter((entry) => signatureKeySet.size === 0 || entry.overlap < Math.max(1, Math.ceil(signatureKeySet.size / 2)))
        .sort((left, right) => right.similarity - left.similarity || left.record.name.localeCompare(right.record.name))
        .slice(0, 5)
        .map((entry) => ({
          recordKey: entry.record.recordKey,
          name: entry.record.name,
          similarity: entry.similarity,
        }));

      const sizeFactor = Math.min(1, members.length / 6);
      const similarityFactor = Math.max(0, Math.min(1, averageSimilarity));
      const density = signatureKeys.length > 0
        ? members.reduce((total, member) => {
          const featureSet = allFeatureKeysByRecordKey.get(member.recordKey) ?? new Set<string>();
          const overlap = signatureKeys.filter((key) => featureSet.has(key)).length;
          return total + (overlap / signatureKeys.length);
        }, 0) / Math.max(1, members.length)
        : 0;
      const liftFactor = clusterAnchors.length > 0
        ? Math.max(0, Math.min(1, (clusterAnchors[0]!.lift ?? 0) / 6))
        : 0;
      const score = Math.max(0, Math.min(1, (sizeFactor * 0.35) + (similarityFactor * 0.25) + (density * 0.25) + (liftFactor * 0.15)));

      return {
        signature: uniqueSorted(signatureDisplay).slice(0, 4),
        size: members.length,
        averageSimilarity,
        sharedTraits: uniqueSorted(sharedTraits),
        anchorSupport: clusterAnchors[0]?.support ?? 0,
        anchorLift: clusterAnchors[0]?.lift ?? 0,
        score,
        recommendation: recommendCluster(score),
        representativeRecords: memberSimilarities
          .sort((left, right) => right.similarity - left.similarity || left.record.name.localeCompare(right.record.name))
          .slice(0, 5)
          .map((entry) => ({
            recordKey: entry.record.recordKey,
            name: entry.record.name,
            similarity: entry.similarity,
          })),
        contrastRecords,
      } satisfies UntaggedCohortCluster;
    })
    .filter((cluster) => cluster.signature.length > 0 || cluster.size >= 3)
    .sort((left, right) =>
      right.score - left.score ||
      right.size - left.size ||
      right.averageSimilarity - left.averageSimilarity ||
      left.signature.join(" ").localeCompare(right.signature.join(" ")))
    .slice(0, cohortLimit);
}

export function discoverUntaggedCohorts(
  db: DatabaseSync,
  options: UntaggedCohortOptions,
): UntaggedCohortReport {
  const untagged = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    untaggedOnly: true,
    includeVectors: true,
  });
  const baseline = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    includeVectors: true,
  });
  if (untagged.length === 0) {
    throw new Error("No untagged canonical records matched the requested scope.");
  }

  const cohorts = buildCandidateCohorts(untagged, baseline, options);
  const anchorTerms = uniqueSorted(cohorts.flatMap((cohort) => cohort.signature))
    .map((value) => {
      const members = cohorts.filter((cohort) => cohort.signature.includes(value));
      const exemplar = members[0];
      return exemplar
        ? {
          value,
          support: exemplar.anchorSupport,
          baselineSupport: 0,
          lift: exemplar.anchorLift,
          score: exemplar.score,
        }
        : null;
    })
    .filter((entry): entry is UntaggedCohortAnchor => Boolean(entry))
    .sort((left, right) => right.score - left.score || right.lift - left.lift || left.value.localeCompare(right.value))
    .slice(0, Math.max(1, Math.min(options.anchorLimit ?? DEFAULT_ANCHOR_LIMIT, 50)));

  return {
    category: options.category,
    subcategory: options.subcategory ?? null,
    untaggedRecordCount: untagged.length,
    baselineRecordCount: baseline.length,
    anchorTerms,
    cohorts,
  };
}
