import type { DiscoveryAnalysisRecord } from "./discovery-records.js";

export type DiscoverySourceScope = "source-slice" | "publication" | "pack";

export type DiscoverySourceSummary = {
  sourceCount: number;
  topSources: string[];
  publicationCount: number;
  topPublications: string[];
  sourceSliceCount: number;
  topSourceSlices: string[];
  dominantSourceShare: number;
  sourceScope: DiscoverySourceScope | null;
  sourceLabel: string | null;
  hasUsableSourceSignals: boolean;
};

type DimensionSummary = {
  count: number;
  topValues: string[];
  dominantShare: number;
};

function summarizeDimension(values: string[]): DimensionSummary {
  const counts = new Map<string, number>();
  for (const value of values) {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }

  const sortedValues = [...counts.entries()]
    .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]));
  const total = sortedValues.reduce((sum, [, count]) => sum + count, 0);
  return {
    count: sortedValues.length,
    topValues: sortedValues.slice(0, 3).map(([value]) => value),
    dominantShare: total > 0 ? (sortedValues[0]?.[1] ?? 0) / total : 0,
  };
}

function collectPackValues(records: DiscoveryAnalysisRecord[]): string[] {
  return records
    .filter((record) => Boolean(record.packName) && record.packName !== record.category)
    .map((record) => record.packName);
}

function collectPublicationValues(records: DiscoveryAnalysisRecord[]): string[] {
  return records
    .map((record) => record.publicationTitle?.trim() ?? "")
    .filter(Boolean);
}

function collectSourceSliceValues(records: DiscoveryAnalysisRecord[]): string[] {
  return records
    .map((record) => record.sourcePathSlice?.trim() ?? "")
    .filter(Boolean);
}

function choosePrimaryScope(
  sourceSlices: DimensionSummary,
  publications: DimensionSummary,
  packs: DimensionSummary,
): { scope: DiscoverySourceScope | null; summary: DimensionSummary | null } {
  if (sourceSlices.count === 1 || sourceSlices.dominantShare >= 0.75) {
    return { scope: "source-slice", summary: sourceSlices };
  }
  if (publications.count === 1 || publications.dominantShare >= 0.75) {
    return { scope: "publication", summary: publications };
  }
  if (packs.count === 1 || packs.dominantShare >= 0.75) {
    return { scope: "pack", summary: packs };
  }
  if (sourceSlices.count > 0) {
    return { scope: "source-slice", summary: sourceSlices };
  }
  if (publications.count > 0) {
    return { scope: "publication", summary: publications };
  }
  if (packs.count > 0) {
    return { scope: "pack", summary: packs };
  }
  return { scope: null, summary: null };
}

export function summarizeDiscoverySources(records: DiscoveryAnalysisRecord[]): DiscoverySourceSummary {
  const packs = summarizeDimension(collectPackValues(records));
  const publications = summarizeDimension(collectPublicationValues(records));
  const sourceSlices = summarizeDimension(collectSourceSliceValues(records));
  const primary = choosePrimaryScope(sourceSlices, publications, packs);

  return {
    sourceCount: packs.count,
    topSources: packs.topValues,
    publicationCount: publications.count,
    topPublications: publications.topValues,
    sourceSliceCount: sourceSlices.count,
    topSourceSlices: sourceSlices.topValues,
    dominantSourceShare: primary.summary?.dominantShare ?? 0,
    sourceScope: primary.scope,
    sourceLabel: primary.summary?.topValues[0] ?? null,
    hasUsableSourceSignals: Boolean(primary.scope),
  };
}
