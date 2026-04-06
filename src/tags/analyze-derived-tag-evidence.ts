#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../app/config.js";
import { SearchCategory, SearchSubcategory } from "../types.js";
import {
  analyzeDiscoveryEvidence,
  type DiscoveryEvidenceOptions,
  type DiscoveryEvidenceReport,
} from "./evidence-analyzer.js";
import { resolveDiscoveryGramRange } from "./discovery-normalization.js";

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

function hasFlag(args: MultiValueArgs, key: string): boolean {
  return key in args && (args[key]?.length ?? 0) === 0;
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

export function parseOptions(argv: string[]): DiscoveryEvidenceOptions {
  const args = parseCliArgs(argv);
  const options = {
    category: lastValue(args, "category") as SearchCategory | undefined,
    subcategory: lastValue(args, "subcategory") as SearchSubcategory | undefined,
    recordKeys: args["record-key"]?.map((value) => value.trim()).filter(Boolean),
    excludeRecordKeys: args["exclude-record-key"]?.map((value) => value.trim()).filter(Boolean),
    tag: lastValue(args, "tag"),
    excludeDerivedTag: lastValue(args, "exclude-derived-tag"),
    untaggedOnly: hasFlag(args, "untagged"),
    limit: parseInteger(lastValue(args, "limit"), "--limit"),
    exampleLimit: parseInteger(lastValue(args, "example-limit"), "--example-limit"),
    minGramLength: parseInteger(lastValue(args, "min-gram-length"), "--min-gram-length"),
    maxGramLength: parseInteger(lastValue(args, "max-gram-length"), "--max-gram-length"),
  };
  resolveDiscoveryGramRange(options);
  return options;
}

function formatTerms(label: string, terms: DiscoveryEvidenceReport["nameTokens"]): string[] {
  return [
    `${label}:`,
    ...(terms.length > 0
      ? terms.map((term) =>
        `- ${term.value} support=${term.cohortSupport}/${Math.max(1, term.baselineSupport)} lift=${term.lift.toFixed(2)} examples=${term.examples.join(" | ")}`)
      : ["- (none)"]),
  ];
}

export function formatEvidenceReport(report: DiscoveryEvidenceReport): string {
  const scope = report.subcategory ? `${report.category}/${report.subcategory}` : (report.category ?? "all canonical records");
  const lines = [
    "Evidence summary:",
    `- Scope: ${scope}`,
    `- Cohort size: ${report.cohortSize}`,
    `- Baseline size: ${report.baselineSize}`,
    "",
    "Representative records:",
    ...(report.representativeRecords.length > 0
      ? report.representativeRecords.map((record) => `- ${record.name} (${record.recordKey}) traits=${record.traits.join(", ") || "(none)"}`)
      : ["- (none)"]),
    "",
    ...formatTerms("Name tokens", report.nameTokens),
    "",
    ...formatTerms("Name phrases", report.namePhrases),
    "",
    ...formatTerms("Description tokens", report.descriptionTokens),
    "",
    ...formatTerms("Description phrases", report.descriptionPhrases),
    "",
    ...formatTerms("Traits", report.traits),
    "",
    ...formatTerms("Reference features", report.references),
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
    const report = analyzeDiscoveryEvidence(db, options);
    console.log(formatEvidenceReport(report));
  } finally {
    db.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag evidence analysis failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
