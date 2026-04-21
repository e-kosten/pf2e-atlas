import { DERIVED_TAG_LEGACY_RULES_BY_CATEGORY } from "../../legacy-rules/index.js";
import { DERIVED_TAG_REGISTRATION_CATEGORIES } from "../../manifest.js";
import { DERIVED_TAG_ONTOLOGY_BY_CATEGORY, flattenDerivedTagAuthoredCategoryOntology } from "../../ontology/index.js";
import { compileAuthoredDerivedTagRules } from "../../rules/index.js";
import {
  buildDerivedTagExplicitAssignmentIndex,
  type AuthoredDerivedTagAssignment,
} from "../../runtime/derivation/assignments.js";
import { deriveRecordTagsFromRules, type DerivedTagContext, type DerivedTagRule } from "../../runtime/matcher/engine.js";
import { DERIVED_TAG_SEED_LOOKUP } from "../../runtime/publication/catalog-seed-records.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  deriveCatalogTagDerivation,
  publishDerivedTagOntology,
  type DerivedTagDerivation,
  type PublishedDerivedTagOntology,
} from "../../runtime/publication/catalog.js";
import { DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY } from "../../legacy-seed-migrations/index.js";
import { getCurrentDerivedTagAuthoredState, getCurrentDerivedTagAuthoredStateRevision } from "./authored-state.js";

type DerivedTagAssignmentGroup = {
  category: (typeof DERIVED_TAG_REGISTRATION_CATEGORIES)[number];
  assignments: AuthoredDerivedTagAssignment[];
};

type DerivedTagWorkingRuntime = {
  authoredRules: DerivedTagRule[];
  legacyRules: DerivedTagRule[];
  ontology: PublishedDerivedTagOntology;
  legacySeedMigrations: ReturnType<typeof buildDerivedTagLegacySeedMigrationIndex>;
  explicitAssignments: ReturnType<typeof buildDerivedTagExplicitAssignmentIndex>;
};

let workingRuntimeCache: { revision: number; runtime: DerivedTagWorkingRuntime } | null = null;

function buildCurrentDerivedTagWorkingRuntime(): DerivedTagWorkingRuntime {
  const state = getCurrentDerivedTagAuthoredState();
  const flattenedOntologies = DERIVED_TAG_REGISTRATION_CATEGORIES.map((category) =>
    flattenDerivedTagAuthoredCategoryOntology(DERIVED_TAG_ONTOLOGY_BY_CATEGORY[category]),
  );
  const ontology = publishDerivedTagOntology(
    flattenedOntologies.flatMap((category) => category.families),
    flattenedOntologies.flatMap((category) => category.tags),
  );
  const authoredRules = compileAuthoredDerivedTagRules(
    ontology,
    DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => state.authoredRules[category]),
  );
  const legacyRules = DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => DERIVED_TAG_LEGACY_RULES_BY_CATEGORY[category]);
  const explicitAssignments = buildDerivedTagExplicitAssignmentIndex(
    ontology,
    DERIVED_TAG_REGISTRATION_CATEGORIES.map(
      (category) =>
        ({
          category,
          assignments: state.assignments[category],
        }) satisfies DerivedTagAssignmentGroup,
    ),
  );
  const legacySeedMigrations = buildDerivedTagLegacySeedMigrationIndex(
    ontology,
    DERIVED_TAG_SEED_LOOKUP,
    DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap((category) => {
      const migration = DERIVED_TAG_LEGACY_SEED_MIGRATIONS_BY_CATEGORY[category];
      return migration ? [migration] : [];
    }),
  );

  return {
    authoredRules,
    legacyRules,
    ontology,
    legacySeedMigrations,
    explicitAssignments,
  };
}

export function getCurrentDerivedTagWorkingRuntime(): DerivedTagWorkingRuntime {
  const revision = getCurrentDerivedTagAuthoredStateRevision();
  if (!workingRuntimeCache || workingRuntimeCache.revision !== revision) {
    workingRuntimeCache = {
      revision,
      runtime: buildCurrentDerivedTagWorkingRuntime(),
    };
  }
  return workingRuntimeCache.runtime;
}

export function deriveCurrentRecordTagDerivation(input: DerivedTagContext): DerivedTagDerivation {
  const runtime = getCurrentDerivedTagWorkingRuntime();
  return deriveCatalogTagDerivation(
    runtime.ontology,
    input,
    {
      authoredRuleTags: deriveRecordTagsFromRules(runtime.authoredRules, input),
      legacyRuleTags: deriveRecordTagsFromRules(runtime.legacyRules, input),
    },
    runtime.explicitAssignments,
    runtime.legacySeedMigrations,
  );
}
