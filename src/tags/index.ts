import { deriveRecordTagsFromRules } from "./matcher.js";
import { normalizeDerivedTag, DerivedTagContext } from "./shared.js";
import { uniqueSorted } from "../utils.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  buildDerivedTagSeedIndex,
  deriveCatalogTagDerivation,
  listConfiguredDerivedTagLegacySeedMigrations,
  publishDerivedTagOntology,
  resolveCatalogSeedRecordKeys,
  type DerivedTagLegacySeedMigrationDefinition,
  type PublishedDerivedTagOntology,
  type DerivedTagDerivation,
} from "./catalog-utils.js";
import { DERIVED_TAG_SEED_LOOKUP } from "./catalog-seed-records.js";
import { CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "./legacy-seed-migrations/creature.js";
import {
  AFFLICTION_DERIVED_TAG_ONTOLOGY,
} from "./ontology/affliction.js";
import {
  CREATURE_DERIVED_TAG_ONTOLOGY,
} from "./ontology/creature.js";
import {
  EQUIPMENT_DERIVED_TAG_ONTOLOGY,
} from "./ontology/equipment.js";
import {
  HAZARD_DERIVED_TAG_ONTOLOGY,
} from "./ontology/hazard.js";
import {
  SPELL_DERIVED_TAG_ONTOLOGY,
} from "./ontology/spell.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "./ontology/utils.js";
import {
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "./assignments.js";
import { AFFLICTION_DERIVED_TAG_RULES } from "./rules/affliction.js";
import { CREATURE_DERIVED_TAG_RULES } from "./rules/creature.js";
import { EQUIPMENT_DERIVED_TAG_RULES } from "./rules/equipment.js";
import { HAZARD_DERIVED_TAG_RULES } from "./rules/hazard.js";
import { SPELL_DERIVED_TAG_RULES } from "./rules/spell.js";

export { normalizeDerivedTag } from "./shared.js";

const DERIVED_TAG_RULES = [
  ...EQUIPMENT_DERIVED_TAG_RULES,
  ...SPELL_DERIVED_TAG_RULES,
  ...HAZARD_DERIVED_TAG_RULES,
  ...AFFLICTION_DERIVED_TAG_RULES,
  ...CREATURE_DERIVED_TAG_RULES,
];

const AUTHORED_DERIVED_TAG_ONTOLOGIES = [
  EQUIPMENT_DERIVED_TAG_ONTOLOGY,
  SPELL_DERIVED_TAG_ONTOLOGY,
  HAZARD_DERIVED_TAG_ONTOLOGY,
  AFFLICTION_DERIVED_TAG_ONTOLOGY,
  CREATURE_DERIVED_TAG_ONTOLOGY,
];
const FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES = AUTHORED_DERIVED_TAG_ONTOLOGIES
  .map((ontology) => flattenDerivedTagAuthoredCategoryOntology(ontology));
const AUTHORED_DERIVED_TAG_ONTOLOGY_FAMILIES = FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES
  .flatMap((ontology) => ontology.families);
const AUTHORED_DERIVED_TAG_ONTOLOGY_TAGS = FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES
  .flatMap((ontology) => ontology.tags);

const DERIVED_TAG_ONTOLOGY: PublishedDerivedTagOntology = publishDerivedTagOntology(
  AUTHORED_DERIVED_TAG_ONTOLOGY_FAMILIES,
  AUTHORED_DERIVED_TAG_ONTOLOGY_TAGS,
);
export const DERIVED_TAG_ONTOLOGY_FAMILIES = DERIVED_TAG_ONTOLOGY.families;
export const DERIVED_TAG_ONTOLOGY_TAGS = DERIVED_TAG_ONTOLOGY.tags;
const DERIVED_TAG_SEED_INDEX = buildDerivedTagSeedIndex(DERIVED_TAG_ONTOLOGY, DERIVED_TAG_SEED_LOOKUP);
const DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX = buildDerivedTagLegacySeedMigrationIndex(
  DERIVED_TAG_ONTOLOGY,
  DERIVED_TAG_SEED_LOOKUP,
  [CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS],
);
const DERIVED_TAG_EXPLICIT_ASSIGNMENT_INDEX = createDerivedTagExplicitAssignmentIndex(DERIVED_TAG_ONTOLOGY);

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagDerivation(input).tags;
}

export function deriveRecordTagDerivation(
  input: DerivedTagContext,
): DerivedTagDerivation {
  const ruleTags = deriveRecordTagsFromRules(DERIVED_TAG_RULES, input);
  return deriveCatalogTagDerivation(
    DERIVED_TAG_ONTOLOGY,
    DERIVED_TAG_SEED_INDEX,
    input,
    ruleTags,
    DERIVED_TAG_EXPLICIT_ASSIGNMENT_INDEX,
    DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX,
  );
}

export function getDerivedTagSeedRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveCatalogSeedRecordKeys(DERIVED_TAG_SEED_INDEX, normalizeDerivedTag(tag), scope);
}

export function listDerivedTagLegacySeedMigrations(
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): DerivedTagLegacySeedMigrationDefinition[] {
  return listConfiguredDerivedTagLegacySeedMigrations(DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX, scope);
}

export function getDerivedTagFamilyTags(
  family: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  const normalizedFamily = normalizeDerivedTag(family);
  const tags = new Set<string>();

  for (const ontologyFamily of DERIVED_TAG_ONTOLOGY.families) {
    if (normalizeDerivedTag(ontologyFamily.family) !== normalizedFamily) {
      continue;
    }
    if (scope.category && ontologyFamily.category !== scope.category) {
      continue;
    }
    if (
      scope.subcategory !== undefined &&
      ontologyFamily.subcategories &&
      (scope.subcategory === null || !ontologyFamily.subcategories.includes(scope.subcategory))
    ) {
      continue;
    }
    const familyKey = `${ontologyFamily.category}:${normalizeDerivedTag(ontologyFamily.family)}` as const;
    for (const tag of DERIVED_TAG_ONTOLOGY.tagsByFamilyKey.get(familyKey) ?? []) {
      tags.add(normalizeDerivedTag(tag.tag));
    }
  }

  const resolved = uniqueSorted([...tags]);
  if (resolved.length === 0) {
    const renderedScope = scope.category
      ? scope.subcategory
        ? `${scope.category}/${scope.subcategory}`
        : scope.category
      : "all categories";
    throw new Error(`No derived tag family "${normalizedFamily}" matched scope "${renderedScope}".`);
  }

  return resolved;
}

export function getVariantInheritableTags(
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  const tags = new Set<string>();

  for (const tag of DERIVED_TAG_ONTOLOGY.tags) {
    if (scope.category && tag.category !== scope.category) {
      continue;
    }
    const familyKey = `${tag.category}:${normalizeDerivedTag(tag.family)}` as const;
    const family = DERIVED_TAG_ONTOLOGY.familyByKey.get(familyKey);
    if (!family) {
      continue;
    }
    if (
      scope.subcategory !== undefined &&
      family.subcategories &&
      (scope.subcategory === null || !family.subcategories.includes(scope.subcategory))
    ) {
      continue;
    }
    if (tag.variantInheritance ?? family.variantInheritance ?? false) {
      tags.add(normalizeDerivedTag(tag.tag));
    }
  }

  return uniqueSorted([...tags]);
}

export function validateConfiguredDerivedTagAssignments(
  records: Iterable<{ recordKey: string; name: string; category: DerivedTagContext["category"] }>,
): void {
  validateDerivedTagExplicitAssignmentsAgainstRecords(records, DERIVED_TAG_EXPLICIT_ASSIGNMENT_INDEX);
}
