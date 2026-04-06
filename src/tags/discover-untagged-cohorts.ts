#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../app/config.js";
import { ConsoleProgressReporter } from "../progress.js";
import { SearchCategory, SearchSubcategory } from "../types.js";
import {
  discoverUntaggedCohorts,
  type UntaggedCohortOptions,
  type UntaggedCohortReport,
} from "./untagged-cohort-discovery.js";

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

function lastValue(args: MultiValueArgs, key: string): string | undefined {
  return args[key]?.at(-1);
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

export function parseOptions(argv: string[]): UntaggedCohortOptions {
  const args = parseCliArgs(argv);
  const category = lastValue(args, "category") as SearchCategory | undefined;
  if (!category) {
    throw new Error("Missing required --category <category> argument.");
  }

  return {
    category,
    subcategory: lastValue(args, "subcategory") as SearchSubcategory | undefined,
    cohortLimit: parseInteger(lastValue(args, "cohort-limit"), "--cohort-limit"),
    anchorLimit: parseInteger(lastValue(args, "anchor-limit"), "--anchor-limit"),
    minFeatureSupport: parseInteger(lastValue(args, "min-feature-support"), "--min-feature-support"),
    minFeatureLift: parseFloatValue(lastValue(args, "min-feature-lift"), "--min-feature-lift"),
  };
}

export function formatUntaggedCohortReport(report: UntaggedCohortReport): string {
  const scope = report.subcategory ? `${report.category}/${report.subcategory}` : report.category;
  const lines = [
    "Untagged cohort summary:",
    `- Scope: ${scope}`,
    `- Untagged records: ${report.untaggedRecordCount}`,
    `- Baseline records: ${report.baselineRecordCount}`,
    "",
    "Top anchors:",
    ...(report.anchorTerms.length > 0
      ? report.anchorTerms.map((anchor) =>
        `- ${anchor.value} support=${anchor.support} baseline=${anchor.baselineSupport} lift=${anchor.lift.toFixed(2)} score=${anchor.score.toFixed(2)}`)
      : ["- (none)"]),
    "",
    "Recommended cohorts:",
    ...(report.cohorts.length > 0
      ? report.cohorts.flatMap((cohort) => [
        `- ${cohort.recommendation} score=${cohort.score.toFixed(2)} size=${cohort.size} families=${cohort.distinctVariantFamilies} sources=${cohort.sourceCount} signature=${cohort.signature.join(", ")}`,
        `  non_name=${cohort.nonNameAnchors.join(", ") || "(none)"} flags=${cohort.reviewFlags.join(", ") || "(none)"} top_sources=${cohort.topSources.join(", ") || "(none)"}`,
        ...cohort.representativeRecords.map((record) => `  ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`),
        ...(cohort.contrastRecords.length > 0
          ? [
            "  contrast:",
            ...cohort.contrastRecords.map((record) => `    ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`),
          ]
          : []),
      ])
      : ["- (none)"]),
  ];

  return lines.join("\n");
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const options = parseOptions(argv);
  const config = await loadConfig(argv);
  const progress = new ConsoleProgressReporter(process.stderr);

  try {
    await access(config.indexPath, constants.R_OK);
  } catch {
    throw new Error(`Index not found at ${config.indexPath}. Run 'npm run refresh-index' first.`);
  }

  const db = new DatabaseSync(config.indexPath);
  try {
    progress.log(`Running untagged cohort discovery against ${config.indexPath}.`);
    const report = discoverUntaggedCohorts(db, {
      ...options,
      progressLogger: (message) => progress.log(message),
      progressStatusLogger: (message) => progress.status(message),
    });
    progress.clear();
    console.log(formatUntaggedCohortReport(report));
  } finally {
    progress.clear();
    db.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Untagged cohort discovery failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
