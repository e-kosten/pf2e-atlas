#!/usr/bin/env node

import {
  lastValue,
  parseCliArgs,
  parseInteger,
} from "../shared/arg-parsing.js";
import { openEditorialConfiguredIndex } from "../../editorial/configured-index.js";
import { writeDerivedTagReviewSummary } from "../../editorial/writeback/review-summary.js";
import { renderDerivedTagReviewSessionSummary } from "../../editorial/ui/render.js";
import { writeDerivedTagReviewSession } from "../../editorial/sessions/session-store.js";
import { buildDerivedTagReviewSession } from "../../editorial/sessions/session-builder.js";
import type { DerivedTagWorkbenchMode } from "../../editorial/types.js";
import {
  parseOptionalScopedSearchSubcategoryArg,
  parseOptionalSearchCategoryArg,
} from "../shared/search-scope-args.js";

const MODES: DerivedTagWorkbenchMode[] = [
  "review_queue",
  "proposal_review",
  "legacy_seed",
  "legacy_rule",
  "exemplar_cleanup",
];

function parseMode(value: string | undefined): DerivedTagWorkbenchMode | undefined {
  switch (value) {
    case undefined:
      return undefined;
    case "new_tagging":
      return "proposal_review";
    case "review_queue":
    case "proposal_review":
    case "legacy_seed":
    case "legacy_rule":
    case "exemplar_cleanup":
      return value;
    default:
      return undefined;
  }
}

function parseDecisionKind(value: string | undefined): "assignment" | "exemplar" | undefined {
  switch (value) {
    case "assignment":
    case "exemplar":
      return value;
    case undefined:
      return undefined;
    default:
      throw new Error(`Expected --decision-kind to be "assignment" or "exemplar", received "${value}".`);
  }
}

function renderHelp(): string {
  return [
    "Usage: npm run create-derived-tag-migration-session -- --mode <mode> [options]",
    "",
    `Modes: ${MODES.join(", ")}`,
    "Options:",
    "  --category <category>",
    "  --subcategory <subcategory>",
    "  --decision-kind <assignment|exemplar>",
    "  --family <family>",
    "  --tag <tag>",
    "  --limit <n>",
    "  --exemplar-limit <n>",
  ].join("\n");
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(renderHelp());
    return;
  }

  const args = parseCliArgs(argv);
  const mode = parseMode(lastValue(args, "mode"));
  if (!mode) {
    throw new Error(
      `Pass --mode with one of: ${MODES.join(", ")}. "new_tagging" remains available as a compatibility alias for "proposal_review".`,
    );
  }

  const { db } = await openEditorialConfiguredIndex(argv);
  try {
    const category = parseOptionalSearchCategoryArg(lastValue(args, "category"), "--category");
    const session = buildDerivedTagReviewSession(db, {
      mode,
      category,
      subcategory: parseOptionalScopedSearchSubcategoryArg(category, lastValue(args, "subcategory"), "--subcategory"),
      decisionKind: parseDecisionKind(lastValue(args, "decision-kind")),
      family: lastValue(args, "family"),
      tag: lastValue(args, "tag"),
      limit: parseInteger(lastValue(args, "limit"), "--limit"),
      exemplarLimit: parseInteger(lastValue(args, "exemplar-limit"), "--exemplar-limit"),
    });

    await writeDerivedTagReviewSession(process.cwd(), session);
    const summary = renderDerivedTagReviewSessionSummary(session);
    await writeDerivedTagReviewSummary(process.cwd(), session.manifest.id, summary);
    console.log(summary);
    console.log(`\nSession written to scratch/migration-sessions/${session.manifest.id}`);
  } finally {
    db.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to create derived-tag migration session: ${(error as Error).message}`);
    process.exit(1);
  });
}
