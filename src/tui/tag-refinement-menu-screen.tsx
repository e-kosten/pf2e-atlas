import React from "react";

import type { DerivedTagMigrationMode, DerivedTagReviewQueueSummaryItem } from "../tags/editorial/types.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  type DerivedTagTerminalActionTargetOption,
} from "./action-target.js";
import type { DerivedTagTerminalLine } from "./framework/types.js";
import { type TerminalInteractionAction } from "./interaction-bindings.js";
import {
  createMergedReturnFooterBinding,
  createSharedReturnInteractionActions,
} from "./shell-navigation-copy.js";
import { TerminalActionMenuScreen, type TerminalMenuScreenInteractions } from "./shared-screens.js";

type TagRefinementCommandId = "review_all" | "legacy_seed" | "legacy_rule" | "exemplar_cleanup" | "proposal_review";

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
      const scope =
        item.kind === "assignment"
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
    const scope =
      item.kind === "assignment"
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
    { id: "select", helpText: "open the selected row" },
    { id: "actions", helpText: "focus the tag-refinement actions rail" },
    { id: "help", helpText: "show this help" },
    ...createSharedReturnInteractionActions("top level").map((action) => ({
      ...action,
      helpText: "return to the top level",
    })),
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
  return buildDerivedTagTerminalActionTargetHelpLines({
    orientation: "horizontal",
    visibility: "onDemand",
    actions: actionEntries,
    contentHelpText: "The action rail replaces the old command palette on this screen.",
  });
}

function createTagRefinementInteractions(
  actionEntries: DerivedTagTerminalActionTargetOption<TagRefinementCommandId>[],
): TerminalMenuScreenInteractions {
  return {
    actions: getTagRefinementInteractionActions(),
    footerBindings: [
      { kind: "action", action: { id: "select" } },
      { kind: "action", action: { id: "actions" } },
      { kind: "action", action: { id: "help" } },
      createMergedReturnFooterBinding("top level"),
    ],
    help: {
      title: "Tag Refinement Help",
      sections: [
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
          actions: getTagRefinementInteractionActions(),
        },
      ],
      appendix: buildTagRefinementHelpLines(actionEntries),
    },
  };
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
  const menuItems = buildTagRefinementMenuItems(queueItems);
  const actionEntries = buildTagRefinementActionEntries(queueItems.length > 0);
  const interactions = createTagRefinementInteractions(actionEntries);
  const clampedSelectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

  React.useEffect(() => {
    if (clampedSelectedIndex !== selectedIndex) {
      onMove(0, menuItems.length);
    }
  }, [clampedSelectedIndex, menuItems.length, onMove, selectedIndex]);

  const runActionTargetCommand = React.useCallback(
    (commandId: TagRefinementCommandId) => {
      if (commandId === "review_all") {
        onQuickAction("review_all");
        return;
      }
      onQuickAction(commandId);
    },
    [onQuickAction],
  );

  return (
    <TerminalActionMenuScreen
      title="Tag Refinement"
      subtitle={`${queueItems.length} queue slice${queueItems.length === 1 ? "" : "s"} pending review`}
      leftTitle="Menu"
      rightTitle="Pending Review Queue"
      leftWidth={48}
      items={menuItems}
      selectedIndex={clampedSelectedIndex}
      interactions={interactions}
      actionEntries={actionEntries}
      buildRightLines={() => buildQueueLines(queueItems)}
      buildStatusLine={({ selectedItem }) => ({
        text: `Selected: ${selectedItem?.label ?? "(none)"}`,
        tone: "accent",
      })}
      onMove={onMove}
      onSelect={() => {
        onOpenSelected(menuItems);
      }}
      onBack={onBack}
      onAction={runActionTargetCommand}
    />
  );
}
