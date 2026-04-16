#!/usr/bin/env node

import { openConfiguredIndex, lastValue, parseCliArgs, parseInteger, writeDerivedTagMigrationSummary } from "../migration/cli-utils.js";
import { renderDerivedTagMigrationSessionSummary } from "../migration/render.js";
import { writeDerivedTagMigrationSession } from "../migration/session-store.js";
import { buildDerivedTagMigrationSession } from "../migration/session-builder.js";
import type { DerivedTagMigrationMode } from "../migration/types.js";
import type { SearchCategory, SearchSubcategory } from "../../types.js";

const MODES: DerivedTagMigrationMode[] = [
  "review_queue",
  "legacy_seed",
  "legacy_rule",
  "exemplar_cleanup",
  "new_tagging",
];

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
  const mode = lastValue(args, "mode") as DerivedTagMigrationMode | undefined;
  if (!mode || !MODES.includes(mode)) {
    throw new Error(`Pass --mode with one of: ${MODES.join(", ")}.`);
  }

  const { db } = await openConfiguredIndex(argv);
  try {
    const session = buildDerivedTagMigrationSession(db, {
      mode,
      category: lastValue(args, "category") as SearchCategory | undefined,
      subcategory: lastValue(args, "subcategory") as SearchSubcategory | undefined,
      decisionKind: lastValue(args, "decision-kind") as "assignment" | "exemplar" | undefined,
      family: lastValue(args, "family"),
      tag: lastValue(args, "tag"),
      limit: parseInteger(lastValue(args, "limit"), "--limit"),
      exemplarLimit: parseInteger(lastValue(args, "exemplar-limit"), "--exemplar-limit"),
    });

    await writeDerivedTagMigrationSession(process.cwd(), session);
    const summary = renderDerivedTagMigrationSessionSummary(session);
    await writeDerivedTagMigrationSummary(process.cwd(), session.manifest.id, summary);
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
