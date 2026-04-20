import type { OntologyDomainModel, OntologyNode, OntologyNodeQuery } from "../../domain/ontology-types.js";
import {
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import type {
  DerivedTagTerminalCommandOption,
  DerivedTagTerminalLine,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalTwoPaneScreenProps,
} from "../framework/types.js";
import type { OntologyExplorerControllerContext } from "./controller.js";
import type { OntologyBrowserSnapshot } from "./ui.js";

function isOntologyExplorerRootLevel(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): boolean {
  return controller.state.activePane === "list" && controller.effectiveState.depth === 0;
}

function isOntologyExplorerDetailContext(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): boolean {
  return controller.layoutMode === "detail-only" || controller.state.activePane === "detail";
}

function getOntologyBrowserBackAction(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): TerminalInteractionAction {
  if (isOntologyExplorerDetailContext(controller)) {
    return { id: "back" };
  }
  return isOntologyExplorerRootLevel(controller) ? { id: "back", label: "return" } : { id: "back" };
}

function getOntologyBrowserBackHelpText(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): string {
  if (isOntologyExplorerDetailContext(controller)) {
    return "return to the entry list";
  }
  return isOntologyExplorerRootLevel(controller) ? "return from ontology browsing" : "return to the previous level";
}

export function getOntologyBrowserInteractionActions(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): TerminalInteractionAction[] {
  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "layout", label: "split-view" },
      getOntologyBrowserBackAction(controller),
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
      getOntologyBrowserBackAction(controller),
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
    getOntologyBrowserBackAction(controller),
    { id: "search" },
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

export function buildOntologyCommandEntries(
  controller: Pick<OntologyExplorerControllerContext, "selectedQuery">,
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void,
  mode: "browse" | "inspect-and-open" = "browse",
): DerivedTagTerminalCommandOption<"openQuery" | "openResults">[] {
  if (!onOpenQuery || !controller.selectedQuery) {
    return [];
  }

  if (mode === "inspect-and-open") {
    return [
      ...(controller.selectedQuery.kind === "listRecords"
        ? [
            {
              value: "openResults" as const,
              label: "Open Results Page",
              description: "Run the focused ontology query in the shared search results page.",
              keywords: ["results", "reader", "records", "open"],
            },
          ]
        : []),
      {
        value: "openQuery",
        label: "Open Search Query",
        description: "Seed the browse/search editor from the focused ontology query.",
        keywords: ["search", "browse", "editor", "query"],
      },
    ];
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

export function buildOntologyBrowserHelpLines(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState" | "selectedQuery">,
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void,
  mode: "browse" | "inspect-and-open" = "browse",
): DerivedTagTerminalLine[] {
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
          ? mode === "inspect-and-open"
            ? "drill into the focused node or inspect its matching children"
            : "drill into the focused node or open its query"
          : action.id === "focus"
            ? "switch focus between list and detail"
            : action.id === "layout"
              ? "toggle split and detail-only layouts"
              : action.id === "cancel"
                ? "clear the current filter without leaving this level"
                : action.id === "back"
                  ? getOntologyBrowserBackHelpText(controller)
                  : action.id === "search"
                    ? "start live filtering"
                    : action.id === "commands"
                      ? "open the ontology command palette"
                      : action.id === "help"
                        ? "show this help"
                        : "leave ontology browsing",
      label: action.id === "focus" ? "toggle pane" : action.label,
    }));

  const commandEntries = buildOntologyCommandEntries(controller, onOpenQuery, mode);
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
      commands: commandEntries.map((command) => ({
        label: command.label,
        description: command.description ?? "No additional details.",
        aliases: command.aliases,
      })),
      lines:
        commandEntries.length === 0
          ? [{ text: "No additional palette commands are available for the current node.", tone: "dim" }]
          : [],
    },
  ]);
}

export function buildOntologyBrowserScreenModel({
  model,
  controller,
  leftLines,
}: {
  model: OntologyDomainModel;
  controller: OntologyExplorerControllerContext;
  leftLines: DerivedTagTerminalLine[];
}):
  | { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps }
  | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
  if (controller.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: model.label,
        subtitle: `${controller.breadcrumb} | depth ${controller.effectiveState.depth} | focused detail${controller.searchIndicator}`,
        pane: {
          title: `[FOCUSED DETAIL] ${controller.detailTitle}`,
          lines: controller.visibleDetailLines,
          active: true,
        },
        footer: [
          {
            text: controller.state.searchMode
              ? TERMINAL_LIVE_FILTER_FOOTER
              : formatTerminalInteractionFooter(getOntologyBrowserInteractionActions(controller)),
            tone: "dim",
          },
          {
            text: controller.state.searchMode
              ? `Search /${controller.state.searchInput}`
              : `detail focus | focused detail view | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
            tone: "accent",
          },
        ],
      },
    };
  }

  return {
    kind: "two-pane",
    props: {
      title: model.label,
      subtitle: `${controller.breadcrumb} | depth ${controller.effectiveState.depth}${controller.searchIndicator}`,
      left: {
        title: controller.state.activePane === "list" ? "[LIST] Ontology Entries" : "Ontology Entries",
        lines: leftLines,
        active: controller.state.activePane === "list",
      },
      right: {
        title: controller.state.activePane === "detail" ? `[DETAIL] ${controller.detailTitle}` : controller.detailTitle,
        lines: controller.visibleDetailLines,
        active: controller.state.activePane === "detail",
      },
      footer: [
        {
          text: controller.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
            : formatTerminalInteractionFooter(getOntologyBrowserInteractionActions(controller)),
          tone: "dim",
        },
        {
          text: controller.state.searchMode
            ? `Search /${controller.state.searchInput}`
            : `${controller.state.activePane} focus | ${controller.layoutMode} layout | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
          tone: "accent",
        },
      ],
      leftWidth: 46,
    },
  };
}
