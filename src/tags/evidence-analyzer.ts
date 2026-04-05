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

const DEFAULT_EVIDENCE_LIMIT = 12;
const DEFAULT_EXAMPLE_LIMIT = 3;

export type DiscoveryEvidenceTerm = {
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
  cohortSize: number;
  baselineSize: number;
  nameTokens: DiscoveryEvidenceTerm[];
  namePhrases: DiscoveryEvidenceTerm[];
  descriptionTokens: DiscoveryEvidenceTerm[];
  descriptionPhrases: DiscoveryEvidenceTerm[];
  traits: DiscoveryEvidenceTerm[];
  references: DiscoveryEvidenceTerm[];
  representativeRecords: Array<{
    recordKey: string;
    name: string;
    traits: string[];
  }>;
};

export type DiscoveryEvidenceOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  recordKeys?: string[];
  excludeRecordKeys?: string[];
  tag?: string;
  excludeDerivedTag?: string;
  untaggedOnly?: boolean;
  limit?: number;
  exampleLimit?: number;
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
    const values = tokenizeDiscoveryText(record.name, { filterStopwords: true });
    for (const value of values) {
      appendExample(value, record.name);
    }
    return { values, examplesByValue };
  }

  if (featureType === "namePhrases") {
    const phrases = [
      ...extractDiscoveryNgrams(record.name, 2, { filterStopwords: true }),
      ...extractDiscoveryNgrams(record.name, 3, { filterStopwords: true }),
    ];
    for (const phrase of phrases) {
      appendExample(phrase.normalized, record.name);
    }
    return { values: phrases.map((phrase) => phrase.normalized), examplesByValue };
  }

  if (featureType === "descriptionTokens") {
    const values = tokenizeDiscoveryText(record.descriptionText ?? "", { filterStopwords: true });
    for (const value of values) {
      appendExample(value, record.descriptionText ?? record.name);
    }
    return { values, examplesByValue };
  }

  if (featureType === "descriptionPhrases") {
    const phrases = [
      ...extractDiscoveryNgrams(record.descriptionText ?? "", 2, { filterStopwords: true }),
      ...extractDiscoveryNgrams(record.descriptionText ?? "", 3, { filterStopwords: true }),
    ];
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
): Map<string, FeatureAccumulator> {
  const support = new Map<string, FeatureAccumulator>();
  for (const record of records) {
    const featureSet = collectRecordFeatureSet(record, featureType);
    incrementFeatureSupport(support, featureSet.values, featureSet.examplesByValue, exampleLimit);
  }

  return support;
}

function rankEvidenceTerms(
  cohortSupport: Map<string, FeatureAccumulator>,
  baselineSupport: Map<string, FeatureAccumulator>,
  cohortSize: number,
  baselineSize: number,
  limit: number,
  exampleLimit: number,
): DiscoveryEvidenceTerm[] {
  return [...cohortSupport.entries()]
    .map(([value, cohort]) => {
      const baseline = baselineSupport.get(value) ?? createFeatureAccumulator();
      const cohortRatio = cohortSize > 0 ? cohort.support / cohortSize : 0;
      const baselineRatio = baselineSize > 0 ? baseline.support / baselineSize : 0;
      const lift = baselineRatio > 0 ? cohortRatio / baselineRatio : cohortRatio > 0 ? cohort.support : 0;
      return {
        value,
        support: cohort.support,
        cohortSupport: cohort.support,
        baselineSupport: baseline.support,
        lift,
        score: cohort.support * Math.max(1, lift),
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

export function analyzeDiscoveryEvidenceFromRecords(
  cohort: DiscoveryAnalysisRecord[],
  baseline: DiscoveryAnalysisRecord[],
  options: Pick<DiscoveryEvidenceOptions, "limit" | "exampleLimit"> = {},
): Omit<DiscoveryEvidenceReport, "category" | "subcategory" | "representativeRecords"> {
  const limit = Math.max(1, Math.min(options.limit ?? DEFAULT_EVIDENCE_LIMIT, 50));
  const exampleLimit = Math.max(1, Math.min(options.exampleLimit ?? DEFAULT_EXAMPLE_LIMIT, 5));

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
      const cohortSupport = collectFeatureSupport(cohort, featureType, exampleLimit);
      const baselineSupport = collectFeatureSupport(baseline, featureType, exampleLimit);
      return [featureType, rankEvidenceTerms(cohortSupport, baselineSupport, cohort.length, baseline.length, limit, exampleLimit)];
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

export function analyzeDiscoveryEvidence(
  db: DatabaseSync,
  options: DiscoveryEvidenceOptions,
): DiscoveryEvidenceReport {
  const cohort = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    recordKeys: options.recordKeys,
    excludeRecordKeys: options.excludeRecordKeys,
    requireTag: options.tag,
    excludeDerivedTag: options.excludeDerivedTag,
    untaggedOnly: options.untaggedOnly,
    includeVectors: false,
  });
  const baseline = loadDiscoveryRecords(db, {
    category: options.category,
    subcategory: options.subcategory,
    includeVectors: false,
  });
  const report = analyzeDiscoveryEvidenceFromRecords(cohort, baseline, options);
  return {
    category: options.category ?? null,
    subcategory: options.subcategory ?? null,
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
