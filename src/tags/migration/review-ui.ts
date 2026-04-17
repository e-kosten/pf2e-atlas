import { importDerivedTagMigrationSession } from "./importer.js";
import { lintDerivedTagMigrationSession } from "./linter.js";
import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  summarizeDerivedTagMigrationReviewProgress,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "./review-session.js";
import { buildDerivedTagMigrationRecordPageLines } from "./review-detail-content.js";
import { writeDerivedTagMigrationSummary } from "./cli-utils.js";
import { writeDerivedTagMigrationSession } from "./session-store.js";
import {
  getRenderedTerminalLineCount,
  getTerminalTwoPaneDimensions,
  getTerminalTwoPaneDetailWidth,
  getTerminalPaneBodyHeight,
  moveSelection,
  moveSelectionWrapped,
  normalizeTerminalTwoPaneLayoutMode,
  pauseForAnyKey,
  readTerminalKey,
  readTerminalKeyOrResize,
  renderTerminalPaneScreen,
  renderTerminalTextScreen,
  renderTerminalTwoPaneScreen,
  sliceRenderedTerminalLines,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalSession,
  type DerivedTagTerminalTwoPaneFocus,
  type DerivedTagTerminalTwoPaneLayoutMode,
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
const REVIEW_LEFT_WIDTH = 46;

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
  const record = session.records.find((entry) => entry.entityRecord.recordKey === recordDecision.recordKey)!;
  const entityRecord = record.entityRecord;
  const selectionNotes = record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)";

  return [
    { text: `${entityRecord.name}`, tone: "section" },
    { text: `${entityRecord.recordKey}`, tone: "dim" },
    { text: `Item ${session.reviewState.currentIndex + 1}/${items.length}` },
    { text: `Scope: ${entityRecord.category}${entityRecord.subcategory ? `/${entityRecord.subcategory}` : ""} | level ${entityRecord.level ?? "-"}` },
    { text: `Resolution: ${recordDecision.resolutionStatus}` },
    { text: `Decision: ${formatDecisionSummary(decision)}` },
    { text: `Status: ${decision.status}` },
    { text: `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}` },
    { text: `Selection reasons: ${selectionNotes}` },
    { text: "Rationale:", tone: "section" },
    { text: decision.rationale || "(none)", indent: 2 },
    { text: "" },
    ...buildDerivedTagMigrationRecordPageLines(record),
  ];
}

function getReviewDetailPaneWidth(
  terminalSession: DerivedTagTerminalSession,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
): number {
  return getTerminalTwoPaneDetailWidth(terminalSession, layoutMode, REVIEW_LEFT_WIDTH);
}

function buildVisibleSelectedReviewDetailLines(
  terminalSession: DerivedTagTerminalSession,
  session: DerivedTagMigrationSession,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  detailScroll: number,
): DerivedTagTerminalLine[] {
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  return sliceRenderedTerminalLines(
    buildSelectedReviewDetailLines(session),
    getReviewDetailPaneWidth(terminalSession, layoutMode),
    detailScroll,
    bodyHeight,
  );
}

function renderReviewHelp(
  terminalSession: DerivedTagTerminalSession,
  selectedActionIndex: number,
): void {
  renderTerminalTextScreen(terminalSession, {
    title: "Derived-Tag Review Help",
    body: [
      { text: "Navigation", tone: "section" },
      { text: "Tab or w: switch focus between the review queue and detail panes" },
      { text: "z: toggle focused detail view while detail has focus" },
      { text: "With list focus, Up / Down or j / k move between review items" },
      { text: "With list focus, Ctrl+U / Ctrl+D and Space / b jump through the queue" },
      { text: "With detail focus, Up / Down or j / k scroll the selected item detail" },
      { text: "With detail focus, Ctrl+U / Ctrl+D and Space / b jump through detail text" },
      { text: "Home / End: jump to the start or end of the focused pane" },
      { text: "Esc or Backspace: leave detail focus and return to the queue" },
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
  let activePane: DerivedTagTerminalTwoPaneFocus = "list";
  let layoutMode: DerivedTagTerminalTwoPaneLayoutMode = "split";
  let detailScroll = 0;

  while (true) {
    layoutMode = normalizeTerminalTwoPaneLayoutMode(layoutMode, activePane);
    const items = getDerivedTagMigrationReviewItems(session);
    const progress = summarizeDerivedTagMigrationReviewProgress(session);
    const progressText = progress.actionableRecordCount > 0
      ? `${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount} actionable records resolved`
      : `${progress.candidateRecordCount} candidate records | 0 actionable review items`;
    const detailLines = buildSelectedReviewDetailLines(session);
    const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(terminalSession, {
      hasSubtitle: true,
      footerLineCount: 2,
    }));
    const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
    const detailJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
    const pageSize = Math.max(1, bodyHeight - 1);
    const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, getReviewDetailPaneWidth(terminalSession, layoutMode));
    const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
    if (detailScroll > maxDetailScroll) {
      detailScroll = maxDetailScroll;
    }
    const subtitle = `Session ${session.manifest.id} | ${progressText} | ${items.length} visible item${items.length === 1 ? "" : "s"} | unresolved only ${session.reviewState.unresolvedOnly ? "on" : "off"}`;
    const detailFooterText = `${formatActionBar(selectedActionIndex)} | ${activePane} focus | ${layoutMode} layout | Detail scroll ${detailScroll}/${maxDetailScroll}`;
    if (layoutMode === "detail-only") {
      renderTerminalPaneScreen(terminalSession, {
        title: "Derived-Tag Review",
        subtitle: `${subtitle} | focused detail`,
        pane: {
          title: "[FOCUSED DETAIL] Selected Item",
          lines: buildVisibleSelectedReviewDetailLines(terminalSession, session, layoutMode, detailScroll),
          active: true,
        },
        footer: [
          { text: "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  Left/Right or h/l choose action  Enter apply  ? help  q quit", tone: "dim" },
          { text: detailFooterText, tone: "accent" },
        ],
      });
    } else {
      renderTerminalTwoPaneScreen(terminalSession, {
        title: "Derived-Tag Review",
        subtitle,
        left: {
          title: activePane === "list" ? "[QUEUE] Review Queue" : "Review Queue",
          lines: buildReviewListLines(terminalSession, session),
          active: activePane === "list",
        },
        right: {
          title: activePane === "detail" ? "[DETAIL] Selected Item" : "Selected Item",
          lines: buildVisibleSelectedReviewDetailLines(terminalSession, session, layoutMode, detailScroll),
          active: activePane === "detail",
        },
        footer: [
          {
            text: activePane === "list"
              ? "Tab/w focus  z detail-only  Up/Down or j/k move  Ctrl+U/D jump queue  Space/b page queue  Home/End queue edge  Left/Right or h/l choose action  Enter apply  ? help  q quit"
              : "Tab/w focus  z detail-only  Up/Down or j/k scroll  Ctrl+U/D jump detail  Space/b page detail  Home/End detail edge  Esc/backspace queue  Left/Right or h/l choose action  Enter apply  ? help  q quit",
            tone: "dim",
          },
          { text: detailFooterText, tone: "accent" },
        ],
        leftWidth: REVIEW_LEFT_WIDTH,
      });
    }

    const key = await readTerminalKeyOrResize(terminalSession);
    const normalized = key.normalizedName;

    if (normalized === "ctrl_c") {
      break;
    }

    let requestedAction: ReviewActionId | undefined;

    if (normalized === "tab" || normalized === "shift_tab" || normalized === "w") {
      activePane = toggleTerminalTwoPaneFocus(activePane);
      layoutMode = normalizeTerminalTwoPaneLayoutMode(layoutMode, activePane);
    } else if (normalized === "z") {
      layoutMode = toggleTerminalTwoPaneLayoutMode(layoutMode, activePane);
    } else if (normalized === "?") {
      renderReviewHelp(terminalSession, selectedActionIndex);
      await readTerminalKey(terminalSession);
      continue;
    } else if (activePane === "detail" && (normalized === "escape" || normalized === "backspace")) {
      activePane = "list";
      layoutMode = "split";
    } else if (activePane === "list" && (normalized === "up" || normalized === "k")) {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, -1, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && (normalized === "down" || normalized === "j")) {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelectionWrapped(session.reviewState.currentIndex, 1, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && normalized === "ctrl_d") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelection(session.reviewState.currentIndex, selectionJumpSize, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && normalized === "ctrl_u") {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelection(session.reviewState.currentIndex, -selectionJumpSize, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && (normalized === "space" || normalized === "page_down")) {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelection(session.reviewState.currentIndex, pageSize, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && (normalized === "b" || normalized === "page_up")) {
      if (items.length > 0) {
        session.reviewState.currentIndex = moveSelection(session.reviewState.currentIndex, -pageSize, items.length);
        detailScroll = 0;
      }
    } else if (activePane === "list" && normalized === "home") {
      if (items.length > 0) {
        session.reviewState.currentIndex = 0;
        detailScroll = 0;
      }
    } else if (activePane === "list" && normalized === "end") {
      if (items.length > 0) {
        session.reviewState.currentIndex = items.length - 1;
        detailScroll = 0;
      }
    } else if (activePane === "detail" && (normalized === "up" || normalized === "k")) {
      detailScroll = Math.max(0, detailScroll - 1);
    } else if (activePane === "detail" && (normalized === "down" || normalized === "j")) {
      detailScroll = Math.min(maxDetailScroll, detailScroll + 1);
    } else if (activePane === "detail" && normalized === "ctrl_d") {
      detailScroll = Math.min(maxDetailScroll, detailScroll + detailJumpSize);
    } else if (activePane === "detail" && normalized === "ctrl_u") {
      detailScroll = Math.max(0, detailScroll - detailJumpSize);
    } else if (activePane === "detail" && (normalized === "space" || normalized === "page_down")) {
      detailScroll = Math.min(maxDetailScroll, detailScroll + pageSize);
    } else if (activePane === "detail" && (normalized === "b" || normalized === "page_up")) {
      detailScroll = Math.max(0, detailScroll - pageSize);
    } else if (activePane === "detail" && normalized === "home") {
      detailScroll = 0;
    } else if (activePane === "detail" && normalized === "end") {
      detailScroll = maxDetailScroll;
    } else if (normalized === "left" || normalized === "h") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, -1, REVIEW_ACTIONS.length);
    } else if (normalized === "right" || normalized === "l") {
      selectedActionIndex = moveSelectionWrapped(selectedActionIndex, 1, REVIEW_ACTIONS.length);
    } else if (normalized === "enter" || normalized === "kp_enter") {
      requestedAction = REVIEW_ACTIONS[selectedActionIndex]?.id;
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
      detailScroll = 0;
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
