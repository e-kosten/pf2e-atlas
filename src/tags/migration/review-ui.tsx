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
  buildDerivedTagTerminalActionTargetHelpLines,
  createDerivedTagTerminalActionTargetState,
  formatDerivedTagTerminalActionTargetBar,
  getDerivedTagTerminalActionTargetInteractionActions,
  reduceDerivedTagTerminalActionTargetState,
  resolveDerivedTagTerminalActionTargetIntent,
  type DerivedTagTerminalActionTargetAction,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
} from "../../tui/action-target.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  moveSelection,
  moveSelectionWrapped,
  resolveDerivedTagTerminalListNavigationAction,
  sliceRenderedTerminalLines,
  runDerivedTagTerminalApp,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
  type DerivedTagTerminalTwoPaneLayoutMode,
} from "../../tui/terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
  type TerminalInteractionAction,
} from "../../tui/interaction-bindings.js";
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
  type DerivedTagTerminalTwoPaneState,
} from "../../tui/two-pane-state.js";
import type { DerivedTagMigrationSession } from "./types.js";

export type DerivedTagMigrationReviewResult = {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

type ReviewUiState = DerivedTagTerminalTwoPaneState & DerivedTagTerminalActionTargetState & {
  imported: boolean;
  session: DerivedTagMigrationSession;
};

type ReviewUiAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "set_session"; session: DerivedTagMigrationSession }
  | { type: "set_imported"; imported: boolean }
  | { type: "move_list_wrapped"; delta: number; itemCount: number }
  | { type: "move_list_clamped"; delta: number; itemCount: number }
  | { type: "list_boundary"; boundary: "start" | "end"; itemCount: number }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number }
  | DerivedTagTerminalActionTargetAction
  | { type: "apply_decision_status"; item: { recordIndex: number; decisionIndex: number }; status: DerivedTagMigrationSession["decisions"][number]["decisions"][number]["status"] }
  | { type: "toggle_unresolved" };

const REVIEW_ACTIONS = [
  { id: "approve", label: "Approve", description: "Mark the current review item approved." },
  { id: "reject", label: "Reject", description: "Mark the current review item rejected." },
  { id: "needs_review", label: "Needs Review", description: "Keep the item unresolved for later follow-up." },
  { id: "toggle_unresolved", label: "Toggle Unresolved", description: "Switch the queue between all items and unresolved-only items." },
  { id: "import", label: "Lint + Import", description: "Run import for the current session after validation." },
  { id: "quit", label: "Quit", description: "Finish the review UI and return the current session state." },
] as const satisfies readonly DerivedTagTerminalActionTargetOption[];

type ReviewActionId = (typeof REVIEW_ACTIONS)[number]["id"];
const REVIEW_LEFT_WIDTH = 46;

function createInitialReviewState(initialSession: DerivedTagMigrationSession): ReviewUiState {
  return {
    activePane: "list",
    ...createDerivedTagTerminalActionTargetState(),
    detailScroll: 0,
    imported: false,
    layoutMode: "split",
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
    case "toggle_layout":
    case "leave_detail":
      return {
        ...state,
        ...reduceDerivedTagTerminalTwoPaneState(state, action),
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
    case "detail_boundary":
      return reduceDerivedTagTerminalTwoPaneState(state, action);
    case "toggle_target":
    case "leave_actions":
    case "move_action":
      return reduceDerivedTagTerminalActionTargetState(state, action);
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

function getReviewContentNavigationActions(activePane: DerivedTagTerminalTwoPaneState["activePane"]): TerminalInteractionAction[] {
  return activePane === "list"
    ? [
      { id: "move", helpText: "move between review items" },
      { id: "jump", helpText: "jump through the review queue" },
      { id: "page", helpText: "page through the review queue" },
      { id: "edge", helpText: "jump to the first or last review item" },
    ]
    : [
      { id: "scroll", helpText: "scroll the selected record detail" },
      { id: "jump", helpText: "jump through the selected record detail" },
      { id: "page", helpText: "page through the selected record detail" },
      { id: "edge", helpText: "jump to the start or end of the selected detail" },
    ];
}

function getReviewPaneInteractionActions(activePane: DerivedTagTerminalTwoPaneState["activePane"]): TerminalInteractionAction[] {
  return [
    { id: "focus", helpText: "switch between the review queue and selected-item detail" },
    { id: "layout", helpText: "toggle split view vs focused detail while detail has focus" },
    ...(activePane === "detail"
      ? [{ id: "close" as const, label: "queue focus", helpText: "return focus to the review queue" }]
      : []),
    { id: "help", helpText: "show this help" },
  ];
}

function buildReviewHelpLines(state: ReviewUiState): DerivedTagTerminalLine[] {
  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Content Navigation",
        actions: getReviewContentNavigationActions(state.activePane),
      },
      {
        title: "Pane Controls",
        actions: getReviewPaneInteractionActions(state.activePane),
      },
    ]),
    { text: "" },
    ...buildDerivedTagTerminalActionTargetHelpLines({
      orientation: "horizontal",
      visibility: "persistent",
      actions: [...REVIEW_ACTIONS],
      contentHelpText: "While the rail is focused, only the action-target keys act on it. Use : or Escape to return to content navigation.",
    }),
    { text: "" },
    { text: "Current Action Rail", tone: "section" },
    { text: formatDerivedTagTerminalActionTargetBar(REVIEW_ACTIONS, state), tone: "accent" },
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
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());

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

  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode(state);
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
  const actionBarText = formatDerivedTagTerminalActionTargetBar(REVIEW_ACTIONS, state);
  const detailFooterText = `${actionBarText} | ${state.activePane} focus | ${layoutMode} layout | Detail scroll ${detailScroll}/${maxDetailScroll}`;

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

  const activeNavigationActions = getReviewContentNavigationActions(state.activePane);
  const paneInteractionActions = getReviewPaneInteractionActions(state.activePane);
  const actionTargetInteractionActions = getDerivedTagTerminalActionTargetInteractionActions(state, "horizontal");
  const footerInteractionActions = state.activeTarget === "actions"
    ? [...actionTargetInteractionActions, { id: "help" as const }]
    : [...activeNavigationActions, ...paneInteractionActions, ...actionTargetInteractionActions];

  useDerivedTagTerminalInput((input, key) => {
    if (busy) {
      return;
    }
    const normalized = getNormalizedKeyName(input, key);
    const actionTargetIntent = resolveDerivedTagTerminalActionTargetIntent(normalized, state, "horizontal");
    const interactionAction = resolveTerminalInteractionAction(normalized, paneInteractionActions);

    if (normalized === "ctrl_c") {
      void requestAction("quit");
      return;
    }
    if (actionTargetIntent?.kind === "toggle_target") {
      dispatch({ type: "toggle_target" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    if (actionTargetIntent?.kind === "leave_actions") {
      dispatch({ type: "leave_actions" });
      return;
    }
    if (actionTargetIntent?.kind === "move_action") {
      dispatch({ type: "move_action", delta: actionTargetIntent.delta, actionCount: REVIEW_ACTIONS.length });
      return;
    }
    if (actionTargetIntent?.kind === "apply_action") {
      const requestedAction = REVIEW_ACTIONS[state.selectedActionIndex]?.id;
      if (requestedAction) {
        void requestAction(requestedAction);
      }
      return;
    }
    if (interactionAction?.id === "help") {
      void terminal.showDialog({
        title: "Derived-Tag Review Help",
        body: buildReviewHelpLines(state),
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
      return;
    }
    if (state.activeTarget === "actions") {
      return;
    }
    if (interactionAction?.id === "focus") {
      dispatch({ type: "toggle_focus" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    if (interactionAction?.id === "layout") {
      dispatch({ type: "toggle_layout" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    if (interactionAction?.id === "close" && state.activePane === "detail") {
      dispatch({ type: "leave_detail" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    const navigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize,
      jumpSize: state.activePane === "list" ? selectionJumpSize : detailJumpSize,
    }, navigationStateRef.current);
    navigationStateRef.current = navigation.state;

    if (navigation.action?.kind === "move") {
      if (state.activePane === "list") {
        if (Math.abs(navigation.action.delta) === 1) {
          dispatch({ type: "move_list_wrapped", delta: navigation.action.delta, itemCount: items.length });
        } else {
          dispatch({ type: "move_list_clamped", delta: navigation.action.delta, itemCount: items.length });
        }
        return;
      }
      dispatch({ type: "move_detail", delta: navigation.action.delta, maxDetailScroll });
      return;
    }
    if (navigation.action?.kind === "boundary") {
      if (state.activePane === "list") {
        dispatch({ type: "list_boundary", boundary: navigation.action.boundary, itemCount: items.length });
        return;
      }
      dispatch({ type: "detail_boundary", boundary: navigation.action.boundary, maxDetailScroll });
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
          { text: formatTerminalInteractionFooter(footerInteractionActions), tone: "dim" },
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
        { text: formatTerminalInteractionFooter(footerInteractionActions), tone: "dim" },
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
