import { normalizeDerivedTag } from "../tags/index.js";
import { uniqueSorted } from "../utils.js";
import type { NormalizedIndexRecord } from "./index-types.js";

type VariantBaseResolution = {
  baseRecordKey: string | null;
  baseTags: string[];
};

function normalizedTags(tags: string[]): string[] {
  return uniqueSorted(tags.map((tag) => normalizeDerivedTag(tag)));
}

function intersectTags(tagSets: string[][]): string[] {
  if (tagSets.length === 0) {
    return [];
  }

  let intersection = new Set(tagSets[0]);
  for (const tagSet of tagSets.slice(1)) {
    const current = new Set(tagSet);
    intersection = new Set([...intersection].filter((tag) => current.has(tag)));
  }

  return uniqueSorted([...intersection]);
}

function resolveVariantBaseTags(
  records: NormalizedIndexRecord[],
  directTagsByRecordKey: Map<string, string[]>,
  inheritableTags: Set<string>,
): Map<string, VariantBaseResolution> {
  const recordsByFamily = new Map<string, NormalizedIndexRecord[]>();

  for (const record of records) {
    if (record.category !== "creature" || !record.variantFamilyKey) {
      continue;
    }

    const bucket = recordsByFamily.get(record.variantFamilyKey) ?? [];
    bucket.push(record);
    recordsByFamily.set(record.variantFamilyKey, bucket);
  }

  const baseByFamily = new Map<string, VariantBaseResolution>();

  for (const [familyKey, familyRecords] of recordsByFamily) {
    const explicitBase = familyRecords.find((record) =>
      record.variantLabel === null &&
      record.variantBaseName !== null &&
      record.name === record.variantBaseName);

    if (explicitBase) {
      const baseTags = (directTagsByRecordKey.get(explicitBase.recordKey) ?? [])
        .filter((tag) => inheritableTags.has(tag));
      baseByFamily.set(familyKey, {
        baseRecordKey: explicitBase.recordKey,
        baseTags,
      });
      continue;
    }

    const nonUniqueSiblingTags = familyRecords
      .filter((record) => !record.isUnique)
      .map((record) => (directTagsByRecordKey.get(record.recordKey) ?? [])
        .filter((tag) => inheritableTags.has(tag)));

    // Many creature families have only labeled forms like age brackets, so fall back
    // to the stable inheritable tag intersection across standard non-unique siblings.
    if (nonUniqueSiblingTags.length < 2) {
      continue;
    }

    baseByFamily.set(familyKey, {
      baseRecordKey: null,
      baseTags: intersectTags(nonUniqueSiblingTags),
    });
  }

  return baseByFamily;
}

export function applyVariantBaseTagInheritance(
  records: NormalizedIndexRecord[],
  inheritableTagsInput: string[],
): Map<string, string[]> {
  const inheritableTags = new Set(normalizedTags(inheritableTagsInput));
  if (inheritableTags.size === 0) {
    return new Map();
  }

  const directTagsByRecordKey = new Map(records.map((record) => [
    record.recordKey,
    normalizedTags(record.derivedTags),
  ]));
  const baseByFamily = resolveVariantBaseTags(records, directTagsByRecordKey, inheritableTags);
  const inheritedTagsByRecordKey = new Map<string, string[]>();

  for (const record of records) {
    if (record.category !== "creature" || !record.variantFamilyKey) {
      continue;
    }

    const resolvedBase = baseByFamily.get(record.variantFamilyKey);
    if (!resolvedBase || resolvedBase.baseTags.length === 0) {
      continue;
    }
    if (resolvedBase.baseRecordKey === record.recordKey) {
      continue;
    }

    const directTags = directTagsByRecordKey.get(record.recordKey) ?? [];
    const inheritedTags = resolvedBase.baseTags.filter((tag) => !directTags.includes(tag));
    if (inheritedTags.length === 0) {
      continue;
    }

    record.derivedTags = uniqueSorted([...record.derivedTags, ...inheritedTags]);
    inheritedTagsByRecordKey.set(record.recordKey, inheritedTags);
  }

  return inheritedTagsByRecordKey;
}
