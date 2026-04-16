#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../app/config.js";
import { SearchCategory, SearchSubcategory } from "../../types.js";
import {
  isReviewedDiscoveryReason,
  type ReviewedDiscoveryApplicationSummary,
} from "../discovery/discovery-reviewed-records.js";
import {
  analyzeDiscoveryEvidence,
  type DiscoveryEvidenceOptions,
  type DiscoveryEvidenceReport,
} from "../evaluation/evidence-analyzer.js";
import { resolveDiscoveryGramRange } from "../discovery/discovery-normalization.js";

type MultiValueArgs = Record<string, string[]>;

function wantsHelp(argv: string[]): boolean {
  return argv.includes("--help") || argv.includes("-h");
}

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
    family: lastValue(args, "family"),
    excludeDerivedTag: lastValue(args, "exclude-derived-tag"),
    untaggedOnly: hasFlag(args, "untagged"),
    familyGapSignals: hasFlag(args, "family-gap-signals"),
    includeReviewed: hasFlag(args, "include-reviewed"),
    reviewReason: lastValue(args, "review-reason"),
    limit: parseInteger(lastValue(args, "limit"), "--limit"),
    exampleLimit: parseInteger(lastValue(args, "example-limit"), "--example-limit"),
    minGramLength: parseInteger(lastValue(args, "min-gram-length"), "--min-gram-length"),
    maxGramLength: parseInteger(lastValue(args, "max-gram-length"), "--max-gram-length"),
  };
  if (options.tag && options.family) {
    throw new Error("Choose either --tag <derived-tag> or --family <derived-tag-family>, not both.");
  }
  if (options.family && !options.category) {
    throw new Error("Pass --category <category> when using --family <derived-tag-family>.");
  }
  if (options.familyGapSignals && !options.family) {
    throw new Error("Pass --family <derived-tag-family> when using --family-gap-signals.");
  }
  if ((options.includeReviewed || options.reviewReason) && !options.family) {
    throw new Error("Pass --family <derived-tag-family> when using reviewed-discovery controls.");
  }
  if ((options.includeReviewed || options.reviewReason) && !options.untaggedOnly && !options.familyGapSignals) {
    throw new Error("Reviewed-discovery controls require --untagged or --family-gap-signals.");
  }
  if (options.reviewReason && !options.includeReviewed) {
    throw new Error("Pass --include-reviewed when using --review-reason.");
  }
  if (options.reviewReason && !isReviewedDiscoveryReason(options.reviewReason)) {
    throw new Error(`Unknown --review-reason "${options.reviewReason}".`);
  }
  resolveDiscoveryGramRange(options);
  return {
    ...options,
    reviewReason: options.reviewReason && isReviewedDiscoveryReason(options.reviewReason)
      ? options.reviewReason
      : undefined,
  };
}

export function formatHelp(): string {
  return [
    "Usage:",
    "  npm run analyze-derived-tag-evidence -- --category <category> [options]",
    "",
    "Scope:",
    "  --category <category>              Required for normal category-scoped and family-scoped analysis",
    "  --subcategory <subcategory>       Narrow the analysis scope within the category",
    "  --record-key <canonical-key>      Repeatable explicit cohort records",
    "  --exclude-record-key <key>        Repeatable exclusions",
    "",
    "Cohort selection:",
    "  --tag <derived-tag>               Analyze one existing derived tag",
    "  --family <derived-tag-family>     Analyze all records with tags from one derived-tag family",
    "  --untagged                        Analyze records missing tags in the selected family, or fully untagged records when no family is given",
    "  --exclude-derived-tag <tag>       Exclude records that already have one derived tag",
    "  --family-gap-signals              Re-rank family-scoped evidence toward missing-family concepts instead of raw family-missing evidence",
    "  --include-reviewed                Include reviewed-negative family-gap records instead of excluding them by default",
    "  --review-reason <reason>          Audit one reviewed-negative reason bucket (requires --include-reviewed)",
    "",
    "Output shaping:",
    "  --limit <n>                       Maximum ranked evidence terms per section",
    "  --example-limit <n>               Maximum examples per surfaced term",
    "  --min-gram-length <n>             Minimum phrase length, default analyzer range",
    "  --max-gram-length <n>             Maximum phrase length, default analyzer range",
    "",
    "Examples:",
    "  npm run analyze-derived-tag-evidence -- --category creature --family setting",
    "  npm run analyze-derived-tag-evidence -- --category creature --family setting --untagged --limit 12",
    "  npm run analyze-derived-tag-evidence -- --category creature --family setting --family-gap-signals --include-reviewed --review-reason not_family_salient",
    "  npm run analyze-derived-tag-evidence -- --category creature --tag fortress_setting",
  ].join("\n");
}

function formatReviewedSummary(summary: ReviewedDiscoveryApplicationSummary): string[] {
  const label = summary.mode === "excluded"
    ? "Excluded reviewed records"
    : summary.mode === "included"
      ? "Included reviewed records"
      : `Filtered reviewed records${summary.reviewReason ? ` (${summary.reviewReason})` : ""}`;
  return [
    `${label}: ${summary.appliedCount}/${summary.scopedCount}`,
    `Reviewed reason counts: ${summary.reasonCounts.length > 0
      ? summary.reasonCounts.map((entry) => `${entry.reason}=${entry.count}`).join(", ")
      : "(none)"}`,
  ];
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
    ...(report.family ? [`- Family: ${report.family}`] : []),
    `- Cohort size: ${report.cohortSize}`,
    `- Baseline size: ${report.baselineSize}`,
    ...(report.familyGap
      ? [
        `- Covered family records: ${report.familyGap.coveredCount}`,
        `- Uncovered family records: ${report.familyGap.uncoveredCount}`,
        `- Live family tags: ${report.familyGap.liveTags.join(", ") || "(none)"}`,
      ]
      : []),
    ...(report.reviewedRecords
      ? formatReviewedSummary(report.reviewedRecords).map((line) => `- ${line}`)
      : []),
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
    ...(report.familyGap
      ? [
        "",
        "Likely new concepts:",
        ...(report.familyGap.likelyNewConcepts.length > 0
          ? report.familyGap.likelyNewConcepts.map((term) =>
            `- ${term.value} gap_lift=${term.gapLift.toFixed(2)} covered=${term.coveredSupport} baseline=${term.baselineSupport} score=${term.score.toFixed(2)} examples=${term.examples.join(" | ")}`)
          : ["- (none)"]),
        "",
        "Likely existing-tag coverage gaps:",
        ...(report.familyGap.existingTagCoverageGaps.length > 0
          ? report.familyGap.existingTagCoverageGaps.map((term) =>
            `- ${term.value} overlaps=${term.existingTagOverlaps.join(", ")} gap_lift=${term.gapLift.toFixed(2)} covered=${term.coveredSupport} score=${term.score.toFixed(2)} examples=${term.examples.join(" | ")}`)
          : ["- (none)"]),
        "",
        "Suppressed generic anchors:",
        ...(report.familyGap.suppressedTerms.length > 0
          ? report.familyGap.suppressedTerms.map((term) =>
            `- ${term.value} reason=${term.suppressionReason ?? "suppressed"} score=${term.score.toFixed(2)} examples=${term.examples.join(" | ")}`)
          : ["- (none)"]),
      ]
      : []),
  ];

  return lines.join("\n");
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  if (wantsHelp(argv)) {
    console.log(formatHelp());
    return;
  }
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
