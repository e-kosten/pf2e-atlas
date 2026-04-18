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
  return [
    { text: "Top-Level Help", tone: "section" },
    { text: "Up / Down or j / k: move between areas" },
    { text: "Ctrl-U / Ctrl-D: jump through the area list" },
    { text: "PageUp / PageDown or b / Space: page through the area list" },
    { text: "gg / G or Home / End: jump to the first or last area" },
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
      includeVimHorizontalConfirmKeys: true,
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
        { text: "Up/Down move  Ctrl-U/D jump  PgUp/PgDn page  gg/G or Home/End edge  Enter/right/l select  ? help  q quit", tone: "dim" },
        {
          text: `${selectedArea ? formatAreaAudience(selectedArea.audience) : "-"} | ${pendingReviewCount} pending queue slice${pendingReviewCount === 1 ? "" : "s"}`,
          tone: "accent",
        },
      ]}
      leftWidth={32}
    />
  );
}
