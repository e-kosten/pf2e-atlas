#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../app/config.js";
import { SearchCategory, SearchSubcategory } from "../../types.js";
import {
  discoverRuleableCohorts,
  type RuleableCohortOptions,
  type RuleableCohortReport,
} from "../discovery/cohort-discovery.js";
import { formatDiscoverySourceContext } from "../discovery/discovery-source-report.js";

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

export function parseOptions(argv: string[]): RuleableCohortOptions {
  const args = parseCliArgs(argv);
  return {
    category: lastValue(args, "category") as SearchCategory | undefined,
    subcategory: lastValue(args, "subcategory") as SearchSubcategory | undefined,
    tag: lastValue(args, "tag"),
    exemplarNames: args.name?.map((value) => value.trim()).filter(Boolean),
    exemplarRecordKeys: args["record-key"]?.map((value) => value.trim()).filter(Boolean),
    candidateLimit: parseInteger(lastValue(args, "candidate-limit"), "--candidate-limit"),
    cohortLimit: parseInteger(lastValue(args, "cohort-limit"), "--cohort-limit"),
    minSimilarity: parseFloatValue(lastValue(args, "min-similarity"), "--min-similarity"),
  };
}

export function formatClusterReport(report: RuleableCohortReport): string {
  const scope = report.subcategory ? `${report.category}/${report.subcategory}` : report.category;
  const lines = [
    "Cohort summary:",
    `- Scope: ${scope}`,
    `- Source tag: ${report.sourceTag ?? "(seed exemplars)"}`,
    `- Exemplars: ${report.exemplarCount}`,
    `- Ranked candidates: ${report.candidateCount}`,
    "",
    "Anchor terms:",
    ...(report.anchorTerms.length > 0
      ? report.anchorTerms.map((term) => `- ${term.value} score=${term.score.toFixed(2)} lift=${term.lift.toFixed(2)}`)
      : ["- (none)"]),
    "",
    "Cohorts:",
    ...(report.cohorts.length > 0
      ? report.cohorts.flatMap((cohort) => [
        `- signature=${cohort.signature.join(", ") || "(semantic only)"} size=${cohort.size} families=${cohort.distinctVariantFamilies} sources=${cohort.sourceCount} publications=${cohort.publicationCount} source_slices=${cohort.sourceSliceCount} avg_similarity=${cohort.averageSimilarity.toFixed(3)} score=${cohort.score.toFixed(2)} recommendation=${cohort.recommendation}`,
        `  non_name=${cohort.nonNameAnchors.join(", ") || "(none)"} flags=${cohort.reviewFlags.join(", ") || "(none)"}`,
        `  ${formatDiscoverySourceContext(cohort)}`,
        ...cohort.representativeRecords.map((record) => `  ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`),
      ])
      : ["- (none)"]),
    "",
    "Contrast records:",
    ...(report.contrastRecords.length > 0
      ? report.contrastRecords.map((record) => `- ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`)
      : ["- (none)"]),
  ];

  return lines.join("\n");
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
    console.log(formatClusterReport(discoverRuleableCohorts(db, options)));
  } finally {
    db.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag cohort clustering failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
