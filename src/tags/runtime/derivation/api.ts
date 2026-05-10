import { uniqueSorted } from "../../../shared/utils.js";
import {
  type DerivedTagCanonicalConcept,
  type DerivedTagLegacySeedMigrationCategory,
} from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_EXEMPLARS_BY_CATEGORY } from "../../exemplars/index.js";
import { DERIVED_TAG_ASSIGNMENTS } from "../../assignments/index.js";
import { DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY } from "../../legacy-seed-migrations/index.js";
import {
  DERIVED_TAG_REGISTRATION_CATEGORIES,
} from "../../manifest.js";
import { getDerivedTagCanonicalOntology } from "../../canonical/index.js";
import {
  buildPublishedDerivedTagTranslations,
  buildPublishedDerivedTagTranslationsByKey,
} from "../../translations/index.js";
import {
  getCurrentDerivedTagFamilyTranslationDefaultsRevision,
  getCurrentDerivedTagTranslationOverridesRevision,
} from "../../translations/state.js";
import { getLegacyDerivedTagFamilyAliases } from "../compat/family-compatibility.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  validateDerivedTagExplicitAssignmentsAgainstRecords,
  type AuthoredDerivedTagAssignment,
} from "./assignments.js";
import { normalizeDerivedTag, type DerivedTagContext } from "../matcher/engine.js";
import {
  buildVisibleDerivedTagOntology,
  listConfiguredDerivedTagLegacySeedMigrations,
  resolveLegacySeedMigrationRecordKeys,
  type DerivedTagDerivation,
  type DerivedTagLegacySeedMigrationDefinition,
  type PublishedDerivedTagOntology,
} from "../publication/catalog.js";
import {
  publishDerivedTagExemplars,
  resolveDerivedTagExemplarRecordKeys,
  validateDerivedTagExemplarsAgainstRecords,
  type PublishedDerivedTagExemplars,
  type PublishedDerivedTagExemplarSet,
} from "../publication/exemplars.js";
import {
  buildDerivedTagRuntimeArtifacts,
  deriveRecordTagDerivationFromRuntime,
  type DerivedTagRuntimeArtifacts,
} from "./runtime-builder.js";

export { groupDerivedTagOntology } from "../publication/catalog.js";
export { normalizeDerivedTag } from "../matcher/engine.js";

type DerivedTagRuntimeAuthoringInputs = {
  explicitAssignments?: AuthoredDerivedTagAssignment[];
};

type DerivedTagRuntimeAuthoringRules = {
  explicitAssignments: AuthoredDerivedTagAssignment[];
};

const DERIVED_TAG_DEFAULT_RUNTIME_AUTHORING: DerivedTagRuntimeAuthoringRules = {
  explicitAssignments: [...DERIVED_TAG_ASSIGNMENTS],
};
const DERIVED_TAG_LEGACY_SEED_MIGRATION_CATEGORIES: DerivedTagLegacySeedMigrationCategory[] =
  DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => {
    const migration = DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY[category];
    return migration ? [migration] : [];
  });

let derivedTagRuntimeCache:
  | {
      familyDefaultsRevision: number;
      translationRevision: number;
      runtime: DerivedTagRuntimeArtifacts;
    }
  | null = null;

let derivedTagExemplarsCache:
  | {
      familyDefaultsRevision: number;
      translationRevision: number;
      exemplars: PublishedDerivedTagExemplars;
    }
  | null = null;

function buildCurrentDerivedTagRuntimeInputs(inputs: DerivedTagRuntimeAuthoringInputs = {}): DerivedTagRuntimeArtifacts {
  const ontology = getDerivedTagCanonicalOntology();
  const visibleOntology = buildVisibleDerivedTagOntology(ontology);
  const legacyTranslations = buildPublishedDerivedTagTranslations();
  const legacyTranslationsByKey = buildPublishedDerivedTagTranslationsByKey();
  const explicitAssignments = buildDerivedTagExplicitAssignmentIndex(
    ontology,
    inputs.explicitAssignments ?? DERIVED_TAG_DEFAULT_RUNTIME_AUTHORING.explicitAssignments,
  );

  return buildDerivedTagRuntimeArtifacts({
    ontology,
    visibleOntology,
    legacyTranslations,
    legacyTranslationsByKey,
    explicitAssignments,
    legacySeedMigrationCategories: DERIVED_TAG_LEGACY_SEED_MIGRATION_CATEGORIES,
  });
}

export function buildDerivedTagRuntimeArtifactsFromAuthoredState(
  inputs: DerivedTagRuntimeAuthoringInputs,
): DerivedTagRuntimeArtifacts {
  return buildCurrentDerivedTagRuntimeInputs({
    explicitAssignments: inputs.explicitAssignments,
  });
}

function getCurrentDerivedTagRuntime(): DerivedTagRuntimeArtifacts {
  const familyDefaultsRevision = getCurrentDerivedTagFamilyTranslationDefaultsRevision();
  const translationRevision = getCurrentDerivedTagTranslationOverridesRevision();

  if (
    !derivedTagRuntimeCache ||
    derivedTagRuntimeCache.familyDefaultsRevision !== familyDefaultsRevision ||
    derivedTagRuntimeCache.translationRevision !== translationRevision
  ) {
    derivedTagRuntimeCache = {
      familyDefaultsRevision,
      translationRevision,
      runtime: buildCurrentDerivedTagRuntimeInputs(),
    };
  }

  return derivedTagRuntimeCache.runtime;
}

export function getCurrentDerivedTagPublishedRuntime(): DerivedTagRuntimeArtifacts {
  return getCurrentDerivedTagRuntime();
}

export function getCurrentDerivedTagOntology(): PublishedDerivedTagOntology {
  return getCurrentDerivedTagPublishedRuntime().ontology;
}

export function getCurrentPublicDerivedTagOntology(): PublishedDerivedTagOntology {
  return getCurrentDerivedTagPublishedRuntime().visibleOntology;
}

export function getCurrentDerivedTagOntologyFamilies(): PublishedDerivedTagOntology["families"] {
  return getCurrentDerivedTagOntology().families;
}

export function getCurrentDerivedTagOntologyTags(): PublishedDerivedTagOntology["tags"] {
  return getCurrentDerivedTagOntology().tags;
}

export function getCurrentPublicDerivedTagOntologyFamilies(): PublishedDerivedTagOntology["families"] {
  return getCurrentPublicDerivedTagOntology().families;
}

export function getCurrentPublicDerivedTagOntologyTags(): PublishedDerivedTagOntology["tags"] {
  return getCurrentPublicDerivedTagOntology().tags;
}

export function getCurrentDerivedTagOntologyConcepts(): readonly DerivedTagCanonicalConcept[] {
  return getCurrentDerivedTagOntology().conceptModel.concepts;
}
export function getCurrentDerivedTagOntologyTranslations(): DerivedTagRuntimeArtifacts["legacyTranslations"] {
  return getCurrentDerivedTagPublishedRuntime().legacyTranslations;
}
export function getCurrentDerivedTagOntologyTranslationByKey(): ReadonlyMap<
  `${string}:${string}`,
  DerivedTagRuntimeArtifacts["legacyTranslations"][number]
> {
  return getCurrentDerivedTagPublishedRuntime().legacyTranslationsByKey;
}

function getCurrentDerivedTagExemplars(): PublishedDerivedTagExemplars {
  const familyDefaultsRevision = getCurrentDerivedTagFamilyTranslationDefaultsRevision();
  const translationRevision = getCurrentDerivedTagTranslationOverridesRevision();

  if (
    !derivedTagExemplarsCache ||
    derivedTagExemplarsCache.familyDefaultsRevision !== familyDefaultsRevision ||
    derivedTagExemplarsCache.translationRevision !== translationRevision
  ) {
    derivedTagExemplarsCache = {
      familyDefaultsRevision,
      translationRevision,
      exemplars: publishDerivedTagExemplars(
        getCurrentDerivedTagOntology(),
        DERIVED_TAG_REGISTRATION_CATEGORIES.map((category) => DERIVED_TAG_EXEMPLARS_BY_CATEGORY[category]),
      ),
    };
  }

  return derivedTagExemplarsCache.exemplars;
}

export function deriveRecordTags(input: DerivedTagContext): string[] {
  return deriveRecordTagDerivation(input).tags;
}

export function deriveRecordTagDerivation(input: DerivedTagContext): DerivedTagDerivation {
  return deriveRecordTagDerivationFromRuntime(getCurrentDerivedTagRuntime(), input);
}

export function getDerivedTagExemplarRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveDerivedTagExemplarRecordKeys(getCurrentDerivedTagExemplars(), normalizeDerivedTag(tag), scope);
}

export function getDerivedTagExemplars(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): PublishedDerivedTagExemplarSet[] {
  return getCurrentDerivedTagExemplars().exemplars
    .filter((entry) => entry.tag === normalizeDerivedTag(tag))
    .filter((entry) => !scope.category || entry.category === scope.category);
}

export function listDerivedTagLegacySeedMigrations(
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): DerivedTagLegacySeedMigrationDefinition[] {
  return listConfiguredDerivedTagLegacySeedMigrations(getCurrentDerivedTagRuntime().legacySeedMigrations, scope);
}

export function getDerivedTagLegacySeedMigrationRecordKeys(
  tag: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  return resolveLegacySeedMigrationRecordKeys(
    getCurrentDerivedTagRuntime().legacySeedMigrations,
    normalizeDerivedTag(tag),
    scope,
  );
}

export function getDerivedTagFamilyTags(
  family: string,
  scope: { category?: DerivedTagContext["category"]; subcategory?: DerivedTagContext["subcategory"] } = {},
): string[] {
  const runtime = getCurrentDerivedTagRuntime();
  const ontology = runtime.ontology;
  const normalizedFamily = normalizeDerivedTag(family);
  const requestedFamilies = new Set<string>([normalizedFamily]);
  const tags = new Set<string>();

  for (const ontologyFamily of ontology.families) {
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
    for (const tag of ontology.tagsByFamilyKey.get(familyKey) ?? []) {
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
  const ontology = getCurrentDerivedTagRuntime().ontology;
  const tags = new Set<string>();

  for (const tag of ontology.tags) {
    if (scope.category && tag.category !== scope.category) {
      continue;
    }
    const familyKey = `${tag.category}:${normalizeDerivedTag(tag.family)}` as const;
    const family = ontology.familyByKey.get(familyKey);
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
  const recordSummaries = [...records];
  const recordPacks = new Set(recordSummaries.map((record) => record.recordKey.split(":")[0]));
  const runtime = getCurrentDerivedTagRuntime();
  const assignmentsForIndexedPacks = DERIVED_TAG_ASSIGNMENTS.filter((assignment) =>
    recordPacks.has(assignment.recordKey.split(":")[0]),
  );
  const recordBackedAssignmentIndex = buildDerivedTagExplicitAssignmentIndex(
    runtime.ontology,
    assignmentsForIndexedPacks,
    recordSummaries,
  );
  validateDerivedTagExplicitAssignmentsAgainstRecords(recordSummaries, recordBackedAssignmentIndex);
  validateDerivedTagExemplarsAgainstRecords(recordSummaries, getCurrentDerivedTagExemplars());
}

export type { DerivedTagDerivation };
export { deriveRecordTagDerivationFromRuntime };
