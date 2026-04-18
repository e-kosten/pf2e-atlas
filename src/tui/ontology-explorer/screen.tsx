import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  useDerivedTagTerminalApp,
} from "../terminal-ui.js";
import { useOntologyExplorerController } from "./controller.js";
import {
  buildOntologyBrowserHelpLines,
  buildOntologyBrowserListLines,
} from "./ui.js";

export function OntologyBrowserScreen({
  model,
  onExit,
  onOpenQuery,
}: {
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const controller = useOntologyExplorerController({
    model,
    onExit,
    onOpenQuery,
    onConfirm: ({ currentNode, currentNodeHasChildren }) => {
      if (
        !currentNodeHasChildren &&
        currentNode?.query?.kind === "listRecords" &&
        model.id !== "derivedTags"
      ) {
        onOpenQuery?.(currentNode.query);
        return true;
      }
      return false;
    },
    onKey: ({ normalizedKey }) => {
      if (normalizedKey !== "?") {
        return false;
      }
      void terminal.showDialog({
        title: "Ontology Browser Help",
        body: buildOntologyBrowserHelpLines(),
        footer: [{ text: "Press any key to return.", tone: "dim" }],
      });
      return true;
    },
  });

  if (controller.layoutMode === "detail-only") {
    return (
      <TerminalPaneScreen
        title={model.label}
        subtitle={`${controller.breadcrumb} | depth ${controller.effectiveState.depth} | focused detail${controller.searchIndicator}`}
        pane={{
          title: `[FOCUSED DETAIL] ${controller.detailTitle}`,
          lines: controller.visibleDetailLines,
          active: true,
        }}
        footer={[
          {
            text: controller.state.searchMode
              ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
              : "z split-view  Tab/w list focus  Up/Down or j/k scroll  Ctrl+U/D jump  Space/b page  Home/End edge  Enter/right open query or detail  Left/backspace/esc list  o open query  / search  ? help  q back",
            tone: "dim",
          },
          {
            text: controller.state.searchMode
              ? `Search /${controller.state.searchInput}`
              : `detail focus | focused detail view | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}${controller.selectedQuery ? " | query ready" : ""}`,
            tone: "accent",
          },
        ]}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      title={model.label}
      subtitle={`${controller.breadcrumb} | depth ${controller.effectiveState.depth}${controller.searchIndicator}`}
      left={{
        title: controller.state.activePane === "list" ? "[LIST] Ontology Entries" : "Ontology Entries",
        lines: buildOntologyBrowserListLines(model, controller.effectiveState, controller.bodyHeight),
        active: controller.state.activePane === "list",
      }}
      right={{
        title: controller.state.activePane === "detail"
          ? `[DETAIL] ${controller.detailTitle}`
          : controller.detailTitle,
        lines: controller.visibleDetailLines,
        active: controller.state.activePane === "detail",
      }}
      footer={[
        {
          text: controller.state.searchMode
            ? "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out"
            : "Tab/w focus  z detail-only  Up/Down or j/k move-scroll  Ctrl+U/D jump  Space/b page  gg/G edge  Enter/right open query or detail  o open query  Left/backspace up  / search  Esc back/clear  ? help  q back",
          tone: "dim",
        },
        {
          text: controller.state.searchMode
            ? `Search /${controller.state.searchInput}`
            : `${controller.state.activePane} focus | ${controller.layoutMode} layout | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}${controller.selectedQuery ? " | query ready" : ""}`,
          tone: "accent",
        },
      ]}
      leftWidth={46}
    />
  );
}
