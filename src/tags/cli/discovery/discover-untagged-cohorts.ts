#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../../app/config.js";
import { ConsoleProgressReporter } from "../../../shared/progress.js";
import {
  parseOptionalScopedSearchSubcategoryArg,
  parseRequiredSearchCategoryArg,
} from "../shared/search-scope-args.js";
import {
  isReviewedDiscoveryReason,
  type ReviewedDiscoveryApplicationSummary,
} from "../../reviews/discovery-reviewed-records.js";
import {
  discoverUntaggedCohorts,
  type UntaggedCohortOptions,
  type UntaggedCohortReport,
} from "../../discovery/untagged-cohort-discovery.js";
import { resolveDiscoveryGramRange } from "../../discovery/discovery-normalization.js";
import { formatDiscoverySourceContext } from "../../discovery/discovery-source-report.js";

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
  const category = parseRequiredSearchCategoryArg(lastValue(args, "category"), "--category");

  const options = {
    category,
    subcategory: parseOptionalScopedSearchSubcategoryArg(category, lastValue(args, "subcategory"), "--subcategory"),
    family: lastValue(args, "family"),
    familyGapSignals: hasFlag(args, "family-gap-signals"),
    includeReviewed: hasFlag(args, "include-reviewed"),
    reviewReason: lastValue(args, "review-reason"),
    cohortLimit: parseInteger(lastValue(args, "cohort-limit"), "--cohort-limit"),
    anchorLimit: parseInteger(lastValue(args, "anchor-limit"), "--anchor-limit"),
    minFeatureSupport: parseInteger(lastValue(args, "min-feature-support"), "--min-feature-support"),
    minFeatureLift: parseFloatValue(lastValue(args, "min-feature-lift"), "--min-feature-lift"),
    minGramLength: parseInteger(lastValue(args, "min-gram-length"), "--min-gram-length"),
    maxGramLength: parseInteger(lastValue(args, "max-gram-length"), "--max-gram-length"),
  };
  if (options.familyGapSignals && !options.family) {
    throw new Error("Pass --family <derived-tag-family> when using --family-gap-signals.");
  }
  if ((options.includeReviewed || options.reviewReason) && !options.family) {
    throw new Error("Pass --family <derived-tag-family> when using reviewed-discovery controls.");
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
    reviewReason:
      options.reviewReason && isReviewedDiscoveryReason(options.reviewReason) ? options.reviewReason : undefined,
  };
}

export function formatHelp(): string {
  return [
    "Usage:",
    "  npm run discover-untagged-cohorts -- --category <category> [options]",
    "",
    "Scope:",
    "  --category <category>             Required category scope",
    "  --subcategory <subcategory>      Narrow the discovery scope within the category",
    "  --family <derived-tag-family>    Restrict the missing-tag slice to one derived-tag family",
    "  --family-gap-signals             Re-rank family-scoped cohorts toward missing-family concepts and existing-tag overlap",
    "  --include-reviewed               Include reviewed-negative family-gap records instead of excluding them by default",
    "  --review-reason <reason>         Audit one reviewed-negative reason bucket (requires --include-reviewed)",
    "",
    "Discovery tuning:",
    "  --cohort-limit <n>               Maximum recommended cohorts to emit",
    "  --anchor-limit <n>               Maximum top anchor terms to emit",
    "  --min-feature-support <n>        Minimum support for surfaced features",
    "  --min-feature-lift <n>           Minimum lift for surfaced features",
    "  --min-gram-length <n>            Minimum phrase length, default analyzer range",
    "  --max-gram-length <n>            Maximum phrase length, default analyzer range",
    "",
    "Semantics:",
    "  Without --family, this command scans fully untagged canonical records.",
    "  With --family, it scans records missing tags from that family even if they already have tags from other families.",
    "",
    "Examples:",
    "  npm run discover-untagged-cohorts -- --category creature --family setting --cohort-limit 8 --anchor-limit 16",
    "  npm run discover-untagged-cohorts -- --category creature --family setting --include-reviewed --review-reason not_family_salient",
    "  npm run discover-untagged-cohorts -- --category equipment --subcategory gear --cohort-limit 8 --anchor-limit 16",
  ].join("\n");
}

function formatReviewedSummary(summary: ReviewedDiscoveryApplicationSummary): string[] {
  const label =
    summary.mode === "excluded"
      ? "Excluded reviewed records"
      : summary.mode === "included"
        ? "Included reviewed records"
        : `Filtered reviewed records${summary.reviewReason ? ` (${summary.reviewReason})` : ""}`;
  return [
    `- ${label}: ${summary.appliedCount}/${summary.scopedCount}`,
    `- Reviewed reason counts: ${
      summary.reasonCounts.length > 0
        ? summary.reasonCounts.map((entry) => `${entry.reason}=${entry.count}`).join(", ")
        : "(none)"
    }`,
  ];
}

export function formatUntaggedCohortReport(report: UntaggedCohortReport): string {
  const scope = report.subcategory ? `${report.category}/${report.subcategory}` : report.category;
  const lines = [
    "Untagged cohort summary:",
    `- Scope: ${scope}`,
    ...(report.family ? [`- Family: ${report.family}`] : []),
    `- Untagged records: ${report.untaggedRecordCount}`,
    `- Baseline records: ${report.baselineRecordCount}`,
    ...(report.coveredRecordCount !== undefined ? [`- Covered family records: ${report.coveredRecordCount}`] : []),
    ...(report.liveTags ? [`- Live family tags: ${report.liveTags.join(", ") || "(none)"}`] : []),
    ...(report.reviewedRecords ? formatReviewedSummary(report.reviewedRecords) : []),
    "",
    "Top anchors:",
    ...(report.anchorTerms.length > 0
      ? report.anchorTerms.map(
          (anchor) =>
            `- ${anchor.value} support=${anchor.support} baseline=${anchor.baselineSupport} lift=${anchor.lift.toFixed(2)} score=${anchor.score.toFixed(2)}${anchor.existingTagOverlaps && anchor.existingTagOverlaps.length > 0 ? ` overlaps=${anchor.existingTagOverlaps.join(", ")}` : ""}`,
        )
      : ["- (none)"]),
    "",
    "Recommended cohorts:",
    ...(report.cohorts.length > 0
      ? report.cohorts.flatMap((cohort) => [
          `- ${cohort.recommendation} score=${cohort.score.toFixed(2)} size=${cohort.size} families=${cohort.distinctVariantFamilies} sources=${cohort.sourceCount} publications=${cohort.publicationCount} source_slices=${cohort.sourceSliceCount} signature=${cohort.signature.join(", ")}${cohort.classification ? ` classification=${cohort.classification}` : ""}${cohort.familyGapRecommendation ? ` family_gap=${cohort.familyGapRecommendation}` : ""}`,
          ...(cohort.overlappingTags && cohort.overlappingTags.length > 0
            ? [`  overlaps=${cohort.overlappingTags.join(", ")}`]
            : []),
          `  non_name=${cohort.nonNameAnchors.join(", ") || "(none)"} flags=${cohort.reviewFlags.join(", ") || "(none)"}`,
          `  ${formatDiscoverySourceContext(cohort)}`,
          ...cohort.representativeRecords.map(
            (record) => `  ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`,
          ),
          ...(cohort.contrastRecords.length > 0
            ? [
                "  contrast:",
                ...cohort.contrastRecords.map(
                  (record) => `    ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`,
                ),
              ]
            : []),
        ])
      : ["- (none)"]),
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
