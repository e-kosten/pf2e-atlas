import { getDerivedTagMigrationReviewItems, summarizeDerivedTagMigrationReviewProgress } from "./review-session.js";
import { buildDerivedTagMigrationRecordPageLines } from "./review-detail-content.js";
import {
  DERIVED_TAG_MIGRATION_REVIEW_ACTIONS,
  type DerivedTagMigrationReviewScreenState,
} from "./review-screen-state.js";
import type { DerivedTagMigrationSession } from "./types.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  buildDerivedTagTerminalActionTargetLine,
  getDerivedTagTerminalActionTargetInteractionActions,
} from "../../tui/action-target.js";
import {
  type DerivedTagTerminalLine,
  type DerivedTagTerminalPaneScreenProps,
  type DerivedTagTerminalTwoPaneLayoutMode,
  type DerivedTagTerminalTwoPaneScreenProps,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
} from "../../tui/terminal-ui.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../../tui/interaction-bindings.js";

export type DerivedTagMigrationReviewScreenModel =
  | { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps }
  | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps };

export type DerivedTagMigrationReviewViewModel = {
  actionTargetInteractionActions: TerminalInteractionAction[];
  detailJumpSize: number;
  helpLines: DerivedTagTerminalLine[];
  items: ReturnType<typeof getDerivedTagMigrationReviewItems>;
  maxDetailScroll: number;
  paneInteractionActions: TerminalInteractionAction[];
  pageSize: number;
  screen: DerivedTagMigrationReviewScreenModel;
  selectionJumpSize: number;
};

const REVIEW_LEFT_WIDTH = 46;

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

function buildReviewListLines(session: DerivedTagMigrationSession, bodyHeight: number): DerivedTagTerminalLine[] {
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
    {
      text: `Scope: ${entityRecord.category}${entityRecord.subcategory ? `/${entityRecord.subcategory}` : ""} | level ${entityRecord.level ?? "-"}`,
    },
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

function getReviewDetailPaneWidth(width: number, layoutMode: DerivedTagTerminalTwoPaneLayoutMode): number {
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

function getReviewContentNavigationActions(
  activePane: DerivedTagMigrationReviewScreenState["activePane"],
): TerminalInteractionAction[] {
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

function getReviewPaneInteractionActions(
  activePane: DerivedTagMigrationReviewScreenState["activePane"],
): TerminalInteractionAction[] {
  return [
    { id: "focus", helpText: "switch between the review queue and selected-item detail" },
    { id: "layout", helpText: "toggle split view vs focused detail while detail has focus" },
    ...(activePane === "detail"
      ? [{ id: "close" as const, label: "queue focus", helpText: "return focus to the review queue" }]
      : []),
    { id: "help", helpText: "show this help" },
  ];
}

function buildReviewHelpLines(
  state: DerivedTagMigrationReviewScreenState,
  paneInteractionActions: TerminalInteractionAction[],
): DerivedTagTerminalLine[] {
  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Content Navigation",
        actions: getReviewContentNavigationActions(state.activePane),
      },
      {
        title: "Pane Controls",
        actions: paneInteractionActions,
      },
    ]),
    { text: "" },
    ...buildDerivedTagTerminalActionTargetHelpLines({
      orientation: "horizontal",
      visibility: "persistent",
      actions: [...DERIVED_TAG_MIGRATION_REVIEW_ACTIONS],
      contentHelpText:
        "While the rail is focused, only the action-target keys act on it. Use : or Escape to return to content navigation.",
    }),
    { text: "" },
    { text: "Current Action Rail", tone: "section" },
    buildDerivedTagTerminalActionTargetLine(DERIVED_TAG_MIGRATION_REVIEW_ACTIONS, state),
  ];
}

export function buildDerivedTagMigrationReviewViewModel({
  persistError,
  size,
  state,
}: {
  persistError: string | null;
  size: { width: number; height: number };
  state: DerivedTagMigrationReviewScreenState;
}): DerivedTagMigrationReviewViewModel {
  const layoutMode = state.layoutMode;
  const footerLineCount = 3 + (persistError ? 1 : 0);
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount,
    }),
  );
  const items = getDerivedTagMigrationReviewItems(state.session);
  const progress = summarizeDerivedTagMigrationReviewProgress(state.session);
  const progressText =
    progress.actionableRecordCount > 0
      ? `${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount} actionable records resolved`
      : `${progress.candidateRecordCount} candidate records | 0 actionable review items`;
  const detailLines = buildSelectedReviewDetailLines(state.session);
  const selectionJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const detailJumpSize = Math.max(1, Math.floor(bodyHeight / 2));
  const pageSize = Math.max(1, bodyHeight - 1);
  const renderedDetailLineCount = getRenderedTerminalLineCount(
    detailLines,
    getReviewDetailPaneWidth(size.width, layoutMode),
  );
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);
  const subtitle = `Session ${state.session.manifest.id} | ${progressText} | ${items.length} visible item${items.length === 1 ? "" : "s"} | unresolved only ${state.session.reviewState.unresolvedOnly ? "on" : "off"}`;
  const actionBarLine = buildDerivedTagTerminalActionTargetLine(DERIVED_TAG_MIGRATION_REVIEW_ACTIONS, state);
  const detailFooterText = `${state.activePane} focus | ${layoutMode} layout | Detail scroll ${detailScroll}/${maxDetailScroll}`;
  const paneInteractionActions = getReviewPaneInteractionActions(state.activePane);
  const actionTargetInteractionActions = getDerivedTagTerminalActionTargetInteractionActions(state, "horizontal");
  const footerInteractionActions: TerminalInteractionAction[] =
    state.activeTarget === "actions"
      ? [...actionTargetInteractionActions, { id: "help" }]
      : [
          ...getReviewContentNavigationActions(state.activePane),
          ...paneInteractionActions,
          ...actionTargetInteractionActions,
        ];
  const helpLines = buildReviewHelpLines(state, paneInteractionActions);
  const commonFooter = [
    { text: formatTerminalInteractionFooter(footerInteractionActions), tone: "dim" as const },
    actionBarLine,
    { text: detailFooterText, tone: "accent" as const },
    ...(persistError ? [{ text: `Persist error: ${persistError}`, tone: "danger" as const }] : []),
  ];

  if (layoutMode === "detail-only") {
    return {
      actionTargetInteractionActions,
      detailJumpSize,
      helpLines,
      items,
      maxDetailScroll,
      pageSize,
      paneInteractionActions,
      screen: {
        kind: "detail-only",
        props: {
          title: "Derived-Tag Review",
          subtitle: `${subtitle} | focused detail`,
          pane: {
            title: "[FOCUSED DETAIL] Selected Item",
            lines: buildVisibleSelectedReviewDetailLines(
              state.session,
              layoutMode,
              detailScroll,
              bodyHeight,
              size.width,
            ),
            active: true,
          },
          footer: commonFooter,
        },
      },
      selectionJumpSize,
    };
  }

  return {
    actionTargetInteractionActions,
    detailJumpSize,
    helpLines,
    items,
    maxDetailScroll,
    pageSize,
    paneInteractionActions,
    screen: {
      kind: "two-pane",
      props: {
        title: "Derived-Tag Review",
        subtitle,
        left: {
          title: state.activePane === "list" ? "[QUEUE] Review Queue" : "Review Queue",
          lines: buildReviewListLines(state.session, bodyHeight),
          active: state.activePane === "list",
        },
        right: {
          title: state.activePane === "detail" ? "[DETAIL] Selected Item" : "Selected Item",
          lines: buildVisibleSelectedReviewDetailLines(state.session, layoutMode, detailScroll, bodyHeight, size.width),
          active: state.activePane === "detail",
        },
        footer: commonFooter,
        leftWidth: REVIEW_LEFT_WIDTH,
      },
    },
    selectionJumpSize,
  };
}
