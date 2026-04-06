import type {
  DerivedTagCatalogEntry,
  DerivedTagCatalogTag,
  DerivedTagSeedRecordResolution,
  SearchCategory,
  SearchSubcategory,
} from "../types.js";
import { normalizeText, uniqueSorted } from "../utils.js";
import type { DerivedTagContext } from "./matcher.js";
import { normalizeDerivedTag } from "./shared.js";

export type DerivedTagSource = "rule" | "seed" | "both";

export type DerivedTagDerivation = {
  tags: string[];
  sources: Map<string, DerivedTagSource>;
};

export type DerivedTagSeedLookup = Map<string, string[]>;

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

function normalizeSeedReference(pack: string, name: string): string {
  return `${normalizeText(pack)}:${normalizeText(name)}`;
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

function resolveSeedRecordKeys(
  seedLookup: DerivedTagSeedLookup,
  seedRecords: DerivedTagCatalogTag["seedRecords"],
  fieldName: "seedRecords" | "excludeSeedRecords",
  tagValue: string,
): string[] {
  const resolvedRecordKeys: string[] = [];

  for (const seedRecord of seedRecords ?? []) {
    const lookupKey = normalizeSeedReference(seedRecord.pack, seedRecord.name);
    const matches = uniqueSorted(seedLookup.get(lookupKey) ?? []);

    if (matches.length === 0) {
      throw new Error(
        `Derived tag ${fieldName} entry "${seedRecord.pack}:${seedRecord.name}" for "${tagValue}" did not resolve to a canonical record key.`,
      );
    }

    if (matches.length > 1) {
      throw new Error(
        `Derived tag ${fieldName} entry "${seedRecord.pack}:${seedRecord.name}" for "${tagValue}" resolved ambiguously to ${matches.length} record keys.`,
      );
    }

    resolvedRecordKeys.push(matches[0]!);
  }

  return uniqueSorted(resolvedRecordKeys);
}

export function buildDerivedTagSeedLookup(
  resolutions: DerivedTagSeedRecordResolution[],
): DerivedTagSeedLookup {
  const lookup = new Map<string, string[]>();

  for (const resolution of resolutions) {
    const key = normalizeSeedReference(resolution.pack, resolution.name);
    const current = lookup.get(key) ?? [];
    current.push(resolution.recordKey);
    lookup.set(key, uniqueSorted(current));
  }

  return lookup;
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
  seedLookup: DerivedTagSeedLookup,
): DerivedTagSeedIndex {
  const assignmentsByRecordKey = new Map<string, CatalogSeedAssignment[]>();
  const excludedAssignmentsByRecordKey = new Map<string, CatalogSeedAssignment[]>();
  const seedDefinitionsByTag = new Map<string, CatalogSeedDefinition[]>();

  for (const entry of catalog) {
    for (const tag of entry.tags) {
      const normalizedTag = normalizeDerivedTag(tag.value);
      const normalizedRecordKeys = resolveSeedRecordKeys(seedLookup, tag.seedRecords, "seedRecords", normalizedTag);
      const excludedRecordKeys = resolveSeedRecordKeys(seedLookup, tag.excludeSeedRecords, "excludeSeedRecords", normalizedTag);

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
