import { importDerivedTagMigrationSession } from "./importer.js";
import { lintDerivedTagMigrationSession } from "./linter.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  summarizeDerivedTagMigrationReviewProgress,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "./review-session.js";
import { buildDerivedTagMigrationRecordContextLines } from "./review-detail-content.js";
import { writeDerivedTagMigrationSummary } from "./cli-utils.js";
import { writeDerivedTagMigrationSession } from "./session-store.js";
import {
  getTerminalPaneBodyHeight,
  moveSelectionWrapped,
  pauseForAnyKey,
  readTerminalKey,
  readTerminalKeyOrResize,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSession,
} from "./terminal-ui.js";
import type { DerivedTagMigrationSession } from "./types.js";

export type DerivedTagMigrationReviewResult = {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

async function persistSession(rootPath: string, session: DerivedTagMigrationSession): Promise<void> {
  const progress = summarizeDerivedTagMigrationReviewProgress(session);
  const actionableSummary = progress.actionableRecordCount > 0
    ? `Actionable records resolved: ${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount}`
    : "Actionable review items: 0";
  await writeDerivedTagMigrationSession(rootPath, session);
  await writeDerivedTagMigrationSummary(rootPath, session.manifest.id, [
    `Session: ${session.manifest.id}`,
    `Mode: ${session.manifest.mode}`,
    `Candidate records: ${progress.candidateRecordCount}`,
    actionableSummary,
    `Visible review items: ${progress.visibleItemCount}`,
    `Updated at: ${session.reviewState.updatedAt}`,
  ].join("\n"));
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

function formatActionBar(selectedActionIndex: number): string {
  return `Actions: ${REVIEW_ACTIONS.map((action, index) => index === selectedActionIndex ? `[${action.label}]` : action.label).join("  ")}`;
}

function formatDecisionSummary(decision: DerivedTagMigrationSession["decisions"][number]["decisions"][number]): string {
  if (decision.kind === "assignment") {
    return `${decision.family}.${decision.tag} ${decision.mode}`;
  }
  if (decision.kind === "exemplar") {
    return `${decision.tag} exemplar ${decision.action}`;
  }
  return `${decision.tag} ${decision.decision}`;
}

function clampWindowStart(selectedIndex: number, itemCount: number, visibleCount: number): number {
  if (visibleCount <= 0 || itemCount <= visibleCount) {
    return 0;
  }
  const centered = selectedIndex - Math.floor(visibleCount / 2);
  return Math.max(0, Math.min(centered, itemCount - visibleCount));
}

function buildReviewListLines(
  terminalSession: DerivedTagTerminalSession,
  session: DerivedTagMigrationSession,
): DerivedTagTerminalLine[] {
  const items = getDerivedTagMigrationReviewItems(session);
  if (items.length === 0) {
    return [{ text: "No review items match the current filter.", tone: "dim" }];
  }

  const visibleCount = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const windowStart = clampWindowStart(session.reviewState.currentIndex, items.length, visibleCount);

  return items.slice(windowStart, windowStart + visibleCount).map((item, offset) => {
    const recordDecision = session.decisions[item.recordIndex]!;
    const decision = recordDecision.decisions[item.decisionIndex]!;
    const isSelected = windowStart + offset === session.reviewState.currentIndex;
    return {
      text: `${recordDecision.name} | ${decision.status} | ${formatDecisionSummary(decision)}`,
      tone: isSelected ? "selected" : "default",
      noWrap: true,
    };
  });
}

function buildSelectedReviewDetailLines(session: DerivedTagMigrationSession): DerivedTagTerminalLine[] {
  const items = getDerivedTagMigrationReviewItems(session);
  if (items.length === 0) {
    return [
      { text: `Session ${session.manifest.id}`, tone: "section" },
      { text: "No review items matched the current filters.", tone: "dim" },
    ];
  }

  const item = items[session.reviewState.currentIndex]!;
  const recordDecision = session.decisions[item.recordIndex]!;
  const decision = recordDecision.decisions[item.decisionIndex]!;
  const record = session.records.find((entry) => entry.recordKey === recordDecision.recordKey)!;
  const selectionNotes = record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)";

  return [
    { text: `${record.name}`, tone: "section" },
    { text: `${record.recordKey}`, tone: "dim" },
    { text: `Item ${session.reviewState.currentIndex + 1}/${items.length}` },
    { text: `Scope: ${record.category}${record.subcategory ? `/${record.subcategory}` : ""} | level ${record.level ?? "-"}` },
    ...buildDerivedTagMigrationRecordContextLines(record, decision),
    { text: `Resolution: ${recordDecision.resolutionStatus}` },
    { text: `Decision: ${formatDecisionSummary(decision)}` },
    { text: `Status: ${decision.status}` },
    { text: `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}` },
    { text: `Selection reasons: ${selectionNotes}` },
    { text: "Rationale:", tone: "section" },
    { text: decision.rationale || "(none)", indent: 2 },
  ];
}

function renderReviewHelp(
  terminalSession: DerivedTagTerminalSession,
  selectedActionIndex: number,
): void {
  renderTerminalTextScreen(terminalSession, {
    title: "Derived-Tag Review Help",
    body: [
      { text: "Navigation", tone: "section" },
      { text: "Up / Down or j / k: move between review items" },
      { text: "Left / Right or h / l: move between actions" },
      { text: "Enter: apply the highlighted action" },
      { text: "" },
      { text: "Direct actions", tone: "section" },
      { text: "a approve  r reject  n needs_review  t toggle unresolved  i import  q quit" },
      { text: "" },
      { text: "Current action bar", tone: "section" },
      { text: formatActionBar(selectedActionIndex), tone: "accent" },
    ],
    footer: [{ text: "Press any key to return.", tone: "dim" }],
  });
}

export async function runDerivedTagMigrationReviewUi(
  rootPath: string,
  initialSession: DerivedTagMigrationSession,
  terminalSession: DerivedTagTerminalSession,
): Promise<DerivedTagMigrationReviewResult> {
  let session = clampDerivedTagMigrationReviewIndex(initialSession);
  let imported = false;
  let selectedActionIndex = 0;

  while (true) {
    const items = getDerivedTagMigrationReviewItems(session);
    const progress = summarizeDerivedTagMigrationReviewProgress(session);
    const progressText = progress.actionableRecordCount > 0
      ? `${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount} actionable records resolved`
      : `${progress.candidateRecordCount} candidate records | 0 actionable review items`;
    renderTerminalTwoPaneScreen(terminalSession, {
      title: "Derived-Tag Review",
      subtitle: `Session ${session.manifest.id} | ${progressText} | ${items.length} visible item${items.length === 1 ? "" : "s"} | unresolved only ${session.reviewState.unresolvedOnly ? "on" : "off"}`,
      left: {
        title: "Review Queue",
        lines: buildReviewListLines(terminalSession, session),
      },
      right: {
        title: "Selected Item",
        lines: buildSelectedReviewDetailLines(session),
      },
      footer: [
        { text: "Up/Down or j/k move  Left/Right or h/l choose action  Enter apply  ? help  q quit", tone: "dim" },
        { text: formatActionBar(selectedActionIndex), tone: "accent" },
      ],
    });

    const key = await readTerminalKeyOrResize(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c") {
      break;
    }

    let requestedAction: ReviewActionId | undefined;

    if (normalized === "up" || normalized === "k") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, -1, items.length);
      }
    } else if (normalized === "down" || normalized === "j") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, 1, items.length);
      }
    } else if (normalized === "left" || normalized === "h") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, -1, REVIEW_ACTIONS.length);
    } else if (normalized === "right" || normalized === "l") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, 1, REVIEW_ACTIONS.length);
    } else if (normalized === "enter" || normalized === "kp_enter") {
      requestedAction = REVIEW_ACTIONS[selectedActionIndex]?.id;
    } else if (normalized === "?") {
      renderReviewHelp(terminalSession, selectedActionIndex);
      await readTerminalKey(terminalSession);
      continue;
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
        await pauseForAnyKey(terminalSession, `Imported session ${session.manifest.id}.`);
        break;
      } catch (error) {
        await pauseForAnyKey(terminalSession, `Import failed: ${(error as Error).message}`);
      }
    }

    await persistSession(rootPath, session);
  }

  await persistSession(rootPath, session);
  return { imported, session };
}
