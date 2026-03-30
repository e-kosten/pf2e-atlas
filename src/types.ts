export interface AppConfig {
  dataPath: string;
  rootPath: string;
  manifestPath: string;
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
  folderId: string | null;
  sourcePath: string;
  raw: Record<string, unknown>;
}

export interface SearchFilters {
  nameQuery?: string;
  pack?: string;
  documentType?: string;
  recordType?: string;
  levelMin?: number;
  levelMax?: number;
  rarity?: string;
  traitsAll?: string[];
  traitsAny?: string[];
  publicationTitle?: string;
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
