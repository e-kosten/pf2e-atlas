import { deriveRecordTagsFromRules } from "./matcher.js";
import { normalizeDerivedTag, DerivedTagContext } from "./shared.js";
import { uniqueSorted } from "../../shared/utils.js";
import { DERIVED_TAG_REGISTRATION_CATEGORIES } from "../manifest.js";
import { DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY, compileAuthoredDerivedTagRules } from "../authored-rules/index.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  deriveCatalogTagDerivation,
  listConfiguredDerivedTagLegacySeedMigrations,
  publishDerivedTagOntology,
  resolveLegacySeedMigrationRecordKeys,
  type DerivedTagLegacySeedMigrationDefinition,
  type PublishedDerivedTagOntology,
  type DerivedTagDerivation,
} from "./catalog-utils.js";
import { DERIVED_TAG_SEED_LOOKUP } from "./catalog-seed-records.js";
import {
  publishDerivedTagExemplars,
  resolveDerivedTagExemplarRecordKeys,
  validateDerivedTagExemplarsAgainstRecords,
  type PublishedDerivedTagExemplars,
  type PublishedDerivedTagExemplarSet,
} from "./exemplar-utils.js";
import { DERIVED_TAG_EXEMPLARS_BY_CATEGORY } from "../exemplars/index.js";
import { DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY } from "../legacy-seed-migrations/index.js";
import { DERIVED_TAG_ONTOLOGY_BY_CATEGORY, flattenDerivedTagAuthoredCategoryOntology } from "../ontology/index.js";
import {
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "./assignments.js";
import { getLegacyDerivedTagFamilyAliases } from "./family-compatibility.js";
import { DERIVED_TAG_LEGACY_RULES_BY_CATEGORY } from "../legacy-rules/index.js";

export { normalizeDerivedTag } from "./shared.js";
export { groupDerivedTagOntology } from "./catalog-utils.js";

const LEGACY_DERIVED_TAG_RULES = [
  ...DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => DERIVED_TAG_LEGACY_RULES_BY_CATEGORY[category]),
];
const AUTHORED_DERIVED_TAG_RULES = [
  ...DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY[category]),
];

const AUTHORED_DERIVED_TAG_ONTOLOGIES = DERIVED_TAG_REGISTRATION_CATEGORIES.map(
  (category) => DERIVED_TAG_ONTOLOGY_BY_CATEGORY[category],
);
const FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES = AUTHORED_DERIVED_TAG_ONTOLOGIES.map((ontology) =>
  flattenDerivedTagAuthoredCategoryOntology(ontology),
);
const AUTHORED_DERIVED_TAG_ONTOLOGY_FAMILIES = FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES.flatMap(
  (ontology) => ontology.families,
);
const AUTHORED_DERIVED_TAG_ONTOLOGY_TAGS = FLATTENED_AUTHORED_DERIVED_TAG_ONTOLOGIES.flatMap(
  (ontology) => ontology.tags,
);

const DERIVED_TAG_ONTOLOGY: PublishedDerivedTagOntology = publishDerivedTagOntology(
  AUTHORED_DERIVED_TAG_ONTOLOGY_FAMILIES,
  AUTHORED_DERIVED_TAG_ONTOLOGY_TAGS,
);
export const DERIVED_TAG_ONTOLOGY_FAMILIES = DERIVED_TAG_ONTOLOGY.families;
export const DERIVED_TAG_ONTOLOGY_TAGS = DERIVED_TAG_ONTOLOGY.tags;
const COMPILED_AUTHORED_DERIVED_TAG_RULES = compileAuthoredDerivedTagRules(
  DERIVED_TAG_ONTOLOGY,
  AUTHORED_DERIVED_TAG_RULES,
);
const DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX = buildDerivedTagLegacySeedMigrationIndex(
  DERIVED_TAG_ONTOLOGY,
  DERIVED_TAG_SEED_LOOKUP,
  DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => {
    const migration = DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY[category];
    return migration ? [migration] : [];
  }),
);
const DERIVED_TAG_EXEMPLARS: PublishedDerivedTagExemplars = publishDerivedTagExemplars(
  DERIVED_TAG_ONTOLOGY,
  DERIVED_TAG_REGISTRATION_CATEGORIES.map((category) => DERIVED_TAG_EXEMPLARS_BY_CATEGORY[category]),
);
const DERIVED_TAG_EXPLICIT_ASSIGNMENT_INDEX = createDerivedTagExplicitAssignmentIndex(DERIVED_TAG_ONTOLOGY);

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagDerivation(input).tags;
}

export function deriveRecordTagDerivation(input: DerivedTagContext): DerivedTagDerivation {
  const authoredRuleTags = deriveRecordTagsFromRules(COMPILED_AUTHORED_DERIVED_TAG_RULES, input);
  const legacyRuleTags = deriveRecordTagsFromRules(LEGACY_DERIVED_TAG_RULES, input);
  return deriveCatalogTagDerivation(
    DERIVED_TAG_ONTOLOGY,
    input,
    {
      authoredRuleTags,
      legacyRuleTags,
    },
    DERIVED_TAG_EXPLICIT_ASSIGNMENT_INDEX,
    DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX,
  );
}

export function getDerivedTagExemplarRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveDerivedTagExemplarRecordKeys(DERIVED_TAG_EXEMPLARS, normalizeDerivedTag(tag), scope);
}

export function getDerivedTagExemplars(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): PublishedDerivedTagExemplarSet[] {
  return DERIVED_TAG_EXEMPLARS.exemplars
    .filter((entry) => entry.tag === normalizeDerivedTag(tag))
    .filter((entry) => !scope.category || entry.category === scope.category);
}

export function listDerivedTagLegacySeedMigrations(
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): DerivedTagLegacySeedMigrationDefinition[] {
  return listConfiguredDerivedTagLegacySeedMigrations(DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX, scope);
}

export function getDerivedTagLegacySeedMigrationRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveLegacySeedMigrationRecordKeys(DERIVED_TAG_LEGACY_SEED_MIGRATION_INDEX, normalizeDerivedTag(tag), scope);
}

export function getDerivedTagFamilyTags(
  family: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  const normalizedFamily = normalizeDerivedTag(family);
  const requestedFamilies = new Set<string>([normalizedFamily]);
  const tags = new Set<string>();

  for (const ontologyFamily of DERIVED_TAG_ONTOLOGY.families) {
    for (const alias of getLegacyDerivedTagFamilyAliases(ontologyFamily.category, normalizedFamily)) {
      requestedFamilies.add(alias);
    }

    if (!requestedFamilies.has(normalizeDerivedTag(ontologyFamily.family))) {
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
  validateDerivedTagExemplarsAgainstRecords(records, DERIVED_TAG_EXEMPLARS);
}
