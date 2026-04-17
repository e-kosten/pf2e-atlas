import React from "react";

import {
  TerminalTwoPaneScreen,
  getNormalizedKeyName,
  getTerminalPaneBodyHeight,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import {
  isApplicationExitKey,
  isConfirmKey,
  isHelpKey,
  isMoveDownKey,
  isMoveUpKey,
} from "./keymap.js";
import { buildScrollableLines } from "./list-utils.js";

export type Pf2eTopLevelArea = {
  id: "tag_refinement" | "ontology_search" | "search";
  label: string;
  description: string;
};

function buildAreaDetailLines(pendingReviewCount: number): DerivedTagTerminalLine[] {
  return [
    { text: "Tag Refinement", tone: "section" },
    { text: `${pendingReviewCount} pending review queue slice${pendingReviewCount === 1 ? "" : "s"}` },
    { text: "" },
    { text: "Ontology Search", tone: "section" },
    { text: "Browse the published ontology and drill from tags into live records." },
    { text: "" },
    { text: "Search", tone: "section" },
    { text: "Reserved for the future first-class search surface powered by the indexed PF2E data service." },
  ];
}

function buildTopLevelHelpLines(): DerivedTagTerminalLine[] {
  return [
    { text: "Top-Level Help", tone: "section" },
    { text: "Up / Down or j / k: move between areas" },
    { text: "Enter: open the selected area" },
    { text: "q: exit the terminal app" },
  ];
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
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (isApplicationExitKey(normalized) || normalized === "escape") {
      onQuit();
      return;
    }
    if (isMoveUpKey(normalized)) {
      onMove(-1);
      return;
    }
    if (isMoveDownKey(normalized)) {
      onMove(1);
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
    if (isConfirmKey(normalized)) {
      onOpenSelectedArea();
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
        title: "Selected Area",
        lines: [
          { text: areas[selectedAreaIndex]?.label ?? "", tone: "section" },
          { text: areas[selectedAreaIndex]?.description ?? "" },
          { text: "" },
          ...buildAreaDetailLines(pendingReviewCount),
        ],
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter select  ? help  q quit", tone: "dim" },
        { text: `${pendingReviewCount} pending queue slice${pendingReviewCount === 1 ? "" : "s"}`, tone: "accent" },
      ]}
      leftWidth={32}
    />
  );
}
