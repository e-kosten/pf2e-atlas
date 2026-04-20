#!/usr/bin/env node

import { constants } from "node:fs";
import { access } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../../app/config.js";
import { expandHome } from "../../../shared/utils.js";
import { parseOptionalSearchCategoryArg } from "../shared/search-scope-args.js";
import {
  evaluateDerivedTagMovement,
  type DerivedTagCategoryMovement,
  type DerivedTagMovementEvaluationOptions,
  type DerivedTagTagMovement,
} from "../../evaluation/movement-evaluator.js";

type CliOptions = DerivedTagMovementEvaluationOptions & {
  baselineIndexPath: string;
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

export function parseOptions(argv: string[]): CliOptions {
  const args = parseCliArgs(argv);
  const baselineIndexPath = args["baseline-index-path"]?.trim();
  if (!baselineIndexPath) {
    throw new Error("Missing required --baseline-index-path <path-to-baseline-index> argument.");
  }

  return {
    baselineIndexPath: path.resolve(expandHome(baselineIndexPath)),
    category: parseOptionalSearchCategoryArg(args.category, "--category"),
    tags: args.tags
      ?.split(",")
      .map((tag) => tag.trim())
      .filter((tag) => tag.length > 0),
    limit: parseInteger(args.limit, "--limit"),
    sampleLimit: parseInteger(args["sample-limit"], "--sample-limit"),
    warnCategoryGainBelowPoints: parseFloatValue(
      args["warn-category-gain-below-points"],
      "--warn-category-gain-below-points",
    ),
    warnCategoryDropPoints: parseFloatValue(args["warn-category-drop-points"], "--warn-category-drop-points"),
    warnTagGainBelowCount: parseInteger(args["warn-tag-gain-below-count"], "--warn-tag-gain-below-count"),
    warnTagDropCount: parseInteger(args["warn-tag-drop-count"], "--warn-tag-drop-count"),
    warnTagDropPoints: parseFloatValue(args["warn-tag-drop-points"], "--warn-tag-drop-points"),
  };
}

function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

function formatSigned(value: number, fractionDigits: number): string {
  const formatted = value.toFixed(fractionDigits);
  return value > 0 ? `+${formatted}` : formatted;
}

function formatCategoryMovement(movement: DerivedTagCategoryMovement): string {
  return `- ${movement.category}: ${movement.baselineTagged}/${movement.baselineTotal} (${formatPercent(movement.baselineCoveragePercent)}) -> ${movement.currentTagged}/${movement.currentTotal} (${formatPercent(movement.currentCoveragePercent)}) | delta=${formatSigned(movement.deltaTagged, 0)} tagged, ${formatSigned(movement.deltaCoveragePoints, 1)} pp`;
}

function formatTagMovement(movement: DerivedTagTagMovement): string {
  return `- ${movement.tag}: ${movement.baselineCount} (${formatPercent(movement.baselineCoveragePercent)}) -> ${movement.currentCount} (${formatPercent(movement.currentCoveragePercent)}) | delta=${formatSigned(movement.deltaCount, 0)}, ${formatSigned(movement.deltaCoveragePoints, 1)} pp`;
}

function formatRecordMovement(label: string, records: DerivedTagTagMovement["gainedRecords"]): string[] {
  if (records.length === 0) {
    return [];
  }

  return [`  ${label}:`, ...records.map((record) => `    - ${record.name} [${record.recordKey}]`)];
}

async function assertReadable(pathToCheck: string, errorMessage: string): Promise<void> {
  try {
    await access(pathToCheck, constants.R_OK);
  } catch {
    throw new Error(errorMessage);
  }
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const options = parseOptions(argv);
  const config = await loadConfig(argv);

  await assertReadable(
    options.baselineIndexPath,
    `Baseline index not found at ${options.baselineIndexPath}. Pass --baseline-index-path /path/to/baseline.sqlite.`,
  );
  await assertReadable(
    config.indexPath,
    `Current index not found at ${config.indexPath}. Run 'npm run refresh-index' first or pass --index-path /path/to/current.sqlite.`,
  );

  const baselineDb = new DatabaseSync(options.baselineIndexPath, { readOnly: true });
  const currentDb = new DatabaseSync(config.indexPath, { readOnly: true });

  try {
    const result = evaluateDerivedTagMovement(baselineDb, currentDb, options);

    console.log(`Baseline index: ${options.baselineIndexPath}`);
    console.log(`Current index: ${config.indexPath}`);
    console.log("");
    console.log("Category movement:");
    for (const movement of result.categories) {
      console.log(formatCategoryMovement(movement));
    }

    if (options.category) {
      console.log("");
      console.log(`Tag movement within ${options.category}:`);
      if (result.tags.length === 0) {
        console.log("- (no tag movement)");
      } else {
        for (const movement of result.tags) {
          console.log(formatTagMovement(movement));
          for (const line of formatRecordMovement("gained", movement.gainedRecords)) {
            console.log(line);
          }
          for (const line of formatRecordMovement("lost", movement.lostRecords)) {
            console.log(line);
          }
        }
      }
    }

    if (result.warnings.length > 0) {
      console.log("");
      console.log("Warnings:");
      for (const warning of result.warnings) {
        console.log(`- ${warning.message}`);
      }
      process.exitCode = 1;
    }
  } finally {
    baselineDb.close();
    currentDb.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag movement evaluation failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
