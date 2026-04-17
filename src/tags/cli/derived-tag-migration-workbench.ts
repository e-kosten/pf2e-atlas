#!/usr/bin/env node

import { runPf2eTerminalApp } from "../../tui/pf2e-app.js";

async function main(): Promise<void> {
  await runPf2eTerminalApp(process.cwd(), process.argv.slice(2));
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration workbench failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
