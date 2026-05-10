import type { SearchCategory } from "../../domain/search-types.js";
import type { SearchSubcategory } from "../../domain/search-types.js";
import type {
  DerivedTagCanonicalConcept,
  DerivedTagCategoryProjection,
  DerivedTagConceptText,
  DerivedTagTranslationStatus,
  DerivedTagConceptRelation,
} from "../../domain/derived-tag-types.js";
import type {
  DerivedTagAssignmentMode,
  DerivedTagOntologyAxis,
  DerivedTagOntologyCategory,
  DerivedTagOntologyFamily,
} from "../../domain/record-types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../manifest.js";
import { CANONICAL_VOCABULARY } from "./vocabulary.js";
import type { CanonicalFacet } from "./facets.js";

export type SchemaKind = (typeof CANONICAL_VOCABULARY.SCHEMA.KIND)[keyof typeof CANONICAL_VOCABULARY.SCHEMA.KIND];

export type CanonicalConceptDescriptor = Omit<DerivedTagCanonicalConcept, "schemaKind" | "id" | "label"> & {
  id?: string;
  label?: string;
  schemaKind?: SchemaKind;
};

export type CanonicalConceptSeed = Omit<CanonicalConceptDescriptor, "schemaKind"> & {
  schemaKind?: SchemaKind;
};

export type FacetlessConceptSeed = Omit<CanonicalConceptSeed, "primaryFacetKind" | "primaryFacetValue">;

export function defineFacetConcepts(
  primaryFacet: CanonicalFacet,
  concepts: Record<string, FacetlessConceptSeed>,
): Record<string, CanonicalConceptSeed> {
  return Object.fromEntries(
    Object.entries(concepts).map(([id, concept]) => [
      id,
      {
        ...concept,
        primaryFacetKind: primaryFacet.kind,
        primaryFacetValue: primaryFacet.value,
      },
    ]),
  ) as Record<string, CanonicalConceptSeed>;
}

export function mergeCanonicalConceptSeeds(
  seeds: readonly Record<string, CanonicalConceptSeed>[],
): Record<string, CanonicalConceptSeed> {
  const merged: Record<string, CanonicalConceptSeed> = {};
  for (const seed of seeds) {
    for (const [id, concept] of Object.entries(seed)) {
      if (merged[id] !== undefined) {
        throw new Error(`Duplicate canonical concept id while merging seeds: ${id}`);
      }
      merged[id] = concept;
    }
  }
  return merged;
}

export function defineDescriptiveConcept(args: {
  id: string;
  label?: string;
  facetKind: string;
  facetValue: string;
  text?: DerivedTagConceptText;
  secondaryFacets?: readonly string[];
}): DerivedTagCanonicalConcept {
  const label = args.label ?? args.id;
  return {
    id: args.id,
    label,
    schemaKind: CANONICAL_VOCABULARY.SCHEMA.KIND.DESCRIPTIVE,
    primaryFacetKind: args.facetKind,
    primaryFacetValue: args.facetValue,
    ...(args.text ? { text: args.text } : {}),
    ...(args.secondaryFacets ? { secondaryFacets: [...args.secondaryFacets] } : {}),
  };
}

export function defineOperationalConcept(args: {
  id: string;
  label?: string;
  domainId: string;
  operation: string;
  text?: DerivedTagConceptText;
  secondaryFacets?: readonly string[];
}): DerivedTagCanonicalConcept {
  const label = args.label ?? args.id;
  return {
    id: args.id,
    label,
    schemaKind: CANONICAL_VOCABULARY.SCHEMA.KIND.OPERATIONAL,
    domainId: args.domainId,
    operation: args.operation,
    ...(args.text ? { text: args.text } : {}),
    ...(args.secondaryFacets ? { secondaryFacets: [...args.secondaryFacets] } : {}),
  };
}

export function defineAggregateConcept(args: {
  id: string;
  label?: string;
  text?: DerivedTagConceptText;
  secondaryFacets?: readonly string[];
}): DerivedTagCanonicalConcept {
  const label = args.label ?? args.id;
  return {
    id: args.id,
    label,
    schemaKind: CANONICAL_VOCABULARY.SCHEMA.KIND.AGGREGATE,
    ...(args.text ? { text: args.text } : {}),
    ...(args.secondaryFacets ? { secondaryFacets: [...args.secondaryFacets] } : {}),
  };
}

export function buildCanonicalConceptMap(
  schemaKind: SchemaKind,
  concepts: Record<string, CanonicalConceptSeed>,
): Record<string, DerivedTagCanonicalConcept> {
  return Object.fromEntries(
    Object.entries(concepts).map(([key, concept]) => {
      const id = concept.id ?? key;
      const label = concept.label ?? id;
      const source = concept as Omit<DerivedTagCanonicalConcept, "schemaKind" | "id" | "label">;
      const fromSeed: Omit<DerivedTagCanonicalConcept, "schemaKind"> = {
        id,
        label,
        ...source,
      };
      return [key, { ...fromSeed, schemaKind }];
    }),
  ) as Record<string, DerivedTagCanonicalConcept>;
}

export type CanonicalProjectionSeed = Omit<
  DerivedTagCategoryProjection,
  "id" | "category" | "currentTag" | "label" | "translationStatus"
> & {
  id?: string;
  category?: SearchCategory;
  currentTag?: string;
  label?: string;
  translationStatus?: DerivedTagTranslationStatus;
};

export type CanonicalProjectionByTag = Record<string, CanonicalProjectionSeed>;

export type CategoryProjectionFamilyDescriptor<C extends DerivedTagManagedCategory = DerivedTagManagedCategory> = {
  category: C;
  axis: DerivedTagOntologyAxis<C>;
  family: string;
};

export type CategoryProjectionSeed = Omit<
  CanonicalProjectionSeed,
  "axis" | "category" | "conceptId" | "currentTag" | "family"
> & {
  concept?: string;
};

export type CategoryProjectionTagSeeds = Record<string, CategoryProjectionSeed | CategoryProjectionSeed[]>;

export type CategoryProjectionFamilyBlock<C extends DerivedTagManagedCategory = DerivedTagManagedCategory> = {
  descriptor: CategoryProjectionFamilyDescriptor<C>;
  tags: CategoryProjectionTagSeeds;
};

export type CategoryProjectionDeclaration<C extends DerivedTagManagedCategory = DerivedTagManagedCategory> = {
  category: C;
  families: readonly CategoryProjectionFamilyBlock<C>[];
};

export function projectionFamily<C extends DerivedTagManagedCategory>(
  descriptor: CategoryProjectionFamilyDescriptor<C>,
  tags: CategoryProjectionTagSeeds,
): CategoryProjectionFamilyBlock<C> {
  return { descriptor, tags };
}

export function defineCategoryProjections<C extends DerivedTagManagedCategory>(
  category: C,
  families: readonly CategoryProjectionFamilyBlock<C>[],
): CategoryProjectionDeclaration<C> {
  return { category, families };
}

export function buildProjectionRecordsByCategory(
  conceptsById: Record<string, DerivedTagCanonicalConcept>,
  declarations: readonly CategoryProjectionDeclaration[],
): Record<DerivedTagManagedCategory, Record<string, DerivedTagCategoryProjection>> {
  const recordsByCategory = Object.fromEntries(
    DERIVED_TAG_MANAGED_CATEGORIES.map((category) => [category, {}]),
  ) as Record<DerivedTagManagedCategory, Record<string, DerivedTagCategoryProjection>>;
  const seenProjectionIds = new Set<string>();

  for (const declaration of declarations) {
    if (!DERIVED_TAG_MANAGED_CATEGORIES.includes(declaration.category)) {
      throw new Error(`Canonical projection declaration uses unmanaged category "${declaration.category}".`);
    }

    const categoryRecords = recordsByCategory[declaration.category];
    for (const familyBlock of declaration.families) {
      if (familyBlock.descriptor.category !== declaration.category) {
        throw new Error(
          `Canonical projection family "${familyBlock.descriptor.category}:${familyBlock.descriptor.axis}:${familyBlock.descriptor.family}" does not belong in ${declaration.category} projections.`,
        );
      }

      for (const [currentTag, seedOrSeeds] of Object.entries(familyBlock.tags)) {
        for (const seed of Array.isArray(seedOrSeeds) ? seedOrSeeds : [seedOrSeeds]) {
          const conceptId = seed.concept ?? currentTag;
          if (!conceptsById[conceptId]) {
            throw new Error(`Canonical projection declaration references unknown concept "${conceptId}".`);
          }

          const projectionId = seed.id ?? `${declaration.category}:${currentTag}`;
          if (seenProjectionIds.has(projectionId)) {
            throw new Error(`Duplicate canonical projection id while building projections: ${projectionId}`);
          }
          seenProjectionIds.add(projectionId);

          if (categoryRecords[currentTag]) {
            throw new Error(`Duplicate canonical projection tag "${declaration.category}:${currentTag}".`);
          }

          const { concept: _concept, id: _id, ...projectionSeed } = seed;
          categoryRecords[currentTag] = {
            ...projectionSeed,
            axis: familyBlock.descriptor.axis,
            family: familyBlock.descriptor.family,
            id: projectionId,
            category: declaration.category,
            conceptId,
            currentTag,
            label: seed.label ?? conceptId,
            translationStatus: seed.translationStatus ?? CANONICAL_VOCABULARY.TRANSLATION.STATUS.MAPPED,
          } as DerivedTagCategoryProjection;
        }
      }
    }
  }

  return recordsByCategory;
}

export type CanonicalFamilySeed = Omit<DerivedTagOntologyFamily, "label"> & {
  label?: string;
};

export function buildCanonicalOntologyFamilies(families: CanonicalFamilySeed[]): DerivedTagOntologyFamily[] {
  return families.map((family) => ({
    ...family,
    label: family.label ?? family.family,
  }));
}

export function defineProjection<C extends DerivedTagOntologyCategory>(
  category: C,
  conceptId: string,
  args: {
    axis: DerivedTagOntologyAxis<C>;
    family: string;
    assignmentMode: DerivedTagAssignmentMode;
    description: string;
    legacyTag?: string;
    label?: string;
    subcategories?: readonly SearchSubcategory[];
    nativeOntologyPolicy?: "distinct_required" | "aggregates_native_signals";
    appliesWhen?: readonly string[];
    doesNotApplyWhen?: readonly string[];
    positiveSignals?: readonly string[];
    negativeSignals?: readonly string[];
    adjacentTags?: readonly string[];
    compositeOfAnyTags?: readonly string[];
    variantInheritance?: boolean;
    translationStatus?: DerivedTagTranslationStatus;
  },
): DerivedTagCategoryProjection {
  const currentTag = args.legacyTag ?? conceptId;
  return {
    id: `${category}:${currentTag}`,
    conceptId,
    category,
    axis: args.axis,
    family: args.family,
    currentTag,
    label: args.label ?? conceptId,
    description: args.description,
    assignmentMode: args.assignmentMode,
    ...(args.subcategories ? { subcategories: [...args.subcategories] } : {}),
    ...(args.nativeOntologyPolicy ? { nativeOntologyPolicy: args.nativeOntologyPolicy } : {}),
    ...(args.appliesWhen ? { appliesWhen: [...args.appliesWhen] } : {}),
    ...(args.doesNotApplyWhen ? { doesNotApplyWhen: [...args.doesNotApplyWhen] } : {}),
    ...(args.positiveSignals ? { positiveSignals: [...args.positiveSignals] } : {}),
    ...(args.negativeSignals ? { negativeSignals: [...args.negativeSignals] } : {}),
    ...(args.adjacentTags ? { adjacentTags: [...args.adjacentTags] } : {}),
    ...(args.compositeOfAnyTags ? { compositeOfAnyTags: [...args.compositeOfAnyTags] } : {}),
    ...(args.variantInheritance !== undefined ? { variantInheritance: args.variantInheritance } : {}),
    translationStatus: args.translationStatus ?? CANONICAL_VOCABULARY.TRANSLATION.STATUS.MAPPED,
  };
}

export function buildProjectionRecord<C extends SearchCategory>(
  category: C,
  projections: CanonicalProjectionByTag,
): Record<string, DerivedTagCategoryProjection> {
  return Object.fromEntries(
    Object.entries(projections).map(([legacyTag, projection]) => {
      const normalizedTag = projection.currentTag ?? legacyTag;
      const merged = {
        ...projection,
        category,
        currentTag: normalizedTag,
        id: projection.id ?? `${category}:${normalizedTag}`,
        conceptId: projection.conceptId,
        label: projection.label ?? projection.conceptId,
        translationStatus: projection.translationStatus ?? CANONICAL_VOCABULARY.TRANSLATION.STATUS.MAPPED,
      } as DerivedTagCategoryProjection;
      return [normalizedTag, merged];
    }),
  );
}

export function relate(
  fromConceptId: string,
  relation: DerivedTagConceptRelation,
  toConceptId: string,
): { fromConceptId: string; relation: DerivedTagConceptRelation; toConceptId: string } {
  return { fromConceptId, relation, toConceptId };
}
