import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  type DerivedTagTerminalCommandOption,
  useDerivedTagTerminalApp,
} from "../terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import { useOntologyExplorerController } from "./controller.js";
import { buildOntologyBrowserListLines } from "./ui.js";

function buildOntologyBrowserFooterText(controller: ReturnType<typeof useOntologyExplorerController>): string {
  return formatTerminalInteractionFooter(getOntologyBrowserInteractionActions(controller));
}

function getOntologyBrowserInteractionActions(
  controller: Pick<ReturnType<typeof useOntologyExplorerController>, "layoutMode" | "state" | "effectiveState">,
): TerminalInteractionAction[] {
  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "layout", label: "split-view" },
      { id: "back", label: "list" },
      { id: "search" },
      { id: "commands" },
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  if (controller.state.activePane === "list") {
    return [
      { id: "move" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "open", label: "open" },
      { id: "focus", label: "pane" },
      { id: "layout", label: "detail-only" },
      ...(controller.effectiveState.filter ? [{ id: "cancel" as const, label: "clear filter" }] : []),
      { id: "back", label: "up" },
      { id: "search" },
      { id: "commands" },
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  return [
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    { id: "focus", label: "pane" },
    { id: "layout", label: "detail-only" },
    { id: "back", label: "list" },
    { id: "search" },
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

function buildOntologyCommandEntries(
  controller: Pick<ReturnType<typeof useOntologyExplorerController>, "selectedQuery">,
  onOpenQuery?: (query: OntologyNodeQuery) => void,
): DerivedTagTerminalCommandOption<"openQuery">[] {
  if (!onOpenQuery || !controller.selectedQuery) {
    return [];
  }

  return [
    {
      value: "openQuery",
      label: "Open Query",
      description: "Open the focused ontology query in browse/search.",
      keywords: ["search", "browse", "records"],
    },
  ];
}

function buildOntologyBrowserHelpLines(
  controller: Pick<
    ReturnType<typeof useOntologyExplorerController>,
    "layoutMode" | "state" | "effectiveState" | "selectedQuery"
  >,
  onOpenQuery?: (query: OntologyNodeQuery) => void,
) {
  const navigationActions: TerminalInteractionAction[] = [
    {
      id: controller.state.activePane === "list" && controller.layoutMode !== "detail-only" ? "move" : "scroll",
      helpText: "move through the active pane",
    },
    { id: "jump", helpText: "jump through the active pane" },
    { id: "page", helpText: "page through the active pane" },
    { id: "edge", helpText: "jump to the start or end of the active pane" },
  ];
  const actionActions: TerminalInteractionAction[] = getOntologyBrowserInteractionActions(controller)
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "open"
          ? "drill into the focused node or open its query"
          : action.id === "focus"
            ? "switch focus between list and detail"
            : action.id === "layout"
              ? "toggle split and detail-only layouts"
              : action.id === "cancel"
                ? "clear the current filter without leaving this level"
              : action.id === "back"
                ? "move up a level or leave the active pane"
                : action.id === "search"
                  ? "start live filtering"
                  : action.id === "commands"
                    ? "open the ontology command palette"
                    : action.id === "help"
                      ? "show this help"
                      : "leave ontology browsing",
      label: action.id === "focus" ? "toggle pane" : action.label,
    }));

  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: navigationActions,
    },
    {
      title: "Actions",
      actions: actionActions,
    },
    {
      title: "Commands",
      commands: buildOntologyCommandEntries(controller, onOpenQuery).map((command) => ({
        label: command.label,
        description: command.description ?? "No additional details.",
        aliases: command.aliases,
      })),
      lines:
        buildOntologyCommandEntries(controller, onOpenQuery).length === 0
          ? [{ text: "No additional palette commands are available for the current node.", tone: "dim" }]
          : [],
    },
  ]);
}

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
      if (!currentNodeHasChildren && currentNode?.query?.kind === "listRecords" && model.id !== "derivedTags") {
        onOpenQuery?.(currentNode.query);
        return true;
      }
      return false;
    },
    getInteractionActions: getOntologyBrowserInteractionActions,
    onAction: (action, keyContext) => {
      if (action.id === "commands") {
        const commandEntries = buildOntologyCommandEntries(keyContext, onOpenQuery);
        if (commandEntries.length === 0) {
          return true;
        }
        void terminal
          .promptCommandPalette({
            title: "Ontology Commands",
            prompt: "Filter ontology commands",
            entries: commandEntries,
          })
          .then((selected) => {
            if (selected === "openQuery" && keyContext.selectedQuery) {
              onOpenQuery?.(keyContext.selectedQuery);
            }
          });
        return true;
      }
      if (action.id !== "help") {
        return false;
      }
      void terminal.showDialog({
        title: "Ontology Browser Help",
        body: buildOntologyBrowserHelpLines(keyContext, onOpenQuery),
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
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
              ? TERMINAL_LIVE_FILTER_FOOTER
              : buildOntologyBrowserFooterText(controller),
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
        title: controller.state.activePane === "detail" ? `[DETAIL] ${controller.detailTitle}` : controller.detailTitle,
        lines: controller.visibleDetailLines,
        active: controller.state.activePane === "detail",
      }}
      footer={[
        {
          text: controller.state.searchMode ? TERMINAL_LIVE_FILTER_FOOTER : buildOntologyBrowserFooterText(controller),
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
