#!/usr/bin/env node

import { openConfiguredPf2eApplicationIndex } from "../../../app/storage-service.js";
import { lastValue, parseCliArgs, parseInteger } from "../shared/arg-parsing.js";
import { writeDerivedTagReviewSummary } from "../../editorial/writeback/review-summary.js";
import { renderDerivedTagReviewSessionSummary } from "../../editorial/ui/render.js";
import { writeDerivedTagReviewSession } from "../../editorial/sessions/session-store.js";
import { buildDerivedTagReviewSession } from "../../editorial/sessions/session-builder.js";
import { DERIVED_TAG_WORKBENCH, type DerivedTagReviewDecisionKind } from "../../editorial/types.js";
import {
  DERIVED_TAG_WORKBENCH_MODE_ALIASES_LIST,
  parseDerivedTagWorkbenchMode,
} from "../../editorial/mode-registry.js";
import {
  parseOptionalScopedSearchSubcategoryArg,
  parseOptionalSearchCategoryArg,
} from "../shared/search-scope-args.js";

const MODE_ALIASES_NOTE = DERIVED_TAG_WORKBENCH_MODE_ALIASES_LIST.join(", ");
const DECISION_KIND_BY_TEXT = {
  assignment: "assignment",
  exemplar: "exemplar",
} as const satisfies Record<string, DerivedTagReviewDecisionKind>;

function parseDecisionKind(value: string | undefined): DerivedTagReviewDecisionKind | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (!(value in DECISION_KIND_BY_TEXT)) {
    throw new Error(`Expected --decision-kind to be "assignment" or "exemplar", received "${value}".`);
  }

  return DECISION_KIND_BY_TEXT[value as keyof typeof DECISION_KIND_BY_TEXT];
}

function renderHelp(): string {
  return [
    "Usage: cd src/tags/cli && npm run create-derived-tag-migration-session -- --mode <mode> [options]",
    "",
    `Modes: ${DERIVED_TAG_WORKBENCH.MODES.join(", ")}`,
    ...(MODE_ALIASES_NOTE.length > 0 ? [`Aliases: ${MODE_ALIASES_NOTE}`] : []),
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
  const mode = parseDerivedTagWorkbenchMode(lastValue(args, "mode"));
  if (!mode) {
    throw new Error(
      `Pass --mode with one of: ${DERIVED_TAG_WORKBENCH.MODES.join(", ")}.` +
        `${MODE_ALIASES_NOTE.length > 0 ? ` ${MODE_ALIASES_NOTE} remains available as compatibility aliases.` : ""}`,
    );
  }

  const handle = await openConfiguredPf2eApplicationIndex(argv);
  try {
    const category = parseOptionalSearchCategoryArg(lastValue(args, "category"), "--category");
    const session = buildDerivedTagReviewSession(handle.db, {
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
    handle.close();
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to create derived-tag migration session: ${(error as Error).message}`);
    process.exit(1);
  });
}
