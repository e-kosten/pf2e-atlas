import { DatabaseSync } from "node:sqlite";

import { SearchCategory, SearchSubcategory } from "../types.js";
import { uniqueSorted } from "../utils.js";
import {
  getDerivedTagExemplarRecordKeys,
  getDerivedTagFamilyTags,
  getDerivedTagLegacySeedMigrationRecordKeys,
  normalizeDerivedTag,
} from "./index.js";
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
  normalizeDiscoveryFeature,
  resolveDiscoveryGramRange,
  tokenizeDiscoveryText,
} from "./discovery-normalization.js";
import {
  type DiscoveryAnalysisRecord,
  loadDiscoveryRecords,
} from "./discovery-records.js";

const DEFAULT_EVIDENCE_LIMIT = 12;
const DEFAULT_EXAMPLE_LIMIT = 3;

export type DiscoveryEvidenceKind =
  | "nameToken"
  | "namePhrase"
  | "descriptionToken"
  | "descriptionPhrase"
  | "trait"
  | "reference";

function isNameDerivedEvidenceKind(kind: DiscoveryEvidenceKind): boolean {
  return kind === "nameToken" || kind === "namePhrase";
}

export type DiscoveryEvidenceTerm = {
  kind: DiscoveryEvidenceKind;
  value: string;
  support: number;
  cohortSupport: number;
  baselineSupport: number;
  lift: number;
  score: number;
  examples: string[];
};

export type DiscoveryEvidenceReport = {
  category: SearchCategory | null;
  subcategory: SearchSubcategory | null;
  family: string | null;
  cohortSize: number;
  baselineSize: number;
  reviewedRecords?: ReviewedDiscoveryApplicationSummary;
  nameTokens: DiscoveryEvidenceTerm[];
  namePhrases: DiscoveryEvidenceTerm[];
  descriptionTokens: DiscoveryEvidenceTerm[];
  descriptionPhrases: DiscoveryEvidenceTerm[];
  traits: DiscoveryEvidenceTerm[];
  references: DiscoveryEvidenceTerm[];
  familyGap?: FamilyGapReport;
  representativeRecords: Array<{
    recordKey: string;
    name: string;
    traits: string[];
  }>;
};

export type FamilyGapEvidenceTerm = DiscoveryEvidenceTerm & {
  bucket: FamilyGapFeatureBucket;
  existingTagOverlaps: string[];
  coveredSupport: number;
  gapLift: number;
  suppressionReason: string | null;
};

export type FamilyGapReport = {
  coveredCount: number;
  uncoveredCount: number;
  baselineCount: number;
  liveTags: string[];
  likelyNewConcepts: FamilyGapEvidenceTerm[];
  existingTagCoverageGaps: FamilyGapEvidenceTerm[];
  suppressedTerms: FamilyGapEvidenceTerm[];
};

export type DiscoveryEvidenceOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  recordKeys?: string[];
  excludeRecordKeys?: string[];
  tag?: string;
  family?: string;
  excludeDerivedTag?: string;
  untaggedOnly?: boolean;
  familyGapSignals?: boolean;
  includeReviewed?: boolean;
  reviewReason?: ReviewedDiscoveryReason;
  limit?: number;
  exampleLimit?: number;
  minGramLength?: number;
  maxGramLength?: number;
};

type FeatureAccumulator = {
  support: number;
  examples: Set<string>;
};

function createFeatureAccumulator(): FeatureAccumulator {
  return {
    support: 0,
    examples: new Set<string>(),
  };
}

function incrementFeatureSupport(
  bucket: Map<string, FeatureAccumulator>,
  values: Iterable<string>,
  examplesByValue: Map<string, string[]>,
  exampleLimit: number,
): void {
  for (const value of new Set(values)) {
    const accumulator = bucket.get(value) ?? createFeatureAccumulator();
    accumulator.support += 1;
    for (const example of examplesByValue.get(value) ?? []) {
      if (accumulator.examples.size >= exampleLimit) {
        break;
      }
      accumulator.examples.add(example);
    }
    bucket.set(value, accumulator);
  }
}

function collectRecordFeatureSet(
  record: DiscoveryAnalysisRecord,
  featureType: "nameTokens" | "namePhrases" | "descriptionTokens" | "descriptionPhrases" | "traits" | "references",
  options: Pick<DiscoveryEvidenceOptions, "minGramLength" | "maxGramLength"> = {},
): { values: string[]; examplesByValue: Map<string, string[]> } {
  const examplesByValue = new Map<string, string[]>();
  const appendExample = (value: string, example: string): void => {
    const bucket = examplesByValue.get(value) ?? [];
    if (!bucket.includes(example)) {
      bucket.push(example);
      examplesByValue.set(value, bucket);
    }
  };

  if (featureType === "nameTokens") {
    const values = tokenizeDiscoveryText(record.name, { filterStopwords: true })
      .filter((value) => !isDiscoveryNoiseToken(value));
    for (const value of values) {
      appendExample(value, record.name);
    }
    return { values, examplesByValue };
  }

  if (featureType === "namePhrases") {
    const phrases = extractDiscoveryGramRange(record.name, options, { filterStopwords: true })
      .filter((phrase) => !isDiscoveryNoisePhrase(phrase.normalized));
    for (const phrase of phrases) {
      appendExample(phrase.normalized, record.name);
    }
    return { values: phrases.map((phrase) => phrase.normalized), examplesByValue };
  }

  if (featureType === "descriptionTokens") {
    const values = tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true })
      .filter((value) => !isDiscoveryNoiseToken(value));
    for (const value of values) {
      appendExample(value, record.descriptionText ?? record.name);
    }
    return { values, examplesByValue };
  }

  if (featureType === "descriptionPhrases") {
    const phrases = extractDiscoveryGramRange(record.descriptionText ?? "", options, { filterStopwords: true })
      .filter((phrase) => !isDiscoveryNoisePhrase(phrase.normalized));
    for (const phrase of phrases) {
      appendExample(phrase.normalized, record.descriptionText ?? record.name);
    }
    return { values: phrases.map((phrase) => phrase.normalized), examplesByValue };
  }

  if (featureType === "traits") {
    const values = record.traits.map((trait) => normalizeDiscoveryFeature(trait)).filter(Boolean);
    for (const value of values) {
      appendExample(value, value);
    }
    return { values, examplesByValue };
  }

  const values = [
    ...record.references.map((reference) => `target:${normalizeDiscoveryFeature(reference.targetName)}`),
    ...record.references.map((reference) => `scope:${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`),
  ];
  for (const reference of record.references) {
    appendExample(`target:${normalizeDiscoveryFeature(reference.targetName)}`, reference.targetName);
    appendExample(
      `scope:${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`,
      `${reference.targetCategory}${reference.targetSubcategory ? `/${reference.targetSubcategory}` : ""}`,
    );
  }
  return { values, examplesByValue };
}

function collectFeatureSupport(
  records: DiscoveryAnalysisRecord[],
  featureType: "nameTokens" | "namePhrases" | "descriptionTokens" | "descriptionPhrases" | "traits" | "references",
  exampleLimit: number,
  options: Pick<DiscoveryEvidenceOptions, "minGramLength" | "maxGramLength"> = {},
): Map<string, FeatureAccumulator> {
  const support = new Map<string, FeatureAccumulator>();
  for (const record of records) {
    const featureSet = collectRecordFeatureSet(record, featureType, options);
    incrementFeatureSupport(support, featureSet.values, featureSet.examplesByValue, exampleLimit);
  }

  return support;
}

function evidenceKindForFeatureType(
  featureType: "nameTokens" | "namePhrases" | "descriptionTokens" | "descriptionPhrases" | "traits" | "references",
): DiscoveryEvidenceKind {
  switch (featureType) {
    case "nameTokens":
      return "nameToken";
    case "namePhrases":
      return "namePhrase";
    case "descriptionTokens":
      return "descriptionToken";
    case "descriptionPhrases":
      return "descriptionPhrase";
    case "traits":
      return "trait";
    case "references":
      return "reference";
  }
}

function evidenceKindWeight(kind: DiscoveryEvidenceKind): number {
  switch (kind) {
    case "reference":
      return 1.2;
    case "trait":
      return 1.1;
    case "namePhrase":
      return 0.9;
    case "nameToken":
      return 0.75;
    case "descriptionPhrase":
      return 0.7;
    case "descriptionToken":
      return 0.55;
  }
}

function evidenceSupportMultiplier(
  kind: DiscoveryEvidenceKind,
  support: number,
  cohortSize: number,
): number {
  const requiresRepeatedSupport = kind === "descriptionToken" || kind === "descriptionPhrase";
  if (!requiresRepeatedSupport || cohortSize < 3 || support >= 2) {
    return 1;
  }

  return kind === "descriptionPhrase" ? 0.3 : 0.18;
}

function evidenceConcentrationMultiplier(
  kind: DiscoveryEvidenceKind,
  support: number,
  cohortSize: number,
): number {
  if (cohortSize <= 1) {
    return 1;
  }

  const supportRatio = support / cohortSize;
  if (kind === "reference" || kind === "trait") {
    return 0.9 + (supportRatio * 0.45);
  }
  if (kind === "descriptionToken" || kind === "descriptionPhrase") {
    return 0.7 + (supportRatio * 0.45);
  }

  return 0.78 + (supportRatio * 0.35);
}

function rankEvidenceTerms(
  cohortSupport: Map<string, FeatureAccumulator>,
  baselineSupport: Map<string, FeatureAccumulator>,
  cohortSize: number,
  baselineSize: number,
  limit: number,
  exampleLimit: number,
  featureType: "nameTokens" | "namePhrases" | "descriptionTokens" | "descriptionPhrases" | "traits" | "references",
): DiscoveryEvidenceTerm[] {
  const kind = evidenceKindForFeatureType(featureType);
  return [...cohortSupport.entries()]
    .map(([value, cohort]) => {
      const baseline = baselineSupport.get(value) ?? createFeatureAccumulator();
      const cohortRatio = cohortSize > 0 ? cohort.support / cohortSize : 0;
      const baselineRatio = baselineSize > 0 ? baseline.support / baselineSize : 0;
      const lift = baselineRatio > 0 ? cohortRatio / baselineRatio : cohortRatio > 0 ? cohort.support : 0;
      const score =
        cohort.support *
        Math.max(1, lift) *
        evidenceKindWeight(kind) *
        evidenceSupportMultiplier(kind, cohort.support, cohortSize) *
        evidenceConcentrationMultiplier(kind, cohort.support, cohortSize);
      return {
        kind,
        value,
        support: cohort.support,
        cohortSupport: cohort.support,
        baselineSupport: baseline.support,
        lift,
        score,
        examples: [...cohort.examples].slice(0, exampleLimit),
      };
    })
    .sort((left, right) =>
      right.score - left.score ||
      right.cohortSupport - left.cohortSupport ||
      right.lift - left.lift ||
      left.value.localeCompare(right.value))
    .slice(0, limit);
}

function mergeFamilyGapTerms(terms: FamilyGapEvidenceTerm[]): FamilyGapEvidenceTerm[] {
  const byValue = new Map<string, FamilyGapEvidenceTerm>();
  for (const term of terms) {
    const existing = byValue.get(term.value);
    if (
      !existing
      || term.score > existing.score
      || (term.score === existing.score && term.cohortSupport > existing.cohortSupport)
    ) {
      byValue.set(term.value, term);
    }
  }

  return [...byValue.values()]
    .sort((left, right) =>
      right.score - left.score ||
      right.gapLift - left.gapLift ||
      right.cohortSupport - left.cohortSupport ||
      left.value.localeCompare(right.value));
}

function rankFamilyGapTerms(
  uncoveredSupport: Map<string, FeatureAccumulator>,
  coveredSupport: Map<string, FeatureAccumulator>,
  baselineSupport: Map<string, FeatureAccumulator>,
  uncoveredSize: number,
  coveredSize: number,
  baselineSize: number,
  limit: number,
  exampleLimit: number,
  featureType: "nameTokens" | "namePhrases" | "descriptionTokens" | "descriptionPhrases" | "traits" | "references",
  family: string,
): FamilyGapEvidenceTerm[] {
  const kind = evidenceKindForFeatureType(featureType);
  return [...uncoveredSupport.entries()]
    .map(([value, uncovered]) => {
      const covered = coveredSupport.get(value) ?? createFeatureAccumulator();
      const baseline = baselineSupport.get(value) ?? createFeatureAccumulator();
      const uncoveredRate = (uncovered.support + 1) / Math.max(1, uncoveredSize + 2);
      const coveredRate = (covered.support + 1) / Math.max(1, coveredSize + 2);
      const baselineRate = (baseline.support + 1) / Math.max(1, baselineSize + 2);
      const gapLift = coveredRate > 0 ? uncoveredRate / coveredRate : uncoveredRate;
      const baselineLift = baselineRate > 0 ? uncoveredRate / baselineRate : uncoveredRate;
      const classification = classifyFamilyGapFeature(family, kind, value);
      const qualityMultiplier = Math.max(
        0.3,
        (classification.cueLocality - (classification.cueAmbiguityPenalty * 0.55)) *
        (1 - (classification.boilerplateRisk * 0.4)),
      );
      const score =
        uncovered.support *
        Math.max(1, gapLift) *
        Math.max(1, baselineLift) *
        evidenceKindWeight(kind) *
        evidenceSupportMultiplier(kind, uncovered.support, uncoveredSize) *
        evidenceConcentrationMultiplier(kind, uncovered.support, uncoveredSize) *
        classification.familyConceptWeight *
        qualityMultiplier;

      return {
        kind,
        value: classification.primaryCue ?? value,
        support: uncovered.support,
        cohortSupport: uncovered.support,
        baselineSupport: baseline.support,
        coveredSupport: covered.support,
        lift: baselineLift,
        gapLift,
        score,
        examples: [...uncovered.examples].slice(0, exampleLimit),
        bucket: classification.bucket,
        existingTagOverlaps: classification.existingTagOverlaps,
        suppressionReason: classification.suppressionReason,
      } satisfies FamilyGapEvidenceTerm;
    })
    .sort((left, right) =>
      right.score - left.score ||
      right.gapLift - left.gapLift ||
      right.cohortSupport - left.cohortSupport ||
      left.value.localeCompare(right.value))
    .slice(0, Math.max(limit * 3, limit));
}

export function analyzeFamilyGapEvidenceFromRecords(
  uncovered: DiscoveryAnalysisRecord[],
  covered: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
  family: string,
  liveTags: string[],
  options: Pick<DiscoveryEvidenceOptions, "limit" | "exampleLimit" | "minGramLength" | "maxGramLength"> = {},
): FamilyGapReport {
  const limit = Math.max(1, Math.min(options.limit ?? DEFAULT_EVIDENCE_LIMIT, 50));
  const exampleLimit = Math.max(1, Math.min(options.exampleLimit ?? DEFAULT_EXAMPLE_LIMIT, 5));
  const gramRange = resolveDiscoveryGramRange(options);
  const featureTypes = [
    "nameTokens",
    "namePhrases",
    "descriptionTokens",
    "descriptionPhrases",
    "traits",
    "references",
  ] as const;

  const allTerms = featureTypes.flatMap((featureType) => {
    const uncoveredSupport = collectFeatureSupport(uncovered, featureType, exampleLimit, gramRange);
    const coveredSupport = collectFeatureSupport(covered, featureType, exampleLimit, gramRange);
    const baselineSupport = collectFeatureSupport(baseline, featureType, exampleLimit, gramRange);
    return rankFamilyGapTerms(
      uncoveredSupport,
      coveredSupport,
      baselineSupport,
      uncovered.length,
      covered.length,
      baseline.length,
      limit,
      exampleLimit,
      featureType,
      family,
    );
  });

  const mergedTerms = mergeFamilyGapTerms(allTerms);
  const surfacedTerms = mergedTerms.filter((term) => !isNameDerivedEvidenceKind(term.kind));
  const likelyNewConcepts = mergedTerms
    .filter((term) => !isNameDerivedEvidenceKind(term.kind))
    .filter((term) => term.bucket === "possible_place_anchor" && term.existingTagOverlaps.length === 0)
    .slice(0, limit);
  const existingTagCoverageGaps = surfacedTerms
    .filter((term) => term.bucket === "possible_place_anchor" && term.existingTagOverlaps.length > 0)
    .slice(0, limit);
  const suppressedTerms = mergedTerms
    .filter((term) => term.suppressionReason !== null)
    .slice(0, limit);

  return {
    coveredCount: covered.length,
    uncoveredCount: uncovered.length,
    baselineCount: baseline.length,
    liveTags,
    likelyNewConcepts,
    existingTagCoverageGaps,
    suppressedTerms,
  };
}

export function analyzeDiscoveryEvidenceFromRecords(
  cohort: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
  options: Pick<DiscoveryEvidenceOptions, "limit" | "exampleLimit" | "minGramLength" | "maxGramLength"> = {},
): Omit<DiscoveryEvidenceReport, "category" | "subcategory" | "family" | "representativeRecords"> {
  const limit = Math.max(1, Math.min(options.limit ?? DEFAULT_EVIDENCE_LIMIT, 50));
  const exampleLimit = Math.max(1, Math.min(options.exampleLimit ?? DEFAULT_EXAMPLE_LIMIT, 5));
  const gramRange = resolveDiscoveryGramRange(options);

  const featureTypes = [
    "nameTokens",
    "namePhrases",
    "descriptionTokens",
    "descriptionPhrases",
    "traits",
    "references",
  ] as const;

  const ranked = Object.fromEntries(
    featureTypes.map((featureType) => {
      const cohortSupport = collectFeatureSupport(cohort, featureType, exampleLimit, gramRange);
      const baselineSupport = collectFeatureSupport(baseline, featureType, exampleLimit, gramRange);
      return [featureType, rankEvidenceTerms(cohortSupport, baselineSupport, cohort.length, baseline.length, limit, exampleLimit, featureType)];
    }),
  ) as Record<(typeof featureTypes)[number], DiscoveryEvidenceTerm[]>;

  return {
    cohortSize: cohort.length,
    baselineSize: baseline.length,
    nameTokens: ranked.nameTokens,
    namePhrases: ranked.namePhrases,
    descriptionTokens: ranked.descriptionTokens,
    descriptionPhrases: ranked.descriptionPhrases,
    traits: ranked.traits,
    references: ranked.references,
  };
}

function mergeUniqueRecords(records: DiscoveryAnalysisRecord[]): DiscoveryAnalysisRecord[] {
  const uniqueRecords = new Map<string, DiscoveryAnalysisRecord>();
  for (const record of records) {
    if (!uniqueRecords.has(record.recordKey)) {
      uniqueRecords.set(record.recordKey, record);
    }
  }

  return [...uniqueRecords.values()];
}

export function analyzeDiscoveryEvidence(
  db: DatabaseSync,
  options: DiscoveryEvidenceOptions,
): DiscoveryEvidenceReport {
  const explicitRecordKeys = options.recordKeys;
  const normalizedTag = options.tag ? normalizeDerivedTag(options.tag) : undefined;
  const normalizedFamily = options.family ? normalizeDerivedTag(options.family) : undefined;
  const familyTags = normalizedFamily
    ? getDerivedTagFamilyTags(normalizedFamily, {
      category: options.category,
      subcategory: options.subcategory,
    })
    : [];
  if (options.familyGapSignals && !normalizedFamily) {
    throw new Error("Pass --family <derived-tag-family> when using --family-gap-signals.");
  }
  const reviewedSelection = normalizedFamily && (options.familyGapSignals || options.untaggedOnly)
    ? getReviewedDiscoverySelection({
      category: options.category,
      subcategory: options.subcategory,
      family: normalizedFamily,
      includeReviewed: options.includeReviewed,
      reviewReason: options.reviewReason,
    })
    : undefined;
  const applyReviewedSelection = !explicitRecordKeys || explicitRecordKeys.length === 0;
  const reviewedRecordKeys = applyReviewedSelection && reviewedSelection?.mode === "filtered"
    ? reviewedSelection.recordKeys
    : explicitRecordKeys;
  const reviewedExcludedRecordKeys = uniqueSorted([
    ...(options.excludeRecordKeys ?? []),
    ...(applyReviewedSelection && reviewedSelection?.mode === "excluded" ? reviewedSelection.recordKeys : []),
  ]);
  const reviewedSummary = reviewedSelection
    ? summarizeReviewedDiscoverySelection(
      reviewedSelection,
      applyReviewedSelection ? reviewedSelection.recordKeys.length : 0,
    )
    : undefined;
  const seedRecordKeys = normalizedTag && !explicitRecordKeys
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
  const taggedRecords = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: reviewedRecordKeys,
    excludeRecordKeys: reviewedExcludedRecordKeys,
    requireTag: normalizedTag,
    requireAnyDerivedTags: normalizedTag || options.untaggedOnly ? undefined : familyTags,
    excludeDerivedTag: options.excludeDerivedTag,
    excludeAnyDerivedTags: options.untaggedOnly ? familyTags : undefined,
    untaggedOnly: options.untaggedOnly && familyTags.length === 0,
    includeVectors: false,
  });
  const seededRecords = seedRecordKeys.length > 0
    ? loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      recordKeys: seedRecordKeys,
      excludeRecordKeys: reviewedExcludedRecordKeys,
      includeVectors: false,
    })
    : [];
  const cohort = mergeUniqueRecords([...taggedRecords, ...seededRecords]);
  const baseline = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    includeVectors: false,
  });
  if (options.familyGapSignals && normalizedFamily) {
    const uncovered = loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      recordKeys: reviewedRecordKeys,
      excludeRecordKeys: reviewedExcludedRecordKeys,
      excludeAnyDerivedTags: familyTags,
      includeVectors: false,
    });
    const covered = loadDiscoveryRecords(db, {
      category: options.category,
      subcategory: options.subcategory,
      requireAnyDerivedTags: familyTags,
      includeVectors: false,
    });
    const liveTags = uniqueSorted([...new Set(covered.flatMap((record) =>
      record.derivedTags.filter((tag) => familyTags.includes(normalizeDerivedTag(tag))),
    ))]);
    const report = analyzeDiscoveryEvidenceFromRecords(uncovered, baseline, options);
    return {
      category: options.category ?? null,
      subcategory: options.subcategory ?? null,
      family: normalizedFamily,
      reviewedRecords: reviewedSummary,
      ...report,
      familyGap: analyzeFamilyGapEvidenceFromRecords(uncovered, covered, baseline, normalizedFamily, liveTags, options),
      representativeRecords: uncovered
        .slice(0, Math.min(5, uncovered.length))
        .map((record) => ({
          recordKey: record.recordKey,
          name: record.name,
          traits: uniqueSorted([...record.traits]),
        })),
    };
  }

  const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, options);
  return {
    category: options.category ?? null,
    subcategory: options.subcategory ?? null,
    family: normalizedFamily ?? null,
    reviewedRecords: reviewedSummary,
    ...report,
    representativeRecords: cohort
      .slice(0, Math.min(5, cohort.length))
      .map((record) => ({
        recordKey: record.recordKey,
        name: record.name,
        traits: uniqueSorted([...record.traits]),
      })),
  };
}
