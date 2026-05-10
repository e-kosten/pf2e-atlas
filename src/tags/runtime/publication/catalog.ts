import type {
  DerivedTagCatalogEntry,
  PublishedDerivedTagConceptModel,
  DerivedTagSeedRecordReference,
  DerivedTagLegacySeedMigrationCategory,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
  DerivedTagSeedRecordResolution,
  SearchCategory,
  SearchSubcategory,
} from "../../../domain/derived-tag-types.js";
import { normalizeText, uniqueSorted } from "../../../shared/utils.js";
import type { DerivedTagExplicitAssignmentIndex } from "../derivation/assignments.js";
import type { DerivedTagContext } from "../matcher/engine.js";
import { normalizeDerivedTag } from "../matcher/engine.js";

type PrimaryDerivedTagSource = "authored_rule" | "legacy_rule" | "seed_migration" | "assignment";
export type DerivedTagSource =
  | "seed_migration"
  | "assignment"
  | "authored_rule"
  | "legacy_rule"
  | "authored_rule+legacy_rule"
  | "authored_rule+seed_migration"
  | "authored_rule+assignment"
  | "legacy_rule+seed_migration"
  | "legacy_rule+assignment"
  | "seed_migration+assignment"
  | "authored_rule+legacy_rule+seed_migration"
  | "authored_rule+legacy_rule+assignment"
  | "authored_rule+seed_migration+assignment"
  | "legacy_rule+seed_migration+assignment"
  | "authored_rule+legacy_rule+seed_migration+assignment";

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

export type DerivedTagLegacySeedMigrationDefinition = TagOwnedRecordDefinition;

export type DerivedTagLegacySeedMigrationIndex = {
  assignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  excludedAssignmentsByRecordKey: Map<string, TagOwnedRecordAssignment[]>;
  definitionsByTag: Map<string, DerivedTagLegacySeedMigrationDefinition[]>;
};

export type PublishedDerivedTagOntology = {
  families: DerivedTagOntologyFamily[];
  tags: DerivedTagOntologyTag[];
  conceptModel: PublishedDerivedTagConceptModel;
  familyByKey: Map<OntologyFamilyKey, DerivedTagOntologyFamily>;
  tagByKey: Map<OntologyTagKey, DerivedTagOntologyTag>;
  tagsByFamilyKey: Map<OntologyFamilyKey, DerivedTagOntologyTag[]>;
};

function familyKey(category: SearchCategory, family: string): OntologyFamilyKey {
  return `${category}:${normalizeDerivedTag(family)}`;
}

function tagKey(category: SearchCategory, tag: string): OntologyTagKey {
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
  if (
    !current.some(
      (candidate) =>
        candidate.tag === assignment.tag &&
        candidate.category === assignment.category &&
        JSON.stringify(candidate.subcategories ?? []) === JSON.stringify(assignment.subcategories ?? []),
    )
  ) {
    current.push(assignment);
    bucket.set(recordKey, current);
  }
}

function pushSeedDefinition(
  bucket: Map<string, TagOwnedRecordDefinition[]>,
  definition: TagOwnedRecordDefinition,
): void {
  const current = bucket.get(definition.tag) ?? [];
  const existing = current.find(
    (candidate) =>
      candidate.category === definition.category &&
      JSON.stringify(candidate.subcategories ?? []) === JSON.stringify(definition.subcategories ?? []),
  );

  if (!existing) {
    current.push(definition);
    bucket.set(definition.tag, current);
    return;
  }

  existing.recordKeys = uniqueSorted([...existing.recordKeys, ...definition.recordKeys]);
}

const PRIMARY_SOURCE_ORDER: PrimaryDerivedTagSource[] = [
  "authored_rule",
  "legacy_rule",
  "seed_migration",
  "assignment",
];

function normalizeSourceSet(sourceSet: Set<PrimaryDerivedTagSource>): DerivedTagSource {
  return PRIMARY_SOURCE_ORDER.filter((source) => sourceSet.has(source)).join("+") as DerivedTagSource;
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

function resolveRecordReferences(
  seedLookup: DerivedTagSeedLookup,
  recordReferences: DerivedTagSeedRecordReference[] | undefined,
  fieldName: string,
  tagValue: string,
): string[] {
  const resolvedRecordKeys: string[] = [];

  for (const recordReference of recordReferences ?? []) {
    const lookupKey = normalizeSeedReference(recordReference.pack, recordReference.name);
    const matches = uniqueSorted(seedLookup.get(lookupKey) ?? []);

    if (matches.length === 0) {
      throw new Error(
        `Derived tag ${fieldName} entry "${recordReference.pack}:${recordReference.name}" for "${tagValue}" did not resolve to a canonical record key.`,
      );
    }

    if (matches.length > 1) {
      throw new Error(
        `Derived tag ${fieldName} entry "${recordReference.pack}:${recordReference.name}" for "${tagValue}" resolved ambiguously to ${matches.length} record keys.`,
      );
    }

    resolvedRecordKeys.push(matches[0]!);
  }

  return uniqueSorted(resolvedRecordKeys);
}

function buildTagsByFamilyKey(tags: DerivedTagOntologyTag[]): Map<OntologyFamilyKey, DerivedTagOntologyTag[]> {
  const tagsByFamilyKey = new Map<OntologyFamilyKey, DerivedTagOntologyTag[]>();

  for (const tag of tags) {
    const key = familyKey(tag.category, tag.family);
    const current = tagsByFamilyKey.get(key) ?? [];
    current.push(tag);
    tagsByFamilyKey.set(key, current);
  }

  return tagsByFamilyKey;
}

export function buildDerivedTagSeedLookup(resolutions: DerivedTagSeedRecordResolution[]): DerivedTagSeedLookup {
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
  includeRecords?: DerivedTagSeedRecordReference[];
  excludeRecords?: DerivedTagSeedRecordReference[];
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
    const normalizedRecordKeys = resolveRecordReferences(
      seedLookup,
      record.includeRecords,
      record.includeFieldName,
      normalizedTag,
    );
    const excludedRecordKeys = resolveRecordReferences(
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

export function buildVisibleDerivedTagOntology(ontology: PublishedDerivedTagOntology): PublishedDerivedTagOntology {
  const visibleTagKeys = new Set(
    ontology.tags
      .filter((tag) => tag.translationStatus === "mapped")
      .map((tag) => tagKey(tag.category, tag.tag)),
  );
  const visibleTags = ontology.tags
    .filter((tag) => visibleTagKeys.has(tagKey(tag.category, tag.tag)))
    .map((tag) => ({
      ...tag,
      adjacentTags: tag.adjacentTags?.filter((adjacentTag) => visibleTagKeys.has(tagKey(tag.category, adjacentTag))),
      compositeOfAnyTags: tag.compositeOfAnyTags?.filter((childTag) => visibleTagKeys.has(tagKey(tag.category, childTag))),
    }));
  const visibleFamilyKeys = new Set(
    visibleTags.map((tag) => familyKey(tag.category, tag.family)),
  );
  const visibleFamilies = ontology.families.filter((family) =>
    visibleFamilyKeys.has(familyKey(family.category, family.family)),
  );
  const visibleProjectionKeys = new Set(visibleTags.map((tag) => `${tag.category}:${tag.tag}`));
  const visibleTranslations = ontology.conceptModel.translations.filter(
    (translation) =>
      translation.translationStatus === "mapped" &&
      visibleProjectionKeys.has(`${translation.currentCategory}:${translation.currentTag}`),
  );
  const visibleConceptIds = new Set(visibleTranslations.map((translation) => translation.canonicalConceptId));
  const visibleConcepts = ontology.conceptModel.concepts.filter((concept) => visibleConceptIds.has(concept.id));
  const visibleProjections = ontology.conceptModel.projections.filter((projection) =>
    visibleProjectionKeys.has(`${projection.category}:${projection.currentTag}`),
  );
  const visibleRelations = ontology.conceptModel.relations.filter(
    (relation) => visibleConceptIds.has(relation.fromConceptId) && visibleConceptIds.has(relation.toConceptId),
  );

  return publishDerivedTagOntology(visibleFamilies, visibleTags, {
    concepts: visibleConcepts,
    conceptById: new Map(visibleConcepts.map((concept) => [concept.id, concept])),
    projections: visibleProjections,
    projectionsById: new Map(visibleProjections.map((projection) => [projection.id, projection])),
    projectionsByTagKey: new Map(
      visibleProjections.map((projection) => [`${projection.category}:${projection.currentTag}`, projection] as const),
    ),
    translations: visibleTranslations,
    translationsByTagKey: new Map(
      visibleTranslations.map((translation) => [
        `${translation.currentCategory}:${translation.currentTag}`,
        translation,
      ] as const),
    ),
    relations: visibleRelations,
  });
}

export function publishDerivedTagOntology(
  families: DerivedTagOntologyFamily[],
  tags: DerivedTagOntologyTag[],
  conceptModel: PublishedDerivedTagConceptModel,
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
    conceptModel,
    familyByKey,
    tagByKey,
    tagsByFamilyKey: buildTagsByFamilyKey(tags),
  };
}

export function groupDerivedTagOntology(
  ontology: Pick<PublishedDerivedTagOntology, "families" | "tags"> &
    Partial<Pick<PublishedDerivedTagOntology, "tagsByFamilyKey">>,
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
        label: family.label,
        axis: family.axis,
        description: family.description,
        variantInheritance: family.variantInheritance,
        tags: tags.map((tag) => ({
          value: tag.tag,
          label: tag.label,
          description: tag.description,
          assignmentMode: tag.assignmentMode,
          nativeOntologyPolicy: tag.nativeOntologyPolicy,
          appliesWhen: tag.appliesWhen,
          doesNotApplyWhen: tag.doesNotApplyWhen,
          positiveSignals: tag.positiveSignals,
          negativeSignals: tag.negativeSignals,
          adjacentTags: tag.adjacentTags,
          compositeOfAnyTags: tag.compositeOfAnyTags,
          variantInheritance: tag.variantInheritance,
          canonicalConceptId: tag.canonicalConceptId,
          translationStatus: tag.translationStatus,
          schemaKind: tag.schemaKind,
          domainId: tag.domainId,
          operation: tag.operation,
          primaryFacetKind: tag.primaryFacetKind,
          primaryFacetValue: tag.primaryFacetValue,
          secondaryFacets: tag.secondaryFacets,
        })),
      };
    })
    .filter((entry) => entry.tags.length > 0);
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
    .sort(
      (left, right) =>
        left.category.localeCompare(right.category) ||
        left.tag.localeCompare(right.tag) ||
        JSON.stringify(left.subcategories ?? []).localeCompare(JSON.stringify(right.subcategories ?? [])),
    );
}

export function resolveLegacySeedMigrationRecordKeys(
  migrationIndex: DerivedTagLegacySeedMigrationIndex,
  tag: string,
  scope: { category?: SearchCategory; subcategory?: SearchSubcategory | null } = {},
): string[] {
  const normalizedTag = normalizeDerivedTag(tag);
  return uniqueSorted(
    (migrationIndex.definitionsByTag.get(normalizedTag) ?? [])
      .filter((definition) => definitionAppliesToScope(definition, scope))
      .flatMap((definition) => definition.recordKeys),
  );
}

export function deriveCatalogTagDerivation(
  ontology: PublishedDerivedTagOntology,
  input: Pick<DerivedTagContext, "recordKey" | "category" | "subcategory">,
  ruleTags:
    | string[]
    | {
        authoredRuleTags?: string[];
        legacyRuleTags?: string[];
      },
  explicitAssignmentIndex?: DerivedTagExplicitAssignmentIndex,
  legacySeedMigrationIndex?: DerivedTagLegacySeedMigrationIndex,
): DerivedTagDerivation {
  const sourceSets = new Map<string, Set<PrimaryDerivedTagSource>>();
  const authoredRuleTags = Array.isArray(ruleTags) ? [] : (ruleTags.authoredRuleTags ?? []);
  const legacyRuleTags = Array.isArray(ruleTags) ? ruleTags : (ruleTags.legacyRuleTags ?? []);

  for (const tag of authoredRuleTags) {
    addPrimarySource(sourceSets, tag, "authored_rule");
  }

  for (const tag of legacyRuleTags) {
    addPrimarySource(sourceSets, tag, "legacy_rule");
  }

  if (input.recordKey) {
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
    if (explicitAssignment && (!explicitAssignment.category || explicitAssignment.category === input.category)) {
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

      const compositeSources = tag.compositeOfAnyTags.reduce<Set<PrimaryDerivedTagSource> | undefined>(
        (currentSource, childTag) => {
          const childSource = sourceSets.get(normalizeDerivedTag(childTag));
          if (!childSource) {
            return currentSource;
          }
          const merged = currentSource ?? new Set<PrimaryDerivedTagSource>();
          for (const source of childSource) {
            merged.add(source);
          }
          return merged;
        },
        undefined,
      );

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
