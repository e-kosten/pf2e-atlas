import type {
  DerivedTagAuthoredCategoryOntology,
  DerivedTagCanonicalConcept,
  DerivedTagCanonicalConceptRelation,
  DerivedTagCategoryProjection,
  DerivedTagTranslationRecord,
  PublishedDerivedTagConceptModel,
} from "../../domain/derived-tag-types.js";
import type { DerivedTagOntologyFamily, DerivedTagOntologyTag } from "../../domain/record-types.js";
import type { DerivedTagManagedCategory } from "../manifest.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../ontology/utils.js";
import { getDerivedTagFamilyTranslationDefaults } from "./family-defaults.js";
import { inferOperationalTranslationDefaults } from "./inference.js";
import { getDerivedTagTranslationOverride } from "./tag-overrides.js";

function tagKey(category: string, tag: string): `${string}:${string}` {
  return `${category}:${tag}`;
}

function mergeNotes(...parts: Array<string | undefined>): string | undefined {
  const merged = parts.filter(Boolean).join(" ");
  return merged || undefined;
}

function buildTranslationRecord(
  family: DerivedTagOntologyFamily,
  tag: DerivedTagOntologyTag,
): DerivedTagTranslationRecord {
  const config = getDerivedTagFamilyTranslationDefaults(`${tag.category}:${tag.family}`);
  if (!config) {
    throw new Error(`Missing derived-tag translation defaults for ${tag.category}:${tag.family}.`);
  }

  const row: DerivedTagTranslationRecord = {
    currentCategory: tag.category,
    currentBrowseAxis: family.axis,
    currentFamily: tag.family,
    currentTag: tag.tag,
    currentAssignmentMode: tag.assignmentMode,
    translationStatus: config.translationStatus,
    canonicalConceptId: tag.tag,
    canonicalConceptLabel: tag.tag,
    schemaKind: config.schemaKind,
    domainId: "",
    operation: "",
    primaryFacetKind: config.primaryFacetKind,
    primaryFacetValue: config.primaryFacetValue,
    secondaryFacets: [],
    projectionAxis: family.axis,
    projectionFamily: tag.family,
    notes: config.notes,
    publishTag: true,
  };

  if (row.schemaKind === "operational") {
    Object.assign(row, inferOperationalTranslationDefaults(tag.tag));
  }

  const override = getDerivedTagTranslationOverride(`${tag.category}:${tag.tag}`);
  if (override) {
    Object.assign(row, override);
    row.notes = mergeNotes(config.notes, override.notes);
  }

  if (tag.assignmentMode === "composite" && row.publishTag) {
    row.schemaKind = "aggregate";
  }

  return row;
}

function buildCanonicalConcepts(translations: DerivedTagTranslationRecord[]): DerivedTagCanonicalConcept[] {
  const concepts = new Map<string, DerivedTagCanonicalConcept>();

  for (const translation of translations) {
    if (translation.translationStatus === "dropped") {
      continue;
    }
    const current = concepts.get(translation.canonicalConceptId);
    if (current) {
      continue;
    }
    concepts.set(translation.canonicalConceptId, {
      id: translation.canonicalConceptId,
      label: translation.canonicalConceptLabel,
      schemaKind: translation.schemaKind,
      domainId: translation.domainId || undefined,
      operation: translation.operation || undefined,
      primaryFacetKind: translation.primaryFacetKind || undefined,
      primaryFacetValue: translation.primaryFacetValue || undefined,
      secondaryFacets: translation.secondaryFacets?.length ? [...translation.secondaryFacets] : undefined,
      text: translation.notes ? { definition: translation.notes } : undefined,
    });
  }

  return [...concepts.values()].sort((left, right) => left.id.localeCompare(right.id));
}

function buildConceptRelations(translations: DerivedTagTranslationRecord[]): DerivedTagCanonicalConceptRelation[] {
  const relations = new Map<string, DerivedTagCanonicalConceptRelation>();

  const remediationsByDomain = new Map<string, string[]>();
  const applicationsByDomain = new Map<string, string[]>();

  for (const translation of translations) {
    if (!translation.domainId || !translation.operation || translation.translationStatus === "dropped") {
      continue;
    }
    if (translation.operation === "remediate") {
      const current = remediationsByDomain.get(translation.domainId) ?? [];
      current.push(translation.canonicalConceptId);
      remediationsByDomain.set(translation.domainId, current);
    }
    if (translation.operation === "apply") {
      const current = applicationsByDomain.get(translation.domainId) ?? [];
      current.push(translation.canonicalConceptId);
      applicationsByDomain.set(translation.domainId, current);
    }
  }

  for (const [domainId, remediations] of remediationsByDomain.entries()) {
    const applications = applicationsByDomain.get(domainId) ?? [];
    for (const fromConceptId of remediations) {
      for (const toConceptId of applications) {
        const key = `${fromConceptId}:counteracts:${toConceptId}`;
        relations.set(key, { fromConceptId, relation: "counteracts", toConceptId });
      }
    }
  }

  return [...relations.values()].sort(
    (left, right) =>
      left.fromConceptId.localeCompare(right.fromConceptId) ||
      left.relation.localeCompare(right.relation) ||
      left.toConceptId.localeCompare(right.toConceptId),
  );
}

function buildProjections(
  families: DerivedTagOntologyFamily[],
  tags: DerivedTagOntologyTag[],
  translations: DerivedTagTranslationRecord[],
): DerivedTagCategoryProjection[] {
  const familyByKey = new Map<string, DerivedTagOntologyFamily>(
    families.map((family) => [tagKey(family.category, family.family), family]),
  );
  const tagByKey = new Map<string, DerivedTagOntologyTag>(
    tags.map((tag) => [tagKey(tag.category, tag.tag), tag]),
  );

  const projections: DerivedTagCategoryProjection[] = [];

  for (const translation of translations) {
    if (!translation.publishTag) {
      continue;
    }
    const tag = tagByKey.get(tagKey(translation.currentCategory, translation.currentTag));
    const family = familyByKey.get(tagKey(translation.currentCategory, translation.currentFamily));
    if (!tag || !family) {
      throw new Error(`Missing source ontology entry for ${translation.currentCategory}:${translation.currentFamily}:${translation.currentTag}.`);
    }
    projections.push({
      conceptId: translation.canonicalConceptId,
      category: translation.currentCategory,
      axis: translation.projectionAxis,
      family: translation.projectionFamily,
      currentTag: translation.currentTag,
      description: tag.description,
      assignmentMode: tag.assignmentMode,
      subcategories: family.subcategories,
      nativeOntologyPolicy: tag.nativeOntologyPolicy,
      appliesWhen: tag.appliesWhen,
      doesNotApplyWhen: tag.doesNotApplyWhen,
      positiveSignals: tag.positiveSignals,
      negativeSignals: tag.negativeSignals,
      adjacentTags: tag.adjacentTags,
      compositeOfAnyTags: tag.compositeOfAnyTags,
      variantInheritance: tag.variantInheritance,
      translationStatus: translation.translationStatus,
    });
  }

  return projections.sort(
    (left, right) =>
      left.category.localeCompare(right.category) ||
      left.axis.localeCompare(right.axis) ||
      left.family.localeCompare(right.family) ||
      left.currentTag.localeCompare(right.currentTag),
  );
}

export function buildPublishedDerivedTagConceptModel(
  ontologyByCategory: Record<DerivedTagManagedCategory, DerivedTagAuthoredCategoryOntology>,
): PublishedDerivedTagConceptModel {
  const flattened = Object.values(ontologyByCategory).map((ontology) => flattenDerivedTagAuthoredCategoryOntology(ontology));
  const families = flattened.flatMap((entry) => entry.families);
  const tags = flattened.flatMap((entry) => entry.tags);
  const familyByKey = new Map<string, DerivedTagOntologyFamily>(
    families.map((family) => [tagKey(family.category, family.family), family]),
  );
  const translations = tags.map((tag) => {
    const family = familyByKey.get(tagKey(tag.category, tag.family));
    if (!family) {
      throw new Error(`Missing source family for ${tag.category}:${tag.family}.`);
    }
    return buildTranslationRecord(family, tag);
  });

  const concepts = buildCanonicalConcepts(translations);
  const conceptById = new Map(concepts.map((concept) => [concept.id, concept]));
  const projections = buildProjections(families, tags, translations);
  const projectionsByTagKey = new Map(
    projections.map((projection) => [tagKey(projection.category, projection.currentTag), projection]),
  );
  const translationsByTagKey = new Map(
    translations.map((translation) => [tagKey(translation.currentCategory, translation.currentTag), translation]),
  );

  return {
    concepts,
    conceptById,
    projections,
    projectionsByTagKey,
    translations,
    translationsByTagKey,
    relations: buildConceptRelations(translations),
  };
}

export function buildProjectedDerivedTagOntologyPublication(
  ontologyByCategory: Record<DerivedTagManagedCategory, DerivedTagAuthoredCategoryOntology>,
  conceptModel: PublishedDerivedTagConceptModel,
): {
  families: DerivedTagOntologyFamily[];
  tags: DerivedTagOntologyTag[];
} {
  const flattened = Object.values(ontologyByCategory).map((ontology) => flattenDerivedTagAuthoredCategoryOntology(ontology));
  const baseFamilies = flattened.flatMap((entry) => entry.families);
  const baseTags = flattened.flatMap((entry) => entry.tags);
  const baseFamilyByKey = new Map(baseFamilies.map((family) => [tagKey(family.category, family.family), family]));
  const baseTagByKey = new Map(baseTags.map((tag) => [tagKey(tag.category, tag.tag), tag]));

  const families = new Map<string, DerivedTagOntologyFamily>();
  const tags: DerivedTagOntologyTag[] = [];

  for (const projection of conceptModel.projections) {
    const sourceTag = baseTagByKey.get(tagKey(projection.category, projection.currentTag));
    const sourceFamily = baseFamilyByKey.get(tagKey(projection.category, projection.family));
    const translation = conceptModel.translationsByTagKey.get(
      tagKey(projection.category, projection.currentTag),
    );
    const concept = conceptModel.conceptById.get(projection.conceptId);

    if (!sourceTag || !translation || !concept) {
      throw new Error(
        `Missing source ontology pieces for published projection ${projection.category}:${projection.family}:${projection.currentTag}.`,
      );
    }

    const familyKey = tagKey(projection.category, projection.family);
    if (!families.has(familyKey)) {
      const fallbackFamily = sourceFamily ?? baseFamilyByKey.get(tagKey(sourceTag.category, sourceTag.family));
      families.set(familyKey, {
        category: projection.category as DerivedTagOntologyFamily["category"],
        family: projection.family,
        label: fallbackFamily?.label ?? projection.family,
        axis: projection.axis,
        subcategories: projection.subcategories ?? fallbackFamily?.subcategories,
        description: fallbackFamily?.description ?? sourceTag.description,
        variantInheritance: fallbackFamily?.variantInheritance,
      });
    }

    tags.push({
      ...sourceTag,
      family: projection.family,
      label: concept.label,
      description: projection.description,
      canonicalConceptId: concept.id,
      translationStatus: projection.translationStatus,
      schemaKind: concept.schemaKind,
      domainId: concept.domainId,
      operation: concept.operation,
      primaryFacetKind: concept.primaryFacetKind,
      primaryFacetValue: concept.primaryFacetValue,
      secondaryFacets: concept.secondaryFacets,
      renameNote: translation.renameNote,
      translationNotes: translation.notes,
    });
  }

  return {
    families: [...families.values()].sort(
      (left, right) =>
        left.category.localeCompare(right.category) ||
        left.axis.localeCompare(right.axis) ||
        left.family.localeCompare(right.family),
    ),
    tags: tags.sort(
      (left, right) =>
        left.category.localeCompare(right.category) ||
        left.family.localeCompare(right.family) ||
        left.tag.localeCompare(right.tag),
    ),
  };
}
