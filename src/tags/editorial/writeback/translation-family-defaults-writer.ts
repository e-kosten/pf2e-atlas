import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";

import type { DerivedTagFamilyTranslationDefaults } from "../../translations/family-defaults.js";
import { setCurrentDerivedTagFamilyTranslationDefaults } from "../../translations/state.js";

const TRANSLATION_FAMILY_DEFAULTS_FILE_PATH = path.join("src", "tags", "translations", "family-defaults.ts");

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

function renderFamilyDefaultsFile(
  defaults: ReadonlyMap<string, DerivedTagFamilyTranslationDefaults>,
): string {
  const renderedEntries = [...defaults.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => `  [${JSON.stringify(key)}, ${renderTsValue(value, 1)}],`)
    .join("\n");

  return [
    'import type { DerivedTagConceptSchemaKind, DerivedTagTranslationStatus } from "../../domain/derived-tag-types.js";',
    "",
    "export type DerivedTagFamilyTranslationDefaults = {",
    "  schemaKind: DerivedTagConceptSchemaKind;",
    "  translationStatus: DerivedTagTranslationStatus;",
    "  primaryFacetKind?: string;",
    "  primaryFacetValue?: string;",
    "  notes?: string;",
    "};",
    "",
    "export const DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS = new Map<string, DerivedTagFamilyTranslationDefaults>([",
    renderedEntries,
    "]);",
    "",
    "export function getDerivedTagFamilyTranslationDefaults(key: string): DerivedTagFamilyTranslationDefaults | undefined {",
    "  return DERIVED_TAG_FAMILY_TRANSLATION_DEFAULTS.get(key);",
    "}",
    "",
  ].join("\n");
}

export async function writeDerivedTagFamilyTranslationDefaults(
  rootPath: string,
  defaults: ReadonlyMap<string, DerivedTagFamilyTranslationDefaults>,
): Promise<void> {
  const outputPath = path.join(rootPath, TRANSLATION_FAMILY_DEFAULTS_FILE_PATH);
  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, renderFamilyDefaultsFile(defaults), "utf8");
  setCurrentDerivedTagFamilyTranslationDefaults(defaults);
}
