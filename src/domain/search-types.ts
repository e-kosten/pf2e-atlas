import type { NormalizedRecord } from "./record-types.js";

export type SearchProfile = "lexical" | "balanced" | "concept";
export type SearchSort = "ranked" | "alphabetical" | "levelAsc" | "levelDesc" | "random";

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

export type SearchCategoryInput =
  | SearchCategory
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

export type SearchSubcategoryInput =
  | SearchSubcategory
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

export const FILTER_VALUE_FIELDS = [
  "traits",
  "families",
  "derivedTags",
  "traditions",
  "spellKinds",
  "damageTypes",
  "languages",
  "speedTypes",
  "senses",
  "immunities",
  "resistances",
  "weaknesses",
  "disableSkills",
  "variantAxes",
  "sourceCategory",
  "size",
  "usage",
  "weaponGroup",
  "armorGroup",
  "itemCategory",
  "baseItem",
  "saveType",
  "areaType",
  "durationUnit",
  "rarity",
  "variantFamilyKey",
  "publicationTitle",
  "rangeText",
  "durationText",
  "targetText",
  "disableText",
  "variantBaseName",
  "variantLabel",
  "level",
  "priceCp",
  "bulkValue",
  "actionCost",
  "hands",
  "rangeValue",
  "areaValue",
  "hasDescription",
  "publicationRemaster",
  "sustained",
  "basicSave",
  "isComplex",
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

export type SearchResultRecord = NormalizedRecord & {
  matchType?: LookupResult["matchType"];
};

export interface SearchResult {
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  sort: SearchSort;
  total: number;
  offset: number;
  limit: number;
  hasMore: boolean;
  nextOffset: number | null;
  records: SearchResultRecord[];
  explain?: SearchExplainResult;
}

export interface SearchCountResult {
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  total: number;
}

export interface SearchWindow {
  id: string;
  searchProfile: SearchProfile | null;
  mode: SearchMode;
  sort: SearchSort;
  sortSeed: number | null;
  total: number;
}

export interface SearchWindowPage extends SearchWindow {
  offset: number;
  limit: number;
  hasMore: boolean;
  nextOffset: number | null;
  records: SearchResultRecord[];
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
