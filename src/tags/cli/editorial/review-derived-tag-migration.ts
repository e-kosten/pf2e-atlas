#!/usr/bin/env node

import { lastValue, parseCliArgs } from "../../editorial/cli-utils.js";
import { runDerivedTagMigrationReviewUi } from "../../editorial/ui/review-ui.js";
import { readDerivedTagMigrationSession } from "../../editorial/sessions/session-store.js";

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const args = parseCliArgs(argv);
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  const session = await readDerivedTagMigrationSession(process.cwd(), sessionId);
  await runDerivedTagMigrationReviewUi(process.cwd(), session);
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration review failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
