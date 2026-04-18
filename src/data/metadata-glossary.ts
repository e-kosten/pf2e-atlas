import { existsSync, readFileSync } from "node:fs";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

import type { MetadataGlossaryArtifact, MetadataGlossaryEntry } from "../types.js";

const TRAIT_LABEL_PATTERN = /["']?([a-z0-9-]+)["']?:\s*"PF2E\.Trait(?!Description)([A-Za-z0-9]+)"/g;
const TRAIT_DESCRIPTION_PATTERN = /["']?([a-z0-9-]+)["']?:\s*"PF2E\.TraitDescription([A-Za-z0-9]+)"/g;

function getPf2eLangEntries(rawLang: unknown): Record<string, string> {
  if (!rawLang || typeof rawLang !== "object") {
    return {};
  }
  const pf2e = (rawLang as { PF2E?: unknown }).PF2E;
  if (!pf2e || typeof pf2e !== "object") {
    return {};
  }

  return Object.fromEntries(
    Object.entries(pf2e).filter((entry): entry is [string, string] => typeof entry[1] === "string"),
  );
}

function collectTraitLocalizationKeys(
  source: string,
  pattern: RegExp,
  prefix: "Trait" | "TraitDescription",
): Map<string, string> {
  const keys = new Map<string, string>();

  for (const match of source.matchAll(pattern)) {
    const slug = match[1];
    const suffix = match[2];
    if (!slug || !suffix || keys.has(slug)) {
      continue;
    }
    keys.set(slug, `${prefix}${suffix}`);
  }

  return keys;
}

function fallbackLabel(value: string): string {
  return value
    .split(/[_\s-]+/)
    .filter((segment) => segment.length > 0)
    .map((segment) => `${segment[0]!.toUpperCase()}${segment.slice(1)}`)
    .join(" ");
}

export function extractTraitGlossaryEntries(
  traitsConfigSource: string,
  rawLang: unknown,
): Record<string, MetadataGlossaryEntry> {
  const labelsBySlug = collectTraitLocalizationKeys(traitsConfigSource, TRAIT_LABEL_PATTERN, "Trait");
  const descriptionsBySlug = collectTraitLocalizationKeys(
    traitsConfigSource,
    TRAIT_DESCRIPTION_PATTERN,
    "TraitDescription",
  );
  const pf2eLang = getPf2eLangEntries(rawLang);
  const slugs = [...new Set([...labelsBySlug.keys(), ...descriptionsBySlug.keys()])].sort((left, right) =>
    left.localeCompare(right),
  );

  return Object.fromEntries(
    slugs.map((slug): [string, MetadataGlossaryEntry] => {
      const labelKey = labelsBySlug.get(slug);
      const descriptionKey = descriptionsBySlug.get(slug);
      return [
        slug,
        {
          value: slug,
          label: labelKey ? (pf2eLang[labelKey] ?? fallbackLabel(slug)) : fallbackLabel(slug),
          description: descriptionKey ? (pf2eLang[descriptionKey] ?? null) : null,
        },
      ];
    }),
  );
}

export function getMetadataGlossaryArtifactPath(indexPath: string): string {
  const extension = path.extname(indexPath);
  const basePath = extension ? indexPath.slice(0, -extension.length) : indexPath;
  return `${basePath}.metadata-glossary.json`;
}

export async function buildMetadataGlossaryArtifact(rootPath: string): Promise<MetadataGlossaryArtifact> {
  const [traitsConfigSource, rawLangSource] = await Promise.all([
    readFile(path.join(rootPath, "src", "scripts", "config", "traits.ts"), "utf8"),
    readFile(path.join(rootPath, "static", "lang", "en.json"), "utf8"),
  ]);

  return {
    generatedAt: new Date().toISOString(),
    fields: {
      traits: extractTraitGlossaryEntries(traitsConfigSource, JSON.parse(rawLangSource)),
    },
  };
}

export async function writeMetadataGlossaryArtifact(
  rootPath: string,
  indexPath: string,
): Promise<MetadataGlossaryArtifact> {
  const artifact = await buildMetadataGlossaryArtifact(rootPath);
  const artifactPath = getMetadataGlossaryArtifactPath(indexPath);
  await mkdir(path.dirname(artifactPath), { recursive: true });
  await writeFile(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`, "utf8");
  return artifact;
}

export function readMetadataGlossaryArtifact(indexPath: string): MetadataGlossaryArtifact | null {
  const artifactPath = getMetadataGlossaryArtifactPath(indexPath);
  if (!existsSync(artifactPath)) {
    return null;
  }

  try {
    return JSON.parse(readFileSync(artifactPath, "utf8")) as MetadataGlossaryArtifact;
  } catch {
    return null;
  }
}
