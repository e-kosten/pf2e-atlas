export interface AppConfig {
  dataPath: string;
  rootPath: string;
  manifestPath: string;
  indexPath: string;
  embeddings: EmbeddingConfig;
  ranking: RankingRuntimeConfig;
}

export interface RankingRuntimeConfig {
  configPath: string;
}

export type EmbeddingProviderKind = "hash" | "hf-local";

export interface EmbeddingConfig {
  provider: EmbeddingProviderKind;
  modelId: string;
  modelRevision: string | null;
  cachePath: string;
  localModelPath: string | null;
}

export interface PackManifestEntry {
  name: string;
  label: string;
  path: string;
  type: string;
}

export interface PackInfo {
  name: string;
  label: string;
  documentType: string;
  declaredPath: string;
  resolvedPath: string;
  recordCount: number;
}

export interface LinkedRecordSummary {
  recordKey: string;
  name: string;
}

export type SourceCategory = "core" | "rules" | "adventure" | "unknown";

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

export interface DerivedTagCatalogTag {
  value: string;
  description?: string;
}

export interface DerivedTagCatalogEntry {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  family: string;
  description: string;
  tags: DerivedTagCatalogTag[];
}

export interface NormalizedRecord {
  recordKey: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
  category: SearchCategory;
  subcategory: SearchSubcategory | null;
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traits: string[];
  derivedTags: string[];
  publicationTitle: string | null;
  publicationRemaster: boolean;
  descriptionText: string | null;
  hasDescription: boolean;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  families: string[];
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditions: string[];
  spellKinds: string[];
  aliases: string[];
  legacyRecordLinks: LinkedRecordSummary[];
  raw: Record<string, unknown>;
}

export type SearchMode = "structured" | "lexical" | "hybrid";

export interface SearchFilters {
  searchProfile?: SearchProfile;
  explain?: boolean;
  nameQuery?: string;
  query?: string;
  pack?: string;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  traitsAll?: string[];
  traitsAny?: string[];
  excludeTraits?: string[];
  familiesAll?: string[];
  familiesAny?: string[];
  excludeFamilies?: string[];
  derivedTagsAll?: string[];
  derivedTagsAny?: string[];
  excludeDerivedTags?: string[];
  sources?: SourceCategory[];
  excludeSources?: SourceCategory[];
  traditions?: string[];
  spellKinds?: string[];
  publicationTitle?: string;
  excludeUnique?: boolean;
  excludeMissingDescription?: boolean;
  size?: string;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
}

export type FilterValueField =
  | "traits"
  | "families"
  | "derivedTags"
  | "rarity"
  | "size"
  | "publicationTitle"
  | "traditions"
  | "spellKinds"
  | "sources"
  | "categories"
  | "subcategories"
  | "packs";

export interface FilterValueQuery {
  field: FilterValueField;
  category?: SearchCategoryInput;
  subcategory?: SearchSubcategoryInput;
  scopes?: SearchScope[];
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
  rankingConfig: RankingConfigStatus;
  records: SearchRecordExplanation[];
}

export interface RankingConfigStatus {
  path: string;
  source: "default" | "file";
  revision: number;
  loadedAt: string;
  lastError: string | null;
}

export interface RuleReferenceEdge {
  fromRecordKey: string;
  toRecordKey: string;
  displayText: string | null;
  referenceText: string;
  direction: "outgoing" | "backlink";
  relationshipType: "references" | "referenced_by";
  sourcePackName: string;
  sourceRecordType: string;
  sourceDocumentType: string;
  sourceCategory: SourceCategory;
}

export interface RuleGraphResult {
  records: NormalizedRecord[];
  edges: RuleReferenceEdge[];
}

export interface RuleGraphCollectionResult {
  outgoing: RuleGraphResult;
  backlinks: RuleGraphResult;
  edges: RuleReferenceEdge[];
}

export interface CollectRuleQuestionContextInput {
  rules?: string[];
  question?: string;
  coreOnly?: boolean;
  maxOutgoingPerPrimary?: number;
  maxBacklinksPerPrimary?: number;
  includeBacklinks?: boolean;
}

export interface CollectRuleQuestionContextResult extends RuleGraphCollectionResult {
  primary: LookupResult[];
}
