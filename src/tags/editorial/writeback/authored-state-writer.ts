import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import type {
  AuthoredDerivedTagRule,
  DerivedTagExemplarCategory,
  DerivedTagExemplarReviewCategory,
} from "../../../domain/derived-tag-types.js";
import {
  DERIVED_TAG_MANAGED_CATEGORIES,
  getDerivedTagCategoryManifestEntry,
  type DerivedTagManagedCategory,
} from "../../manifest.js";
import type {
  AuthoredDerivedTagAssignment,
  DerivedTagAssignmentDecision,
  DerivedTagAssignmentMemoryCategory,
  DerivedTagAssignmentReviewCategory,
} from "../../runtime/derivation/assignments.js";
import { setCurrentDerivedTagAuthoredState } from "../state/authored-state.js";
import type { DerivedTagAuthoredState } from "../types.js";

type DerivedTagManagedRegistry<T> = Record<DerivedTagManagedCategory, T>;

const CATEGORY_FILE_PATHS = Object.fromEntries(
  DERIVED_TAG_MANAGED_CATEGORIES.map((category) => [
    category,
    {
      exemplar: path.join("src", "tags", "exemplars", `${category}.ts`),
      authoredRule: path.join("src", "tags", "rules", `${category}.ts`),
    },
  ]),
) as Record<
  DerivedTagManagedCategory,
  {
    exemplar: string;
    authoredRule: string;
  }
>;

const REVIEW_REGISTRY_FILE_PATHS = {
  assignmentReview: path.join("src", "tags", "reviews", "assignment-reviews", "index.ts"),
  assignmentMemory: path.join("src", "tags", "reviews", "assignment-memory", "index.ts"),
  exemplarReview: path.join("src", "tags", "reviews", "exemplar-reviews", "index.ts"),
} as const;

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

function getPackExportName(packName: string): string {
  return `${packName.toUpperCase().replace(/[^A-Z0-9]+/g, "_")}_DERIVED_TAG_ASSIGNMENTS`;
}

function getPackName(recordKey: string): string {
  const packName = recordKey.split(":")[0];
  if (!packName) {
    throw new Error(`Cannot render assignment for record key without pack name: "${recordKey}".`);
  }
  return packName;
}

function renderAssignmentDecision(decision: DerivedTagAssignmentDecision, level: number): string {
  const lines = [
    `${indent(level)}tag(${JSON.stringify(decision.tag)}, {`,
    `${indent(level + 1)}source: ${JSON.stringify(decision.source)},`,
    ...(decision.confidence ? [`${indent(level + 1)}confidence: ${JSON.stringify(decision.confidence)},`] : []),
    `${indent(level + 1)}rationale: ${JSON.stringify(decision.rationale)},`,
    `${indent(level)}})`,
  ];
  return lines.join("\n");
}

function renderAssignmentDecisionList(
  decisions: DerivedTagAssignmentDecision[] | undefined,
  level: number,
): string | undefined {
  if (!decisions || decisions.length === 0) {
    return undefined;
  }
  const rendered = decisions
    .map((decision) => renderAssignmentDecision(decision, level + 1))
    .join(",\n");
  return `[\n${rendered}\n${indent(level)}]`;
}

function renderAssignmentFile(
  packName: string,
  assignments: AuthoredDerivedTagAssignment[],
): string {
  const exportName = getPackExportName(packName);
  const renderedAssignments =
    assignments.length === 0
      ? "{}"
      : `{\n${assignments
          .map((assignment) => {
            const lines = [
              `${indent(1)}${JSON.stringify(assignment.recordKey)}: {`,
              `${indent(2)}name: ${JSON.stringify(assignment.name)},`,
            ];
            const applied = renderAssignmentDecisionList(assignment.applied, 2);
            if (applied) {
              lines.push(`${indent(2)}applied: ${applied},`);
            }
            const excluded = renderAssignmentDecisionList(assignment.excluded, 2);
            if (excluded) {
              lines.push(`${indent(2)}excluded: ${excluded},`);
            }
            lines.push(`${indent(1)}}`);
            return lines.join("\n");
          })
          .join(",\n")}\n}`;
  return [
    'import { defineAssignments, tag } from "../builders.js";',
    "",
    `export const ${exportName} = defineAssignments(${renderedAssignments});`,
    "",
  ].join("\n");
}

function groupAssignmentsByPack(
  assignments: AuthoredDerivedTagAssignment[],
): Map<string, AuthoredDerivedTagAssignment[]> {
  const grouped = new Map<string, AuthoredDerivedTagAssignment[]>();
  for (const assignment of assignments) {
    const packName = getPackName(assignment.recordKey);
    const packAssignments = grouped.get(packName) ?? [];
    packAssignments.push(assignment);
    grouped.set(packName, packAssignments);
  }
  return grouped;
}

function renderAssignmentIndexFile(packNames: string[]): string {
  const imports = packNames.map((packName) => {
    const exportName = getPackExportName(packName);
    return `import { ${exportName} } from "./packs/${packName}.js";`;
  });
  const spreads = packNames.map((packName) => `  ...${getPackExportName(packName)},`);
  return [
    'import type { AuthoredDerivedTagAssignment } from "../runtime/derivation/assignments.js";',
    ...imports,
    "",
    "export const DERIVED_TAG_ASSIGNMENTS: AuthoredDerivedTagAssignment[] = [",
    ...spreads,
    "];",
    "",
  ].join("\n");
}

function renderExemplarFile(category: DerivedTagManagedCategory, exemplars: DerivedTagExemplarCategory): string {
  const exportName = `${getCategoryExportPrefix(category)}_DERIVED_TAG_EXEMPLARS`;
  return [
    'import type { DerivedTagExemplarCategory } from "../../domain/derived-tag-types.js";',
    "",
    `export const ${exportName} = ${renderTsValue(exemplars)} satisfies DerivedTagExemplarCategory;`,
    "",
  ].join("\n");
}

function renderAuthoredRuleFile(category: DerivedTagManagedCategory, rules: AuthoredDerivedTagRule[]): string {
  const exportName = `${getCategoryExportPrefix(category)}_AUTHORED_DERIVED_TAG_RULES`;
  return [
    'import type { AuthoredDerivedTagRule } from "../../domain/derived-tag-types.js";',
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
    'import type { DerivedTagAssignmentReviewCategory } from "../../runtime/derivation/assignments.js";',
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
    'import type { DerivedTagAssignmentMemoryCategory } from "../../runtime/derivation/assignments.js";',
    "",
    `export const DERIVED_TAG_ASSIGNMENT_MEMORY_BY_CATEGORY = ${renderTsValue(assignmentMemory)} satisfies Record<DerivedTagManagedCategory, DerivedTagAssignmentMemoryCategory>;`,
    "",
  ].join("\n");
}

function renderExemplarReviewRegistryFile(
  exemplarReviews: DerivedTagManagedRegistry<DerivedTagExemplarReviewCategory>,
): string {
  return [
    'import type { DerivedTagExemplarReviewCategory } from "../../../domain/derived-tag-types.js";',
    'import type { DerivedTagManagedCategory } from "../../manifest.js";',
    "",
    `export const DERIVED_TAG_EXEMPLAR_REVIEWS_BY_CATEGORY = ${renderTsValue(exemplarReviews)} satisfies Record<DerivedTagManagedCategory, DerivedTagExemplarReviewCategory>;`,
    "",
  ].join("\n");
}

export async function writeDerivedTagAuthoredState(
  rootPath: string,
  state: DerivedTagAuthoredState,
  categories: DerivedTagManagedCategory[],
): Promise<void> {
  const assignmentsByPack = groupAssignmentsByPack(state.assignments);
  const packNames = [...assignmentsByPack.keys()].sort();
  for (const [packName, assignments] of assignmentsByPack) {
    const assignmentFile = path.join("src", "tags", "assignments", "packs", `${packName}.ts`);
    await mkdir(path.dirname(path.join(rootPath, assignmentFile)), { recursive: true });
    await writeFile(
      path.join(rootPath, assignmentFile),
      renderAssignmentFile(packName, assignments),
      "utf8",
    );
  }
  await writeFile(
    path.join(rootPath, "src", "tags", "assignments", "index.ts"),
    renderAssignmentIndexFile(packNames),
    "utf8",
  );

  for (const category of categories) {
    const files = CATEGORY_FILE_PATHS[category];
    await mkdir(path.dirname(path.join(rootPath, files.exemplar)), { recursive: true });
    await mkdir(path.dirname(path.join(rootPath, files.authoredRule)), { recursive: true });

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

  setCurrentDerivedTagAuthoredState(state);
}
