import { deriveRecordTagsFromRules } from "./matcher.js";
import { normalizeDerivedTag, DerivedTagContext } from "./shared.js";
import { uniqueSorted } from "../../utils.js";
import {
  AFFLICTION_AUTHORED_DERIVED_TAG_RULES,
  CREATURE_AUTHORED_DERIVED_TAG_RULES,
  EQUIPMENT_AUTHORED_DERIVED_TAG_RULES,
  HAZARD_AUTHORED_DERIVED_TAG_RULES,
  SPELL_AUTHORED_DERIVED_TAG_RULES,
  compileAuthoredDerivedTagRules,
} from "../authored-rules/index.js";
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
import {
  AFFLICTION_DERIVED_TAG_EXEMPLARS,
  CREATURE_DERIVED_TAG_EXEMPLARS,
  EQUIPMENT_DERIVED_TAG_EXEMPLARS,
  HAZARD_DERIVED_TAG_EXEMPLARS,
  SPELL_DERIVED_TAG_EXEMPLARS,
} from "../exemplars/index.js";
import { CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "../legacy-seed-migrations/creature.js";
import { HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "../legacy-seed-migrations/hazard.js";
import { SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS } from "../legacy-seed-migrations/spell.js";
import { AFFLICTION_DERIVED_TAG_ONTOLOGY } from "../ontology/affliction.js";
import { CREATURE_DERIVED_TAG_ONTOLOGY } from "../ontology/creature.js";
import { EQUIPMENT_DERIVED_TAG_ONTOLOGY } from "../ontology/equipment.js";
import { HAZARD_DERIVED_TAG_ONTOLOGY } from "../ontology/hazard.js";
import { SPELL_DERIVED_TAG_ONTOLOGY } from "../ontology/spell.js";
import { flattenDerivedTagAuthoredCategoryOntology } from "../ontology/utils.js";
import {
  createDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
} from "./assignments.js";
import { getLegacyDerivedTagFamilyAliases } from "./family-compatibility.js";
import {
  AFFLICTION_LEGACY_DERIVED_TAG_RULES,
  CREATURE_LEGACY_DERIVED_TAG_RULES,
  EQUIPMENT_LEGACY_DERIVED_TAG_RULES,
  HAZARD_LEGACY_DERIVED_TAG_RULES,
  SPELL_LEGACY_DERIVED_TAG_RULES,
} from "../legacy-rules/index.js";

export { normalizeDerivedTag } from "./shared.js";
export { groupDerivedTagOntology } from "./catalog-utils.js";

const LEGACY_DERIVED_TAG_RULES = [
  ...EQUIPMENT_LEGACY_DERIVED_TAG_RULES,
  ...SPELL_LEGACY_DERIVED_TAG_RULES,
  ...HAZARD_LEGACY_DERIVED_TAG_RULES,
  ...AFFLICTION_LEGACY_DERIVED_TAG_RULES,
  ...CREATURE_LEGACY_DERIVED_TAG_RULES,
];
const AUTHORED_DERIVED_TAG_RULES = [
  ...EQUIPMENT_AUTHORED_DERIVED_TAG_RULES,
  ...SPELL_AUTHORED_DERIVED_TAG_RULES,
  ...HAZARD_AUTHORED_DERIVED_TAG_RULES,
  ...AFFLICTION_AUTHORED_DERIVED_TAG_RULES,
  ...CREATURE_AUTHORED_DERIVED_TAG_RULES,
];

const AUTHORED_DERIVED_TAG_ONTOLOGIES = [
  EQUIPMENT_DERIVED_TAG_ONTOLOGY,
  SPELL_DERIVED_TAG_ONTOLOGY,
  HAZARD_DERIVED_TAG_ONTOLOGY,
  AFFLICTION_DERIVED_TAG_ONTOLOGY,
  CREATURE_DERIVED_TAG_ONTOLOGY,
];
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
  [
    SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
    HAZARD_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
    CREATURE_DERIVED_TAG_LEGACY_SEED_MIGRATIONS,
  ],
);
const DERIVED_TAG_EXEMPLARS: PublishedDerivedTagExemplars = publishDerivedTagExemplars(DERIVED_TAG_ONTOLOGY, [
  EQUIPMENT_DERIVED_TAG_EXEMPLARS,
  SPELL_DERIVED_TAG_EXEMPLARS,
  HAZARD_DERIVED_TAG_EXEMPLARS,
  AFFLICTION_DERIVED_TAG_EXEMPLARS,
  CREATURE_DERIVED_TAG_EXEMPLARS,
]);
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
