#!/usr/bin/env node

import { lastValue, parseCliArgs } from "../../editorial/cli-utils.js";
import { importDerivedTagMigrationSession } from "../../editorial/importer.js";
import { readDerivedTagMigrationSession } from "../../editorial/session-store.js";

async function main(): Promise<void> {
  const args = parseCliArgs(process.argv.slice(2));
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  const session = await readDerivedTagMigrationSession(process.cwd(), sessionId);
  await importDerivedTagMigrationSession(process.cwd(), session);
  console.log(`Imported derived-tag migration session ${sessionId}.`);
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to import derived-tag migration session: ${(error as Error).message}`);
    process.exit(1);
  });
}
