import type {
  DerivedTagCanonicalConcept,
  DerivedTagCategoryProjection,
  DerivedTagOntologyTag,
  PublishedDerivedTagConceptModel,
} from "../../domain/derived-tag-types.js";
import {
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  DERIVED_TAG_CANONICAL_FAMILIES,
  DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY,
  DERIVED_TAG_CANONICAL_RELATIONS,
} from "./registry.js";
import { publishDerivedTagOntology, type PublishedDerivedTagOntology } from "../runtime/publication/catalog.js";
import { buildPublishedDerivedTagTranslations } from "../translations/publication.js";

function conceptSort(left: DerivedTagCanonicalConcept, right: DerivedTagCanonicalConcept): number {
  return left.id.localeCompare(right.id);
}

export const DERIVED_TAG_CANONICAL_CONCEPTS = Object.values(DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID).sort(conceptSort);
const CANONICAL_CONCEPTS_BY_ID = DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID;

const GENERATED_CANONICAL_PROJECTIONS = Object.values(DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY)
  .flatMap((categoryProjections) => Object.values(categoryProjections))
  .sort(
    (left, right) =>
      left.category.localeCompare(right.category) ||
      left.axis.localeCompare(right.axis) ||
      left.family.localeCompare(right.family) ||
      left.currentTag.localeCompare(right.currentTag),
  );

function translationKey(category: string, tag: string): `${string}:${string}` {
  return `${category}:${tag}`;
}

function buildTranslationsByKey(translations: PublishedDerivedTagConceptModel["translations"]): PublishedDerivedTagConceptModel["translationsByTagKey"] {
  return new Map(
    translations.map((translation) => [
      translationKey(translation.currentCategory, translation.currentTag),
      translation,
    ] as const),
  );
}

function buildCanonicalProjections(
  translationsByKey: PublishedDerivedTagConceptModel["translationsByTagKey"],
): DerivedTagCategoryProjection[] {
  return GENERATED_CANONICAL_PROJECTIONS.map((projection) => {
    const translation = translationsByKey.get(translationKey(projection.category, projection.currentTag));
    if (!translation) {
      return projection;
    }
    return {
      ...projection,
      axis: translation.projectionAxis,
      family: translation.projectionFamily,
      translationStatus: translation.translationStatus,
    };
  });
}

function buildCanonicalTags(
  projections: DerivedTagCategoryProjection[],
  translationsByKey: PublishedDerivedTagConceptModel["translationsByTagKey"],
): DerivedTagOntologyTag[] {
  return projections.map((projection) => {
    const concept = CANONICAL_CONCEPTS_BY_ID[projection.conceptId];
    if (!concept) {
      throw new Error(`Missing canonical concept "${projection.conceptId}" for projection "${projection.id}".`);
    }
    const translation = translationsByKey.get(translationKey(projection.category, projection.currentTag));
    const schemaKind = translation?.schemaKind ?? concept.schemaKind;
    const domainId = translation?.domainId || concept.domainId;
    const operation = translation?.operation || concept.operation;
    const primaryFacetKind = translation?.primaryFacetKind || concept.primaryFacetKind;
    const primaryFacetValue = translation?.primaryFacetValue || concept.primaryFacetValue;
    const secondaryFacets = translation?.secondaryFacets ?? concept.secondaryFacets;
    return {
      category: projection.category,
      family: projection.family,
      tag: projection.currentTag,
      label: projection.label,
      description: projection.description,
      assignmentMode: projection.assignmentMode,
      nativeOntologyPolicy: projection.nativeOntologyPolicy,
      appliesWhen: projection.appliesWhen,
      doesNotApplyWhen: projection.doesNotApplyWhen,
      positiveSignals: projection.positiveSignals,
      negativeSignals: projection.negativeSignals,
      adjacentTags: projection.adjacentTags,
      compositeOfAnyTags: projection.compositeOfAnyTags,
      variantInheritance: projection.variantInheritance,
      canonicalConceptId: concept.id,
      translationStatus: projection.translationStatus,
      schemaKind,
      domainId,
      operation,
      primaryFacetKind,
      primaryFacetValue,
      secondaryFacets,
      renameNote: translation?.renameNote,
      translationNotes: translation?.notes,
    };
  });
}

function buildCanonicalConceptModel(
  projections: DerivedTagCategoryProjection[],
  translations: PublishedDerivedTagConceptModel["translations"],
): PublishedDerivedTagConceptModel {
  const translationsByKey = buildTranslationsByKey(translations);
  return {
    concepts: DERIVED_TAG_CANONICAL_CONCEPTS,
    conceptById: new Map(DERIVED_TAG_CANONICAL_CONCEPTS.map((concept) => [concept.id, concept])),
    projections,
    projectionsById: new Map(projections.map((projection) => [projection.id, projection])),
    projectionsByTagKey: new Map(
      projections.map((projection) => [translationKey(projection.category, projection.currentTag), projection] as const),
    ),
    translations,
    translationsByTagKey: translationsByKey,
    relations: DERIVED_TAG_CANONICAL_RELATIONS,
  };
}

function buildCanonicalOntology(): PublishedDerivedTagOntology {
  const translations = buildPublishedDerivedTagTranslations();
  const translationsByKey = buildTranslationsByKey(translations);
  const projections = buildCanonicalProjections(translationsByKey);
  const conceptModel = buildCanonicalConceptModel(projections, translations);
  const tags = buildCanonicalTags(projections, translationsByKey);
  return publishDerivedTagOntology(DERIVED_TAG_CANONICAL_FAMILIES, tags, conceptModel);
}

export const DERIVED_TAG_CANONICAL_TRANSLATIONS = buildPublishedDerivedTagTranslations();
const TRANSLATIONS_BY_KEY = buildTranslationsByKey(DERIVED_TAG_CANONICAL_TRANSLATIONS);

export const DERIVED_TAG_CANONICAL_PROJECTIONS = buildCanonicalProjections(TRANSLATIONS_BY_KEY);

export const DERIVED_TAG_CANONICAL_CONCEPT_MODEL: PublishedDerivedTagConceptModel = buildCanonicalConceptModel(
  DERIVED_TAG_CANONICAL_PROJECTIONS,
  DERIVED_TAG_CANONICAL_TRANSLATIONS,
);

export const DERIVED_TAG_CANONICAL_TAGS: DerivedTagOntologyTag[] = buildCanonicalTags(
  DERIVED_TAG_CANONICAL_PROJECTIONS,
  TRANSLATIONS_BY_KEY,
);

export const DERIVED_TAG_CANONICAL_ONTOLOGY: PublishedDerivedTagOntology = publishDerivedTagOntology(
  DERIVED_TAG_CANONICAL_FAMILIES,
  DERIVED_TAG_CANONICAL_TAGS,
  DERIVED_TAG_CANONICAL_CONCEPT_MODEL,
);

export function getDerivedTagCanonicalOntology(): PublishedDerivedTagOntology {
  return buildCanonicalOntology();
}
