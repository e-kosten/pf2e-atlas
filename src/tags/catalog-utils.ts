import type {
  DerivedTagCatalogEntry,
  DerivedTagLegacySeedMigrationCategory,
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

type PrimaryDerivedTagSource = "rule" | "seed" | "seed_migration" | "assignment";
export type DerivedTagSource =
  | "rule"
  | "seed"
  | "seed_migration"
  | "assignment"
  | "rule+seed"
  | "rule+seed_migration"
  | "rule+assignment"
  | "seed+seed_migration"
  | "seed+assignment"
  | "seed_migration+assignment"
  | "rule+seed+seed_migration"
  | "rule+seed+assignment"
  | "rule+seed_migration+assignment"
  | "seed+seed_migration+assignment"
  | "rule+seed+seed_migration+assignment";

export type DerivedTagDerivation = {
  tags: string[];
  sources: Map<string, DerivedTagSource>;
};

export type DerivedTagSeedLookup = Map<string, string[]>;

type OntologyFamilyKey = `${SearchCategory}:${string}`;
type OntologyTagKey = `${SearchCategory}:${string}`;

type TagOwnedRecordDefinition = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  recordKeys: string[];
};

type TagOwnedRecordAssignment = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type DerivedTagSeedIndex = {
  assignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  seedDefinitionsByTag: Map<string, TagOwnedRecordDefinition[]>;
};

export type DerivedTagLegacySeedMigrationDefinition = TagOwnedRecordDefinition;

export type DerivedTagLegacySeedMigrationIndex = {
  assignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  definitionsByTag: Map<string, DerivedTagLegacySeedMigrationDefinition[]>;
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
  assignment: TagOwnedRecordAssignment,
  input: Pick<DerivedTagContext, "category" | "subcategory">,
): boolean {
  if (assignment.category !== input.category) {
    return false;
  }

  return familyAppliesToSubcategory(assignment, input.subcategory);
}

function definitionAppliesToScope(
  definition: TagOwnedRecordDefinition,
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
  bucket: Map<string, TagOwnedRecordAssignment[]>,
  recordKey: string,
  assignment: TagOwnedRecordAssignment,
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
  bucket: Map<string, TagOwnedRecordDefinition[]>,
  definition: TagOwnedRecordDefinition,
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

const PRIMARY_SOURCE_ORDER: PrimaryDerivedTagSource[] = [
  "rule",
  "seed",
  "seed_migration",
  "assignment",
];

function normalizeSourceSet(sourceSet: Set<PrimaryDerivedTagSource>): DerivedTagSource {
  return PRIMARY_SOURCE_ORDER
    .filter((source) => sourceSet.has(source))
    .join("+") as DerivedTagSource;
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
  fieldName: string,
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

type TagOwnedRecordIndex = {
  assignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  definitionsByTag: Map<string, TagOwnedRecordDefinition[]>;
};

type TagOwnedRecordConfig = {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  includeRecords?: DerivedTagOntologyTag["seedRecords"];
  excludeRecords?: DerivedTagOntologyTag["excludeSeedRecords"];
  includeFieldName: string;
  excludeFieldName: string;
};

function buildTagOwnedRecordIndex(
  records: TagOwnedRecordConfig[],
  seedLookup: DerivedTagSeedLookup,
): TagOwnedRecordIndex {
  const assignmentsByRecordKey = new Map<string, TagOwnedRecordAssignment[]>();
  const excludedAssignmentsByRecordKey = new Map<string, TagOwnedRecordAssignment[]>();
  const definitionsByTag = new Map<string, TagOwnedRecordDefinition[]>();

  for (const record of records) {
    const normalizedTag = normalizeDerivedTag(record.tag);
    const normalizedRecordKeys = resolveSeedRecordKeys(
      seedLookup,
      record.includeRecords,
      record.includeFieldName,
      normalizedTag,
    );
    const excludedRecordKeys = resolveSeedRecordKeys(
      seedLookup,
      record.excludeRecords,
      record.excludeFieldName,
      normalizedTag,
    );

    if (normalizedRecordKeys.length > 0) {
      const definition: TagOwnedRecordDefinition = {
        tag: normalizedTag,
        category: record.category,
        subcategories: record.subcategories,
        recordKeys: normalizedRecordKeys,
      };
      pushSeedDefinition(definitionsByTag, definition);

      for (const recordKey of normalizedRecordKeys) {
        pushAssignment(assignmentsByRecordKey, recordKey, {
          tag: normalizedTag,
          category: record.category,
          subcategories: record.subcategories,
        });
      }
    }

    for (const recordKey of excludedRecordKeys) {
      pushAssignment(excludedAssignmentsByRecordKey, recordKey, {
        tag: normalizedTag,
        category: record.category,
        subcategories: record.subcategories,
      });
    }
  }

  return {
    assignmentsByRecordKey,
    excludedAssignmentsByRecordKey,
    definitionsByTag,
  };
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
  const seedRecords = ontology.tags.map((tag) => {
    const normalizedTag = normalizeDerivedTag(tag.tag);
    const family = ontology.familyByKey.get(familyKey(tag.category, tag.family));
    if (!family) {
      throw new Error(
        `Derived tag "${normalizedTag}" in category "${tag.category}" references unknown family "${normalizeDerivedTag(tag.family)}".`,
      );
    }

    return {
      tag: normalizedTag,
      category: tag.category,
      subcategories: family.subcategories,
      includeRecords: tag.seedRecords,
      excludeRecords: tag.excludeSeedRecords,
      includeFieldName: "seedRecords",
      excludeFieldName: "excludeSeedRecords",
    };
  });

  const index = buildTagOwnedRecordIndex(seedRecords, seedLookup);
  return {
    assignmentsByRecordKey: index.assignmentsByRecordKey,
    excludedAssignmentsByRecordKey: index.excludedAssignmentsByRecordKey,
    seedDefinitionsByTag: index.definitionsByTag,
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

export function buildDerivedTagLegacySeedMigrationIndex(
  ontology: PublishedDerivedTagOntology,
  seedLookup: DerivedTagSeedLookup,
  migrations: DerivedTagLegacySeedMigrationCategory[],
): DerivedTagLegacySeedMigrationIndex {
  const authoredTags = new Set<string>();
  const migrationRecords: TagOwnedRecordConfig[] = [];

  for (const migrationCategory of migrations) {
    for (const migrationTag of migrationCategory.tags) {
      const normalizedTag = normalizeDerivedTag(migrationTag.tag);
      const authoredKey = tagKey(migrationCategory.category, normalizedTag);
      if (authoredTags.has(authoredKey)) {
        throw new Error(
          `Duplicate legacy seed migration tag "${normalizedTag}" in category "${migrationCategory.category}".`,
        );
      }
      authoredTags.add(authoredKey);

      const ontologyTag = ontology.tagByKey.get(authoredKey);
      if (!ontologyTag) {
        throw new Error(
          `Legacy seed migration tag "${normalizedTag}" in category "${migrationCategory.category}" does not exist in the published ontology.`,
        );
      }

      const family = ontology.familyByKey.get(familyKey(ontologyTag.category, ontologyTag.family));
      if (!family) {
        throw new Error(
          `Legacy seed migration tag "${normalizedTag}" in category "${migrationCategory.category}" references unknown family "${normalizeDerivedTag(ontologyTag.family)}".`,
        );
      }

      migrationRecords.push({
        tag: normalizedTag,
        category: migrationCategory.category,
        subcategories: family.subcategories,
        includeRecords: migrationTag.includeRecords,
        excludeRecords: migrationTag.excludeRecords,
        includeFieldName: "includeRecords",
        excludeFieldName: "excludeRecords",
      });
    }
  }

  const index = buildTagOwnedRecordIndex(migrationRecords, seedLookup);
  return {
    assignmentsByRecordKey: index.assignmentsByRecordKey,
    excludedAssignmentsByRecordKey: index.excludedAssignmentsByRecordKey,
    definitionsByTag: index.definitionsByTag,
  };
}

export function listConfiguredDerivedTagLegacySeedMigrations(
  migrationIndex: DerivedTagLegacySeedMigrationIndex,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): DerivedTagLegacySeedMigrationDefinition[] {
  return uniqueSorted([...migrationIndex.definitionsByTag.keys()])
    .flatMap((tag) => migrationIndex.definitionsByTag.get(tag) ?? [])
    .filter((definition) => definitionAppliesToScope(definition, scope))
    .sort((left, right) =>
      left.category.localeCompare(right.category)
      || left.tag.localeCompare(right.tag)
      || JSON.stringify(left.subcategories ?? []).localeCompare(JSON.stringify(right.subcategories ?? [])));
}

export function resolveLegacySeedMigrationRecordKeys(
  migrationIndex: DerivedTagLegacySeedMigrationIndex,
  tag: string,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): string[] {
  const normalizedTag = normalizeDerivedTag(tag);
  return uniqueSorted((migrationIndex.definitionsByTag.get(normalizedTag) ?? [])
    .filter((definition) => definitionAppliesToScope(definition, scope))
    .flatMap((definition) => definition.recordKeys));
}

export function deriveCatalogTagDerivation(
  ontology: PublishedDerivedTagOntology,
  seedIndex: DerivedTagSeedIndex,
  input: Pick<DerivedTagContext, "recordKey" | "category" | "subcategory">,
  ruleTags: string[],
  explicitAssignmentIndex?: DerivedTagExplicitAssignmentIndex,
  legacySeedMigrationIndex?: DerivedTagLegacySeedMigrationIndex,
): DerivedTagDerivation {
  const sourceSets = new Map<string, Set<PrimaryDerivedTagSource>>();

  for (const tag of ruleTags) {
    addPrimarySource(sourceSets, tag, "rule");
  }

  if (input.recordKey) {
    const blockedSeedTags = new Set(
      (seedIndex.excludedAssignmentsByRecordKey.get(input.recordKey) ?? [])
        .filter((assignment) => assignmentAppliesToContext(assignment, input))
        .map((assignment) => assignment.tag),
    );
    for (const assignment of seedIndex.assignmentsByRecordKey.get(input.recordKey) ?? []) {
      if (!assignmentAppliesToContext(assignment, input) || blockedSeedTags.has(assignment.tag)) {
        continue;
      }
      addPrimarySource(sourceSets, assignment.tag, "seed");
    }

    const blockedMigrationTags = new Set(
      (legacySeedMigrationIndex?.excludedAssignmentsByRecordKey.get(input.recordKey) ?? [])
        .filter((assignment) => assignmentAppliesToContext(assignment, input))
        .map((assignment) => assignment.tag),
    );
    for (const assignment of legacySeedMigrationIndex?.assignmentsByRecordKey.get(input.recordKey) ?? []) {
      if (!assignmentAppliesToContext(assignment, input) || blockedMigrationTags.has(assignment.tag)) {
        continue;
      }
      addPrimarySource(sourceSets, assignment.tag, "seed_migration");
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
