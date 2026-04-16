import { constants } from "node:fs";
import { access, writeFile } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../app/config.js";
import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { migrationSessionDirectory } from "./session-store.js";

type MultiValueArgs = Record<string, string[]>;

export function parseCliArgs(argv: string[]): MultiValueArgs {
  const parsed: MultiValueArgs = {};

  for (let index = 0; index < argv.length; index += 1) {
    const current = argv[index];
    if (!current || !current.startsWith("--")) {
      continue;
    }

    const [rawKey, inlineValue] = current.slice(2).split("=", 2);
    if (!rawKey) {
      continue;
    }

    const nextValue = inlineValue ?? argv[index + 1];
    if (!nextValue || nextValue.startsWith("--")) {
      parsed[rawKey] = parsed[rawKey] ?? [];
      continue;
    }

    const bucket = parsed[rawKey] ?? [];
    bucket.push(nextValue);
    parsed[rawKey] = bucket;
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  return parsed;
}

export function lastValue(args: MultiValueArgs, key: string): string | undefined {
  return args[key]?.at(-1);
}

export function parseInteger(value: string | undefined, flagName: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected ${flagName} to be an integer, received "${value}".`);
  }
  return parsed;
}

export async function openConfiguredIndex(argv: string[]): Promise<{ db: DatabaseSync; config: Awaited<ReturnType<typeof loadConfig>> }> {
  const config = await loadConfig(argv);
  await access(config.indexPath, constants.R_OK);
  return {
    config,
    db: new DatabaseSync(config.indexPath),
  };
}

export async function writeDerivedTagMigrationSummary(
  rootPath: string,
  sessionId: string,
  summary: string,
): Promise<void> {
  const directory = migrationSessionDirectory(rootPath, sessionId);
  await writeFile(path.join(directory, "summary.md"), `${summary}\n`, "utf8");
}

export type ParsedMigrationScope = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
};
