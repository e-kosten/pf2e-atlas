#!/usr/bin/env node

import { lastValue, parseCliArgs } from "../shared/arg-parsing.js";
import { lintDerivedTagReviewSession } from "../../editorial/writeback/linter.js";
import { readDerivedTagReviewSession } from "../../editorial/sessions/session-store.js";
import { renderDerivedTagReviewSessionSummary } from "../../editorial/ui/render.js";

async function main(): Promise<void> {
  const args = parseCliArgs(process.argv.slice(2));
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  const session = await readDerivedTagReviewSession(process.cwd(), sessionId);
  lintDerivedTagReviewSession(session);
  console.log(renderDerivedTagReviewSessionSummary(session));
  console.log("\nSession lint passed.");
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to lint derived-tag migration session: ${(error as Error).message}`);
    process.exit(1);
  });
}
