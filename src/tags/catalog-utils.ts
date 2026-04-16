import type {
  DerivedTagCatalogEntry,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
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

type OntologyFamilyKey = `${SearchCategory}:${string}`;
type OntologyTagKey = `${SearchCategory}:${string}`;

type OntologySeedDefinition = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  recordKeys: string[];
};

type OntologySeedAssignment = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type DerivedTagSeedIndex = {
  assignmentsByRecordKey: Map<string, OntologySeedAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, OntologySeedAssignment[]>;
  seedDefinitionsByTag: Map<string, OntologySeedDefinition[]>;
};

export type PublishedDerivedTagOntology = {
  families: DerivedTagOntologyFamily[];
  tags: DerivedTagOntologyTag[];
  familyByKey: Map<OntologyFamilyKey, DerivedTagOntologyFamily>;
  tagByKey: Map<OntologyTagKey, DerivedTagOntologyTag>;
  tagsByFamilyKey: Map<OntologyFamilyKey, DerivedTagOntologyTag[]>;
};

function familyKey(
  category: SearchCategory,
  family: string,
): OntologyFamilyKey {
  return `${category}:${normalizeDerivedTag(family)}`;
}

function tagKey(
  category: SearchCategory,
  tag: string,
): OntologyTagKey {
  return `${category}:${normalizeDerivedTag(tag)}`;
}

function familyAppliesToSubcategory(
  family: Pick<DerivedTagOntologyFamily, "subcategories">,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!family.subcategories || family.subcategories.length === 0) {
    return true;
  }

  return subcategory !== null && family.subcategories.includes(subcategory);
}

function assignmentAppliesToContext(
  assignment: OntologySeedAssignment,
  input: Pick<DerivedTagContext, "category" | "subcategory">,
): boolean {
  if (assignment.category !== input.category) {
    return false;
  }

  return familyAppliesToSubcategory(assignment, input.subcategory);
}

function definitionAppliesToScope(
  definition: OntologySeedDefinition,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null },
): boolean {
  if (scope.category && definition.category !== scope.category) {
    return false;
  }

  if (scope.subcategory !== undefined && !familyAppliesToSubcategory(definition, scope.subcategory)) {
    return false;
  }

  return true;
}

function normalizeSeedReference(pack: string, name: string): string {
  return `${normalizeText(pack)}:${normalizeText(name)}`;
}

function pushAssignment(
  bucket: Map<string, OntologySeedAssignment[]>,
  recordKey: string,
  assignment: OntologySeedAssignment,
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
  bucket: Map<string, OntologySeedDefinition[]>,
  definition: OntologySeedDefinition,
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
  seedRecords: DerivedTagOntologyTag["seedRecords"],
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

function buildTagsByFamilyKey(
  tags: DerivedTagOntologyTag[],
): Map<OntologyFamilyKey, DerivedTagOntologyTag[]> {
  const tagsByFamilyKey = new Map<OntologyFamilyKey, DerivedTagOntologyTag[]>();

  for (const tag of tags) {
    const key = familyKey(tag.category, tag.family);
    const current = tagsByFamilyKey.get(key) ?? [];
    current.push(tag);
    tagsByFamilyKey.set(key, current);
  }

  return tagsByFamilyKey;
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

export function publishDerivedTagOntology(
  families: DerivedTagOntologyFamily[],
  tags: DerivedTagOntologyTag[],
): PublishedDerivedTagOntology {
  const familyByKey = new Map<OntologyFamilyKey, DerivedTagOntologyFamily>();
  const tagByKey = new Map<OntologyTagKey, DerivedTagOntologyTag>();

  for (const family of families) {
    const normalizedFamilyKey = familyKey(family.category, family.family);
    if (familyByKey.has(normalizedFamilyKey)) {
      throw new Error(
        `Duplicate derived tag family "${normalizeDerivedTag(family.family)}" in category "${family.category}".`,
      );
    }
    familyByKey.set(normalizedFamilyKey, family);
  }

  for (const tag of tags) {
    const normalizedTagKey = tagKey(tag.category, tag.tag);
    if (!tag.assignmentMode) {
      throw new Error(`Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${tag.category}" is missing assignmentMode.`);
    }

    const normalizedFamilyKey = familyKey(tag.category, tag.family);
    if (!familyByKey.has(normalizedFamilyKey)) {
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${tag.category}" references unknown family "${normalizeDerivedTag(tag.family)}".`,
      );
    }
    if (tagByKey.has(normalizedTagKey)) {
      const existing = tagByKey.get(normalizedTagKey)!;
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${tag.category}" belongs to both "${normalizeDerivedTag(existing.family)}" and "${normalizeDerivedTag(tag.family)}".`,
      );
    }
    tagByKey.set(normalizedTagKey, tag);
  }

  for (const tag of tags) {
    for (const adjacentTag of tag.adjacentTags ?? []) {
      if (!tagByKey.has(tagKey(tag.category, adjacentTag))) {
        throw new Error(
          `Derived tag "${tag.tag}" in category "${tag.category}" references unknown adjacent tag "${adjacentTag}".`,
        );
      }
    }
    for (const childTag of tag.compositeOfAnyTags ?? []) {
      if (!tagByKey.has(tagKey(tag.category, childTag))) {
        throw new Error(
          `Derived tag "${tag.tag}" in category "${tag.category}" references unknown composite child "${childTag}".`,
        );
      }
    }
  }

  return {
    families,
    tags,
    familyByKey,
    tagByKey,
    tagsByFamilyKey: buildTagsByFamilyKey(tags),
  };
}

export function groupDerivedTagOntology(
  ontology: Pick<PublishedDerivedTagOntology, "families" | "tags"> & Partial<Pick<PublishedDerivedTagOntology, "tagsByFamilyKey">>,
): DerivedTagCatalogEntry[] {
  const tagsByFamilyKey = ontology.tagsByFamilyKey ?? buildTagsByFamilyKey(ontology.tags);

  return ontology.families
    .map((family) => {
      const key = familyKey(family.category, family.family);
      const tags = tagsByFamilyKey.get(key) ?? [];

      return {
        category: family.category,
        subcategories: family.subcategories,
        family: family.family,
        description: family.description,
        variantInheritance: family.variantInheritance,
        tags: tags.map((tag) => ({
          value: tag.tag,
          description: tag.description,
          assignmentMode: tag.assignmentMode,
          nativeOntologyPolicy: tag.nativeOntologyPolicy,
          appliesWhen: tag.appliesWhen,
          doesNotApplyWhen: tag.doesNotApplyWhen,
          positiveSignals: tag.positiveSignals,
          negativeSignals: tag.negativeSignals,
          adjacentTags: tag.adjacentTags,
          compositeOfAnyTags: tag.compositeOfAnyTags,
          seedRecords: tag.seedRecords,
          excludeSeedRecords: tag.excludeSeedRecords,
          variantInheritance: tag.variantInheritance,
        })),
      };
    })
    .filter((entry) => entry.tags.length > 0);
}

export function buildDerivedTagSeedIndex(
  ontology: PublishedDerivedTagOntology,
  seedLookup: DerivedTagSeedLookup,
): DerivedTagSeedIndex {
  const assignmentsByRecordKey = new Map<string, OntologySeedAssignment[]>();
  const excludedAssignmentsByRecordKey = new Map<string, OntologySeedAssignment[]>();
  const seedDefinitionsByTag = new Map<string, OntologySeedDefinition[]>();

  for (const tag of ontology.tags) {
    const normalizedTag = normalizeDerivedTag(tag.tag);
    const family = ontology.familyByKey.get(familyKey(tag.category, tag.family));
    if (!family) {
      throw new Error(
        `Derived tag "${normalizedTag}" in category "${tag.category}" references unknown family "${normalizeDerivedTag(tag.family)}".`,
      );
    }

    const normalizedRecordKeys = resolveSeedRecordKeys(seedLookup, tag.seedRecords, "seedRecords", normalizedTag);
    const excludedRecordKeys = resolveSeedRecordKeys(seedLookup, tag.excludeSeedRecords, "excludeSeedRecords", normalizedTag);

    if (normalizedRecordKeys.length > 0) {
      const definition: OntologySeedDefinition = {
        tag: normalizedTag,
        category: tag.category,
        subcategories: family.subcategories,
        recordKeys: normalizedRecordKeys,
      };
      pushSeedDefinition(seedDefinitionsByTag, definition);

      for (const recordKey of normalizedRecordKeys) {
        pushAssignment(assignmentsByRecordKey, recordKey, {
          tag: normalizedTag,
          category: tag.category,
          subcategories: family.subcategories,
        });
      }
    }

    for (const recordKey of excludedRecordKeys) {
      pushAssignment(excludedAssignmentsByRecordKey, recordKey, {
        tag: normalizedTag,
        category: tag.category,
        subcategories: family.subcategories,
      });
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
  ontology: PublishedDerivedTagOntology,
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

  let appliedComposite = true;
  while (appliedComposite) {
    appliedComposite = false;

    for (const tag of ontology.tags) {
      if (tag.category !== input.category) {
        continue;
      }

      const family = ontology.familyByKey.get(familyKey(tag.category, tag.family));
      if (!family || !familyAppliesToSubcategory(family, input.subcategory)) {
        continue;
      }

      if (!tag.compositeOfAnyTags || tag.compositeOfAnyTags.length === 0) {
        continue;
      }

      const compositeSources = tag.compositeOfAnyTags.reduce<Set<PrimaryDerivedTagSource> | undefined>((currentSource, childTag) => {
        const childSource = sourceSets.get(normalizeDerivedTag(childTag));
        if (!childSource) {
          return currentSource;
        }
        const merged = currentSource ?? new Set<PrimaryDerivedTagSource>();
        for (const source of childSource) {
          merged.add(source);
        }
        return merged;
      }, undefined);

      if (!compositeSources || compositeSources.size === 0) {
        continue;
      }

      const normalizedTag = normalizeDerivedTag(tag.tag);
      const existingSource = sourceSets.get(normalizedTag);
      const existingSignature = existingSource ? normalizeSourceSet(existingSource) : null;
      addSourceSet(sourceSets, normalizedTag, compositeSources);
      const updatedSource = sourceSets.get(normalizedTag);
      const updatedSignature = updatedSource ? normalizeSourceSet(updatedSource) : null;
      if (updatedSignature !== existingSignature) {
        appliedComposite = true;
      }
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
