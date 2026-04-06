import type {
  ActorMetricMap,
  ItemMetricMap,
  PackInfo,
  SearchCategory,
  SearchSubcategory,
  SourceCategory,
  VariantSource,
} from "../types.js";

export type StageTiming = {
  label: string;
  durationMs: number;
};

export type PackBuildInfo = Omit<PackInfo, "recordCount">;

export type NormalizedIndexRecord = {
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
  baseItem: string | null;
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
  searchText: string;
};

export type ExtractedReference = {
  packName: string | null;
  recordLocator: string;
  displayText: string | null;
  referenceText: string;
};

export type ResolvedBuildReference = {
  targetRecordKey: string;
  targetRecord: NormalizedIndexRecord;
  displayText: string | null;
  referenceText: string;
};

export type RecordAliasRow = {
  canonicalRecordKey: string;
  aliasText: string;
  normalizedAlias: string;
  sourceKind: string;
  sourceRef: string;
};

export type RecordLegacyLinkRow = {
  canonicalRecordKey: string;
  legacyRecordKey: string;
  sourceKind: string;
  sourceRef: string;
};

export type PendingCanonicalEmbedding = {
  record: NormalizedIndexRecord;
  encodedEmbeddingInput: string;
};

export type PendingCanonicalEmbeddingWithHash = PendingCanonicalEmbedding & {
  semanticInputHash: string;
};

export type ActorIndexData = {
  size: string | null;
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
};

export type ItemIndexData = {
  itemCategory: string | null;
  baseItem: string | null;
  priceCp: number | null;
  bulkValue: number | null;
  usage: string | null;
  hands: number | null;
  damageTypes: string[];
  weaponGroup: string | null;
  armorGroup: string | null;
  actionCost: number | null;
  itemMetrics: ItemMetricMap;
};

export type SpellIndexData = {
  actionCost: number | null;
  traditions: string[];
  spellKinds: string[];
  rangeText: string | null;
  rangeValue: number | null;
  saveType: string | null;
  areaType: string | null;
  durationText: string | null;
  durationUnit: string | null;
  targetText: string | null;
  areaValue: number | null;
  sustained: boolean;
  basicSave: boolean;
  damageTypes: string[];
};

export type BuildSourceEntry = {
  pack: PackBuildInfo;
  filePath: string;
  raw: Record<string, unknown>;
  record: NormalizedIndexRecord | null;
  actorData: ActorIndexData | null;
  itemData: ItemIndexData | null;
  spellData: SpellIndexData | null;
  references: ExtractedReference[];
  resolvedReferences: ResolvedBuildReference[];
};

export type BuildIndexResult = {
  packs: PackInfo[];
  warnings: string[];
  recordCount: number;
  stageTimings: StageTiming[];
  reusedCanonicalEmbeddingCount: number;
  regeneratedCanonicalEmbeddingCount: number;
};
