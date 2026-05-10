import type {
  DerivedTagCanonicalConcept,
  DerivedTagCategoryProjection,
  DerivedTagOntologyTag,
  DerivedTagOntologyFamily,
  SearchCategory,
  PublishedDerivedTagConceptModel,
} from "../../domain/derived-tag-types.js";
import {
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  DERIVED_TAG_CANONICAL_FAMILIES,
  DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY,
  DERIVED_TAG_CANONICAL_RELATIONS,
} from "./registry.js";
import { type PublishedDerivedTagOntology } from "../runtime/publication/catalog.js";
import { buildPublishedDerivedTagTranslations } from "../translations/publication.js";
import { normalizeDerivedTag } from "../runtime/matcher/engine.js";
import {
  getCurrentDerivedTagFamilyTranslationDefaultsRevision,
  getCurrentDerivedTagTranslationOverridesRevision,
} from "../translations/state.js";

type OntologyFamilyKey = `${SearchCategory}:${string}`;
type OntologyTagKey = `${SearchCategory}:${string}`;

function familyKey(category: SearchCategory, family: string): OntologyFamilyKey {
  return `${category}:${normalizeDerivedTag(family)}`;
}

function tagKey(category: SearchCategory, tag: string): OntologyTagKey {
  return `${category}:${normalizeDerivedTag(tag)}`;
}

function buildTagsByFamilyKey(tags: DerivedTagOntologyTag[]): Map<OntologyFamilyKey, DerivedTagOntologyTag[]> {
  const tagsByFamilyKey = new Map<OntologyFamilyKey, DerivedTagOntologyTag[]>();

  for (const tag of tags) {
    const key = familyKey(tag.category, tag.family);
    const current = tagsByFamilyKey.get(key) ?? [];
    current.push(tag);
    tagsByFamilyKey.set(key, current);
  }

  return tagsByFamilyKey;
}

function publishDerivedTagOntology(
  families: DerivedTagOntologyFamily[],
  tags: DerivedTagOntologyTag[],
  conceptModel: PublishedDerivedTagConceptModel,
): PublishedDerivedTagOntology {
  const familyByKey = new Map<OntologyFamilyKey, DerivedTagOntologyFamily>();
  const tagByKey = new Map<OntologyTagKey, DerivedTagOntologyTag>();

  for (const family of families) {
    const normalizedFamilyKey = familyKey(family.category, family.family);
    if (familyByKey.has(normalizedFamilyKey)) {
      throw new Error(
        `Duplicate derived tag family "${normalizeDerivedTag(family.family)}" in category "${family.category}".`,
      );
    }
    familyByKey.set(normalizedFamilyKey, family);
  }

  for (const tag of tags) {
    const normalizedTagKey = tagKey(tag.category, tag.tag);
    const normalizedFamilyKey = familyKey(tag.category, tag.family);
    if (!familyByKey.has(normalizedFamilyKey)) {
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${tag.category}" references unknown family "${normalizeDerivedTag(
          tag.family,
        )}".`,
      );
    }
    if (tagByKey.has(normalizedTagKey)) {
      const existing = tagByKey.get(normalizedTagKey)!;
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${tag.category}" belongs to both "${normalizeDerivedTag(
          existing.family,
        )}" and "${normalizeDerivedTag(tag.family)}".`,
      );
    }
    tagByKey.set(normalizedTagKey, tag);
  }

  for (const tag of tags) {
    for (const adjacentTag of tag.adjacentTags ?? []) {
      if (!tagByKey.has(tagKey(tag.category, adjacentTag))) {
        throw new Error(`Derived tag "${tag.tag}" in category "${tag.category}" references unknown adjacent tag "${adjacentTag}".`);
      }
    }
    for (const childTag of tag.compositeOfAnyTags ?? []) {
      if (!tagByKey.has(tagKey(tag.category, childTag))) {
        throw new Error(
          `Derived tag "${tag.tag}" in category "${tag.category}" references unknown composite child "${childTag}".`,
        );
      }
    }
  }

  return {
    families,
    tags,
    conceptModel,
    familyByKey,
    tagByKey,
    tagsByFamilyKey: buildTagsByFamilyKey(tags),
  };
}

function conceptSort(left: DerivedTagCanonicalConcept, right: DerivedTagCanonicalConcept): number {
  return left.id.localeCompare(right.id);
}

const DERIVED_TAG_CANONICAL_CONCEPTS = Object.values(DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID).sort(conceptSort);
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
      schemaKind: concept.schemaKind,
      domainId: concept.domainId,
      operation: concept.operation,
      primaryFacetKind: concept.primaryFacetKind,
      primaryFacetValue: concept.primaryFacetValue,
      secondaryFacets: concept.secondaryFacets,
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

type CanonicalOntologyCache = {
  familyDefaultsRevision: number;
  translationRevision: number;
  ontology: PublishedDerivedTagOntology;
};

let cachedCanonicalOntology: CanonicalOntologyCache | null = null;

export function getDerivedTagCanonicalConcepts(): readonly DerivedTagCanonicalConcept[] {
  return DERIVED_TAG_CANONICAL_CONCEPTS;
}

export function getDerivedTagCanonicalOntology(): PublishedDerivedTagOntology {
  const familyDefaultsRevision = getCurrentDerivedTagFamilyTranslationDefaultsRevision();
  const translationRevision = getCurrentDerivedTagTranslationOverridesRevision();

  if (
    !cachedCanonicalOntology ||
    cachedCanonicalOntology.familyDefaultsRevision !== familyDefaultsRevision ||
    cachedCanonicalOntology.translationRevision !== translationRevision
  ) {
    cachedCanonicalOntology = {
      familyDefaultsRevision,
      translationRevision,
      ontology: buildCanonicalOntology(),
    };
  }

  return cachedCanonicalOntology.ontology;
}
