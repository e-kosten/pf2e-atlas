#!/usr/bin/env node

import { createInterface } from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";

import { lastValue, parseCliArgs, writeDerivedTagMigrationSummary } from "../migration/cli-utils.js";
import { renderDerivedTagMigrationReviewItem, renderDerivedTagMigrationSessionSummary } from "../migration/render.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "../migration/review-session.js";
import { readDerivedTagMigrationSession, writeDerivedTagMigrationSession } from "../migration/session-store.js";

function clearScreen(): void {
  output.write("\x1B[2J\x1B[0f");
}

async function main(): Promise<void> {
  const argv = process.argv.slice(2);
  const args = parseCliArgs(argv);
  const sessionId = lastValue(args, "session");
  if (!sessionId) {
    throw new Error("Pass --session <session-id>.");
  }

  let session = await readDerivedTagMigrationSession(process.cwd(), sessionId);
  session = clampDerivedTagMigrationReviewIndex(session);
  const rl = createInterface({ input, output });

  try {
    while (true) {
      clearScreen();
      const items = getDerivedTagMigrationReviewItems(session);
      console.log(items.length > 0
        ? renderDerivedTagMigrationReviewItem(session, session.reviewState.currentIndex)
        : renderDerivedTagMigrationSessionSummary(session));

      const answer = (await rl.question("\nreview> ")).trim().toLowerCase();
      if (answer === "q") {
        break;
      }
      if (answer === "j" || answer === "next") {
        if (items.length > 0) {
          session.reviewState.currentIndex = Math.min(session.reviewState.currentIndex + 1, items.length - 1);
        }
      } else if (answer === "k" || answer === "prev") {
        if (items.length > 0) {
          session.reviewState.currentIndex = Math.max(session.reviewState.currentIndex - 1, 0);
        }
      } else if (answer === "t") {
        session = clampDerivedTagMigrationReviewIndex(toggleDerivedTagMigrationUnresolvedOnly(session));
      } else if ((answer === "a" || answer === "approve") && items.length > 0) {
        session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
          session,
          items[session.reviewState.currentIndex]!,
          "approved",
        ));
      } else if ((answer === "r" || answer === "reject") && items.length > 0) {
        session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
          session,
          items[session.reviewState.currentIndex]!,
          "rejected",
        ));
      } else if ((answer === "n" || answer === "needs-review") && items.length > 0) {
        session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
          session,
          items[session.reviewState.currentIndex]!,
          "needs_review",
        ));
      }

      await writeDerivedTagMigrationSession(process.cwd(), session);
      await writeDerivedTagMigrationSummary(process.cwd(), session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
    }
  } finally {
    rl.close();
  }

  await writeDerivedTagMigrationSession(process.cwd(), session);
  await writeDerivedTagMigrationSummary(process.cwd(), session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
}

if (import.meta.url === new URL(process.argv[1] ?? "", "file:").href) {
  main().catch((error) => {
    console.error(`Derived-tag migration review failed: ${(error as Error).message}`);
    process.exit(1);
  });
}
