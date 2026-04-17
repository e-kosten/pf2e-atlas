import React from "react";

import {
  clampDerivedTagMigrationReviewIndex,
  getDerivedTagMigrationReviewItems,
  summarizeDerivedTagMigrationReviewProgress,
  toggleDerivedTagMigrationUnresolvedOnly,
  updateDerivedTagMigrationDecisionStatus,
} from "./review-session.js";
import { buildDerivedTagMigrationRecordPageLines } from "./review-detail-content.js";
import {
  DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
  importDerivedTagMigrationReviewSession,
  persistDerivedTagMigrationReviewSession,
  type DerivedTagMigrationReviewServices,
} from "./review-controller.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  moveSelectionWrapped,
  normalizeTerminalTwoPaneLayoutMode,
  sliceRenderedTerminalLines,
  toggleTerminalTwoPaneFocus,
  toggleTerminalTwoPaneLayoutMode,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalTwoPaneFocus,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "./terminal-ui.js";
import type { DerivedTagMigrationSession } from "./types.js";

export type DerivedTagMigrationReviewResult = {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

type ReviewUiState = {
  activePane: DerivedTagTerminalTwoPaneFocus;
  detailScroll: number;
  imported: boolean;
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode;
  selectedActionIndex: number;
  session: DerivedTagMigrationSession;
};

type ReviewUiAction =
  | { type: "toggle_focus" }
  | { type: "toggle_layout" }
  | { type: "leave_detail" }
  | { type: "set_session"; session: DerivedTagMigrationSession }
  | { type: "set_imported"; imported: boolean }
  | { type: "move_list_wrapped"; delta: number; itemCount: number }
  | { type: "move_list_clamped"; delta: number; itemCount: number }
  | { type: "list_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | { type: "select_action"; delta: number; actionCount: number }
  | { type: "apply_decision_status"; item: { recordIndex: number; decisionIndex: number }; status: DerivedTagMigrationSession["decisions"][number]["decisions"][number]["status"] }
  | { type: "toggle_unresolved" };

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

function createInitialReviewState(initialSession: DerivedTagMigrationSession): ReviewUiState {
  return {
    activePane: "list",
    detailScroll: 0,
    imported: false,
    layoutMode: "split",
    selectedActionIndex: 0,
    session: clampDerivedTagMigrationReviewIndex(initialSession),
  };
}

function setReviewCurrentIndex(
  session: DerivedTagMigrationSession,
  nextIndex: number,
): DerivedTagMigrationSession {
  const next = structuredClone(session);
  next.reviewState.currentIndex = nextIndex;
  return next;
}

function reviewReducer(state: ReviewUiState, action: ReviewUiAction): ReviewUiState {
  switch (action.type) {
    case "toggle_focus":
      return {
        ...state,
        activePane: toggleTerminalTwoPaneFocus(state.activePane),
      };
    case "toggle_layout":
      return {
        ...state,
        layoutMode: toggleTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane),
      };
    case "leave_detail":
      return {
        ...state,
        activePane: "list",
        layoutMode: "split",
      };
    case "set_session":
      return {
        ...state,
        session: clampDerivedTagMigrationReviewIndex(action.session),
      };
    case "set_imported":
      return {
        ...state,
        imported: action.imported,
      };
    case "move_list_wrapped": {
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(
          state.session,
          moveSelectionWrapped(state.session.reviewState.currentIndex, action.delta, action.itemCount),
        ),
      };
    }
    case "move_list_clamped": {
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(
          state.session,
          moveSelection(state.session.reviewState.currentIndex, action.delta, action.itemCount),
        ),
      };
    }
    case "list_boundary":
      if (action.itemCount <= 0) {
        return state;
      }
      return {
        ...state,
        detailScroll: 0,
        session: setReviewCurrentIndex(state.session, action.boundary === "start" ? 0 : action.itemCount - 1),
      };
    case "move_detail":
      return {
        ...state,
        detailScroll: Math.max(0, Math.min(action.maxDetailScroll, state.detailScroll + action.delta)),
      };
    case "detail_boundary":
      return {
        ...state,
        detailScroll: action.boundary === "start" ? 0 : action.maxDetailScroll,
      };
    case "select_action":
      return {
        ...state,
        selectedActionIndex: moveSelectionWrapped(state.selectedActionIndex, action.delta, action.actionCount),
      };
    case "apply_decision_status":
      return {
        ...state,
        session: clampDerivedTagMigrationReviewIndex(updateDerivedTagMigrationDecisionStatus(
          state.session,
          action.item,
          action.status,
        )),
      };
    case "toggle_unresolved":
      return {
        ...state,
        detailScroll: 0,
        session: clampDerivedTagMigrationReviewIndex(toggleDerivedTagMigrationUnresolvedOnly(state.session)),
      };
    default:
      return state;
  }
}

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
  session: DerivedTagMigrationSession,
  bodyHeight: number,
): DerivedTagTerminalLine[] {
  const items = getDerivedTagMigrationReviewItems(session);
  if (items.length === 0) {
    return [{ text: "No review items match the current filter.", tone: "dim" }];
  }

  const visibleCount = Math.max(1, bodyHeight);
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
    { text: entityRecord.name, tone: "section" },
    { text: entityRecord.recordKey, tone: "dim" },
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
  width: number,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
): number {
  return getTerminalTwoPaneDetailWidth(width, layoutMode, REVIEW_LEFT_WIDTH);
}

function buildVisibleSelectedReviewDetailLines(
  session: DerivedTagMigrationSession,
  layoutMode: DerivedTagTerminalTwoPaneLayoutMode,
  detailScroll: number,
  bodyHeight: number,
  width: number,
): DerivedTagTerminalLine[] {
  return sliceRenderedTerminalLines(
    buildSelectedReviewDetailLines(session),
    getReviewDetailPaneWidth(width, layoutMode),
    detailScroll,
    bodyHeight,
  );
}

function buildReviewHelpLines(selectedActionIndex: number): DerivedTagTerminalLine[] {
  return [
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
  ];
}

export function DerivedTagMigrationReviewScreen({
  rootPath,
  initialSession,
  onComplete,
  services = DEFAULT_DERIVED_TAG_MIGRATION_REVIEW_SERVICES,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
  services?: DerivedTagMigrationReviewServices;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(reviewReducer, initialSession, createInitialReviewState);
  const [busy, setBusy] = React.useState(false);
  const [persistError, setPersistError] = React.useState<string | null>(null);

  React.useEffect(() => {
    let cancelled = false;

    void persistDerivedTagMigrationReviewSession(rootPath, state.session, services)
      .then(() => {
        if (!cancelled) {
          setPersistError(null);
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setPersistError((error as Error).message);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [rootPath, services, state.session]);

  const layoutMode = normalizeTerminalTwoPaneLayoutMode(state.layoutMode, state.activePane);
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const items = getDerivedTagMigrationReviewItems(state.session);
  const progress = summarizeDerivedTagMigrationReviewProgress(state.session);
  const progressText = progress.actionableRecordCount > 0
    ? `${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount} actionable records resolved`
    : `${progress.candidateRecordCount} candidate records | 0 actionable review items`;
  const detailLines = buildSelectedReviewDetailLines(state.session);
  const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const detailJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const pageSize = Math.max(1, bodyHeight - 1);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, getReviewDetailPaneWidth(size.width, layoutMode));
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);
  const subtitle = `Session ${state.session.manifest.id} | ${progressText} | ${items.length} visible item${items.length === 1 ? "" : "s"} | unresolved only ${state.session.reviewState.unresolvedOnly ? "on" : "off"}`;
  const detailFooterText = `${formatActionBar(state.selectedActionIndex)} | ${state.activePane} focus | ${layoutMode} layout | Detail scroll ${detailScroll}/${maxDetailScroll}`;

  const completeReview = React.useCallback((imported: boolean, session: DerivedTagMigrationSession) => {
    onComplete({ imported, session });
  }, [onComplete]);

  const handleImport = React.useCallback(async () => {
    setBusy(true);
    try {
      await importDerivedTagMigrationReviewSession(rootPath, state.session, services);
      dispatch({ type: "set_imported", imported: true });
      setPersistError(null);
      await terminal.pauseForAnyKey(`Imported session ${state.session.manifest.id}.`);
      completeReview(true, state.session);
    } catch (error) {
      await terminal.pauseForAnyKey(`Import failed: ${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [completeReview, rootPath, services, state.session, terminal]);

  const requestAction = React.useCallback(async (actionId: ReviewActionId) => {
    if (actionId === "quit") {
      completeReview(state.imported, state.session);
      return;
    }
    if (actionId === "toggle_unresolved") {
      dispatch({ type: "toggle_unresolved" });
      return;
    }
    if (actionId === "import") {
      await handleImport();
      return;
    }
    if (items.length === 0) {
      return;
    }
    const item = items[state.session.reviewState.currentIndex];
    if (!item) {
      return;
    }
    if (actionId === "approve") {
      dispatch({ type: "apply_decision_status", item, status: "approved" });
    } else if (actionId === "reject") {
      dispatch({ type: "apply_decision_status", item, status: "rejected" });
    } else if (actionId === "needs_review") {
      dispatch({ type: "apply_decision_status", item, status: "needs_review" });
    }
  }, [completeReview, handleImport, items, state.imported, state.session]);

  useDerivedTagTerminalInput((input, key) => {
    if (busy) {
      return;
    }
    const normalized = getNormalizedKeyName(input, key);
    let requestedAction: ReviewActionId | undefined;

    if (normalized === "ctrl_c") {
      requestedAction = "quit";
    } else if (normalized === "tab" || normalized === "shift_tab" || normalized === "w") {
      dispatch({ type: "toggle_focus" });
      return;
    } else if (normalized === "z") {
      dispatch({ type: "toggle_layout" });
      return;
    } else if (normalized === "?") {
      void terminal.showDialog({
        title: "Derived-Tag Review Help",
        body: buildReviewHelpLines(state.selectedActionIndex),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    } else if (state.activePane === "detail" && (normalized === "escape" || normalized === "backspace")) {
      dispatch({ type: "leave_detail" });
      return;
    } else if (state.activePane === "list" && (normalized === "up" || normalized === "k")) {
      dispatch({ type: "move_list_wrapped", delta: -1, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && (normalized === "down" || normalized === "j")) {
      dispatch({ type: "move_list_wrapped", delta: 1, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && normalized === "ctrl_d") {
      dispatch({ type: "move_list_clamped", delta: selectionJumpSize, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && normalized === "ctrl_u") {
      dispatch({ type: "move_list_clamped", delta: -selectionJumpSize, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && (normalized === "space" || normalized === "page_down")) {
      dispatch({ type: "move_list_clamped", delta: pageSize, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && (normalized === "b" || normalized === "page_up")) {
      dispatch({ type: "move_list_clamped", delta: -pageSize, itemCount: items.length });
      return;
    } else if (state.activePane === "list" && normalized === "home") {
      dispatch({ type: "list_boundary", boundary: "start", itemCount: items.length });
      return;
    } else if (state.activePane === "list" && normalized === "end") {
      dispatch({ type: "list_boundary", boundary: "end", itemCount: items.length });
      return;
    } else if (state.activePane === "detail" && (normalized === "up" || normalized === "k")) {
      dispatch({ type: "move_detail", delta: -1, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && (normalized === "down" || normalized === "j")) {
      dispatch({ type: "move_detail", delta: 1, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && normalized === "ctrl_d") {
      dispatch({ type: "move_detail", delta: detailJumpSize, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && normalized === "ctrl_u") {
      dispatch({ type: "move_detail", delta: -detailJumpSize, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && (normalized === "space" || normalized === "page_down")) {
      dispatch({ type: "move_detail", delta: pageSize, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && (normalized === "b" || normalized === "page_up")) {
      dispatch({ type: "move_detail", delta: -pageSize, maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && normalized === "home") {
      dispatch({ type: "detail_boundary", boundary: "start", maxDetailScroll });
      return;
    } else if (state.activePane === "detail" && normalized === "end") {
      dispatch({ type: "detail_boundary", boundary: "end", maxDetailScroll });
      return;
    } else if (normalized === "left" || normalized === "h") {
      dispatch({ type: "select_action", delta: -1, actionCount: REVIEW_ACTIONS.length });
      return;
    } else if (normalized === "right" || normalized === "l") {
      dispatch({ type: "select_action", delta: 1, actionCount: REVIEW_ACTIONS.length });
      return;
    } else if (normalized === "enter") {
      requestedAction = REVIEW_ACTIONS[state.selectedActionIndex]?.id;
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

    if (requestedAction) {
      void requestAction(requestedAction);
    }
  }, !busy);

  if (layoutMode === "detail-only") {
    return (
      <TerminalPaneScreen
        title="Derived-Tag Review"
        subtitle={`${subtitle} | focused detail`}
        pane={{
          title: "[FOCUSED DETAIL] Selected Item",
          lines: buildVisibleSelectedReviewDetailLines(state.session, layoutMode, detailScroll, bodyHeight, size.width),
          active: true,
        }}
        footer={[
          { text: "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Esc/backspace list  Left/Right or h/l choose action  Enter apply  ? help  q quit", tone: "dim" },
          { text: detailFooterText, tone: "accent" },
          ...(persistError ? [{ text: `Persist error: ${persistError}`, tone: "danger" as const }] : []),
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title="Derived-Tag Review"
      subtitle={subtitle}
      left={{
        title: state.activePane === "list" ? "[QUEUE] Review Queue" : "Review Queue",
        lines: buildReviewListLines(state.session, bodyHeight),
        active: state.activePane === "list",
      }}
      right={{
        title: state.activePane === "detail" ? "[DETAIL] Selected Item" : "Selected Item",
        lines: buildVisibleSelectedReviewDetailLines(state.session, layoutMode, detailScroll, bodyHeight, size.width),
        active: state.activePane === "detail",
      }}
      footer={[
        {
          text: state.activePane === "list"
            ? "Tab/w focus  z detail-only  Up/Down or j/k move  Ctrl+U/D jump queue  Space/b page queue  Home/End queue edge  Left/Right or h/l choose action  Enter apply  ? help  q quit"
            : "Tab/w focus  z detail-only  Up/Down or j/k scroll  Ctrl+U/D jump detail  Space/b page detail  Home/End detail edge  Esc/backspace queue  Left/Right or h/l choose action  Enter apply  ? help  q quit",
          tone: "dim",
        },
        { text: detailFooterText, tone: "accent" },
        ...(persistError ? [{ text: `Persist error: ${persistError}`, tone: "danger" as const }] : []),
      ]}
      leftWidth={REVIEW_LEFT_WIDTH}
    />
  );
}

function DerivedTagMigrationReviewRoot({
  rootPath,
  initialSession,
  onComplete,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
}): React.JSX.Element {
  return (
    <DerivedTagMigrationReviewScreen
      rootPath={rootPath}
      initialSession={initialSession}
      onComplete={onComplete}
    />
  );
}

export async function runDerivedTagMigrationReviewUi(
  rootPath: string,
  initialSession: DerivedTagMigrationSession,
): Promise<DerivedTagMigrationReviewResult> {
  let result: DerivedTagMigrationReviewResult | null = null;

  await runDerivedTagTerminalApp(
    <ReviewRunner
      rootPath={rootPath}
      initialSession={initialSession}
      onComplete={(nextResult) => {
        result = nextResult;
      }}
    />,
  );

  if (!result) {
    throw new Error("Review did not complete.");
  }
  return result;
}

function ReviewRunner({
  rootPath,
  initialSession,
  onComplete,
}: {
  rootPath: string;
  initialSession: DerivedTagMigrationSession;
  onComplete: (result: DerivedTagMigrationReviewResult) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  return (
    <DerivedTagMigrationReviewRoot
      rootPath={rootPath}
      initialSession={initialSession}
      onComplete={(result) => {
        onComplete(result);
        terminal.exitApp(result);
      }}
    />
  );
}
