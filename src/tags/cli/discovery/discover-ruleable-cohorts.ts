#!/usr/bin/env node

import { access } from "node:fs/promises";
import { constants } from "node:fs";
import { DatabaseSync } from "node:sqlite";

import { loadConfig } from "../../../app/config.js";
import { discoverRuleableCohorts, type RuleableCohortReport } from "../../discovery/cohort-discovery.js";
import { formatDiscoverySourceContext } from "../../discovery/discovery-source-report.js";
import { parseOptions } from "./cluster-derived-tag-candidates.js";

export function formatRuleableCohortReport(report: RuleableCohortReport): string {
  const scope = report.subcategory ? `${report.category}/${report.subcategory}` : report.category;
  const lines = [
    "Ruleable cohort summary:",
    `- Scope: ${scope}`,
    `- Source tag: ${report.sourceTag ?? "(seed exemplars)"}`,
    `- Exemplars: ${report.exemplarCount}`,
    `- Ranked candidates: ${report.candidateCount}`,
    "",
    "Top cohorts:",
    ...(report.cohorts.length > 0
      ? report.cohorts.flatMap((cohort) => [
          `- ${cohort.recommendation} score=${cohort.score.toFixed(2)} size=${cohort.size} families=${cohort.distinctVariantFamilies} sources=${cohort.sourceCount} publications=${cohort.publicationCount} source_slices=${cohort.sourceSliceCount} signature=${cohort.signature.join(", ") || "(semantic only)"} non_name=${cohort.nonNameAnchors.join(", ") || "(none)"} flags=${cohort.reviewFlags.join(", ") || "(none)"}`,
          `  ${formatDiscoverySourceContext(cohort)}`,
        ])
      : ["- (none)"]),
    "",
    "Anchor terms:",
    ...(report.anchorTerms.length > 0
      ? report.anchorTerms.map((term) => `- ${term.value} support=${term.cohortSupport} lift=${term.lift.toFixed(2)}`)
      : ["- (none)"]),
    "",
    "Contrast records:",
    ...(report.contrastRecords.length > 0
      ? report.contrastRecords.map(
          (record) => `- ${record.name} (${record.recordKey}) score=${record.similarity.toFixed(3)}`,
        )
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
    console.log(formatRuleableCohortReport(discoverRuleableCohorts(db, options)));
  } finally {
    db.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Ruleable cohort discovery failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
