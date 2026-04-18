import type {
  DerivedTagExemplarCategory,
  DerivedTagExemplarRecord,
  SearchCategory,
  SearchSubcategory,
} from "../../types.js";
import { uniqueSorted } from "../../utils.js";
import type { PublishedDerivedTagOntology } from "./catalog-utils.js";
import { normalizeDerivedTag } from "./shared.js";

type ExemplarKey = `${SearchCategory}:${string}`;

export type PublishedDerivedTagExemplarSet = {
  category: SearchCategory;
  tag: string;
  positives: DerivedTagExemplarRecord[];
  negatives: DerivedTagExemplarRecord[];
  notes?: string;
};

export type PublishedDerivedTagExemplars = {
  categories: DerivedTagExemplarCategory[];
  exemplars: PublishedDerivedTagExemplarSet[];
  exemplarsByKey: Map<ExemplarKey, PublishedDerivedTagExemplarSet>;
};

type DerivedTagRecordSummary = {
  recordKey: string;
  name: string;
  category: SearchCategory;
};

function exemplarKey(category: SearchCategory, tag: string): ExemplarKey {
  return `${category}:${normalizeDerivedTag(tag)}`;
}

function dedupeExemplarRecords(
  records: DerivedTagExemplarRecord[] | undefined,
  bucketName: "positives" | "negatives",
  tag: string,
  category: SearchCategory,
): DerivedTagExemplarRecord[] {
  const seen = new Set<string>();
  const deduped: DerivedTagExemplarRecord[] = [];

  for (const record of records ?? []) {
    if (seen.has(record.recordKey)) {
      throw new Error(
        `Derived tag exemplar "${normalizeDerivedTag(tag)}" in category "${category}" repeats record "${record.recordKey}" in ${bucketName}.`,
      );
    }
    seen.add(record.recordKey);
    deduped.push(record);
  }

  return deduped;
}

export function publishDerivedTagExemplars(
  ontology: PublishedDerivedTagOntology,
  categories: DerivedTagExemplarCategory[],
): PublishedDerivedTagExemplars {
  const exemplars: PublishedDerivedTagExemplarSet[] = [];
  const exemplarsByKey = new Map<ExemplarKey, PublishedDerivedTagExemplarSet>();

  for (const category of categories) {
    for (const exemplarSet of category.exemplars) {
      const normalizedTag = normalizeDerivedTag(exemplarSet.tag);
      const ontologyTag = ontology.tagByKey.get(exemplarKey(category.category, normalizedTag));
      if (!ontologyTag) {
        throw new Error(
          `Derived tag exemplar "${normalizedTag}" in category "${category.category}" does not exist in the published ontology.`,
        );
      }

      const positives = dedupeExemplarRecords(exemplarSet.positives, "positives", normalizedTag, category.category);
      const negatives = dedupeExemplarRecords(exemplarSet.negatives, "negatives", normalizedTag, category.category);
      const positiveRecordKeys = new Set(positives.map((record) => record.recordKey));
      const conflictingNegative = negatives.find((record) => positiveRecordKeys.has(record.recordKey));
      if (conflictingNegative) {
        throw new Error(
          `Derived tag exemplar "${normalizedTag}" in category "${category.category}" lists "${conflictingNegative.recordKey}" as both positive and negative.`,
        );
      }

      const key = exemplarKey(category.category, normalizedTag);
      if (exemplarsByKey.has(key)) {
        throw new Error(`Duplicate derived tag exemplar "${normalizedTag}" in category "${category.category}".`);
      }

      const publishedSet: PublishedDerivedTagExemplarSet = {
        category: category.category,
        tag: normalizedTag,
        positives,
        negatives,
        notes: exemplarSet.notes,
      };
      exemplars.push(publishedSet);
      exemplarsByKey.set(key, publishedSet);
    }
  }

  return {
    categories,
    exemplars,
    exemplarsByKey,
  };
}

export function getPublishedDerivedTagExemplars(
  exemplars: PublishedDerivedTagExemplars,
  tag: string,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): PublishedDerivedTagExemplarSet[] {
  const normalizedTag = normalizeDerivedTag(tag);

  return exemplars.exemplars
    .filter((entry) => entry.tag === normalizedTag)
    .filter((entry) => !scope.category || entry.category === scope.category);
}

export function resolveDerivedTagExemplarRecordKeys(
  exemplars: PublishedDerivedTagExemplars,
  tag: string,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): string[] {
  return uniqueSorted(
    getPublishedDerivedTagExemplars(exemplars, tag, scope).flatMap((entry) =>
      entry.positives.map((record) => record.recordKey),
    ),
  );
}

export function validateDerivedTagExemplarsAgainstRecords(
  records: Iterable<DerivedTagRecordSummary>,
  exemplars: PublishedDerivedTagExemplars,
): void {
  const recordsByKey = new Map<string, DerivedTagRecordSummary>();

  for (const record of records) {
    recordsByKey.set(record.recordKey, record);
  }

  for (const exemplarSet of exemplars.exemplars) {
    for (const record of [...exemplarSet.positives, ...exemplarSet.negatives]) {
      const actualRecord = recordsByKey.get(record.recordKey);
      if (!actualRecord) {
        continue;
      }
      if (actualRecord.category !== exemplarSet.category) {
        throw new Error(
          `Derived tag exemplar "${record.recordKey}" for "${exemplarSet.tag}" expected category "${exemplarSet.category}" but found "${actualRecord.category}".`,
        );
      }
      if (actualRecord.name !== record.name) {
        throw new Error(
          `Derived tag exemplar "${record.recordKey}" for "${exemplarSet.tag}" expected name "${record.name}" but found "${actualRecord.name}".`,
        );
      }
    }
  }
}
