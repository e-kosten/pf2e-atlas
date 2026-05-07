import type { DerivedTagAssignmentMode, DerivedTagOntologyAxis } from "./record-types.js";
import type { SearchCategory, SearchSubcategory } from "./search-types.js";

export type DerivedTagConceptSchemaKind = "descriptive" | "operational" | "aggregate";

export type DerivedTagTranslationStatus = "mapped" | "provisional" | "unmapped" | "dropped";

export type DerivedTagConceptText = {
  definition?: string;
  pathfinderContext?: string;
  examples?: string[];
  nonExamples?: string[];
};

export type DerivedTagConceptFacet = {
  kind: string;
  value: string;
};

export type DerivedTagCanonicalConcept = {
  id: string;
  label: string;
  schemaKind: DerivedTagConceptSchemaKind;
  text?: DerivedTagConceptText;
  domainId?: string;
  operation?: string;
  primaryFacetKind?: string;
  primaryFacetValue?: string;
  secondaryFacets?: string[];
};

export type DerivedTagConceptRelation =
  | "counteracts"
  | "applies"
  | "requires"
  | "enables"
  | "evokes"
  | "overlaps_with"
  | "specializes";

export type DerivedTagCanonicalConceptRelation = {
  fromConceptId: string;
  relation: DerivedTagConceptRelation;
  toConceptId: string;
};

export type DerivedTagCategoryProjection = {
  conceptId: string;
  category: SearchCategory;
  axis: DerivedTagOntologyAxis;
  family: string;
  currentTag: string;
  description: string;
  assignmentMode: DerivedTagAssignmentMode;
  subcategories?: SearchSubcategory[];
  nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
  appliesWhen?: string[];
  doesNotApplyWhen?: string[];
  positiveSignals?: string[];
  negativeSignals?: string[];
  adjacentTags?: string[];
  compositeOfAnyTags?: string[];
  variantInheritance?: boolean;
  translationStatus: DerivedTagTranslationStatus;
};

export type DerivedTagTranslationRecord = {
  currentCategory: SearchCategory;
  currentBrowseAxis: DerivedTagOntologyAxis;
  currentFamily: string;
  currentTag: string;
  currentAssignmentMode: DerivedTagAssignmentMode;
  translationStatus: DerivedTagTranslationStatus;
  canonicalConceptId: string;
  canonicalConceptLabel: string;
  schemaKind: DerivedTagConceptSchemaKind;
  domainId?: string;
  operation?: string;
  primaryFacetKind?: string;
  primaryFacetValue?: string;
  secondaryFacets?: string[];
  projectionAxis: DerivedTagOntologyAxis;
  projectionFamily: string;
  renameNote?: string;
  notes?: string;
  publishTag: boolean;
};

export type PublishedDerivedTagConceptModel = {
  concepts: DerivedTagCanonicalConcept[];
  conceptById: Map<string, DerivedTagCanonicalConcept>;
  projections: DerivedTagCategoryProjection[];
  projectionsByTagKey: Map<`${string}:${string}`, DerivedTagCategoryProjection>;
  translations: DerivedTagTranslationRecord[];
  translationsByTagKey: Map<`${string}:${string}`, DerivedTagTranslationRecord>;
  relations: DerivedTagCanonicalConceptRelation[];
};
