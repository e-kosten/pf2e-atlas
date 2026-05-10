import type { DerivedTagTranslationMapping } from "../../domain/derived-tag-types.js";
import { DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY } from "../canonical/registry.js";

const EXTRA_LEGACY_TRANSLATIONS: DerivedTagTranslationMapping[] = [
  {
    source: {
      currentBrowseAxis: "effect",
      currentCategory: "equipment",
      currentFamily: "function",
      currentTag: "beneficial",
    },
    notes: "Too vague to carry stable retrieval value.",
    renameNote: "Drop vague tag with no canonical replacement.",
    translationStatus: "dropped",
  },
];

function projectionSort(left: DerivedTagTranslationMapping, right: DerivedTagTranslationMapping): number {
  return (
    left.source.currentCategory.localeCompare(right.source.currentCategory) ||
    left.source.currentBrowseAxis.localeCompare(right.source.currentBrowseAxis) ||
    left.source.currentFamily.localeCompare(right.source.currentFamily) ||
    left.source.currentTag.localeCompare(right.source.currentTag)
  );
}

export function buildBaseLegacyDerivedTagTranslations(): DerivedTagTranslationMapping[] {
  const projectionMappings = Object.values(DERIVED_TAG_CANONICAL_PROJECTIONS_BY_CATEGORY)
    .flatMap((categoryProjections) => Object.values(categoryProjections))
    .map(
      (projection): DerivedTagTranslationMapping => ({
        source: {
          currentBrowseAxis: projection.axis,
          currentCategory: projection.category,
          currentFamily: projection.family,
          currentTag: projection.currentTag,
        },
        targetProjectionId: projection.id,
        translationStatus: projection.translationStatus,
      }),
    );

  return [...projectionMappings, ...EXTRA_LEGACY_TRANSLATIONS].sort(projectionSort);
}

export const DERIVED_TAG_BASE_LEGACY_TRANSLATIONS = buildBaseLegacyDerivedTagTranslations();
