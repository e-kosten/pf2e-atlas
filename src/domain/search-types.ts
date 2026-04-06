import type { MetadataFilterNode } from "./metadata-types.js";
import type { NormalizedRecord } from "./record-types.js";
import { METADATA_FIELD_REGISTRY } from "./metadata-field-registry.js";

export type SearchProfile = "lexical" | "balanced" | "concept";

export type SearchCategory =
  | "equipment"
  | "feat"
  | "creature"
  | "hazard"
  | "affliction"
  | "rule"
  | "spell"
  | "characterCreation"
  | "lore";

export type SearchCategoryInput = SearchCategory
  | "feats"
  | "creatures"
  | "hazards"
  | "afflictions"
  | "rules"
  | "spells";

export type SearchSubcategory =
  | "consumable"
  | "gear"
  | "weapon"
  | "armor"
  | "shield"
  | "ammo"
  | "backpack"
  | "treasure"
  | "kit"
  | "vehicle"
  | "class"
  | "ancestry"
  | "skill"
  | "general"
  | "archetype"
  | "boonCurse"
  | "character"
  | "familiar"
  | "haunt"
  | "trap"
  | "curse"
  | "disease"
  | "poison"
  | "action"
  | "condition"
  | "effect"
  | "campaignFeature"
  | "heritage"
  | "background"
  | "deity"
  | "journal";

export type SearchSubcategoryInput = SearchSubcategory
  | "actions"
  | "conditions"
  | "effects"
  | "campaign"
  | "campaigns"
  | "consumables"
  | "weapons"
  | "shields"
  | "backpacks"
  | "treasures"
  | "kits"
  | "vehicles"
  | "archetypes"
  | "haunts"
  | "traps"
  | "curses"
  | "diseases"
  | "poisons"
  | "classes"
  | "ancestries"
  | "heritages"
  | "backgrounds"
  | "deities"
  | "journals";

export interface SearchScope {
  category: SearchCategoryInput;
  subcategories?: SearchSubcategoryInput[];
}

export type SearchMode = "structured" | "lexical" | "hybrid";

export interface SearchFilters {
  searchProfile?: SearchProfile;
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  excludeQuery?: string;
  pack?: string;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  metadata?: MetadataFilterNode;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
}

const METADATA_FILTER_VALUE_FIELDS = METADATA_FIELD_REGISTRY
  .filter((entry) => entry.discoverable)
  .map((entry) => entry.field);

export const FILTER_VALUE_FIELDS = [
  ...METADATA_FILTER_VALUE_FIELDS,
  "actorMetrics",
  "itemMetrics",
  "sources",
  "categories",
  "subcategories",
  "packs",
] as const;

export type FilterValueField = (typeof FILTER_VALUE_FIELDS)[number];

export interface FilterValueQuery {
  field: FilterValueField;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
  metricPrefix?: string;
  metric?: string;
}

export interface FilterValueCount {
  value: string;
  count: number;
}

export interface FilterValueResult {
  field: FilterValueField;
  values: FilterValueCount[];
}

export interface LookupOptions {
  pack?: string;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
}

export type RecordDetail = "minimal" | "standard" | "full";

export interface LookupQuery extends LookupOptions {
  name: string;
}

export interface LookupResult {
  query: LookupQuery;
  match: NormalizedRecord | null;
  alternatives: NormalizedRecord[];
  matchType: "exact" | "normalized_exact" | "fuzzy" | "none";
}

export interface SearchResult {
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  total: number;
  offset: number;
  limit: number;
  records: NormalizedRecord[];
  explain?: SearchExplainResult;
}

export interface SearchQueryAnalysis {
  rawQuery: string;
  normalizedQuery: string;
  queryTokens: string[];
}

export interface SearchRecordExplanation {
  recordKey: string;
  name: string;
  totalScore: number;
  fusionScore: number | null;
  lexicalRank: number | null;
  semanticRank: number | null;
  lexicalRerankScore: number | null;
  matchedTraits: string[];
  matchedNameTokens: string[];
  rerankAdjustments: {
    packQuality: number;
    sourceQuality: number;
    rarityPreference: number;
    sourcePenalty: number;
  };
}

export interface RankingConfigStatus {
  path: string;
  source: "default" | "file";
  revision: number;
  loadedAt: string;
  lastError: string | null;
}

export interface SearchExplainResult {
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  fusionMethod: "weightedRrf" | null;
  fusionProfile: "balanced" | "concept" | null;
  fusionConfig: {
    rrfK: number;
    lexicalWeight: number;
    semanticWeight: number;
    lexicalTopK: number;
    semanticTopK: number;
  } | null;
  lexicalQuery: string;
  semanticQuery: string;
  query: SearchQueryAnalysis | null;
  excludeQuery: SearchQueryAnalysis | null;
  rankingConfig: RankingConfigStatus;
  records: SearchRecordExplanation[];
}
