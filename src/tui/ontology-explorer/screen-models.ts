import type { OntologyDomainModel, OntologyNodeQuery } from "../../types.js";
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
} from "../terminal-ui.js";
import type { OntologyExplorerControllerContext } from "./controller.js";

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

export function buildOntologyCommandEntries(
  controller: Pick<OntologyExplorerControllerContext, "selectedQuery">,
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

export function buildOntologyBrowserHelpLines(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState" | "selectedQuery">,
  onOpenQuery?: (query: OntologyNodeQuery) => void,
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

  const commandEntries = buildOntologyCommandEntries(controller, onOpenQuery);
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
}): { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps } | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
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
              : `detail focus | focused detail view | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}${controller.selectedQuery ? " | query ready" : ""}`,
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
            : `${controller.state.activePane} focus | ${controller.layoutMode} layout | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}${controller.selectedQuery ? " | query ready" : ""}`,
          tone: "accent",
        },
      ],
      leftWidth: 46,
    },
  };
}

export function getFacetPickerInteractionActions(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): TerminalInteractionAction[] {
  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "cycle" },
      { id: "layout", label: "split-view" },
      { id: "back", label: "values" },
      { id: "search" },
      { id: "help" },
      { id: "quit", label: "return" },
    ];
  }

  if (controller.state.activePane === "list") {
    return [
      { id: "move", label: "select" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "cycle" },
      { id: "focus", label: "pane" },
      { id: "layout", label: "detail-only" },
      ...(controller.effectiveState.filter ? [{ id: "cancel" as const, label: "clear filter" }] : []),
      { id: "back", label: "up" },
      { id: "search" },
      { id: "help" },
      { id: "quit", label: "return" },
    ];
  }

  return [
    { id: "scroll" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    { id: "focus", label: "pane" },
    { id: "layout", label: "detail-only" },
    { id: "back", label: "values" },
    { id: "search" },
    { id: "help" },
    { id: "quit", label: "return" },
  ];
}

export function buildFacetPickerHelpLines(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
): DerivedTagTerminalLine[] {
  const actionActions = getFacetPickerInteractionActions(controller)
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "cycle"
          ? "cycle the focused policy through off, any, all, and exclude"
          : action.id === "focus"
            ? "switch focus between values and detail"
            : action.id === "layout"
              ? "toggle split and detail-only layouts"
              : action.id === "cancel"
                ? "clear the current filter without leaving this level"
                : action.id === "back"
                  ? "move up a level or leave the active pane"
                  : action.id === "search"
                    ? "start live filtering"
                    : action.id === "help"
                      ? "show this help"
                      : "apply the current facet state and return",
      label: action.id === "focus" ? "toggle pane" : action.label,
    }));

  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        {
          id: controller.state.activePane === "list" && controller.layoutMode !== "detail-only" ? "move" : "scroll",
          helpText: "move through the active pane",
        },
        { id: "jump", helpText: "jump through the active pane" },
        { id: "page", helpText: "page through the active pane" },
        { id: "edge", helpText: "jump to the start or end of the active pane" },
      ],
    },
    {
      title: "Actions",
      actions: actionActions,
    },
  ]);
}

export function buildFacetPickerScreenModel({
  model,
  controller,
  leftLines,
  focusedPolicyLabel,
}: {
  model: OntologyDomainModel;
  controller: OntologyExplorerControllerContext;
  leftLines: DerivedTagTerminalLine[];
  focusedPolicyLabel: string;
}): { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps } | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
  if (controller.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: "Facet Picker",
        subtitle: `${model.label} | ${controller.breadcrumb}${controller.searchIndicator}`,
        pane: {
          title: "[FOCUSED DETAIL] Detail",
          lines: controller.visibleDetailLines,
          active: true,
        },
        footer: [
          {
            text: controller.state.searchMode
              ? TERMINAL_LIVE_FILTER_FOOTER
              : formatTerminalInteractionFooter(getFacetPickerInteractionActions(controller)),
            tone: "dim",
          },
          {
            text: controller.state.searchMode
              ? `Search /${controller.state.searchInput}`
              : `detail focus | focused detail view | Policy ${focusedPolicyLabel} | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
            tone: "accent",
          },
        ],
      },
    };
  }

  return {
    kind: "two-pane",
    props: {
      title: "Facet Picker",
      subtitle: `${model.label} | ${controller.breadcrumb}${controller.searchIndicator}`,
      left: {
        title: controller.state.activePane === "list" ? "[VALUES]" : "Values",
        lines: leftLines,
        active: controller.state.activePane === "list",
      },
      right: {
        title: controller.state.activePane === "detail" ? "[DETAIL]" : "Detail",
        lines: controller.visibleDetailLines,
        active: controller.state.activePane === "detail",
      },
      footer: [
        {
          text: controller.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
            : formatTerminalInteractionFooter(getFacetPickerInteractionActions(controller)),
          tone: "dim",
        },
        {
          text: controller.state.searchMode
            ? `Search /${controller.state.searchInput}`
            : `Focused: ${controller.currentNode?.label ?? "(none)"} | Policy ${focusedPolicyLabel} | ${controller.state.activePane} focus | Detail scroll ${controller.effectiveState.detailScroll}/${controller.maxDetailScroll}`,
          tone: "accent",
        },
      ],
      leftWidth: 48,
    },
  };
}
