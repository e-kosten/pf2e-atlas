#!/usr/bin/env node

import { lastValue, parseCliArgs } from "../migration/cli-utils.js";
import { lintDerivedTagMigrationSession } from "../migration/linter.js";
import { readDerivedTagMigrationSession } from "../migration/session-store.js";
import { renderDerivedTagMigrationSessionSummary } from "../migration/render.js";

async function main(): Promise<void> {
  const args = parseCliArgs(process.argv.slice(2));
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  const session = await readDerivedTagMigrationSession(process.cwd(), sessionId);
  lintDerivedTagMigrationSession(session);
  console.log(renderDerivedTagMigrationSessionSummary(session));
  console.log("\nSession lint passed.");
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to lint derived-tag migration session: ${(error as Error).message}`);
    process.exit(1);
  });
}
