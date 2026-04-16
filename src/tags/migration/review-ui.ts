import { importDerivedTagMigrationSession } from "./importer.js";
import { lintDerivedTagMigrationSession } from "./linter.js";
import { renderDerivedTagMigrationReviewItem, renderDerivedTagMigrationSessionSummary } from "./render.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "./review-session.js";
import { writeDerivedTagMigrationSummary } from "./cli-utils.js";
import { writeDerivedTagMigrationSession } from "./session-store.js";
import {
  clearTerminalScreen,
  moveSelection,
  moveSelectionWrapped,
  pauseForAnyKey,
  readTerminalKey,
  terminalTheme,
} from "./terminal-ui.js";
import type { DerivedTagMigrationSession } from "./types.js";

export type DerivedTagMigrationReviewResult = {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

async function persistSession(rootPath: string, session: DerivedTagMigrationSession): Promise<void> {
  await writeDerivedTagMigrationSession(rootPath, session);
  await writeDerivedTagMigrationSummary(rootPath, session.manifest.id, renderDerivedTagMigrationSessionSummary(session));
}

const REVIEW_ACTIONS = [
  { id: "approve", label: "Approve" },
  { id: "reject", label: "Reject" },
  { id: "needs_review", label: "Needs Review" },
  { id: "toggle_unresolved", label: "Toggle Unresolved" },
  { id: "import", label: "Lint + Import" },
  { id: "quit", label: "Quit" },
] as const;

type ReviewActionId = (typeof REVIEW_ACTIONS)[number]["id"];

function renderReviewActionBar(selectedActionIndex: number): string {
  const labels = REVIEW_ACTIONS
    .map((action, index) => {
      const baseLabel = action.id === "approve"
        ? terminalTheme.positiveAction(action.label)
        : action.id === "reject"
          ? terminalTheme.negativeAction(action.label)
          : action.id === "needs_review"
            ? terminalTheme.cautionAction(action.label)
            : action.id === "quit"
              ? terminalTheme.warning(action.label)
              : terminalTheme.neutralAction(action.label);
      return index === selectedActionIndex ? terminalTheme.selectedAction(action.label) : baseLabel;
    });
  return [
    terminalTheme.dim("Controls: Up/Down or j/k move item  Left/Right or h/l choose action  Enter apply  ? help  q quit"),
    labels.join("  "),
  ].join("\n");
}

function clampActionIndex(currentIndex: number): number {
  return moveSelection(currentIndex, 0, REVIEW_ACTIONS.length);
}

function renderReviewHelp(): string {
  return [
    terminalTheme.heading("Review Help"),
    "",
    "Navigation:",
    "- Up / Down or j / k: move between review items",
    "- Left / Right or h / l: move between available actions",
    "- Movement wraps at the ends of the list and action bar",
    "",
    "Decision actions:",
    "- Enter: apply the currently highlighted action chip",
    "- a: approve current item",
    "  Confirm that the proposed assignment, exemplar decision, or rule decision is correct and should be imported.",
    "- r: reject current item",
    "  Mark the current proposal as not accepted. Rejected assignment decisions remain as review metadata; rejected exemplar review items are cleared from the pending queue without changing live exemplars.",
    "- n: mark current item as needs_review",
    "  Put the item back into the unresolved queue so it remains visible in future passes.",
    "",
    "Workflow actions:",
    "- t: toggle unresolved-only filtering",
    "  Switch between only unresolved items and the full session, including already reviewed decisions.",
    "- i: lint and import the session",
    "  Validate that the session is internally legal, then write approved outcomes back into authored assignments, exemplars, and future authored rules.",
    "- q: save and quit",
    "  Persist scratch review progress without importing anything.",
    "",
    "Display:",
    "- The current record view is the selected review item",
    "- The highlighted action chip is the action Enter will execute",
    "- approved and auto-applied assignment decisions feed live applied/excluded state; needs_review and rejected remain review metadata",
    "- approved exemplar review decisions update live exemplar files and then leave the review queue",
  ].join("\n");
}

export async function runDerivedTagMigrationReviewUi(
  rootPath: string,
  initialSession: DerivedTagMigrationSession,
): Promise<DerivedTagMigrationReviewResult> {
  let session = clampDerivedTagMigrationReviewIndex(initialSession);
  let imported = false;
  let selectedActionIndex = 0;

  while (true) {
    selectedActionIndex = clampActionIndex(selectedActionIndex);
    clearTerminalScreen();
    const items = getDerivedTagMigrationReviewItems(session);
    console.log(items.length > 0
      ? renderDerivedTagMigrationReviewItem(session, session.reviewState.currentIndex, renderReviewActionBar(selectedActionIndex))
      : renderDerivedTagMigrationReviewItem(session, session.reviewState.currentIndex, renderReviewActionBar(selectedActionIndex)));

    const key = await readTerminalKey();
    const normalized = key.sequence.toLowerCase();

    if (key.ctrl && key.name === "c") {
      break;
    }

    let requestedAction: ReviewActionId | undefined;

    if (key.name === "up" || normalized === "k") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, -1, items.length);
      }
    } else if (key.name === "down" || normalized === "j") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, 1, items.length);
      }
    } else if (key.name === "left" || normalized === "h") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, -1, REVIEW_ACTIONS.length);
    } else if (key.name === "right" || normalized === "l") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, 1, REVIEW_ACTIONS.length);
    } else if (key.name === "return" || key.name === "enter") {
      requestedAction = REVIEW_ACTIONS[selectedActionIndex]?.id;
    } else if (normalized === "?") {
      await pauseForAnyKey(renderReviewHelp());
    } else if (normalized === "a") {
      requestedAction = "approve";
    } else if (normalized === "r") {
      requestedAction = "reject";
    } else if (normalized === "n") {
      requestedAction = "needs_review";
    } else if (normalized === "t") {
      requestedAction = "toggle_unresolved";
    } else if (normalized === "i") {
      requestedAction = "import";
    } else if (normalized === "q") {
      requestedAction = "quit";
    }

    if (requestedAction === "quit") {
      break;
    }
    if (requestedAction === "toggle_unresolved") {
      session = clampDerivedTagMigrationReviewIndex(toggleDerivedTagMigrationUnresolvedOnly(session));
    } else if (requestedAction === "approve" && items.length > 0) {
      session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
        session,
        items[session.reviewState.currentIndex]!,
        "approved",
      ));
    } else if (requestedAction === "reject" && items.length > 0) {
      session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
        session,
        items[session.reviewState.currentIndex]!,
        "rejected",
      ));
    } else if (requestedAction === "needs_review" && items.length > 0) {
      session = clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
        session,
        items[session.reviewState.currentIndex]!,
        "needs_review",
      ));
    } else if (requestedAction === "import") {
      try {
        lintDerivedTagMigrationSession(session);
        await importDerivedTagMigrationSession(rootPath, session);
        imported = true;
        await persistSession(rootPath, session);
        await pauseForAnyKey(`Imported session ${session.manifest.id}.`);
        break;
      } catch (error) {
        await pauseForAnyKey(`Import failed: ${(error as Error).message}`);
      }
    }

    await persistSession(rootPath, session);
  }

  await persistSession(rootPath, session);
  return { imported, session };
}
