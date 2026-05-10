import type { SearchCategory, DerivedTagLegacySeedMigrationCategory, DerivedTagTranslationRecord } from "../../../domain/derived-tag-types.js";
import { DERIVED_TAG_REGISTRATION_CATEGORIES } from "../../manifest.js";
import { DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY, compileAuthoredDerivedTagRules } from "../../rules/index.js";
import { DERIVED_TAG_LEGACY_RULES_BY_CATEGORY } from "../../legacy-rules/index.js";
import { deriveRecordTagsFromRules, type DerivedTagContext } from "../matcher/engine.js";
import type { DerivedTagDerivation, PublishedDerivedTagOntology } from "../publication/catalog.js";
import {
  buildDerivedTagLegacySeedMigrationIndex,
  deriveCatalogTagDerivation,
  type DerivedTagLegacySeedMigrationIndex,
} from "../publication/catalog.js";
import { translateLegacyDerivedTags, translateLegacySeedMigrationCategories } from "../../translations/index.js";
import { DERIVED_TAG_SEED_LOOKUP } from "../publication/catalog-seed-records.js";
import type { DerivedTagExplicitAssignmentIndex } from "./assignments.js";

export type DerivedTagRuntimeArtifacts = {
  ontology: PublishedDerivedTagOntology;
  visibleOntology: PublishedDerivedTagOntology;
  legacyTranslations: DerivedTagTranslationRecord[];
  legacyTranslationsByKey: ReadonlyMap<`${SearchCategory}:${string}`, DerivedTagTranslationRecord>;
  explicitAssignments: DerivedTagExplicitAssignmentIndex;
  legacySeedMigrations: DerivedTagLegacySeedMigrationIndex;
};

export type DerivedTagRuntimeBuildInputs = {
  ontology: PublishedDerivedTagOntology;
  visibleOntology: PublishedDerivedTagOntology;
  legacyTranslations: DerivedTagTranslationRecord[];
  legacyTranslationsByKey: ReadonlyMap<`${SearchCategory}:${string}`, DerivedTagTranslationRecord>;
  explicitAssignments: DerivedTagExplicitAssignmentIndex;
  legacySeedMigrationCategories: DerivedTagLegacySeedMigrationCategory[];
};

const DERIVED_TAG_AUTHORED_RULES = DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap(
  (category) => DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY[category],
);
const DERIVED_TAG_LEGACY_RULES = DERIVED_TAG_REGISTRATION_CATEGORIES.flatMap(
  (category) => DERIVED_TAG_LEGACY_RULES_BY_CATEGORY[category],
);

export function buildDerivedTagRuntimeArtifacts(
  inputs: DerivedTagRuntimeBuildInputs,
): DerivedTagRuntimeArtifacts {
  const legacySeedMigrations = buildDerivedTagLegacySeedMigrationIndex(
    inputs.ontology,
    DERIVED_TAG_SEED_LOOKUP,
    translateLegacySeedMigrationCategories(inputs.legacySeedMigrationCategories, inputs.ontology, inputs.legacyTranslationsByKey),
  );

  return {
    ontology: inputs.ontology,
    visibleOntology: inputs.visibleOntology,
    legacyTranslations: inputs.legacyTranslations,
    legacyTranslationsByKey: inputs.legacyTranslationsByKey,
    explicitAssignments: inputs.explicitAssignments,
    legacySeedMigrations,
  };
}

export function deriveRecordTagDerivationFromRuntime(
  runtime: DerivedTagRuntimeArtifacts,
  input: DerivedTagContext,
): DerivedTagDerivation {
  const authoredRuleTags = deriveRecordTagsFromRules(
    compileAuthoredDerivedTagRules(runtime.ontology, DERIVED_TAG_AUTHORED_RULES),
    input,
  );
  const legacyRuleTags = translateLegacyDerivedTags(
    input.category,
    deriveRecordTagsFromRules(DERIVED_TAG_LEGACY_RULES, input),
    runtime.ontology,
    runtime.legacyTranslationsByKey,
  );

  return deriveCatalogTagDerivation(
    runtime.ontology,
    input,
    {
      authoredRuleTags,
      legacyRuleTags,
    },
    runtime.explicitAssignments,
    runtime.legacySeedMigrations,
  );
}
