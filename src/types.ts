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

export type SourceCategory = "core" | "rules" | "adventure" | "unknown";

export type RankingProfile = "default" | "preferReusableReferenceContent";

export type SearchCategory =
  | "equipment"
  | "feats"
  | "creatures"
  | "hazards"
  | "afflictions"
  | "rules"
  | "spells"
  | "characterCreation"
  | "lore";

export interface NormalizedRecord {
  recordKey: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
  category: SearchCategory;
  subcategories: string[];
  packName: string;
  packLabel: string;
  documentType: string;
  level: number | null;
  rarity: string | null;
  traits: string[];
  publicationTitle: string | null;
  descriptionText: string | null;
  hasDescription: boolean;
  descriptionSnippet: string | null;
  sourceCategory: SourceCategory;
  folderId: string | null;
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  traditions: string[];
  raw: Record<string, unknown>;
}

export type SearchMode = "structured" | "lexical" | "hybrid";

export interface SearchFilters {
  mode?: SearchMode;
  rankingProfile?: RankingProfile;
  explain?: boolean;
  nameQuery?: string;
  themeQuery?: string;
  pack?: string;
  category?: SearchCategory;
  subcategory?: string;
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  traitsAll?: string[];
  traitsAny?: string[];
  tradition?: string;
  publicationTitle?: string;
  excludeUnique?: boolean;
  excludeMissingDescription?: boolean;
  excludeAdventureContent?: boolean;
  coreOnly?: boolean;
  size?: string;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
}

export interface LookupOptions {
  pack?: string;
  category?: SearchCategory;
  subcategory?: string;
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
  lexicalScore: number;
  semanticScore: number;
  matchedTraits: string[];
  matchedNameTokens: string[];
  matchedMetadataTokens: string[];
  components: {
    fullTextSearch: number;
    metadataText: number;
    descriptionText: number;
    themeName: number;
    themeTraits: number;
    themeMetadata: number;
    missingDescriptionNormalization: number;
    sourceQuality: number;
    rarityPreference: number;
    sourcePenalty: number;
    packQuality: number;
    rankingProfile: number;
  };
}

export interface SearchExplainResult {
  mode: SearchMode;
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

export interface CollectRuleQuestionContextInput {
  rules?: string[];
  question?: string;
  coreOnly?: boolean;
  maxOutgoingPerPrimary?: number;
  maxBacklinksPerPrimary?: number;
  includeBacklinks?: boolean;
}

export interface CollectRuleQuestionContextResult {
  primary: LookupResult[];
  outgoing: RuleGraphResult;
  backlinks: RuleGraphResult;
  edges: RuleReferenceEdge[];
}
