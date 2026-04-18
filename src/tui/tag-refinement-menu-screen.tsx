import React from "react";

import type {
  DerivedTagMigrationMode,
  DerivedTagReviewQueueSummaryItem,
} from "../tags/migration/types.js";
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
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
} from "./interaction-bindings.js";
import {
  isBackOrExitKey,
  isHelpKey,
} from "./keymap.js";
import { buildScrollableLines } from "./list-utils.js";

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

function buildTagRefinementHelpLines(): DerivedTagTerminalLine[] {
  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        { id: "move", helpText: "move between tag-refinement rows" },
        { id: "jump", helpText: "jump through the menu" },
        { id: "page", helpText: "page through the menu" },
        { id: "edge", helpText: "jump to the first or last row" },
        { id: "select", helpText: "open the selected row" },
        { id: "back", helpText: "return to the top level" },
        { id: "help", helpText: "show this help" },
      ],
    },
    {
      title: "Quick Actions",
      lines: [
        { text: "Review All  aliases: a  review all queue items" },
        { text: "Legacy Seed  aliases: s  create a legacy-seed session" },
        { text: "Legacy Rule  aliases: r  create a legacy-rule session" },
        { text: "Exemplar Cleanup  aliases: e  create an exemplar-cleanup session" },
        { text: "AI Proposal Review  aliases: p, n  create an AI proposal review session" },
      ],
    },
  ]);
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
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));
  const menuItems = buildTagRefinementMenuItems(queueItems);
  const clampedSelectedIndex = Math.max(0, Math.min(selectedIndex, Math.max(0, menuItems.length - 1)));

  React.useEffect(() => {
    if (clampedSelectedIndex !== selectedIndex) {
      onMove(0, menuItems.length);
    }
  }, [clampedSelectedIndex, menuItems.length, onMove, selectedIndex]);

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const navigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: Math.max(1, bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
      includeConfirmKeys: true,
    }, navigationStateRef.current);
    navigationStateRef.current = navigation.state;

    if (isBackOrExitKey(normalized)) {
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
    if (navigation.action?.kind === "confirm") {
      onOpenSelected(menuItems);
      return;
    }
    if (isHelpKey(normalized)) {
      void terminal.showDialog({
        title: "Tag Refinement Help",
        body: buildTagRefinementHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }
    if (normalized === "a" && queueItems.length > 0) {
      onQuickAction("review_all");
      return;
    }
    if (normalized === "s") {
      onQuickAction("legacy_seed");
      return;
    }
    if (normalized === "r") {
      onQuickAction("legacy_rule");
      return;
    }
    if (normalized === "e") {
      onQuickAction("exemplar_cleanup");
      return;
    }
    if (normalized === "p" || normalized === "n") {
      onQuickAction("proposal_review");
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
        { text: `${formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }, { id: "select" }, { id: "help" }, { id: "back", label: "top level" }])}  a/s/r/e/p actions`, tone: "dim" },
        { text: `Selected: ${menuItems[clampedSelectedIndex]?.label ?? "(none)"}`, tone: "accent" },
      ]}
      leftWidth={48}
    />
  );
}
