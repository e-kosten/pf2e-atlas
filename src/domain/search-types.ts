import type { NormalizedRecord } from "./record-types.js";

export const SEARCH_VOCABULARY = {
  PROFILE: {
    LEXICAL: "lexical" as const,
    BALANCED: "balanced" as const,
    CONCEPT: "concept" as const,
  },
  SORT_KIND: {
    RANKED: "ranked" as const,
    ALPHABETICAL: "alphabetical" as const,
    LEVEL_ASC: "levelAsc" as const,
    LEVEL_DESC: "levelDesc" as const,
    RANDOM: "random" as const,
  },
  MODE: {
    STRUCTURED: "structured" as const,
    LEXICAL: "lexical" as const,
    HYBRID: "hybrid" as const,
  },
} as const;

type SearchVocabulary = typeof SEARCH_VOCABULARY;
export const SEARCH_PROFILE_VALUES = [
  SEARCH_VOCABULARY.PROFILE.LEXICAL,
  SEARCH_VOCABULARY.PROFILE.BALANCED,
  SEARCH_VOCABULARY.PROFILE.CONCEPT,
] as const;
export const SEARCH_SORT_VALUES = [
  SEARCH_VOCABULARY.SORT_KIND.RANKED,
  SEARCH_VOCABULARY.SORT_KIND.ALPHABETICAL,
  SEARCH_VOCABULARY.SORT_KIND.LEVEL_ASC,
  SEARCH_VOCABULARY.SORT_KIND.LEVEL_DESC,
  SEARCH_VOCABULARY.SORT_KIND.RANDOM,
] as const;
export const BROWSE_SEARCH_SORT_VALUES = [
  SEARCH_VOCABULARY.SORT_KIND.ALPHABETICAL,
  SEARCH_VOCABULARY.SORT_KIND.LEVEL_ASC,
  SEARCH_VOCABULARY.SORT_KIND.LEVEL_DESC,
] as const;
export const SEARCH_MODE_VALUES = [
  SEARCH_VOCABULARY.MODE.STRUCTURED,
  SEARCH_VOCABULARY.MODE.LEXICAL,
  SEARCH_VOCABULARY.MODE.HYBRID,
] as const;
export type SearchProfile = SearchVocabulary["PROFILE"][keyof SearchVocabulary["PROFILE"]];
export type SearchSort = SearchVocabulary["SORT_KIND"][keyof SearchVocabulary["SORT_KIND"]];
export type SearchFusionProfile =
  | SearchVocabulary["PROFILE"]["BALANCED"]
  | SearchVocabulary["PROFILE"]["CONCEPT"];

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

export type SearchMode = SearchVocabulary["MODE"][keyof SearchVocabulary["MODE"]];

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
  valueType?: "number" | "text" | "boolean";
  numericMin?: number | null;
  numericMax?: number | null;
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
  fusionProfile: SearchFusionProfile | null;
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
