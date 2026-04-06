import type { SearchCategory, SearchSubcategory } from "./search-types.js";
import type { ActorMetricMap } from "./actor-metrics.js";
import type { ItemMetricMap } from "./item-metrics.js";

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
export type VariantSource = "baseItem" | "slug" | "namePattern" | "sourcePath" | "composite" | "none";

export interface DerivedTagCatalogTag {
  value: string;
  description?: string;
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  seedRecordKeys?: string[];
  excludeSeedRecordKeys?: string[];
}

export interface DerivedTagCatalogEntry {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  family: string;
  description: string;
  tags: DerivedTagCatalogTag[];
  promoteFamilyToTag?: boolean;
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
  variantFamilyKey: string | null;
  variantBaseName: string | null;
  variantLabel: string | null;
  variantAxes: string[];
  variantConfidence: number | null;
  variantSource: VariantSource;
  sourcePath: string;
  isUnique: boolean;
  size: string | null;
  itemCategory: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  actionCost: number | null;
  usage: string | null;
  hands: number | null;
  damageTypes: string[];
  weaponGroup: string | null;
  armorGroup: string | null;
  traditions: string[];
  spellKinds: string[];
  saveType: string | null;
  areaType: string | null;
  rangeText: string | null;
  durationText: string | null;
  durationUnit: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: boolean;
  basicSave: boolean;
  languages: string[];
  speedTypes: string[];
  senses: string[];
  immunities: string[];
  resistances: string[];
  weaknesses: string[];
  disableText: string | null;
  disableSkills: string[];
  isComplex: boolean;
  actorMetrics: ActorMetricMap;
  itemMetrics: ItemMetricMap;
  rangeValue: number | null;
  aliases: string[];
  legacyRecordLinks: LinkedRecordSummary[];
  raw: Record<string, unknown>;
}
