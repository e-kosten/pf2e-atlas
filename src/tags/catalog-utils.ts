import type {
  DerivedTagCatalogEntry,
  DerivedTagCatalogTag,
  DerivedTagSeedRecordResolution,
  SearchCategory,
  SearchSubcategory,
} from "../types.js";
import { normalizeText, uniqueSorted } from "../utils.js";
import type { DerivedTagExplicitAssignmentIndex } from "./assignments.js";
import type { DerivedTagContext } from "./matcher.js";
import { normalizeDerivedTag } from "./shared.js";

type PrimaryDerivedTagSource = "rule" | "seed" | "assignment";
export type DerivedTagSource =
  | "rule"
  | "seed"
  | "assignment"
  | "both"
  | "rule_assignment"
  | "seed_assignment"
  | "rule_seed_assignment";

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

function normalizeSourceSet(sourceSet: Set<PrimaryDerivedTagSource>): DerivedTagSource {
  if (sourceSet.size === 1) {
    return [...sourceSet][0]!;
  }
  if (sourceSet.has("rule") && sourceSet.has("seed") && sourceSet.size === 2) {
    return "both";
  }
  if (sourceSet.has("rule") && sourceSet.has("assignment") && sourceSet.size === 2) {
    return "rule_assignment";
  }
  if (sourceSet.has("seed") && sourceSet.has("assignment") && sourceSet.size === 2) {
    return "seed_assignment";
  }
  return "rule_seed_assignment";
}

function addPrimarySource(
  sources: Map<string, Set<PrimaryDerivedTagSource>>,
  tag: string,
  source: PrimaryDerivedTagSource,
): void {
  const normalizedTag = normalizeDerivedTag(tag);
  const existing = sources.get(normalizedTag) ?? new Set<PrimaryDerivedTagSource>();
  existing.add(source);
  sources.set(normalizedTag, existing);
}

function addSourceSet(
  sources: Map<string, Set<PrimaryDerivedTagSource>>,
  tag: string,
  sourceSet: Set<PrimaryDerivedTagSource>,
): void {
  const normalizedTag = normalizeDerivedTag(tag);
  const existing = sources.get(normalizedTag) ?? new Set<PrimaryDerivedTagSource>();
  for (const source of sourceSet) {
    existing.add(source);
  }
  sources.set(normalizedTag, existing);
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
  explicitAssignmentIndex?: DerivedTagExplicitAssignmentIndex,
): DerivedTagDerivation {
  const sourceSets = new Map<string, Set<PrimaryDerivedTagSource>>();

  for (const tag of ruleTags) {
    addPrimarySource(sourceSets, tag, "rule");
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
      addPrimarySource(sourceSets, assignment.tag, "seed");
    }

    const explicitAssignment = explicitAssignmentIndex?.assignmentsByRecordKey.get(input.recordKey);
    if (explicitAssignment && explicitAssignment.category === input.category) {
      for (const tag of explicitAssignment.includeTags) {
        addPrimarySource(sourceSets, tag, "assignment");
      }
      for (const tag of explicitAssignment.excludeTags) {
        sourceSets.delete(normalizeDerivedTag(tag));
      }
    }
  }

  for (const entry of catalog) {
    if (!entry.promoteFamilyToTag || entry.category !== input.category || !entryAppliesToSubcategory(entry, input.subcategory)) {
      continue;
    }

    const childSource = entry.tags.reduce<Set<PrimaryDerivedTagSource> | undefined>((currentSource, tag) => {
      const tagSource = sourceSets.get(normalizeDerivedTag(tag.value));
      if (!tagSource) {
        return currentSource;
      }
      const merged = currentSource ?? new Set<PrimaryDerivedTagSource>();
      for (const source of tagSource) {
        merged.add(source);
      }
      return merged;
    }, undefined);

    if (childSource && childSource.size > 0) {
      addSourceSet(sourceSets, entry.family, childSource);
    }
  }

  const sources = new Map<string, DerivedTagSource>();
  for (const [tag, sourceSet] of sourceSets) {
    sources.set(tag, normalizeSourceSet(sourceSet));
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
