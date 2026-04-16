import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../../types.js";
import { uniqueSorted } from "../../utils.js";
import { getDerivedTagFamilyTags, normalizeDerivedTag } from "../index.js";
import {
  getReviewedDiscoverySelection,
  summarizeReviewedDiscoverySelection,
  type ReviewedDiscoveryApplicationSummary,
  type ReviewedDiscoveryReason,
} from "./discovery-reviewed-records.js";
import {
  classifyFamilyGapFeature,
  type FamilyGapFeatureBucket,
} from "./family-gap-signals.js";
import {
  extractDiscoveryGramRange,
  isDiscoveryNoisePhrase,
  isDiscoveryNoiseToken,
  isDiscoveryPlaceholder,
  normalizeDiscoveryFeature,
  resolveDiscoveryGramRange,
  tokenizeDiscoveryText,
} from "./discovery-normalization.js";
import {
  type DiscoveryAnalysisRecord,
  loadDiscoveryRecords,
} from "./discovery-records.js";
import { summarizeDiscoverySources } from "./discovery-source-summary.js";
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
  family?: string;
  familyGapSignals?: boolean;
  includeReviewed?: boolean;
  reviewReason?: ReviewedDiscoveryReason;
  cohortLimit?: number;
  anchorLimit?: number;
  minFeatureSupport?: number;
  minFeatureLift?: number;
  minGramLength?: number;
  maxGramLength?: number;
  progressLogger?: (message: string) => void;
  progressStatusLogger?: (message: string) => void;
};

export type UntaggedCohortAnchor = {
  value: string;
  support: number;
  baselineSupport: number;
  lift: number;
  score: number;
  bucket?: FamilyGapFeatureBucket;
  existingTagOverlaps?: string[];
};

export type UntaggedCohortCluster = {
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
  nonNameAnchors: string[];
  reviewFlags: string[];
  anchorSupport: number;
  anchorLift: number;
  score: number;
  recommendation: CohortRecommendation;
  classification?: "new_concept_candidate" | "existing_tag_coverage_gap" | "context_only";
  familyGapRecommendation?: "new-tag" | "extend-existing-tag" | "manual-only";
  overlappingTags?: string[];
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
  family: string | null;
  untaggedRecordCount: number;
  baselineRecordCount: number;
  coveredRecordCount?: number;
  liveTags?: string[];
  reviewedRecords?: ReviewedDiscoveryApplicationSummary;
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
  bucket?: FamilyGapFeatureBucket;
  existingTagOverlaps?: string[];
  cueStrength?: "strong" | "weak" | "novel" | "none";
  cueLocality?: number;
  cueAmbiguityPenalty?: number;
  boilerplateRisk?: number;
};

type RecordNode = {
  record: DiscoveryAnalysisRecord;
  informativeFeatureKeys: Set<string>;
  allFeatureKeys: Set<string>;
};

type SelectedUntaggedCohorts = {
  cohorts: UntaggedCohortCluster[];
  anchorTerms: UntaggedCohortAnchor[];
};

function variantFamilyIdentity(record: DiscoveryAnalysisRecord): string {
  return record.variantFamilyKey ?? record.recordKey;
}

function distinctVariantFamilyCount(records: DiscoveryAnalysisRecord[]): number {
  return new Set(records.map((record) => variantFamilyIdentity(record))).size;
}

function selectRepresentativeRecords(
  members: Array<{ record: DiscoveryAnalysisRecord; similarity: number }>,
  limit = 5,
): Array<{ recordKey: string; name: string; similarity: number }> {
  const selected: Array<{ recordKey: string; name: string; similarity: number }> = [];
  const seenFamilies = new Set<string>();

  for (const entry of members) {
    const familyId = variantFamilyIdentity(entry.record);
    if (seenFamilies.has(familyId)) {
      continue;
    }
    seenFamilies.add(familyId);
    selected.push({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      similarity: entry.similarity,
    });
    if (selected.length >= limit) {
      return selected;
    }
  }

  for (const entry of members) {
    if (selected.some((selectedEntry) => selectedEntry.recordKey === entry.record.recordKey)) {
      continue;
    }
    selected.push({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      similarity: entry.similarity,
    });
    if (selected.length >= limit) {
      break;
    }
  }

  return selected;
}

function selectDistinctContrastRecords(
  members: Array<{ record: DiscoveryAnalysisRecord; similarity: number }>,
  limit = 5,
): Array<{ recordKey: string; name: string; similarity: number }> {
  const selected: Array<{ recordKey: string; name: string; similarity: number }> = [];
  const seenFamilies = new Set<string>();

  for (const entry of members) {
    const familyId = variantFamilyIdentity(entry.record);
    if (seenFamilies.has(familyId)) {
      continue;
    }
    seenFamilies.add(familyId);
    selected.push({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      similarity: entry.similarity,
    });
    if (selected.length >= limit) {
      break;
    }
  }

  return selected;
}

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

function isNameFeatureKind(kind: DiscoveryFeatureKind): boolean {
  return kind === "name" || kind === "name_phrase";
}

function isLexicalFeatureKind(kind: DiscoveryFeatureKind): boolean {
  return kind === "name" || kind === "name_phrase" || kind === "text" || kind === "text_phrase";
}

function deriveReviewFlags(
  anchors: RankedFeature[],
  sharedTraits: string[],
  sourceSummary: ReturnType<typeof summarizeDiscoverySources>,
  distinctVariantFamilies: number,
): string[] {
  const flags: string[] = [];
  const nameAnchors = anchors.filter((anchor) => isNameFeatureKind(anchor.kind));
  const nonNameAnchors = anchors.filter((anchor) => !isNameFeatureKind(anchor.kind));
  const lexicalOnly = anchors.length > 0 && anchors.every((anchor) => isLexicalFeatureKind(anchor.kind));
  const traitOnly = anchors.length > 0 && anchors.every((anchor) => anchor.kind === "trait");
  const dominantNameSupport = nameAnchors[0]?.support ?? 0;
  const nameSeriesSupportFloor = Math.max(2, Math.ceil(distinctVariantFamilies * 0.6));
  const overlapShare = dominantOverlapShare(anchors);
  const locality = averageAnchorLocality(anchors);
  const boilerplateRisk = averageBoilerplateRisk(anchors);

  if (anchors.length === 0) {
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
  if (sourceSummary.hasUsableSourceSignals && sourceSummary.dominantSourceShare > 0 && sourceSummary.sourceScope && (
    (sourceSummary.sourceScope === "source-slice" && sourceSummary.sourceSliceCount === 1) ||
    (sourceSummary.sourceScope === "publication" && sourceSummary.publicationCount === 1) ||
    (sourceSummary.sourceScope === "pack" && sourceSummary.sourceCount === 1)
  )) {
    flags.push("source-local");
  } else if (sourceSummary.hasUsableSourceSignals && sourceSummary.dominantSourceShare >= 0.75) {
    flags.push("source-heavy");
  }
  if (nonNameAnchors.length === 0 && anchors.length > 0) {
    flags.push("name-anchored");
  }
  if (anchors.length > 0 && locality < 0.72) {
    flags.push("weak-locality");
  }
  if (hasMixedSettingCueSignal(anchors, overlapShare)) {
    flags.push("mixed-setting-cues");
  }
  if (boilerplateRisk >= 0.3 || (sourceSummary.dominantSourceShare >= 0.75 && boilerplateRisk >= 0.2)) {
    flags.push("boilerplate-heavy");
  }

  return uniqueSorted(flags);
}

function averageAnchorLocality(anchors: RankedFeature[]): number {
  if (anchors.length === 0) {
    return 0;
  }
  return anchors.reduce((total, anchor) => total + (anchor.cueLocality ?? 0.35), 0) / anchors.length;
}

function averageBoilerplateRisk(anchors: RankedFeature[]): number {
  if (anchors.length === 0) {
    return 0;
  }
  return anchors.reduce((total, anchor) => total + (anchor.boilerplateRisk ?? 0), 0) / anchors.length;
}

function dominantOverlapShare(anchors: RankedFeature[]): number {
  const counts = new Map<string, number>();
  for (const anchor of anchors) {
    for (const overlap of anchor.existingTagOverlaps ?? []) {
      counts.set(overlap, (counts.get(overlap) ?? 0) + 1);
    }
  }

  if (counts.size <= 1) {
    return 1;
  }

  const total = [...counts.values()].reduce((sum, count) => sum + count, 0);
  const dominant = Math.max(...counts.values());
  return total > 0 ? dominant / total : 1;
}

function hasMixedSettingCueSignal(anchors: RankedFeature[], overlapShare: number): boolean {
  const overlappingTags = uniqueSorted(anchors.flatMap((anchor) => anchor.existingTagOverlaps ?? []));
  if (overlappingTags.length >= 3) {
    return true;
  }
  return overlappingTags.length === 2 && overlapShare < 0.67;
}

function collectRecordFeatures(
  record: DiscoveryAnalysisRecord,
  options: Pick<UntaggedCohortOptions, "minGramLength" | "maxGramLength"> = {},
): DiscoveryFeature[] {
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
    if (!isDiscoveryNoiseToken(token)) {
      push("name", token);
    }
  }
  for (const phrase of extractDiscoveryGramRange(record.name, options, { filterStopwords: true })) {
    if (!isDiscoveryNoisePhrase(phrase.normalized)) {
      push("name_phrase", phrase.normalized);
    }
  }
  for (const token of tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true })) {
    if (!isDiscoveryNoiseToken(token)) {
      push("text", token);
    }
  }
  for (const phrase of extractDiscoveryGramRange(record.descriptionText ?? "", options, { filterStopwords: true })) {
    if (!isDiscoveryNoisePhrase(phrase.normalized)) {
      push("text_phrase", phrase.normalized);
    }
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
  options: Pick<UntaggedCohortOptions, "minGramLength" | "maxGramLength"> = {},
): { counts: Map<string, number>; byKey: Map<string, DiscoveryFeature>; featuresByRecordKey: Map<string, DiscoveryFeature[]> } {
  const counts = new Map<string, number>();
  const byKey = new Map<string, DiscoveryFeature>();
  const featuresByRecordKey = new Map<string, DiscoveryFeature[]>();

  for (const record of records) {
    const features = collectRecordFeatures(record, options);
    featuresByRecordKey.set(record.recordKey, features);
    for (const feature of features) {
      byKey.set(feature.key, feature);
      counts.set(feature.key, (counts.get(feature.key) ?? 0) + 1);
    }
  }

  return { counts, byKey, featuresByRecordKey };
}

function isPlaceholderOnlyFeature(feature: DiscoveryFeature): boolean {
  return isDiscoveryPlaceholder(feature.display);
}

function isEligibleSimilarityFeature(
  feature: DiscoveryFeature,
  support: number,
  totalRecords: number,
  minSupport: number,
  options: UntaggedCohortOptions,
): boolean {
  if (support < minSupport) {
    return false;
  }
  if (isPlaceholderOnlyFeature(feature)) {
    return false;
  }
  if (options.familyGapSignals) {
    const classification = classifyFamilyGapFeature(options.family, feature.kind, feature.display);
    if (!classification.eligibleForClustering) {
      return false;
    }
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
  options: UntaggedCohortOptions,
): RecordNode[] {
  const totalRecords = untagged.length;

  return untagged.map((record) => {
    const features = featureSupport.featuresByRecordKey.get(record.recordKey) ?? [];
    const informative = features
      .filter((feature) => isEligibleSimilarityFeature(feature, featureSupport.counts.get(feature.key) ?? 0, totalRecords, minSupport, options))
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
  progressStatusLogger?: (message: string) => void,
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
  const progressInterval = Math.max(25, Math.ceil(nodes.length / 20));
  for (let nodeIndex = 0; nodeIndex < nodes.length; nodeIndex += 1) {
    const node = nodes[nodeIndex]!;
    if (nodeIndex === 0 || (nodeIndex + 1) % progressInterval === 0 || nodeIndex + 1 === nodes.length) {
      const percent = Math.max(1, Math.min(100, Math.round(((nodeIndex + 1) / Math.max(1, nodes.length)) * 100)));
      progressStatusLogger?.(`Building neighbor graph ${percent}% (${nodeIndex + 1}/${nodes.length}).`);
    }

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
      const sameFamily = node.record.variantFamilyKey !== null && node.record.variantFamilyKey === candidate.record.variantFamilyKey;
      const hybridBase = (Math.max(0, cosine) * 0.45) + (featureJaccard * 0.4) + (Math.min(1, sharedFeatures / 4) * 0.15);
      const hybrid = sameFamily ? hybridBase - 0.18 : hybridBase;
      const shouldLink = sameFamily
        ? sharedFeatures >= 3 && hybrid >= 0.34
        : (hybrid >= 0.36 || (sharedFeatures >= 2 && hybrid >= 0.26));
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
  recordsByKey: Map<string, DiscoveryAnalysisRecord>,
  options: UntaggedCohortOptions,
): RankedFeature[] {
  const supportCounts = new Map<string, Set<string>>();
  for (const recordKey of memberKeys) {
    const record = recordsByKey.get(recordKey);
    const familyId = record ? variantFamilyIdentity(record) : recordKey;
    for (const feature of featureSupport.featuresByRecordKey.get(recordKey) ?? []) {
      const bucket = supportCounts.get(feature.key) ?? new Set<string>();
      bucket.add(familyId);
      supportCounts.set(feature.key, bucket);
    }
  }

  const baselineSupportCounts = new Map<string, Set<string>>();
  for (const [recordKey, features] of baselineSupport.featuresByRecordKey.entries()) {
    const record = recordsByKey.get(recordKey);
    const familyId = record ? variantFamilyIdentity(record) : recordKey;
    for (const feature of features) {
      const bucket = baselineSupportCounts.get(feature.key) ?? new Set<string>();
      bucket.add(familyId);
      baselineSupportCounts.set(feature.key, bucket);
    }
  }

  const minLift = Math.max(1, options.minFeatureLift ?? DEFAULT_MIN_FEATURE_LIFT);
  const memberFamilyCount = new Set(memberKeys.map((recordKey) => {
    const record = recordsByKey.get(recordKey);
    return record ? variantFamilyIdentity(record) : recordKey;
  })).size;
  const baselineFamilyCount = new Set([...recordsByKey.values()].map((record) => variantFamilyIdentity(record))).size;
  const rankedEntries: RankedFeature[] = [];
  for (const [key, supportSet] of supportCounts.entries()) {
    const feature = featureSupport.byKey.get(key);
    if (!feature || isPlaceholderOnlyFeature(feature)) {
      continue;
    }

    const support = supportSet.size;
    const baselineCount = baselineSupportCounts.get(key)?.size ?? 0;
    const clusterRatio = (support + 1) / Math.max(1, memberFamilyCount + 2);
    const baselineRatio = (baselineCount + 1) / Math.max(1, baselineFamilyCount + 2);
    const lift = baselineRatio > 0 ? clusterRatio / baselineRatio : support;
    const classification = classifyFamilyGapFeature(options.family, feature.kind, feature.display);
    if (options.familyGapSignals && (classification.bucket !== "possible_place_anchor" || isNameFeatureKind(feature.kind))) {
      continue;
    }
    const qualityMultiplier = Math.max(
      0.3,
      (classification.cueLocality - (classification.cueAmbiguityPenalty * 0.55)) *
      (1 - (classification.boilerplateRisk * 0.4)),
    );
    rankedEntries.push({
      key,
      kind: feature.kind,
      value: classification.primaryCue ?? feature.display,
      support,
      baselineSupport: baselineCount,
      lift,
      score: support * Math.max(1, lift) * featureSpecificityWeight(feature.kind, feature.display) * classification.familyConceptWeight * qualityMultiplier,
      bucket: classification.bucket,
      existingTagOverlaps: classification.existingTagOverlaps,
      cueStrength: classification.cueStrength,
      cueLocality: classification.cueLocality,
      cueAmbiguityPenalty: classification.cueAmbiguityPenalty,
      boilerplateRisk: classification.boilerplateRisk,
    });
  }

  return rankedEntries
    .filter((entry) => entry.support >= Math.max(2, Math.ceil(memberFamilyCount * 0.45)) && entry.lift >= minLift)
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

function recommendClusterForAnchors(
  score: number,
  anchors: RankedFeature[],
  sharedTraits: string[],
  nonNameAnchors: string[],
  reviewFlags: string[],
): CohortRecommendation {
  if (anchors.length === 0) {
    if (score >= 0.3) {
      return "manual-only";
    }
    return "reject";
  }

  const lexicalOnly = anchors.length > 0 && anchors.every((anchor) => isLexicalFeatureKind(anchor.kind));
  const traitOnly = anchors.length > 0 && anchors.every((anchor) => anchor.kind === "trait");

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
  if (reviewFlags.includes("mixed-setting-cues") || reviewFlags.includes("weak-locality")) {
    if (score >= 0.6 && nonNameAnchors.length >= 2 && !reviewFlags.includes("source-heavy")) {
      return "hybrid";
    }
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
    score >= 0.72 &&
    nonNameAnchors.length >= 2 &&
    !reviewFlags.includes("source-local") &&
    !reviewFlags.includes("source-heavy") &&
    !reviewFlags.includes("boilerplate-heavy")
  ) {
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

function buildCandidateCohorts(
  untagged: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
  options: UntaggedCohortOptions,
): SelectedUntaggedCohorts {
  const gramRange = resolveDiscoveryGramRange(options);
  options.progressStatusLogger?.(`Extracting discovery features for ${untagged.length} untagged records.`);
  const featureSupport = collectFeatureSupport(untagged, gramRange);
  options.progressStatusLogger?.(`Extracting baseline features for ${baseline.length} records.`);
  const baselineSupport = collectFeatureSupport(baseline, gramRange);
  const minSupport = Math.max(1, options.minFeatureSupport ?? DEFAULT_MIN_FEATURE_SUPPORT);
  options.progressStatusLogger?.("Selecting informative features per record.");
  const nodes = buildRecordNodes(untagged, featureSupport, minSupport, options);
  const graph = buildNeighborGraph(nodes, options.progressStatusLogger);
  options.progressStatusLogger?.("Collecting connected components.");
  const components = collectComponents(nodes, graph);
  const recordsByKey = new Map([...baseline, ...untagged].map((record) => [record.recordKey, record] as const));
  const allFeatureKeysByRecordKey = new Map(nodes.map((node) => [node.record.recordKey, node.allFeatureKeys] as const));
  const cohortLimit = Math.max(1, Math.min(options.cohortLimit ?? DEFAULT_COHORT_LIMIT, 20));
  const rankedClusters = components
    .map((memberKeys, componentIndex) => {
      if (
        componentIndex === 0
        || (componentIndex + 1) % Math.max(10, Math.ceil(components.length / 10)) === 0
        || componentIndex + 1 === components.length
      ) {
        const percent = Math.max(1, Math.min(100, Math.round(((componentIndex + 1) / Math.max(1, components.length)) * 100)));
        options.progressStatusLogger?.(`Scoring cohort candidates ${percent}% (${componentIndex + 1}/${components.length}).`);
      }

      const members = memberKeys.map((recordKey) => recordsByKey.get(recordKey)).filter((record): record is DiscoveryAnalysisRecord => Boolean(record));
      const clusterAnchors = rankClusterAnchors(memberKeys, featureSupport, baselineSupport, recordsByKey, options);
      const centroid = normalizeVector(averageVectors(members.map((record) => record.vector).filter((vector) => vector.length > 0)));
      const memberSimilarities = members.map((record) => ({
        record,
        similarity: cosineSimilarity(record.vector, centroid),
      }));
      const averageSimilarity = memberSimilarities.reduce((total, entry) => total + entry.similarity, 0) / Math.max(1, memberSimilarities.length);
      const distinctVariantFamilies = distinctVariantFamilyCount(members);
      const sharedTraits = [...new Set(members.flatMap((member) => member.traits))]
        .filter((trait) => members.every((member) => member.traits.includes(trait)));
      const sourceSummary = summarizeDiscoverySources(members);
      const reviewFlags = deriveReviewFlags(clusterAnchors, uniqueSorted(sharedTraits), sourceSummary, distinctVariantFamilies);
      const nonNameAnchors = clusterAnchors
        .filter((anchor) => !isNameFeatureKind(anchor.kind))
        .map((anchor) => anchor.value)
        .slice(0, 4);
      const nonMembers = untagged.filter((record) => !memberKeys.includes(record.recordKey));
      const signatureKeys = clusterAnchors.map((anchor) => anchor.key);
      const signatureDisplay = clusterAnchors.map((anchor) => anchor.value);
      const signatureKeySet = new Set(signatureKeys);
      const localityFactor = averageAnchorLocality(clusterAnchors);
      const overlapShare = dominantOverlapShare(clusterAnchors);
      const boilerplateRisk = averageBoilerplateRisk(clusterAnchors);
      const contrastRecords = selectDistinctContrastRecords(nonMembers
        .map((record) => {
          const similarity = cosineSimilarity(record.vector, centroid);
          const featureSet = allFeatureKeysByRecordKey.get(record.recordKey) ?? new Set<string>();
          const overlap = signatureKeys.filter((key) => featureSet.has(key)).length;
          return { record, similarity, overlap };
        })
        .filter((entry) => signatureKeySet.size === 0 || entry.overlap < Math.max(1, Math.ceil(signatureKeySet.size / 2)))
        .sort((left, right) => right.similarity - left.similarity || left.record.name.localeCompare(right.record.name))
      , 5);

      const sizeFactor = Math.min(1, members.length / 6);
      const familyDiversity = distinctVariantFamilies / Math.max(1, members.length);
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
      const localityBonus = clusterAnchors.length > 0 ? localityFactor * 0.12 : 0;
      const overlapCoherenceBonus = clusterAnchors.length > 0 ? overlapShare * 0.08 : 0;
      const lexicalOnlyPenalty = clusterAnchors.length > 0 && clusterAnchors.every((anchor) => isLexicalFeatureKind(anchor.kind)) && sharedTraits.length === 0
        ? 0.18
        : 0;
      const noAnchorPenalty = clusterAnchors.length === 0 ? 0.18 : 0;
      const nameOnlyPenalty = clusterAnchors.length > 0 && clusterAnchors.every((anchor) => anchor.kind === "name" || anchor.kind === "name_phrase")
        ? 0.08
        : 0;
      const traitOnlyPenalty = clusterAnchors.length > 0 && clusterAnchors.every((anchor) => anchor.kind === "trait") && sharedTraits.length <= 1
        ? 0.12
        : 0;
      const nameSeriesPenalty = reviewFlags.includes("name-series")
        ? nonNameAnchors.length >= 2 ? 0.08 : 0.18
        : 0;
      const sourcePenalty = reviewFlags.includes("source-local")
        ? 0.16
        : reviewFlags.includes("source-heavy")
          ? 0.08
          : 0;
      const weakLocalityPenalty = reviewFlags.includes("weak-locality") ? 0.12 : 0;
      const mixedCuePenalty = reviewFlags.includes("mixed-setting-cues") ? 0.14 : 0;
      const boilerplatePenalty = reviewFlags.includes("boilerplate-heavy")
        ? 0.08
        : boilerplateRisk * 0.08;
      const score = Math.max(
        0,
        Math.min(
          1,
          (sizeFactor * 0.25) +
          (familyDiversity * 0.25) +
          (similarityFactor * 0.2) +
          (density * 0.2) +
          (liftFactor * 0.1) +
          localityBonus +
          overlapCoherenceBonus -
          lexicalOnlyPenalty -
          noAnchorPenalty -
          nameOnlyPenalty -
          traitOnlyPenalty -
          nameSeriesPenalty -
          sourcePenalty -
          weakLocalityPenalty -
          mixedCuePenalty -
          boilerplatePenalty,
        ),
      );
      const overlappingTags = uniqueSorted(clusterAnchors.flatMap((anchor) => anchor.existingTagOverlaps ?? []));
      const classification: "new_concept_candidate" | "existing_tag_coverage_gap" | "context_only" = overlappingTags.length > 0
        ? "existing_tag_coverage_gap"
        : clusterAnchors.some((anchor) => anchor.bucket === "possible_place_anchor")
          ? "new_concept_candidate"
          : "context_only";
      const familyGapRecommendation: "new-tag" | "extend-existing-tag" | "manual-only" = classification === "existing_tag_coverage_gap"
        ? "extend-existing-tag"
        : classification === "new_concept_candidate"
          ? "new-tag"
          : "manual-only";

      return {
        signature: uniqueSorted(signatureDisplay).slice(0, 4),
        size: members.length,
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
        nonNameAnchors,
        reviewFlags,
        anchorSupport: clusterAnchors[0]?.support ?? 0,
        anchorLift: clusterAnchors[0]?.lift ?? 0,
        score,
        recommendation: recommendClusterForAnchors(score, clusterAnchors, uniqueSorted(sharedTraits), nonNameAnchors, reviewFlags),
        classification,
        familyGapRecommendation,
        overlappingTags,
        representativeRecords: selectRepresentativeRecords(
          memberSimilarities
            .sort((left, right) => right.similarity - left.similarity || left.record.name.localeCompare(right.record.name)),
        ),
        contrastRecords,
        rankedAnchors: clusterAnchors,
      };
    })
    .filter((cluster) => cluster.signature.length > 0 || cluster.size >= 3)
    .sort((left, right) =>
      right.score - left.score ||
      right.distinctVariantFamilies - left.distinctVariantFamilies ||
      right.size - left.size ||
      right.averageSimilarity - left.averageSimilarity ||
      left.signature.join(" ").localeCompare(right.signature.join(" ")))
    .slice(0, cohortLimit);
  const anchorTermByValue = new Map<string, UntaggedCohortAnchor>();
  for (const cluster of rankedClusters) {
    for (const anchor of cluster.rankedAnchors) {
      const existing = anchorTermByValue.get(anchor.value);
      const candidate = {
        value: anchor.value,
        support: anchor.support,
        baselineSupport: anchor.baselineSupport,
        lift: anchor.lift,
        score: anchor.score,
        bucket: anchor.bucket,
        existingTagOverlaps: anchor.existingTagOverlaps,
      } satisfies UntaggedCohortAnchor;
      if (
        !existing ||
        candidate.score > existing.score ||
        (candidate.score === existing.score && candidate.support > existing.support) ||
        (candidate.score === existing.score && candidate.support === existing.support && candidate.value.localeCompare(existing.value) < 0)
      ) {
        anchorTermByValue.set(anchor.value, candidate);
      }
    }
  }
  options.progressStatusLogger?.(`Selected top ${rankedClusters.length} cohort candidates.`);
  return {
    cohorts: rankedClusters.map(({ rankedAnchors: _rankedAnchors, ...cluster }) => cluster),
    anchorTerms: [...anchorTermByValue.values()]
      .sort((left, right) => right.score - left.score || right.support - left.support || right.lift - left.lift || left.value.localeCompare(right.value))
      .slice(0, Math.max(1, Math.min(options.anchorLimit ?? DEFAULT_ANCHOR_LIMIT, 50))),
  };
}

export function discoverUntaggedCohorts(
  db: DatabaseSync,
  options: UntaggedCohortOptions,
): UntaggedCohortReport {
  const normalizedFamily = options.family ? normalizeDerivedTag(options.family) : undefined;
  const familyTags = normalizedFamily
    ? getDerivedTagFamilyTags(normalizedFamily, {
      category: options.category,
      subcategory: options.subcategory,
    })
    : [];
  const overallStartTime = Date.now();
  const logPhase = (label: string, startTime: number): void => {
    options.progressLogger?.(`${label} in ${Math.max(0, Date.now() - startTime)}ms.`);
  };
  const reviewedSelection = normalizedFamily
    ? getReviewedDiscoverySelection({
      category: options.category,
      subcategory: options.subcategory,
      family: normalizedFamily,
      includeReviewed: options.includeReviewed,
      reviewReason: options.reviewReason,
    })
    : undefined;
  const reviewedRecordKeys = reviewedSelection?.mode === "filtered" ? reviewedSelection.recordKeys : undefined;
  const reviewedExcludedRecordKeys = reviewedSelection?.mode === "excluded" ? reviewedSelection.recordKeys : undefined;
  const reviewedSummary = reviewedSelection
    ? summarizeReviewedDiscoverySelection(reviewedSelection)
    : undefined;

  options.progressStatusLogger?.("Loading untagged records.");
  const untaggedStartTime = Date.now();
  const untagged = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: reviewedRecordKeys,
    excludeRecordKeys: reviewedExcludedRecordKeys,
    excludeAnyDerivedTags: familyTags,
    untaggedOnly: familyTags.length === 0,
    includeVectors: true,
    includeDerivedTags: false,
  });
  logPhase(`Loaded ${untagged.length} untagged records`, untaggedStartTime);

  options.progressStatusLogger?.("Loading baseline records.");
  const baselineStartTime = Date.now();
  const baseline = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    includeVectors: false,
    includeDerivedTags: false,
  });
  logPhase(`Loaded ${baseline.length} baseline records`, baselineStartTime);
  const covered = normalizedFamily
    ? loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      requireAnyDerivedTags: familyTags,
      includeVectors: false,
    })
    : [];
  if (untagged.length === 0) {
    if (normalizedFamily) {
      throw new Error(`No canonical records without derived tags in family "${normalizedFamily}" matched the requested scope.`);
    }
    throw new Error("No untagged canonical records matched the requested scope.");
  }

  options.progressStatusLogger?.("Building candidate cohorts.");
  const cohortStartTime = Date.now();
  const { cohorts, anchorTerms } = buildCandidateCohorts(untagged, baseline, options);
  logPhase(`Built ${cohorts.length} candidate cohorts`, cohortStartTime);

  options.progressStatusLogger?.("Ranking top anchors.");
  const anchorStartTime = Date.now();
  logPhase(`Ranked ${anchorTerms.length} anchor terms`, anchorStartTime);
  options.progressLogger?.(`Untagged cohort discovery finished in ${Math.max(0, Date.now() - overallStartTime)}ms.`);

  return {
    category: options.category,
    subcategory: options.subcategory ?? null,
    family: normalizedFamily ?? null,
    untaggedRecordCount: untagged.length,
    baselineRecordCount: baseline.length,
    coveredRecordCount: covered.length > 0 ? covered.length : undefined,
    liveTags: covered.length > 0
      ? uniqueSorted([...new Set(covered.flatMap((record) =>
        record.derivedTags.filter((tag) => familyTags.includes(normalizeDerivedTag(tag))),
      ))])
      : undefined,
    reviewedRecords: reviewedSummary,
    anchorTerms,
    cohorts,
  };
}
