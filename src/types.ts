export interface AppConfig {
  dataPath: string;
  rootPath: string;
  manifestPath: string;
  indexPath: string;
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

export interface NormalizedRecord {
  recordKey: string;
  id: string;
  name: string;
  normalizedName: string;
  type: string;
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
  nameQuery?: string;
  themeQuery?: string;
  pack?: string;
  documentType?: string;
  recordType?: string;
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
  itemCategory?: string;
  priceMin?: number;
  priceMax?: number;
  actionCost?: number;
  offset?: number;
  limit?: number;
}

export interface LookupOptions {
  pack?: string;
  documentType?: string;
  recordType?: string;
}

export interface SearchResult {
  total: number;
  offset: number;
  limit: number;
  records: NormalizedRecord[];
}
