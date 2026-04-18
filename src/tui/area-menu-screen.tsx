import React from "react";

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
  isApplicationExitKey,
  isHelpKey,
} from "./keymap.js";
import { buildScrollableLines } from "./list-utils.js";

export type Pf2eTopLevelArea = {
  id: "tag_refinement" | "ontology_search" | "search";
  audience: "user" | "dev";
  label: string;
  description: string;
};

function formatAreaAudience(audience: Pf2eTopLevelArea["audience"]): string {
  return audience === "user" ? "User Surface" : "Development Tool";
}

function buildAreaDetailLines(
  selectedArea: Pf2eTopLevelArea | undefined,
  pendingReviewCount: number,
): DerivedTagTerminalLine[] {
  return [
    { text: "Selected Area", tone: "section" },
    { text: selectedArea?.label ?? "(none)" },
    { text: selectedArea?.description ?? "" },
    { text: "" },
    { text: `Audience: ${selectedArea ? formatAreaAudience(selectedArea.audience) : "-"}` },
    { text: "" },
    { text: "Audience Lanes", tone: "section" },
    { text: "User Surface: Search, ontology browsing, record inspection" },
    { text: "Development Tool: Tag refinement and review workflows" },
    { text: "" },
    { text: `${pendingReviewCount} pending review queue slice${pendingReviewCount === 1 ? "" : "s"}` },
  ];
}

function buildTopLevelHelpLines(): DerivedTagTerminalLine[] {
  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        { id: "move", helpText: "move between top-level areas" },
        { id: "jump", helpText: "jump through the area list" },
        { id: "page", helpText: "page through the area list" },
        { id: "edge", helpText: "jump to the first or last area" },
      ],
    },
    {
      title: "Actions",
      actions: [
        { id: "select", helpText: "open the selected area" },
        { id: "help", helpText: "show this help" },
        { id: "quit", label: "quit", helpText: "exit the terminal app" },
      ],
    },
  ]);
}

export function AreaMenuScreen({
  title,
  selectedAreaIndex,
  areas,
  pendingReviewCount,
  onOpenSelectedArea,
  onMove,
  onQuit,
}: {
  title: string;
  selectedAreaIndex: number;
  areas: Pf2eTopLevelArea[];
  pendingReviewCount: number;
  onOpenSelectedArea: () => void;
  onMove: (delta: number) => void;
  onQuit: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const selectedArea = areas[selectedAreaIndex];
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const navigation = resolveDerivedTagTerminalListNavigationAction(input, key, {
      pageSize: Math.max(1, bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    }, navigationStateRef.current);
    navigationStateRef.current = navigation.state;
    if (isApplicationExitKey(normalized) || normalized === "escape") {
      onQuit();
      return;
    }
    if (navigation.action?.kind === "move") {
      onMove(navigation.action.delta);
      return;
    }
    if (navigation.action?.kind === "confirm") {
      onOpenSelectedArea();
      return;
    }
    if (navigation.action?.kind === "boundary") {
      onMove(navigation.action.boundary === "start" ? -selectedAreaIndex : areas.length - 1 - selectedAreaIndex);
      return;
    }
    if (isHelpKey(normalized)) {
      void terminal.showDialog({
        title: "Top-Level Help",
        body: buildTopLevelHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return;
    }
  });

  return (
    <TerminalTwoPaneScreen
      title={title}
      subtitle="Choose a first-class TUI area"
      left={{
        title: "Areas",
        lines: buildScrollableLines(areas, selectedAreaIndex, bodyHeight),
      }}
      right={{
        title: "Area Details",
        lines: buildAreaDetailLines(selectedArea, pendingReviewCount),
      }}
      footer={[
        { text: formatTerminalInteractionFooter([{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }, { id: "select" }, { id: "help" }, { id: "quit", label: "quit" }]), tone: "dim" },
        {
          text: `${selectedArea ? formatAreaAudience(selectedArea.audience) : "-"} | ${pendingReviewCount} pending queue slice${pendingReviewCount === 1 ? "" : "s"}`,
          tone: "accent",
        },
      ]}
      leftWidth={32}
    />
  );
}
