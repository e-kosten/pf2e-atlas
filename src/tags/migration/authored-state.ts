import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import type { AuthoredDerivedTagRule, DerivedTagExemplarCategory } from "../../types.js";
import { AFFLICTION_DERIVED_TAG_ASSIGNMENTS } from "../assignments/affliction.js";
import { CREATURE_DERIVED_TAG_ASSIGNMENTS } from "../assignments/creature.js";
import { EQUIPMENT_DERIVED_TAG_ASSIGNMENTS } from "../assignments/equipment.js";
import { HAZARD_DERIVED_TAG_ASSIGNMENTS } from "../assignments/hazard.js";
import { SPELL_DERIVED_TAG_ASSIGNMENTS } from "../assignments/spell.js";
import {
  AFFLICTION_AUTHORED_DERIVED_TAG_RULES,
  CREATURE_AUTHORED_DERIVED_TAG_RULES,
  EQUIPMENT_AUTHORED_DERIVED_TAG_RULES,
  HAZARD_AUTHORED_DERIVED_TAG_RULES,
  SPELL_AUTHORED_DERIVED_TAG_RULES,
} from "../authored-rules/index.js";
import {
  AFFLICTION_DERIVED_TAG_EXEMPLARS,
  CREATURE_DERIVED_TAG_EXEMPLARS,
  EQUIPMENT_DERIVED_TAG_EXEMPLARS,
  HAZARD_DERIVED_TAG_EXEMPLARS,
  SPELL_DERIVED_TAG_EXEMPLARS,
} from "../exemplars/index.js";
import type { AuthoredDerivedTagAssignment } from "../runtime/assignments.js";
import type { DerivedTagManagedCategory, DerivedTagMigrationAuthoredState } from "./types.js";

const CATEGORY_UPPER: Record<DerivedTagManagedCategory, string> = {
  affliction: "AFFLICTION",
  creature: "CREATURE",
  equipment: "EQUIPMENT",
  hazard: "HAZARD",
  spell: "SPELL",
};

export const CATEGORY_FILE_PATHS: Record<DerivedTagManagedCategory, {
  assignment: string;
  exemplar: string;
  authoredRule: string;
}> = {
  affliction: {
    assignment: path.join("src", "tags", "assignments", "affliction.ts"),
    exemplar: path.join("src", "tags", "exemplars", "affliction.ts"),
    authoredRule: path.join("src", "tags", "authored-rules", "affliction.ts"),
  },
  creature: {
    assignment: path.join("src", "tags", "assignments", "creature.ts"),
    exemplar: path.join("src", "tags", "exemplars", "creature.ts"),
    authoredRule: path.join("src", "tags", "authored-rules", "creature.ts"),
  },
  equipment: {
    assignment: path.join("src", "tags", "assignments", "equipment.ts"),
    exemplar: path.join("src", "tags", "exemplars", "equipment.ts"),
    authoredRule: path.join("src", "tags", "authored-rules", "equipment.ts"),
  },
  hazard: {
    assignment: path.join("src", "tags", "assignments", "hazard.ts"),
    exemplar: path.join("src", "tags", "exemplars", "hazard.ts"),
    authoredRule: path.join("src", "tags", "authored-rules", "hazard.ts"),
  },
  spell: {
    assignment: path.join("src", "tags", "assignments", "spell.ts"),
    exemplar: path.join("src", "tags", "exemplars", "spell.ts"),
    authoredRule: path.join("src", "tags", "authored-rules", "spell.ts"),
  },
};

function clone<T>(value: T): T {
  return structuredClone(value);
}

function buildImportedDerivedTagMigrationAuthoredState(): DerivedTagMigrationAuthoredState {
  return {
    assignments: {
      affliction: clone(AFFLICTION_DERIVED_TAG_ASSIGNMENTS),
      creature: clone(CREATURE_DERIVED_TAG_ASSIGNMENTS),
      equipment: clone(EQUIPMENT_DERIVED_TAG_ASSIGNMENTS),
      hazard: clone(HAZARD_DERIVED_TAG_ASSIGNMENTS),
      spell: clone(SPELL_DERIVED_TAG_ASSIGNMENTS),
    },
    exemplars: {
      affliction: clone(AFFLICTION_DERIVED_TAG_EXEMPLARS),
      creature: clone(CREATURE_DERIVED_TAG_EXEMPLARS),
      equipment: clone(EQUIPMENT_DERIVED_TAG_EXEMPLARS),
      hazard: clone(HAZARD_DERIVED_TAG_EXEMPLARS),
      spell: clone(SPELL_DERIVED_TAG_EXEMPLARS),
    },
    authoredRules: {
      affliction: clone(AFFLICTION_AUTHORED_DERIVED_TAG_RULES),
      creature: clone(CREATURE_AUTHORED_DERIVED_TAG_RULES),
      equipment: clone(EQUIPMENT_AUTHORED_DERIVED_TAG_RULES),
      hazard: clone(HAZARD_AUTHORED_DERIVED_TAG_RULES),
      spell: clone(SPELL_AUTHORED_DERIVED_TAG_RULES),
    },
  };
}

let currentDerivedTagMigrationAuthoredState: DerivedTagMigrationAuthoredState | null = null;

export function setCurrentDerivedTagMigrationAuthoredState(
  state: DerivedTagMigrationAuthoredState,
): void {
  currentDerivedTagMigrationAuthoredState = clone(state);
}

export function getCurrentDerivedTagMigrationAuthoredState(): DerivedTagMigrationAuthoredState {
  if (!currentDerivedTagMigrationAuthoredState) {
    currentDerivedTagMigrationAuthoredState = buildImportedDerivedTagMigrationAuthoredState();
  }
  return clone(currentDerivedTagMigrationAuthoredState);
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
    const rendered = value
      .map((entry) => `${indent(level + 1)}${renderTsValue(entry, level + 1)}`)
      .join(",\n");
    return `[\n${rendered}\n${indent(level)}]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>).filter(([, entryValue]) => entryValue !== undefined);
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

function renderAssignmentFile(category: DerivedTagManagedCategory, assignments: AuthoredDerivedTagAssignment[]): string {
  const exportName = `${CATEGORY_UPPER[category]}_DERIVED_TAG_ASSIGNMENTS`;
  return [
    "import type { AuthoredDerivedTagAssignment } from \"../runtime/assignments.js\";",
    "",
    `export const ${exportName}: AuthoredDerivedTagAssignment[] = ${renderTsValue(assignments)};`,
    "",
  ].join("\n");
}

function renderExemplarFile(category: DerivedTagManagedCategory, exemplars: DerivedTagExemplarCategory): string {
  const exportName = `${CATEGORY_UPPER[category]}_DERIVED_TAG_EXEMPLARS`;
  return [
    "import type { DerivedTagExemplarCategory } from \"../../types.js\";",
    "",
    `export const ${exportName} = ${renderTsValue(exemplars)} satisfies DerivedTagExemplarCategory;`,
    "",
  ].join("\n");
}

function renderAuthoredRuleFile(category: DerivedTagManagedCategory, rules: AuthoredDerivedTagRule[]): string {
  const exportName = `${CATEGORY_UPPER[category]}_AUTHORED_DERIVED_TAG_RULES`;
  return [
    "import type { AuthoredDerivedTagRule } from \"../../types.js\";",
    "",
    `export const ${exportName}: AuthoredDerivedTagRule[] = ${renderTsValue(rules)};`,
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

  setCurrentDerivedTagMigrationAuthoredState(state);
}
