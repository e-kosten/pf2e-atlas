import React from "react";

import type { DerivedTagTerminalLine } from "./terminal-ui.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "./interaction-bindings.js";
import { TerminalMenuScreen } from "./shared-screens.js";

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
  const actionActions = getAreaMenuInteractionActions().map((action) => ({
    ...action,
    helpText:
      action.id === "select"
        ? "open the selected area"
        : action.id === "help"
          ? "show this help"
          : "exit the terminal app",
  }));

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
      actions: actionActions,
    },
  ]);
}

function getAreaMenuInteractionActions(): TerminalInteractionAction[] {
  return [{ id: "select" }, { id: "help" }, { id: "quit", label: "quit" }];
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
  const selectedArea = areas[selectedAreaIndex];

  return (
    <TerminalMenuScreen
      title={title}
      subtitle="Choose a first-class TUI area"
      leftTitle="Areas"
      rightTitle="Area Details"
      items={areas}
      selectedIndex={selectedAreaIndex}
      interactionActions={getAreaMenuInteractionActions()}
      footer={[
        {
          text: formatTerminalInteractionFooter([
            { id: "move" },
            { id: "jump" },
            { id: "page" },
            { id: "edge" },
            ...getAreaMenuInteractionActions(),
          ]),
          tone: "dim",
        },
      ]}
      status={{
        text: `${selectedArea ? formatAreaAudience(selectedArea.audience) : "-"} | ${pendingReviewCount} pending queue slice${pendingReviewCount === 1 ? "" : "s"}`,
        tone: "accent",
      }}
      helpTitle="Top-Level Help"
      helpBody={buildTopLevelHelpLines()}
      buildDetailLines={(area) => buildAreaDetailLines(area, pendingReviewCount)}
      onMove={(delta) => {
        onMove(delta);
      }}
      onSelect={onOpenSelectedArea}
      onBack={onQuit}
    />
  );
}
