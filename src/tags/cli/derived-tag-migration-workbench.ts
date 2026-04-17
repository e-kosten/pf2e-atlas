#!/usr/bin/env node

import { runDerivedTagMigrationWorkbenchApp } from "../migration/workbench-ui.js";

async function main(): Promise<void> {
  await runDerivedTagMigrationWorkbenchApp(process.cwd(), process.argv.slice(2));
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration workbench failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
