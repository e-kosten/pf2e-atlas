import type { buildLiteralQueryWeights } from "./query-analysis.js";
import type { NormalizedRecord } from "../domain/record-types.js";
import type {
  SearchExplainResult,
  SearchFilters,
  SearchMode,
  SearchProfile,
  SearchRecordExplanation,
} from "../domain/search-types.js";
import type { RankingConfig } from "./ranking-config.js";
import { normalizeText } from "../shared/utils.js";
import { bigramDice } from "./primitives.js";

type RerankAdjustments = SearchRecordExplanation["rerankAdjustments"];
type HybridFusionProfileName = NonNullable<SearchExplainResult["fusionProfile"]>;
type HybridFusionProfile = RankingConfig["hybridFusion"]["balanced"];
type FusionConfigSummary = NonNullable<SearchExplainResult["fusionConfig"]>;

export type LexicalSignal = {
  lexicalScore: number;
  matchedTraits: string[];
  matchedNameTokens: string[];
};

export type LexicalRetrievalRow = {
  recordKey: string;
  rank: number;
};

export type SemanticRetrievalRow = {
  recordKey: string;
  distance: number;
};

function isSocietyPublication(publicationTitle: string | null): boolean {
  const normalized = normalizeText(publicationTitle ?? "");
  return (
    normalized.startsWith("pathfinder society scenario") ||
    normalized.startsWith("pathfinder society special") ||
    normalized.startsWith("pathfinder society intro")
  );
}

function isSocietyPack(packName: string): boolean {
  return normalizeText(packName).startsWith("pfs ");
}

function hasScenarioScaleSuffix(name: string): boolean {
  return /\(\d+\s*-\s*\d+\)\s*$/.test(name.trim());
}

export function packQualityScore(record: NormalizedRecord, rankingConfig: RankingConfig): number {
  const normalizedPack = normalizeText(record.packName);
  let score = 0;

  if (normalizedPack.includes("macro")) {
    score += rankingConfig.packQuality.macroPenalty;
  }

  if (normalizedPack.includes("glossary")) {
    score += rankingConfig.packQuality.glossaryPenalty;
  }

  if (normalizedPack.includes("effect")) {
    score += rankingConfig.packQuality.effectPenalty;
  }

  if (
    normalizedPack === "actions" ||
    normalizedPack === "spells" ||
    normalizedPack === "equipment" ||
    normalizedPack === "feats"
  ) {
    score += rankingConfig.packQuality.utilityPackBoost;
  }

  return score;
}

export function sourceQualityScore(record: NormalizedRecord, rankingConfig: RankingConfig): number {
  if (record.sourceCategory === "core") {
    return rankingConfig.sourceQuality.core;
  }
  if (record.sourceCategory === "rules") {
    return rankingConfig.sourceQuality.rules;
  }
  if (record.sourceCategory === "adventure") {
    return rankingConfig.sourceQuality.adventure;
  }

  return rankingConfig.sourceQuality.unknown;
}

export function rarityPreferenceScore(
  record: NormalizedRecord,
  filters: SearchFilters,
  rankingConfig: RankingConfig,
): number {
  const normalizedRarity = normalizeText(record.rarity ?? "");
  let score = 0;

  if (normalizedRarity === "common" || normalizedRarity === "uncommon") {
    score +=
      normalizedRarity === "common" ? rankingConfig.rarityPreference.common : rankingConfig.rarityPreference.uncommon;
  } else if (normalizedRarity === "rare") {
    score += rankingConfig.rarityPreference.rare;
  } else if (normalizedRarity === "unique") {
    score += rankingConfig.rarityPreference.unique;
  }

  if (record.isUnique && filters.query?.trim()) {
    score += rankingConfig.rarityPreference.themeQueryUniquePenalty;
  }

  return score;
}

export function sourcePenaltyScore(
  record: NormalizedRecord,
  filters: SearchFilters,
  rankingConfig: RankingConfig,
): number {
  if (record.hasDescription || !filters.query?.trim()) {
    return 0;
  }

  let penalty = 0;
  if (isSocietyPublication(record.publicationTitle) || isSocietyPack(record.packName)) {
    penalty += rankingConfig.sourcePenalty.societyMetadataOnlyPenalty;
  }
  if (hasScenarioScaleSuffix(record.name)) {
    penalty += rankingConfig.sourcePenalty.scenarioScaleSuffixPenalty;
  }

  return penalty;
}

export function queryTextScore(query: string, haystack: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const targetTokens = new Set(normalizeText(haystack).split(" ").filter(Boolean));
  if (queryTokens.length === 0 || targetTokens.size === 0) {
    return 0;
  }

  const overlap = queryTokens.filter((token) => targetTokens.has(token)).length;
  return overlap / queryTokens.length;
}

export function tokenize(value: string): string[] {
  const normalized = normalizeText(value);
  return normalized ? normalized.split(" ").filter(Boolean) : [];
}

export function scoreWeightedOverlap(
  weights: Map<string, number>,
  targetTokens: Iterable<string>,
  saturationWeight: number,
): { score: number; matchedTokens: string[] } {
  if (weights.size === 0) {
    return { score: 0, matchedTokens: [] };
  }

  let matchedWeight = 0;
  const matchedTokens: string[] = [];
  for (const token of new Set([...targetTokens].map((value) => normalizeText(value)).filter(Boolean))) {
    const weight = weights.get(token);
    if (!weight) {
      continue;
    }

    matchedWeight += weight;
    matchedTokens.push(token);
  }

  return {
    score: Math.min(1, matchedWeight / Math.max(1, saturationWeight)),
    matchedTokens: matchedTokens.sort((left, right) => left.localeCompare(right)),
  };
}

function buildTraitText(record: NormalizedRecord): string {
  return record.traits.join(" ");
}

export function scoreNameCandidate(query: string, normalizedName: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0.5;
  }

  if (normalizedName === normalizedQuery) {
    return 1;
  }

  if (normalizedName.startsWith(normalizedQuery)) {
    return 0.95;
  }

  if (normalizedName.includes(normalizedQuery)) {
    return 0.9;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const nameTokens = new Set(normalizedName.split(" ").filter(Boolean));
  const overlap = queryTokens.filter((token) => nameTokens.has(token)).length;
  const tokenScore = queryTokens.length > 0 ? overlap / queryTokens.length : 0;
  const dice = bigramDice(normalizedQuery, normalizedName);
  return Math.max(tokenScore * 0.8, dice * 0.75);
}

export function resolveSearchMode(filters: SearchFilters, context: "list" | "search"): SearchMode {
  if (context === "search") {
    if (filters.searchProfile === "lexical") {
      return filters.query?.trim() ? "lexical" : "structured";
    }

    if (filters.searchProfile === "balanced" || filters.searchProfile === "concept") {
      return filters.query?.trim() ? "hybrid" : "structured";
    }
  }

  if (context === "search" && filters.query?.trim()) {
    return "hybrid";
  }

  return "structured";
}

export function hasStructuredFilterSignal(filters: SearchFilters): boolean {
  return Boolean(
    filters.pack ||
    (filters.linksTo && filters.linksTo.length > 0) ||
    (filters.excludeLinksTo && filters.excludeLinksTo.length > 0) ||
    filters.category ||
    filters.subcategory ||
    (filters.scopes && filters.scopes.length > 0) ||
    filters.levelMin !== undefined ||
    filters.levelMax !== undefined ||
    filters.rarity ||
    filters.metadata ||
    filters.priceMin !== undefined ||
    filters.priceMax !== undefined ||
    filters.actionCost !== undefined,
  );
}

export function resolveSearchProfile(
  filters: SearchFilters,
  context: "list" | "search",
  mode: SearchMode,
): SearchProfile | null {
  if (context === "list") {
    return null;
  }

  if (filters.query?.trim()) {
    if (filters.searchProfile) {
      return filters.searchProfile;
    }

    if (mode === "hybrid") {
      return "balanced";
    }

    return "lexical";
  }

  if (filters.nameQuery?.trim()) {
    return "lexical";
  }

  return null;
}

export function resolveHybridFusionProfile(
  searchProfile: SearchProfile | null,
  mode: SearchMode,
  rankingConfig: RankingConfig,
): { profile: HybridFusionProfileName; config: HybridFusionProfile } | null {
  if (mode !== "hybrid") {
    return null;
  }

  if (searchProfile === "concept") {
    return {
      profile: "concept",
      config: rankingConfig.hybridFusion.concept,
    };
  }

  return {
    profile: "balanced",
    config: rankingConfig.hybridFusion.balanced,
  };
}

export function buildRerankAdjustments(
  record: NormalizedRecord,
  filters: SearchFilters,
  rankingConfig: RankingConfig,
): RerankAdjustments {
  return {
    packQuality: packQualityScore(record, rankingConfig),
    sourceQuality: sourceQualityScore(record, rankingConfig),
    rarityPreference: rarityPreferenceScore(record, filters, rankingConfig),
    sourcePenalty: sourcePenaltyScore(record, filters, rankingConfig),
  };
}

export function sumRerankAdjustments(adjustments: RerankAdjustments): number {
  return adjustments.packQuality + adjustments.sourceQuality + adjustments.rarityPreference + adjustments.sourcePenalty;
}

export function buildLexicalSignal(
  record: NormalizedRecord,
  lexicalQuery: string,
  literalQueryWeights: ReturnType<typeof buildLiteralQueryWeights> | null,
  lexicalMatches: Map<string, number>,
  rankingConfig: RankingConfig,
): LexicalSignal {
  const ftsScore = lexicalQuery.length > 0 ? (lexicalMatches.get(record.recordKey) ?? 0) : 0;
  const descriptionTextScore = lexicalQuery.length > 0 ? queryTextScore(lexicalQuery, record.descriptionText ?? "") : 0;
  const traitTextScore = lexicalQuery.length > 0 ? queryTextScore(lexicalQuery, buildTraitText(record)) : 0;
  const themeName = literalQueryWeights
    ? scoreWeightedOverlap(literalQueryWeights.nameWeights, tokenize(record.name), 1.5)
    : { score: 0, matchedTokens: [] };
  const themeTraits = literalQueryWeights
    ? scoreWeightedOverlap(literalQueryWeights.traitWeights, record.traits, 2)
    : { score: 0, matchedTokens: [] };
  const lexicalWeights = rankingConfig.lexicalChannels;
  const lexicalScoreBeforeNormalization =
    ftsScore * lexicalWeights.fullTextSearch +
    descriptionTextScore * lexicalWeights.descriptionText +
    themeName.score * lexicalWeights.themeName +
    Math.max(traitTextScore, themeTraits.score) * lexicalWeights.themeTraits;
  const normalizationMultiplier = !record.hasDescription ? 1 / (1 - lexicalWeights.descriptionText) : 1;
  const lexicalScore = lexicalScoreBeforeNormalization * normalizationMultiplier;

  return {
    lexicalScore,
    matchedTraits: themeTraits.matchedTokens,
    matchedNameTokens: themeName.matchedTokens,
  };
}

export function buildFusionConfigSummary(
  fusionProfile: HybridFusionProfileName | null,
  fusionConfig: HybridFusionProfile | null,
  rankingConfig: RankingConfig,
): FusionConfigSummary | null {
  if (!fusionProfile || !fusionConfig) {
    return null;
  }

  return {
    rrfK: rankingConfig.hybridFusion.rrfK,
    lexicalWeight: fusionConfig.lexicalWeight,
    semanticWeight: fusionConfig.semanticWeight,
    lexicalTopK: fusionConfig.lexicalTopK,
    semanticTopK: fusionConfig.semanticTopK,
  };
}

export function compareOptionalRanks(left: number | null, right: number | null): number {
  if (left === right) {
    return 0;
  }
  if (left === null) {
    return 1;
  }
  if (right === null) {
    return -1;
  }

  return left - right;
}

export function computeWeightedRrfScore(
  lexicalRank: number | null,
  semanticRank: number | null,
  fusionConfig: HybridFusionProfile,
  rrfK: number,
): number {
  const lexicalContribution = lexicalRank === null ? 0 : fusionConfig.lexicalWeight / (rrfK + lexicalRank);
  const semanticContribution = semanticRank === null ? 0 : fusionConfig.semanticWeight / (rrfK + semanticRank);

  return (lexicalContribution + semanticContribution) * (rrfK + 1);
}

export function buildNormalizedRankScoreMap(recordKeysInRankOrder: string[]): Map<string, number> {
  const scores = new Map<string, number>();
  const total = recordKeysInRankOrder.length;
  recordKeysInRankOrder.forEach((recordKey, index) => {
    scores.set(recordKey, total <= 1 ? 1 : 1 - index / (total - 1));
  });
  return scores;
}

export function buildRankMap(recordKeysInRankOrder: string[]): Map<string, number> {
  const ranks = new Map<string, number>();
  recordKeysInRankOrder.forEach((recordKey, index) => {
    ranks.set(recordKey, index + 1);
  });
  return ranks;
}
