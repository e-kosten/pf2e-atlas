import type {
  DerivedTagLegacySeedMigrationCategory,
  DerivedTagTranslationRecord,
  SearchCategory,
} from "../../domain/derived-tag-types.js";
import { normalizeDerivedTag } from "../runtime/matcher/engine.js";
import type { PublishedDerivedTagOntology } from "../runtime/publication/catalog.js";
import { getCurrentDerivedTagFamilyTranslationDefault, getCurrentDerivedTagTranslationOverride } from "./state.js";
import { DERIVED_TAG_BASE_LEGACY_TRANSLATIONS } from "./base-mappings.js";
import { applyDerivedTagTranslationOverride } from "./record-utils.js";

function translationKey(category: SearchCategory, tag: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(tag)}`;
}

function familyKey(category: SearchCategory, family: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(family)}`;
}

function applyFamilyDefaults(
  row: DerivedTagTranslationRecord,
  defaults: ReturnType<typeof getCurrentDerivedTagFamilyTranslationDefault>,
): DerivedTagTranslationRecord {
  if (!defaults) {
    return row;
  }
  const preserveExplicitRowShape =
    row.currentAssignmentMode === "composite" || row.schemaKind === "aggregate" || row.translationStatus === "dropped";
  return {
    ...row,
    ...(!preserveExplicitRowShape ? { schemaKind: defaults.schemaKind, translationStatus: defaults.translationStatus } : {}),
    ...(!preserveExplicitRowShape && defaults.primaryFacetKind !== undefined ? { primaryFacetKind: defaults.primaryFacetKind } : {}),
    ...(!preserveExplicitRowShape && defaults.primaryFacetValue !== undefined ? { primaryFacetValue: defaults.primaryFacetValue } : {}),
    ...(defaults.notes ? { notes: [row.notes, defaults.notes].filter(Boolean).join(" ") } : {}),
  };
}

export function buildPublishedDerivedTagTranslations(
  options: { includeOverrides?: boolean } = {},
): DerivedTagTranslationRecord[] {
  const includeOverrides = options.includeOverrides !== false;

  return DERIVED_TAG_BASE_LEGACY_TRANSLATIONS.map((baseRow): DerivedTagTranslationRecord => {
    let row: DerivedTagTranslationRecord = structuredClone(baseRow) as DerivedTagTranslationRecord;
    if (!includeOverrides) {
      return row;
    }

    row = applyFamilyDefaults(row, getCurrentDerivedTagFamilyTranslationDefault(familyKey(row.currentCategory, row.currentFamily)));
    row = applyDerivedTagTranslationOverride(
      row,
      getCurrentDerivedTagTranslationOverride(translationKey(row.currentCategory, row.currentTag)),
    );
    return row;
  }).sort(
    (left, right) =>
      left.currentCategory.localeCompare(right.currentCategory) ||
      left.currentBrowseAxis.localeCompare(right.currentBrowseAxis) ||
      left.currentFamily.localeCompare(right.currentFamily) ||
      left.currentTag.localeCompare(right.currentTag),
  );
}

export function buildPublishedDerivedTagTranslationsByKey(
  options: { includeOverrides?: boolean } = {},
): Map<`${SearchCategory}:${string}`, DerivedTagTranslationRecord> {
  return new Map(
    buildPublishedDerivedTagTranslations(options).map((translation) => [
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
      !translation.publishTag
    ) {
      continue;
    }
    const normalizedTag = normalizeDerivedTag(translation.currentTag);
    if (!ontology.tagByKey.has(translationKey(category, normalizedTag))) {
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
