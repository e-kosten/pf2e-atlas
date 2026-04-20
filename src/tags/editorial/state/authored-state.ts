import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarReviewCategory,
} from "../../../domain/index.js";
import { DERIVED_TAG_ASSIGNMENTS_BY_CATEGORY } from "../../assignments/index.js";
import { DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY } from "../../reviews/assignment-memory/index.js";
import { DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY } from "../../reviews/assignment-reviews/index.js";
import { DERIVED_TAG_AUTHORED_RULES_BY_CATEGORY } from "../../rules/index.js";
import { DERIVED_TAG_EXEMPLARS_BY_CATEGORY } from "../../exemplars/index.js";
import { DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY } from "../../reviews/exemplar-reviews/index.js";
import {
  DERIVED_TAG_MANAGED_CATEGORIES,
  getDerivedTagCategoryManifestEntry,
  type DerivedTagManagedCategory,
} from "../../manifest.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
} from "../../runtime/assignments.js";
import type { DerivedTagMigrationAuthoredState } from "../types.js";

type DerivedTagManagedRegistry<T> = Record<DerivedTagManagedCategory, T>;

export const CATEGORY_FILE_PATHS = Object.fromEntries(
  DERIVED_TAG_MANAGED_CATEGORIES.map((category) => [
    category,
    {
      assignment: path.join("src", "tags", "assignments", `${category}.ts`),
      exemplar: path.join("src", "tags", "exemplars", `${category}.ts`),
      authoredRule: path.join("src", "tags", "rules", `${category}.ts`),
    },
  ]),
) as Record<
  DerivedTagManagedCategory,
  {
    assignment: string;
    exemplar: string;
    authoredRule: string;
  }
>;

export const REVIEW_REGISTRY_FILE_PATHS = {
  assignmentReview: path.join("src", "tags", "reviews", "assignment-reviews", "index.ts"),
  assignmentMemory: path.join("src", "tags", "reviews", "assignment-memory", "index.ts"),
  exemplarReview: path.join("src", "tags", "reviews", "exemplar-reviews", "index.ts"),
} as const;

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

function isValidIdentifier(value: string): boolean {
  return /^[A-Za-z_$][A-Za-z0-9_$]*$/.test(value);
}

function indent(level: number): string {
  return "  ".repeat(level);
}

function renderTsValue(value: unknown, level = 0): string {
  if (value === null) {
    return "null";
  }
  if (typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return String(value);
  }
  if (Array.isArray(value)) {
    if (value.length === 0) {
      return "[]";
    }
    const rendered = value.map((entry) => `${indent(level + 1)}${renderTsValue(entry, level + 1)}`).join(",\n");
    return `[\n${rendered}\n${indent(level)}]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>).filter(
      ([, entryValue]) => entryValue !== undefined,
    );
    if (entries.length === 0) {
      return "{}";
    }
    const rendered = entries
      .map(([key, entryValue]) => {
        const renderedKey = isValidIdentifier(key) ? key : JSON.stringify(key);
        return `${indent(level + 1)}${renderedKey}: ${renderTsValue(entryValue, level + 1)}`;
      })
      .join(",\n");
    return `{\n${rendered}\n${indent(level)}}`;
  }

  throw new Error(`Unsupported value type in TypeScript serializer: ${typeof value}.`);
}

function getCategoryExportPrefix(category: DerivedTagManagedCategory): string {
  return getDerivedTagCategoryManifestEntry(category).exportPrefix;
}

function renderAssignmentFile(
  category: DerivedTagManagedCategory,
  assignments: AuthoredDerivedTagAssignment[],
): string {
  const exportName = `${getCategoryExportPrefix(category)}_DERIVED_TAG_ASSIGNMENTS`;
  return [
    'import type { AuthoredDerivedTagAssignment } from "../runtime/assignments.js";',
    "",
    `export const ${exportName}: AuthoredDerivedTagAssignment[] = ${renderTsValue(assignments)};`,
    "",
  ].join("\n");
}

function renderExemplarFile(category: DerivedTagManagedCategory, exemplars: DerivedTagExemplarCategory): string {
  const exportName = `${getCategoryExportPrefix(category)}_DERIVED_TAG_EXEMPLARS`;
  return [
    'import type { DerivedTagExemplarCategory } from "../../domain/index.js";',
    "",
    `export const ${exportName} = ${renderTsValue(exemplars)} satisfies DerivedTagExemplarCategory;`,
    "",
  ].join("\n");
}

function renderAuthoredRuleFile(category: DerivedTagManagedCategory, rules: AuthoredDerivedTagRule[]): string {
  const exportName = `${getCategoryExportPrefix(category)}_AUTHORED_DERIVED_TAG_RULES`;
  return [
    'import type { AuthoredDerivedTagRule } from "../../domain/index.js";',
    "",
    `export const ${exportName}: AuthoredDerivedTagRule[] = ${renderTsValue(rules)};`,
    "",
  ].join("\n");
}

function renderAssignmentReviewRegistryFile(
  assignmentReviews: DerivedTagManagedRegistry<DerivedTagAssignmentReviewCategory>,
): string {
  return [
    'import type { DerivedTagManagedCategory } from "../../manifest.js";',
    'import type { DerivedTagAssignmentReviewCategory } from "../../runtime/assignments.js";',
    "",
    `export const DERIVED_TAG_ASSIGNMENT_REVIEWS_BY_CATEGORY = ${renderTsValue(assignmentReviews)} satisfies Record<DerivedTagManagedCategory, DerivedTagAssignmentReviewCategory>;`,
    "",
  ].join("\n");
}

function renderAssignmentMemoryRegistryFile(
  assignmentMemory: DerivedTagManagedRegistry<DerivedTagAssignmentMemoryCategory>,
): string {
  return [
    'import type { DerivedTagManagedCategory } from "../../manifest.js";',
    'import type { DerivedTagAssignmentMemoryCategory } from "../../runtime/assignments.js";',
    "",
    `export const DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY = ${renderTsValue(assignmentMemory)} satisfies Record<DerivedTagManagedCategory, DerivedTagAssignmentMemoryCategory>;`,
    "",
  ].join("\n");
}

function renderExemplarReviewRegistryFile(
  exemplarReviews: DerivedTagManagedRegistry<DerivedTagExemplarReviewCategory>,
): string {
  return [
    'import type { DerivedTagExemplarReviewCategory } from "../../../domain/index.js";',
    'import type { DerivedTagManagedCategory } from "../../manifest.js";',
    "",
    `export const DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY = ${renderTsValue(exemplarReviews)} satisfies Record<DerivedTagManagedCategory, DerivedTagExemplarReviewCategory>;`,
    "",
  ].join("\n");
}

export async function writeDerivedTagMigrationAuthoredState(
  rootPath: string,
  state: DerivedTagMigrationAuthoredState,
  categories: DerivedTagManagedCategory[],
): Promise<void> {
  for (const category of categories) {
    const files = CATEGORY_FILE_PATHS[category];
    await mkdir(path.dirname(path.join(rootPath, files.assignment)), { recursive: true });
    await mkdir(path.dirname(path.join(rootPath, files.exemplar)), { recursive: true });
    await mkdir(path.dirname(path.join(rootPath, files.authoredRule)), { recursive: true });

    await writeFile(
      path.join(rootPath, files.assignment),
      renderAssignmentFile(category, state.assignments[category]),
      "utf8",
    );
    await writeFile(
      path.join(rootPath, files.exemplar),
      renderExemplarFile(category, state.exemplars[category]),
      "utf8",
    );
    await writeFile(
      path.join(rootPath, files.authoredRule),
      renderAuthoredRuleFile(category, state.authoredRules[category]),
      "utf8",
    );
  }

  await mkdir(path.dirname(path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.assignmentReview)), { recursive: true });
  await mkdir(path.dirname(path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.assignmentMemory)), { recursive: true });
  await mkdir(path.dirname(path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.exemplarReview)), { recursive: true });

  await writeFile(
    path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.assignmentReview),
    renderAssignmentReviewRegistryFile(state.assignmentReviews),
    "utf8",
  );
  await writeFile(
    path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.assignmentMemory),
    renderAssignmentMemoryRegistryFile(state.assignmentMemory),
    "utf8",
  );
  await writeFile(
    path.join(rootPath, REVIEW_REGISTRY_FILE_PATHS.exemplarReview),
    renderExemplarReviewRegistryFile(state.exemplarReviews),
    "utf8",
  );

  setCurrentDerivedTagMigrationAuthoredState(state);
}
