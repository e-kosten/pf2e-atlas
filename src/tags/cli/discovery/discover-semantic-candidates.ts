#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { pathToFileURL } from "node:url";

import { loadConfig } from "../../../app/config.js";
import { SearchCategory, SearchSubcategory } from "../../../domain/derived-tag-types.js";
import {
  parseOptionalScopedSearchSubcategoryArg,
  parseOptionalSearchCategoryArg,
} from "../shared/search-scope-args.js";
import {
  discoverSemanticCandidates,
  type SemanticDiscoveryCandidate,
  type SemanticDiscoveryOptions,
  type SemanticDiscoveryResult,
} from "../../discovery/semantic-discovery.js";
import { resolveDiscoveryGramRange } from "../../discovery/discovery-normalization.js";

type CliOptions = SemanticDiscoveryOptions;

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

export function parseOptions(argv: string[]): CliOptions {
  const args = parseCliArgs(argv);
  const exemplarNames = args.name?.map((value) => value.trim()).filter(Boolean) ?? [];
  const exemplarRecordKeys = args["record-key"]?.map((value) => value.trim()).filter(Boolean) ?? [];
  if (exemplarNames.length === 0 && exemplarRecordKeys.length === 0) {
    throw new Error("Provide at least one exemplar via --name <record name> or --record-key <canonical record key>.");
  }

  const category = parseOptionalSearchCategoryArg(lastValue(args, "category"), "--category");
  const options = {
    category,
    subcategory: parseOptionalScopedSearchSubcategoryArg(category, lastValue(args, "subcategory"), "--subcategory"),
    exemplarNames,
    exemplarRecordKeys,
    limit: parseInteger(lastValue(args, "candidate-limit") ?? lastValue(args, "limit"), "--candidate-limit"),
    exemplarLimit: parseInteger(lastValue(args, "exemplar-limit"), "--exemplar-limit"),
    contrastLimit: parseInteger(lastValue(args, "contrast-limit"), "--contrast-limit"),
    commonTraitLimit: parseInteger(lastValue(args, "common-trait-limit"), "--common-trait-limit"),
    sharedTokenLimit: parseInteger(lastValue(args, "shared-token-limit"), "--shared-token-limit"),
    sharedPhraseLimit: parseInteger(lastValue(args, "shared-phrase-limit"), "--shared-phrase-limit"),
    candidateEvidenceLimit: parseInteger(lastValue(args, "candidate-evidence-limit"), "--candidate-evidence-limit"),
    minSimilarity: parseFloatValue(lastValue(args, "min-similarity"), "--min-similarity"),
    excludeDerivedTag: lastValue(args, "exclude-derived-tag"),
    minGramLength: parseInteger(lastValue(args, "min-gram-length"), "--min-gram-length"),
    maxGramLength: parseInteger(lastValue(args, "max-gram-length"), "--max-gram-length"),
  };
  resolveDiscoveryGramRange(options);
  return options;
}

function formatScope(category: SearchCategory, subcategory?: SearchSubcategory | null): string {
  if (subcategory) {
    return `${category}/${subcategory}`;
  }

  return category;
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

function renderCandidate(candidate: SemanticDiscoveryCandidate): string[] {
  const sharedTraits = candidate.sharedTraits.length > 0 ? candidate.sharedTraits.join(", ") : "(none)";
  return [
    `- ${candidate.name} [level ${formatLevel(candidate.level)}] score=${formatSimilarity(candidate.similarity)} shared_traits=${sharedTraits}`,
    `  traits=${formatTraits(candidate.traits)} derived_tags=${formatTraits(candidate.derivedTags)}`,
    `  ${trimDescription(candidate.descriptionText)}`,
  ];
}

export function formatSemanticDiscoveryReport(result: SemanticDiscoveryResult): string {
  const lines: string[] = [];
  lines.push("Query summary:");
  lines.push(`- Candidate scope: ${formatScope(result.category, result.subcategory)}`);
  lines.push(`- Resolved exemplars: ${result.exemplarCount}`);
  lines.push(`- Candidates considered: ${result.candidateCount}`);
  lines.push(`- Candidates at/above threshold: ${result.matchedCandidateCount}`);
  lines.push("");
  lines.push("Resolved exemplars:");
  for (const exemplar of result.resolvedExemplars) {
    lines.push(
      `- ${exemplar.query} -> ${exemplar.name} (${exemplar.recordKey}) [${exemplar.matchedBy}] level=${formatLevel(exemplar.level)} traits=${formatTraits(exemplar.traits)}`,
    );
  }
  lines.push("");
  lines.push("Representative exemplars:");
  for (const exemplar of result.exemplars) {
    lines.push(
      `- ${exemplar.name} [level ${formatLevel(exemplar.level)}] score=${formatSimilarity(exemplar.similarityToCentroid)} traits=${formatTraits(exemplar.traits)}`,
    );
  }
  lines.push("");
  lines.push("Shared evidence:");
  lines.push(`- Common traits: ${formatTraits(result.commonTraits)}`);
  lines.push(
    `- Shared tokens: ${result.sharedTokens.length > 0 ? result.sharedTokens.map((entry) => `${entry.value} (e${entry.exemplarSupport}/c${entry.candidateSupport})`).join(", ") : "(none)"}`,
  );
  lines.push(
    `- Shared phrases: ${result.sharedPhrases.length > 0 ? result.sharedPhrases.map((entry) => `${entry.value} (e${entry.exemplarSupport}/c${entry.candidateSupport})`).join(", ") : "(none)"}`,
  );
  lines.push("");
  lines.push("Top candidates:");
  for (const candidate of result.candidates) {
    lines.push(...renderCandidate(candidate));
  }
  lines.push("");
  lines.push("Contrast records:");
  for (const contrast of result.contrastRecords) {
    lines.push(...renderCandidate(contrast));
  }
  lines.push("");
  lines.push("Coverage hints:");
  for (const bucket of result.similarityBuckets) {
    lines.push(`- >= ${bucket.minSimilarity.toFixed(2)}: ${bucket.count}`);
  }
  lines.push("");
  lines.push("Interpretation notes:");
  lines.push(
    "- Similarity is discovery support only. Convert recurring evidence into explicit, explainable derived-tag rules before tagging.",
  );
  lines.push(
    "- If contrast records look as plausible as top candidates, the concept may be too subjective to model safely.",
  );

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
    const result = discoverSemanticCandidates(db, options);
    console.log(formatSemanticDiscoveryReport(result));
  } finally {
    db.close();
  }
}

function isDirectExecution(): boolean {
  const argvPath = process.argv[1];
  if (!argvPath) {
    return false;
  }

  return import.meta.url === pathToFileURL(argvPath).href;
}

if (isDirectExecution()) {
  main().catch((error) => {
    console.error(`Semantic discovery failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
