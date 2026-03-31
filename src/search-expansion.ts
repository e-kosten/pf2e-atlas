import { NormalizedRecord, SearchQueryAnalysis } from "./types.js";
import { normalizeText } from "./utils.js";

export type SearchQueryAnalysisState = SearchQueryAnalysis & {
  baseTraitWeights: Map<string, number>;
  baseNameWeights: Map<string, number>;
  baseMetadataWeights: Map<string, number>;
};

export type CandidateQueryWeights = {
  traitWeights: Map<string, number>;
  nameWeights: Map<string, number>;
  metadataWeights: Map<string, number>;
};

function addWeight(weights: Map<string, number>, token: string, weight: number): void {
  const normalized = normalizeText(token);
  if (!normalized) {
    return;
  }

  weights.set(normalized, Math.max(weight, weights.get(normalized) ?? 0));
}

export function buildSearchQueryAnalysis(query: string): SearchQueryAnalysisState | null {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return null;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const baseTraitWeights = new Map<string, number>();
  const baseNameWeights = new Map<string, number>();
  const baseMetadataWeights = new Map<string, number>();

  for (const token of queryTokens) {
    addWeight(baseTraitWeights, token, 1);
    addWeight(baseNameWeights, token, 0.65);
    addWeight(baseMetadataWeights, token, 0.75);
  }

  return {
    rawQuery: query,
    normalizedQuery,
    queryTokens,
    baseTraitWeights,
    baseNameWeights,
    baseMetadataWeights,
  };
}

export function buildCandidateQueryWeights(_record: NormalizedRecord, analysis: SearchQueryAnalysisState): CandidateQueryWeights {
  return {
    traitWeights: new Map(analysis.baseTraitWeights),
    nameWeights: new Map(analysis.baseNameWeights),
    metadataWeights: new Map(analysis.baseMetadataWeights),
  };
}
