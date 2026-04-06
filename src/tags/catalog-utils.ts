import type {
  DerivedTagCatalogEntry,
  DerivedTagCatalogTag,
  SearchCategory,
  SearchSubcategory,
} from "../types.js";
import { uniqueSorted } from "../utils.js";
import type { DerivedTagContext } from "./matcher.js";
import { normalizeDerivedTag } from "./shared.js";

export type DerivedTagSource = "rule" | "seed" | "both";

export type DerivedTagDerivation = {
  tags: string[];
  sources: Map<string, DerivedTagSource>;
};

type CatalogSeedDefinition = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  recordKeys: string[];
};

type CatalogSeedAssignment = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type DerivedTagSeedIndex = {
  assignmentsByRecordKey: Map<string, CatalogSeedAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, CatalogSeedAssignment[]>;
  seedDefinitionsByTag: Map<string, CatalogSeedDefinition[]>;
};

function entryAppliesToSubcategory(
  entry: Pick<DerivedTagCatalogEntry, "subcategories">,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!entry.subcategories || entry.subcategories.length === 0) {
    return true;
  }

  return subcategory !== null && entry.subcategories.includes(subcategory);
}

function assignmentAppliesToContext(
  assignment: CatalogSeedAssignment,
  input: Pick<DerivedTagContext, "category" | "subcategory">,
): boolean {
  if (assignment.category !== input.category) {
    return false;
  }

  return entryAppliesToSubcategory(assignment, input.subcategory);
}

function definitionAppliesToScope(
  definition: CatalogSeedDefinition,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null },
): boolean {
  if (scope.category && definition.category !== scope.category) {
    return false;
  }

  if (scope.subcategory !== undefined && !entryAppliesToSubcategory(definition, scope.subcategory)) {
    return false;
  }

  return true;
}

function appendCatalogTag(
  tags: DerivedTagCatalogTag[],
  tag: DerivedTagCatalogTag,
): DerivedTagCatalogTag[] {
  const normalized = normalizeDerivedTag(tag.value);
  if (tags.some((candidate) => normalizeDerivedTag(candidate.value) === normalized)) {
    return tags;
  }

  return [...tags, tag];
}

function pushAssignment(
  bucket: Map<string, CatalogSeedAssignment[]>,
  recordKey: string,
  assignment: CatalogSeedAssignment,
): void {
  const current = bucket.get(recordKey) ?? [];
  if (!current.some((candidate) =>
    candidate.tag === assignment.tag &&
    candidate.category === assignment.category &&
    JSON.stringify(candidate.subcategories ?? []) === JSON.stringify(assignment.subcategories ?? []))) {
    current.push(assignment);
    bucket.set(recordKey, current);
  }
}

function pushSeedDefinition(
  bucket: Map<string, CatalogSeedDefinition[]>,
  definition: CatalogSeedDefinition,
): void {
  const current = bucket.get(definition.tag) ?? [];
  const existing = current.find((candidate) =>
    candidate.category === definition.category &&
    JSON.stringify(candidate.subcategories ?? []) === JSON.stringify(definition.subcategories ?? []));

  if (!existing) {
    current.push(definition);
    bucket.set(definition.tag, current);
    return;
  }

  existing.recordKeys = uniqueSorted([...existing.recordKeys, ...definition.recordKeys]);
}

function mergeSources(left: DerivedTagSource | undefined, right: DerivedTagSource): DerivedTagSource {
  if (!left || left === right) {
    return right;
  }
  return "both";
}

function addSource(
  sources: Map<string, DerivedTagSource>,
  tag: string,
  source: DerivedTagSource,
): void {
  const normalizedTag = normalizeDerivedTag(tag);
  sources.set(normalizedTag, mergeSources(sources.get(normalizedTag), source));
}

export function publishDerivedTagCatalog(
  catalog: DerivedTagCatalogEntry[],
): DerivedTagCatalogEntry[] {
  return catalog.map((entry) => {
    if (!entry.promoteFamilyToTag) {
      return entry;
    }

    return {
      ...entry,
      tags: appendCatalogTag(entry.tags, {
        value: normalizeDerivedTag(entry.family),
        description: entry.description,
      }),
    };
  });
}

export function buildDerivedTagSeedIndex(
  catalog: DerivedTagCatalogEntry[],
): DerivedTagSeedIndex {
  const assignmentsByRecordKey = new Map<string, CatalogSeedAssignment[]>();
  const excludedAssignmentsByRecordKey = new Map<string, CatalogSeedAssignment[]>();
  const seedDefinitionsByTag = new Map<string, CatalogSeedDefinition[]>();

  for (const entry of catalog) {
    for (const tag of entry.tags) {
      const normalizedTag = normalizeDerivedTag(tag.value);
      const normalizedRecordKeys = uniqueSorted((tag.seedRecordKeys ?? []).map((recordKey) => recordKey.trim()).filter(Boolean));
      const excludedRecordKeys = uniqueSorted((tag.excludeSeedRecordKeys ?? []).map((recordKey) => recordKey.trim()).filter(Boolean));

      if (normalizedRecordKeys.length > 0) {
        const definition: CatalogSeedDefinition = {
          tag: normalizedTag,
          category: entry.category,
          subcategories: entry.subcategories,
          recordKeys: normalizedRecordKeys,
        };
        pushSeedDefinition(seedDefinitionsByTag, definition);

        if (entry.promoteFamilyToTag) {
          pushSeedDefinition(seedDefinitionsByTag, {
            ...definition,
            tag: normalizeDerivedTag(entry.family),
          });
        }

        for (const recordKey of normalizedRecordKeys) {
          pushAssignment(assignmentsByRecordKey, recordKey, {
            tag: normalizedTag,
            category: entry.category,
            subcategories: entry.subcategories,
          });
        }
      }

      for (const recordKey of excludedRecordKeys) {
        pushAssignment(excludedAssignmentsByRecordKey, recordKey, {
          tag: normalizedTag,
          category: entry.category,
          subcategories: entry.subcategories,
        });
      }
    }
  }

  return {
    assignmentsByRecordKey,
    excludedAssignmentsByRecordKey,
    seedDefinitionsByTag,
  };
}

export function resolveCatalogSeedRecordKeys(
  seedIndex: DerivedTagSeedIndex,
  tag: string,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): string[] {
  const normalizedTag = normalizeDerivedTag(tag);
  return uniqueSorted((seedIndex.seedDefinitionsByTag.get(normalizedTag) ?? [])
    .filter((definition) => definitionAppliesToScope(definition, scope))
    .flatMap((definition) => definition.recordKeys));
}

export function deriveCatalogTagDerivation(
  catalog: DerivedTagCatalogEntry[],
  seedIndex: DerivedTagSeedIndex,
  input: Pick<DerivedTagContext, "recordKey" | "category" | "subcategory">,
  ruleTags: string[],
): DerivedTagDerivation {
  const sources = new Map<string, DerivedTagSource>();

  for (const tag of ruleTags) {
    addSource(sources, tag, "rule");
  }

  if (input.recordKey) {
    const blockedTags = new Set(
      (seedIndex.excludedAssignmentsByRecordKey.get(input.recordKey) ?? [])
        .filter((assignment) => assignmentAppliesToContext(assignment, input))
        .map((assignment) => assignment.tag),
    );
    for (const assignment of seedIndex.assignmentsByRecordKey.get(input.recordKey) ?? []) {
      if (!assignmentAppliesToContext(assignment, input) || blockedTags.has(assignment.tag)) {
        continue;
      }
      addSource(sources, assignment.tag, "seed");
    }
  }

  for (const entry of catalog) {
    if (!entry.promoteFamilyToTag || entry.category !== input.category || !entryAppliesToSubcategory(entry, input.subcategory)) {
      continue;
    }

    const childSource = entry.tags.reduce<DerivedTagSource | undefined>((currentSource, tag) => {
      const tagSource = sources.get(normalizeDerivedTag(tag.value));
      return tagSource ? mergeSources(currentSource, tagSource) : currentSource;
    }, undefined);

    if (childSource) {
      addSource(sources, entry.family, childSource);
    }
  }

  return {
    tags: uniqueSorted([...sources.keys()]),
    sources,
  };
}

export function applyPromotedFamilyTags(
  catalog: DerivedTagCatalogEntry[],
  input: Pick<DerivedTagContext, "category" | "subcategory">,
  tags: string[],
): string[] {
  const expanded = new Set(tags.map((tag) => normalizeDerivedTag(tag)));

  for (const entry of catalog) {
    if (!entry.promoteFamilyToTag || entry.category !== input.category) {
      continue;
    }
    if (!entryAppliesToSubcategory(entry, input.subcategory)) {
      continue;
    }

    const hasMatchingChildTag = entry.tags.some((tag) => expanded.has(normalizeDerivedTag(tag.value)));
    if (hasMatchingChildTag) {
      expanded.add(normalizeDerivedTag(entry.family));
    }
  }

  return uniqueSorted([...expanded]);
}
