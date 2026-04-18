import React from "react";

import type {
  DerivedTagMigrationMode,
  DerivedTagReviewQueueSummaryItem,
} from "../tags/migration/types.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  createDerivedTagTerminalActionTargetState,
  formatDerivedTagTerminalActionTargetBar,
  getDerivedTagTerminalActionTargetInteractionActions,
  reduceDerivedTagTerminalActionTargetState,
  resolveDerivedTagTerminalActionTargetIntent,
  shouldRenderDerivedTagTerminalActionTarget,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
} from "./action-target.js";
import {
  TerminalTwoPaneScreen,
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  getTerminalPaneBodyHeight,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
  type TerminalInteractionAction,
} from "./interaction-bindings.js";
import { isBackOrExitKey } from "./keymap.js";
import { buildScrollableLines } from "./list-utils.js";

type TagRefinementCommandId =
  | "review_all"
  | "legacy_seed"
  | "legacy_rule"
  | "exemplar_cleanup"
  | "proposal_review";

type TagRefinementUiState = DerivedTagTerminalActionTargetState;

export type TagRefinementMenuItem =
  | { kind: "review_queue_item"; label: string; queueItem: DerivedTagReviewQueueSummaryItem }
  | { kind: "review_all"; label: string }
  | { kind: "create_mode"; label: string; mode: DerivedTagMigrationMode }
  | { kind: "back"; label: string };

export function buildTagRefinementMenuItems(items: DerivedTagReviewQueueSummaryItem[]): TagRefinementMenuItem[] {
  const menuItems: TagRefinementMenuItem[] = [];
  if (items.length > 0) {
    menuItems.push({ kind: "review_all", label: "Review all pending queue items" });
    for (const item of items) {
      const kindLabel = item.kind === "assignment" ? "assignment" : "exemplar";
      const scope = item.kind === "assignment"
        ? `${item.category} ${item.family}.${item.tag}`
        : `${item.category} exemplar.${item.tag}`;
      menuItems.push({
        kind: "review_queue_item",
        label: `Review ${kindLabel} ${scope}  confidence=${item.confidence}  count=${item.count}`,
        queueItem: item,
      });
    }
  }
  menuItems.push(
    { kind: "create_mode", label: "Create legacy-seed review session", mode: "legacy_seed" },
    { kind: "create_mode", label: "Create legacy-rule review session", mode: "legacy_rule" },
    { kind: "create_mode", label: "Create exemplar-cleanup review session", mode: "exemplar_cleanup" },
    { kind: "create_mode", label: "Create AI proposal review session", mode: "proposal_review" },
    { kind: "back", label: "Back to top level" },
  );
  return menuItems;
}

function buildQueueLines(queueItems: DerivedTagReviewQueueSummaryItem[]): DerivedTagTerminalLine[] {
  if (queueItems.length === 0) {
    return [{ text: "No pending authored review items.", tone: "dim" }];
  }

  return queueItems.flatMap((item) => {
    const scope = item.kind === "assignment"
      ? `${item.category} ${item.family}.${item.tag}`
      : `${item.category} exemplar.${item.tag}`;
    return [
      { text: scope, tone: "section" as const },
      { text: `confidence=${item.confidence} count=${item.count}`, indent: 2 },
    ];
  });
}

function getTagRefinementInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "select" },
    { id: "actions" },
    { id: "help" },
    { id: "back", label: "top level" },
    { id: "quit", label: "top level" },
  ];
}

function buildTagRefinementActionEntries(
  hasQueueItems: boolean,
): DerivedTagTerminalActionTargetOption<TagRefinementCommandId>[] {
  const entries: DerivedTagTerminalActionTargetOption<TagRefinementCommandId>[] = [];
  if (hasQueueItems) {
    entries.push({
      id: "review_all",
      label: "Review All Pending Queue Items",
      description: "Create a queue review session covering all pending items.",
    });
  }
  entries.push(
    {
      id: "legacy_seed",
      label: "Create Legacy-Seed Review Session",
      description: "Start a custom legacy-seed review session.",
    },
    {
      id: "legacy_rule",
      label: "Create Legacy-Rule Review Session",
      description: "Start a custom legacy-rule review session.",
    },
    {
      id: "exemplar_cleanup",
      label: "Create Exemplar-Cleanup Review Session",
      description: "Start a custom exemplar-cleanup review session.",
    },
    {
      id: "proposal_review",
      label: "Create AI Proposal Review Session",
      description: "Start a custom AI proposal review session.",
    },
  );
  return entries;
}

function buildTagRefinementHelpLines(
  actionEntries: DerivedTagTerminalActionTargetOption<TagRefinementCommandId>[],
): DerivedTagTerminalLine[] {
  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Navigation",
        actions: [
          { id: "move", helpText: "move between tag-refinement rows" },
          { id: "jump", helpText: "jump through the menu" },
          { id: "page", helpText: "page through the menu" },
          { id: "edge", helpText: "jump to the first or last row" },
        ],
      },
      {
        title: "Actions",
        actions: getTagRefinementInteractionActions().map((action) => ({
          ...action,
          helpText: action.id === "select"
            ? "open the selected row"
            : action.id === "actions"
              ? "focus the tag-refinement actions rail"
              : action.id === "help"
                ? "show this help"
                : "return to the top level",
        })),
      },
    ]),
    { text: "" },
    ...buildDerivedTagTerminalActionTargetHelpLines({
      orientation: "horizontal",
      visibility: "onDemand",
      actions: actionEntries,
      contentHelpText: "The action rail replaces the old command palette on this screen.",
    }),
  ];
}

export function TagRefinementMenuScreen({
  selectedIndex,
  queueItems,
  onBack,
  onMove,
  onOpenSelected,
  onQuickAction,
}: {
  selectedIndex: number;
  queueItems: DerivedTagReviewQueueSummaryItem[];
  onBack: () => void;
  onMove: (delta: number, itemCount: number) => void;
  onOpenSelected: (menuItems: TagRefinementMenuItem[]) => void;
  onQuickAction: (mode: "review_all" | DerivedTagMigrationMode) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const [actionTargetState, dispatchActionTarget] = React.useReducer(
    reduceDerivedTagTerminalActionTargetState<TagRefinementUiState>,
    undefined,
    () => createDerivedTagTerminalActionTargetState(),
  );
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const menuItems = buildTagRefinementMenuItems(queueItems);
  const actionEntries = buildTagRefinementActionEntries(queueItems.length > 0);
  const clampedSelectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

  React.useEffect(() => {
    if (clampedSelectedIndex !== selectedIndex) {
      onMove(0, menuItems.length);
    }
  }, [clampedSelectedIndex, menuItems.length, onMove, selectedIndex]);

  const runActionTargetCommand = React.useCallback((commandId: TagRefinementCommandId) => {
    if (commandId === "review_all") {
      onQuickAction("review_all");
      return;
    }
    onQuickAction(commandId);
  }, [onQuickAction]);

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const actionTargetIntent = resolveDerivedTagTerminalActionTargetIntent(normalized, actionTargetState, "horizontal");
    const navigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: Math.max(1, bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
      includeConfirmKeys: true,
    }, navigationStateRef.current);
    navigationStateRef.current = navigation.state;
    const interactionAction = resolveTerminalInteractionAction(normalized, getTagRefinementInteractionActions());

    if (normalized === "ctrl_c") {
      onBack();
      return;
    }
    if (actionTargetIntent?.kind === "toggle_target") {
      dispatchActionTarget({ type: "toggle_target" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    if (actionTargetIntent?.kind === "leave_actions") {
      dispatchActionTarget({ type: "leave_actions" });
      return;
    }
    if (actionTargetIntent?.kind === "move_action") {
      dispatchActionTarget({ type: "move_action", delta: actionTargetIntent.delta, actionCount: actionEntries.length });
      return;
    }
    if (actionTargetIntent?.kind === "apply_action") {
      const selectedAction = actionEntries[actionTargetState.selectedActionIndex];
      if (selectedAction) {
        runActionTargetCommand(selectedAction.id);
      }
      return;
    }
    if (actionTargetState.activeTarget === "actions") {
      if (interactionAction?.id === "help") {
        void terminal.showDialog({
          title: "Tag Refinement Help",
          body: buildTagRefinementHelpLines(actionEntries),
          footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
        });
      }
      return;
    }
    if (isBackOrExitKey(normalized) || interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onBack();
      return;
    }
    if (navigation.action?.kind === "move") {
      onMove(navigation.action.delta, menuItems.length);
      return;
    }
    if (navigation.action?.kind === "boundary") {
      onMove(
        navigation.action.boundary === "start" ? -clampedSelectedIndex : menuItems.length - 1 - clampedSelectedIndex,
        menuItems.length,
      );
      return;
    }
    if (interactionAction?.id === "select") {
      onOpenSelected(menuItems);
      return;
    }
    if (interactionAction?.id === "help") {
      void terminal.showDialog({
        title: "Tag Refinement Help",
        body: buildTagRefinementHelpLines(actionEntries),
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
      return;
    }
  });

  return (
    <TerminalTwoPaneScreen
      title="Tag Refinement"
      subtitle={`${queueItems.length} queue slice${queueItems.length === 1 ? "" : "s"} pending review`}
      left={{
        title: "Menu",
        lines: buildScrollableLines(menuItems, clampedSelectedIndex, bodyHeight),
      }}
      right={{
        title: "Pending Review Queue",
        lines: buildQueueLines(queueItems),
      }}
      footer={[
        {
          text: formatTerminalInteractionFooter(
            actionTargetState.activeTarget === "actions"
              ? [...getDerivedTagTerminalActionTargetInteractionActions(actionTargetState, "horizontal"), { id: "help" }]
              : [{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }, ...getTagRefinementInteractionActions()],
          ),
          tone: "dim",
        },
        {
          text: shouldRenderDerivedTagTerminalActionTarget(actionTargetState, "onDemand")
            ? formatDerivedTagTerminalActionTargetBar(actionEntries, actionTargetState)
            : `Selected: ${menuItems[clampedSelectedIndex]?.label ?? "(none)"}`,
          tone: "accent",
        },
      ]}
      leftWidth={48}
    />
  );
}
