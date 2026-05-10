import type {
  DerivedTagCanonicalConcept,
  DerivedTagCategoryProjection,
  DerivedTagLegacySeedMigrationCategory,
  DerivedTagTranslationMapping,
  DerivedTagTranslationRecord,
  SearchCategory,
} from "../../domain/derived-tag-types.js";
import {
  DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID,
  DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY,
} from "../canonical/registry.js";
import type { PublishedDerivedTagOntology } from "../runtime/publication/catalog.js";
import { normalizeDerivedTag } from "../runtime/matcher/engine.js";
import { getCurrentDerivedTagFamilyTranslationDefault, getCurrentDerivedTagTranslationOverride } from "./state.js";
import { DERIVED_TAG_BASE_LEGACY_TRANSLATIONS } from "./base-mappings.js";
import { applyDerivedTagTranslationOverride } from "./record-utils.js";
import type { DerivedTagTranslationOverride } from "./tag-overrides.js";

function translationKey(category: SearchCategory, tag: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(tag)}`;
}

function familyKey(category: SearchCategory, family: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(family)}`;
}

function applyFamilyDefaults(
  row: DerivedTagTranslationMapping,
  defaults: ReturnType<typeof getCurrentDerivedTagFamilyTranslationDefault>,
): DerivedTagTranslationMapping {
  if (!defaults) {
    return row;
  }
  if (row.translationStatus === "dropped") {
    return row;
  }
  return {
    ...row,
    translationStatus: defaults.translationStatus,
    ...(defaults.notes ? { notes: [row.notes, defaults.notes].filter(Boolean).join(" ") } : {}),
  };
}

const CANONICAL_PROJECTIONS_BY_ID = new Map<string, DerivedTagCategoryProjection>(
  Object.values(DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY).flatMap((categoryProjections) =>
    Object.values(categoryProjections).map((projection) => [projection.id, projection] as const),
  ),
);

function resolveTargetProjection(
  mapping: DerivedTagTranslationMapping,
): DerivedTagCategoryProjection | undefined {
  return mapping.targetProjectionId ? CANONICAL_PROJECTIONS_BY_ID.get(mapping.targetProjectionId) : undefined;
}

function resolveTargetConcept(
  projection: DerivedTagCategoryProjection | undefined,
): DerivedTagCanonicalConcept | undefined {
  return projection ? DERIVED_TAG_CANONICAL_CONCEPTS_BY_ID[projection.conceptId] : undefined;
}

function hydratePublishedTranslationRecord(
  mapping: DerivedTagTranslationMapping,
): DerivedTagTranslationRecord {
  const targetProjection = resolveTargetProjection(mapping);
  const targetConcept = resolveTargetConcept(targetProjection);

  return {
    currentCategory: mapping.source.currentCategory,
    currentBrowseAxis: mapping.source.currentBrowseAxis,
    currentFamily: mapping.source.currentFamily,
    currentTag: mapping.source.currentTag,
    currentAssignmentMode: mapping.source.currentAssignmentMode,
    translationStatus: mapping.translationStatus,
    canonicalConceptId: targetConcept?.id ?? "",
    canonicalConceptLabel: targetProjection?.label ?? targetConcept?.label ?? "",
    schemaKind: targetConcept?.schemaKind ?? "descriptive",
    ...(targetConcept?.domainId !== undefined ? { domainId: targetConcept.domainId } : {}),
    ...(targetConcept?.operation !== undefined ? { operation: targetConcept.operation } : {}),
    ...(targetConcept?.primaryFacetKind !== undefined ? { primaryFacetKind: targetConcept.primaryFacetKind } : {}),
    ...(targetConcept?.primaryFacetValue !== undefined ? { primaryFacetValue: targetConcept.primaryFacetValue } : {}),
    ...(targetConcept?.secondaryFacets !== undefined ? { secondaryFacets: targetConcept.secondaryFacets } : {}),
    projectionAxis: targetProjection?.axis ?? mapping.source.currentBrowseAxis,
    projectionFamily: targetProjection?.family ?? mapping.source.currentFamily,
    ...(mapping.targetProjectionId ? { targetProjectionId: mapping.targetProjectionId } : {}),
    ...(mapping.renameNote ? { renameNote: mapping.renameNote } : {}),
    ...(mapping.notes ? { notes: mapping.notes } : {}),
  };
}

function buildTranslationMappingFromPublishedRecord(
  record: DerivedTagTranslationRecord,
): DerivedTagTranslationMapping {
  return {
    source: {
      currentAssignmentMode: record.currentAssignmentMode,
      currentBrowseAxis: record.currentBrowseAxis,
      currentCategory: record.currentCategory,
      currentFamily: record.currentFamily,
      currentTag: record.currentTag,
    },
    ...(record.targetProjectionId ? { targetProjectionId: record.targetProjectionId } : {}),
    translationStatus: record.translationStatus,
    ...(record.renameNote ? { renameNote: record.renameNote } : {}),
    ...(record.notes ? { notes: record.notes } : {}),
  };
}

export function buildEffectiveDerivedTagTranslationRecord(
  base: DerivedTagTranslationRecord,
  override: DerivedTagTranslationOverride | undefined,
): DerivedTagTranslationRecord {
  const mapping = buildTranslationMappingFromPublishedRecord(base);
  return hydratePublishedTranslationRecord(applyDerivedTagTranslationOverride(mapping, override));
}

export function buildBaseDerivedTagTranslations(): DerivedTagTranslationRecord[] {
  return DERIVED_TAG_BASE_LEGACY_TRANSLATIONS.map((row) => hydratePublishedTranslationRecord(structuredClone(row))).sort(
    (left, right) =>
      left.currentCategory.localeCompare(right.currentCategory) ||
      left.currentBrowseAxis.localeCompare(right.currentBrowseAxis) ||
      left.currentFamily.localeCompare(right.currentFamily) ||
      left.currentTag.localeCompare(right.currentTag),
  );
}

export function buildBaseDerivedTagTranslationsByKey(): Map<`${SearchCategory}:${string}`, DerivedTagTranslationRecord> {
  return new Map(
    buildBaseDerivedTagTranslations().map((translation) => [
      translationKey(translation.currentCategory, translation.currentTag),
      translation,
    ] as const),
  );
}

export function buildPublishedDerivedTagTranslations(): DerivedTagTranslationRecord[] {
  return buildBaseDerivedTagTranslations().map((baseRow): DerivedTagTranslationRecord => {
    let row: DerivedTagTranslationMapping = buildTranslationMappingFromPublishedRecord(baseRow);
    row = applyFamilyDefaults(
      row,
      getCurrentDerivedTagFamilyTranslationDefault(
        familyKey(row.source.currentCategory, row.source.currentFamily),
      ),
    );
    row = applyDerivedTagTranslationOverride(
      row,
      getCurrentDerivedTagTranslationOverride(translationKey(row.source.currentCategory, row.source.currentTag)),
    );
    return hydratePublishedTranslationRecord(row);
  }).sort(
    (left, right) =>
      left.currentCategory.localeCompare(right.currentCategory) ||
      left.currentBrowseAxis.localeCompare(right.currentBrowseAxis) ||
      left.currentFamily.localeCompare(right.currentFamily) ||
      left.currentTag.localeCompare(right.currentTag),
  );
}

export function buildPublishedDerivedTagTranslationsByKey(): Map<`${SearchCategory}:${string}`, DerivedTagTranslationRecord> {
  return new Map(
    buildPublishedDerivedTagTranslations().map((translation) => [
      translationKey(translation.currentCategory, translation.currentTag),
      translation,
    ] as const),
  );
}

export function translateLegacyDerivedTags(
  category: SearchCategory,
  legacyTags: string[],
  ontology: PublishedDerivedTagOntology,
  translations: ReadonlyMap<`${SearchCategory}:${string}`, DerivedTagTranslationRecord>,
): string[] {
  const mappedTags = new Set<string>();
  for (const legacyTag of legacyTags) {
    const translation = translations.get(translationKey(category, legacyTag));
    if (
      !translation ||
      translation.translationStatus === "unmapped" ||
      translation.translationStatus === "dropped" ||
      !translation.targetProjectionId
    ) {
      continue;
    }
    const targetProjection = ontology.conceptModel.projectionsById.get(translation.targetProjectionId);
    if (!targetProjection || targetProjection.category !== category) {
      continue;
    }
    const normalizedTag = normalizeDerivedTag(targetProjection.currentTag);
    if (!ontology.tagByKey.has(translationKey(targetProjection.category, normalizedTag))) {
      continue;
    }
    mappedTags.add(normalizedTag);
  }
  return [...mappedTags].sort((left, right) => left.localeCompare(right));
}

export function translateLegacySeedMigrationCategories(
  migrations: DerivedTagLegacySeedMigrationCategory[],
  ontology: PublishedDerivedTagOntology,
  translations: ReadonlyMap<`${SearchCategory}:${string}`, DerivedTagTranslationRecord>,
): DerivedTagLegacySeedMigrationCategory[] {
  type LegacySeedTag = DerivedTagLegacySeedMigrationCategory["tags"][number];
  type LegacySeedCategoryBucket = {
    category: SearchCategory;
    tags: Map<string, LegacySeedTag>;
  };
  const translated = new Map<string, LegacySeedCategoryBucket>();

  for (const migration of migrations) {
    const categoryBucket =
      translated.get(migration.category) ?? { category: migration.category, tags: new Map<string, LegacySeedTag>() };

    for (const tag of migration.tags) {
      const [mappedTag] = translateLegacyDerivedTags(
        migration.category,
        [tag.tag],
        ontology,
        translations,
      );
      if (!mappedTag) {
        continue;
      }

      const existing: LegacySeedTag = categoryBucket.tags.get(mappedTag) ?? {
        tag: mappedTag,
        includeRecords: [],
        excludeRecords: [],
      };
      existing.includeRecords = [...(existing.includeRecords ?? []), ...(tag.includeRecords ?? [])];
      existing.excludeRecords = [...(existing.excludeRecords ?? []), ...(tag.excludeRecords ?? [])];
      categoryBucket.tags.set(mappedTag, existing);
    }

    translated.set(migration.category, categoryBucket);
  }

  return [...translated.values()]
    .map((categoryBucket) => ({
      category: categoryBucket.category,
      tags: [...categoryBucket.tags.values()].sort((left, right) => left.tag.localeCompare(right.tag)),
    }))
    .filter((categoryBucket) => categoryBucket.tags.length > 0);
}
