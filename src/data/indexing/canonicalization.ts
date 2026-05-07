import { deriveRecordTags, getVariantInheritableTags } from "../../tags/runtime.js";
import {
  buildDerivedAfflictionArtifacts,
  type DerivedAfflictionBuild,
  type DerivedBuildEntry,
} from "../derived-afflictions.js";
import type { IndexedBuildSourceEntry } from "../index-types.js";
import { applyVariantBaseTagInheritance } from "../variant-tag-inheritance.js";
import {
  createResolutionProgressCounter,
  reportDerivedTagResolutionProgress,
  reportDerivedTagResolutionStarted,
  reportReferenceResolutionSummary,
  type IndexingProgressReporter,
} from "./progress.js";
import type { ReferenceResolutionStageResult } from "./reference-resolution.js";
import { cloneNormalizedIndexRecord } from "./stage-artifacts.js";

export type CanonicalizationStageResult = {
  canonicalEntries: IndexedBuildSourceEntry[];
  derivedAfflictions: DerivedAfflictionBuild;
};

export function canonicalizeIndexRecords(
  indexedEntries: IndexedBuildSourceEntry[],
  references: ReferenceResolutionStageResult,
  progress: IndexingProgressReporter = {},
): CanonicalizationStageResult {
  const derivedAfflictions = buildDerivedAfflictionArtifacts(indexedEntries);
  const canonicalDerivedAfflictions = derivedAfflictions.records.filter((entry) => entry.isSearchCanonical);
  const totalResolutionRecords = indexedEntries.length + canonicalDerivedAfflictions.length;
  const resolutionProgress = createResolutionProgressCounter(totalResolutionRecords);
  let resolvedRecordCount = 0;

  reportDerivedTagResolutionStarted(progress, { totalRecords: totalResolutionRecords });

  const canonicalEntries = indexedEntries.map((entry) => {
    const derivedTags = deriveRecordTags({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      category: entry.record.category,
      subcategory: entry.record.subcategory,
      descriptionText: entry.record.descriptionText,
      blurbText: entry.record.blurbText,
      traits: entry.record.traits,
      families: entry.record.families,
      references: entry.resolvedReferences.map((reference) => ({
        recordKey: reference.targetRecordKey,
        packName: reference.targetRecord.packName,
        name: reference.targetRecord.name,
        category: reference.targetRecord.category,
        subcategory: reference.targetRecord.subcategory,
        traits: reference.targetRecord.traits,
      })),
    });

    resolvedRecordCount += 1;
    if (resolutionProgress.shouldReport(resolvedRecordCount)) {
      reportDerivedTagResolutionProgress(progress, {
        resolvedRecords: resolvedRecordCount,
        totalRecords: totalResolutionRecords,
      });
    }

    return {
      ...entry,
      record: {
        ...cloneNormalizedIndexRecord(entry.record),
        derivedTags,
      },
      resolvedReferences: [...entry.resolvedReferences],
    };
  });

  const canonicalDerivedRecords = derivedAfflictions.records.map((entry): DerivedBuildEntry => {
    if (!entry.isSearchCanonical) {
      return {
        ...entry,
        record: cloneNormalizedIndexRecord(entry.record),
        references: [],
        resolvedReferences: [],
      };
    }

    const derivedTags = deriveRecordTags({
      recordKey: entry.record.recordKey,
      name: entry.record.name,
      category: entry.record.category,
      subcategory: entry.record.subcategory,
      // Canonical derived afflictions can have empty descriptions even when their
      // linked staged-condition text is preserved in searchText.
      descriptionText: entry.record.descriptionText ?? entry.record.searchText,
      blurbText: entry.record.blurbText,
      traits: entry.record.traits,
      families: entry.record.families,
      references: [],
    });

    resolvedRecordCount += 1;
    if (resolutionProgress.shouldReport(resolvedRecordCount)) {
      reportDerivedTagResolutionProgress(progress, {
        resolvedRecords: resolvedRecordCount,
        totalRecords: totalResolutionRecords,
      });
    }

    return {
      ...entry,
      record: {
        ...cloneNormalizedIndexRecord(entry.record),
        derivedTags,
      },
      references: [],
      resolvedReferences: [],
    };
  });

  const creatureVariantInheritableTags = getVariantInheritableTags({ category: "creature" });
  if (creatureVariantInheritableTags.length > 0) {
    applyVariantBaseTagInheritance(
      canonicalEntries.map((entry) => entry.record),
      creatureVariantInheritableTags,
    );
  }

  reportReferenceResolutionSummary(progress, {
    aliasCount: references.aliasRows.length,
    legacyLinkCount: references.legacyLinkRows.length,
    canonicalDerivedAfflictionCount: canonicalDerivedAfflictions.length,
  });

  return {
    canonicalEntries,
    derivedAfflictions: {
      records: canonicalDerivedRecords,
      edges: [...derivedAfflictions.edges],
    },
  };
}
