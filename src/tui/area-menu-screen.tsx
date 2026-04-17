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
  const selectedArea = areas[selectedAreaIndex];
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
    if (isConfirmKey(normalized) || normalized === "right" || normalized === "l") {
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
        title: "Area Details",
        lines: buildAreaDetailLines(selectedArea, pendingReviewCount),
      }}
      footer={[
        { text: "Up/Down or j/k move  Enter/right/l select  ? help  q quit", tone: "dim" },
        {
          text: `${selectedArea ? formatAreaAudience(selectedArea.audience) : "-"} | ${pendingReviewCount} pending queue slice${pendingReviewCount === 1 ? "" : "s"}`,
          tone: "accent",
        },
      ]}
      leftWidth={32}
    />
  );
}
