#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "./config.js";
import { evaluateDerivedTagGaps } from "./derived-tag-gap-evaluator.js";
import { SearchCategory, SearchSubcategory } from "./types.js";

type CliOptions = {
  tag: string;
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  limit?: number;
  exemplarLimit?: number;
  commonTraitLimit?: number;
  minSimilarity?: number;
};

function parseCliArgs(argv: string[]): Record<string, string> {
  const parsed: Record<string, string> = {};

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
      continue;
    }

    parsed[rawKey] = nextValue;
    if (inlineValue === undefined) {
      index += 1;
    }
  }

  return parsed;
}

function parseInteger(value: string | undefined, flagName: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected ${flagName} to be an integer, received "${value}".`);
  }

  return parsed;
}

function parseFloatValue(value: string | undefined, flagName: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }

  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected ${flagName} to be a number, received "${value}".`);
  }

  return parsed;
}

function parseOptions(argv: string[]): CliOptions {
  const args = parseCliArgs(argv);
  const tag = args.tag?.trim();
  if (!tag) {
    throw new Error("Missing required --tag <derived-tag> argument.");
  }

  return {
    tag,
    category: args.category as SearchCategory | undefined,
    subcategory: args.subcategory as SearchSubcategory | undefined,
    limit: parseInteger(args.limit, "--limit"),
    exemplarLimit: parseInteger(args["exemplar-limit"], "--exemplar-limit"),
    commonTraitLimit: parseInteger(args["common-trait-limit"], "--common-trait-limit"),
    minSimilarity: parseFloatValue(args["min-similarity"], "--min-similarity"),
  };
}

function formatScope(category?: SearchCategory, subcategory?: SearchSubcategory): string {
  if (category && subcategory) {
    return `${category}/${subcategory}`;
  }
  if (category) {
    return category;
  }
  return "all canonical records";
}

function formatSimilarity(value: number): string {
  return value.toFixed(3);
}

function formatLevel(level: number | null): string {
  return level === null ? "-" : String(level);
}

function formatTraits(traits: string[]): string {
  return traits.length > 0 ? traits.join(", ") : "(none)";
}

function trimDescription(value: string | null, maxLength = 180): string {
  if (!value) {
    return "(no description)";
  }

  if (value.length <= maxLength) {
    return value;
  }

  return `${value.slice(0, maxLength - 1)}…`;
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const options = parseOptions(argv);
  const config = await loadConfig(argv);

  try {
    await access(config.indexPath, constants.R_OK);
  } catch {
    throw new Error(`Index not found at ${config.indexPath}. Run 'npm run refresh-index' first.`);
  }

  const db = new DatabaseSync(config.indexPath);
  try {
    const result = evaluateDerivedTagGaps(db, options);

    console.log(`Tag: ${result.tag}`);
    console.log(`Scope: ${formatScope(result.category ?? undefined, result.subcategory ?? undefined)}`);
    console.log(`Tagged exemplars: ${result.exemplarCount}`);
    console.log(`Untagged candidates considered: ${result.candidateCount}`);
    console.log(`Common exemplar traits: ${formatTraits(result.commonTraits)}`);
    console.log("");
    console.log("Representative exemplars:");
    for (const exemplar of result.exemplars) {
      console.log(
        `- ${exemplar.name} [level ${formatLevel(exemplar.level)}] score=${formatSimilarity(exemplar.similarityToCentroid)} traits=${formatTraits(exemplar.traits)}`,
      );
    }
    console.log("");
    console.log("Potential gaps:");
    for (const candidate of result.candidates) {
      const sharedTraits = candidate.sharedTraits.length > 0 ? candidate.sharedTraits.join(", ") : "(none)";
      console.log(
        `- ${candidate.name} [level ${formatLevel(candidate.level)}] score=${formatSimilarity(candidate.similarity)} shared_traits=${sharedTraits}`,
      );
      console.log(`  traits=${formatTraits(candidate.traits)}`);
      console.log(`  ${trimDescription(candidate.descriptionText)}`);
    }
  } finally {
    db.close();
  }
}

main().catch((error) => {
  console.error(`Derived-tag gap evaluation failed: ${(error as Error).message}`);
  process.exit(1);
});
