#!/usr/bin/env node

import { summarizeCurrentDerivedTagReviewQueue } from "../migration/runtime-state.js";

async function main(): Promise<void> {
  const summary = summarizeCurrentDerivedTagReviewQueue();
  if (summary.length === 0) {
    console.log("No derived-tag review items are currently pending.");
    return;
  }

  console.log("Derived-tag review queue:");
  for (const item of summary) {
    const scope = item.kind === "assignment"
      ? `${item.category} ${item.family}.${item.tag}`
      : `${item.category} exemplar.${item.tag}`;
    console.log(`- ${scope} confidence=${item.confidence} count=${item.count}`);
  }
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Failed to summarize derived-tag review queue: ${(error as Error).message}`);
    process.exit(1);
  });
}
