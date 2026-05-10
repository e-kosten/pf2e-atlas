import type { DerivedTagConceptSchemaKind, DerivedTagTranslationStatus } from "./derived-tag-concept-types.js";
import type { SearchCategory, SearchSubcategory } from "./search-types.js";
import type { ActorMetricMap } from "./actor-metrics.js";
import type { ItemMetricMap } from "./item-metrics.js";

export type RecordKey = string;

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
  recordKey: RecordKey;
  name: string;
}

export interface DerivedTagSeedRecordReference {
  pack: string;
  name: string;
}

export interface DerivedTagSeedRecordResolution extends DerivedTagSeedRecordReference {
  recordKey: RecordKey;
}

export interface DerivedTagExemplarRecord {
  name: string;
  recordKey: RecordKey;
}

export interface DerivedTagExemplarSet {
  tag: string;
  positives?: DerivedTagExemplarRecord[];
  negatives?: DerivedTagExemplarRecord[];
  notes?: string;
}

export interface DerivedTagExemplarCategory {
  category: SearchCategory;
  exemplars: DerivedTagExemplarSet[];
}

export type DerivedTagExemplarPolarity = "positive" | "negative";

export type DerivedTagExemplarReviewStatus = "needs_review" | "approved" | "rejected";

export type DerivedTagExemplarReviewConfidence = "high" | "medium" | "low";

export type DerivedTagExemplarReviewSource = "human" | "llm";

export interface DerivedTagExemplarReviewDecision {
  name: string;
  recordKey: RecordKey;
  tag: string;
  proposedPolarity: DerivedTagExemplarPolarity | "drop";
  currentPolarity?: DerivedTagExemplarPolarity | "none";
  status: DerivedTagExemplarReviewStatus;
  confidence?: DerivedTagExemplarReviewConfidence;
  rationale: string;
  source?: DerivedTagExemplarReviewSource;
}

export interface DerivedTagExemplarReviewCategory {
  category: SearchCategory;
  decisions: DerivedTagExemplarReviewDecision[];
}

export interface DerivedTagLegacySeedMigrationTag {
  tag: string;
  includeRecords?: DerivedTagSeedRecordReference[];
  excludeRecords?: DerivedTagSeedRecordReference[];
}

export interface DerivedTagLegacySeedMigrationCategory {
  category: SearchCategory;
  tags: DerivedTagLegacySeedMigrationTag[];
}

export type SourceCategory = "core" | "rules" | "adventure" | "unknown";
export type VariantSource = "baseItem" | "slug" | "namePattern" | "sourcePath" | "composite" | "none";
export type DerivedTagOntologyCategory = "equipment" | "creature" | "hazard" | "affliction" | "spell";
export interface DerivedTagOntologyAxisByCategory {
  equipment: "legacy" | "utility" | "party_role" | "item_mechanical" | "effect";
  creature: "legacy" | "setting" | "encounter" | "npc_role" | "presentation" | "specialization";
  hazard: "legacy" | "mechanism" | "encounter" | "setting" | "haunt" | "resolution" | "problem" | "effect";
  affliction: "disease_model" | "response" | "behavior" | "metaphysical" | "effect";
  spell: "legacy" | "utility" | "battlefield" | "transformation" | "influence" | "support" | "summoning" | "effect";
}
export type DerivedTagOntologyAxis<C extends DerivedTagOntologyCategory = DerivedTagOntologyCategory> =
  DerivedTagOntologyAxisByCategory[C];

export interface DerivedTagOntologyFamily<C extends DerivedTagOntologyCategory = DerivedTagOntologyCategory> {
  category: C;
  subcategories?: SearchSubcategory[];
  family: string;
  label?: string;
  axis: DerivedTagOntologyAxis<C>;
  description: string;
  variantInheritance?: boolean;
}

export interface DerivedTagOntologyTag {
  category: SearchCategory;
  family: string;
  tag: string;
  label?: string;
  description: string;
  isComposite?: boolean;
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
  variantInheritance?: boolean;
  canonicalConceptId?: string;
  translationStatus?: DerivedTagTranslationStatus;
  schemaKind?: DerivedTagConceptSchemaKind;
  domainId?: string;
  operation?: string;
  primaryFacetKind?: string;
  primaryFacetValue?: string;
  secondaryFacets?: string[];
  renameNote?: string;
  translationNotes?: string;
}

export interface DerivedTagCatalogTag {
  value: string;
  label?: string;
  description?: string;
  isComposite?: boolean;
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
  variantInheritance?: boolean;
  canonicalConceptId?: string;
  translationStatus?: DerivedTagTranslationStatus;
  schemaKind?: DerivedTagConceptSchemaKind;
  domainId?: string;
  operation?: string;
  primaryFacetKind?: string;
  primaryFacetValue?: string;
  secondaryFacets?: string[];
}

export interface DerivedTagCatalogEntry {
  category: DerivedTagOntologyCategory;
  subcategories?: SearchSubcategory[];
  family: string;
  label?: string;
  axis: DerivedTagOntologyAxis;
  description: string;
  isComposite?: boolean;
  tags: DerivedTagCatalogTag[];
  variantInheritance?: boolean;
}

export interface NormalizedRecord {
  recordKey: RecordKey;
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
  blurbText: string | null;
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
  aliases: string[];
  legacyRecordLinks: LinkedRecordSummary[];
  raw: Record<string, unknown>;
}
