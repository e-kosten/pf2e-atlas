#!/usr/bin/env node

import { lastValue, parseCliArgs } from "../shared/arg-parsing.js";
import { readDerivedTagTranslationReviewSession } from "../../editorial/sessions/translation-session-store.js";
import { importDerivedTagTranslationReviewSession } from "../../editorial/writeback/translation-session-importer.js";

async function main(): Promise<void> {
  const args = parseCliArgs(process.argv.slice(2));
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  const session = await readDerivedTagTranslationReviewSession(process.cwd(), sessionId);
  await importDerivedTagTranslationReviewSession(process.cwd(), session);
  console.log(`Imported derived-tag translation session ${sessionId}.`);
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to import derived-tag translation session: ${(error as Error).message}`);
    process.exit(1);
  });
}
