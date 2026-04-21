import { DERIVED_TAG_ASSIGNMENTS_BY_CATEGORY } from "../../assignments/index.js";
import { DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY } from "../../reviews/assignment-memory/index.js";
import { DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY } from "../../reviews/assignment-reviews/index.js";
import { DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY } from "../../rules/index.js";
import { DERIVED_TAG_EXEMPLARS_BY_CATEGORY } from "../../exemplars/index.js";
import { DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY } from "../../reviews/exemplar-reviews/index.js";
import { DERIVED_TAG_MANAGED_CATEGORIES, type DerivedTagManagedCategory } from "../../manifest.js";
import type { DerivedTagMigrationAuthoredState } from "../types.js";

type DerivedTagManagedRegistry<T> = Record<DerivedTagManagedCategory, T>;

function clone<T>(value: T): T {
  return structuredClone(value);
}

function cloneManagedRegistry<T>(registry: DerivedTagManagedRegistry<T>): DerivedTagManagedRegistry<T> {
  return Object.fromEntries(
    DERIVED_TAG_MANAGED_CATEGORIES.map((category) => [category, clone(registry[category])] as const),
  ) as DerivedTagManagedRegistry<T>;
}

function buildImportedDerivedTagMigrationAuthoredState(): DerivedTagMigrationAuthoredState {
  return {
    assignments: cloneManagedRegistry(DERIVED_TAG_ASSIGNMENTS_BY_CATEGORY),
    assignmentReviews: cloneManagedRegistry(DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY),
    assignmentMemory: cloneManagedRegistry(DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY),
    exemplars: cloneManagedRegistry(DERIVED_TAG_EXEMPLARS_BY_CATEGORY),
    exemplarReviews: cloneManagedRegistry(DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY),
    authoredRules: cloneManagedRegistry(DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY),
  };
}

let currentDerivedTagMigrationAuthoredState: DerivedTagMigrationAuthoredState | null = null;
let currentDerivedTagMigrationAuthoredStateRevision = 0;

export function setCurrentDerivedTagMigrationAuthoredState(state: DerivedTagMigrationAuthoredState): void {
  currentDerivedTagMigrationAuthoredState = clone(state);
  currentDerivedTagMigrationAuthoredStateRevision += 1;
}

export function getCurrentDerivedTagMigrationAuthoredState(): DerivedTagMigrationAuthoredState {
  if (!currentDerivedTagMigrationAuthoredState) {
    currentDerivedTagMigrationAuthoredState = buildImportedDerivedTagMigrationAuthoredState();
  }
  return clone(currentDerivedTagMigrationAuthoredState);
}

export function getCurrentDerivedTagMigrationAuthoredStateRevision(): number {
  return currentDerivedTagMigrationAuthoredStateRevision;
}
