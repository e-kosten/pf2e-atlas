import type { OntologyExplorerControllerContext } from "../ontology-explorer/controller.js";
import { buildOntologyBrowserListRows } from "../ontology-explorer/ui.js";
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
  DerivedTagTerminalSegment,
  DerivedTagTerminalTwoPaneScreenProps,
} from "../framework/types.js";
import { getFilterExplorerTargetState } from "./compose-state.js";
import {
  buildFilterExplorerPolicyBadgeSegments,
  buildFilterExplorerPolicyLabelSegments,
  getFilterExplorerPolicyPresentation,
} from "./policy-presentation.js";
import type {
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerMode,
  FilterExplorerSelectionMap,
} from "./types.js";

function isDetailContext(controller: OntologyExplorerControllerContext): boolean {
  return controller.layoutMode === "detail-only" || controller.state.activePane === "detail";
}

function isAtExitDepth(mode: FilterExplorerMode, controller: OntologyExplorerControllerContext, rootDepth: number): boolean {
  return mode.kind === "compose" && controller.effectiveState.depth === rootDepth;
}

function getBackHelpText(mode: FilterExplorerMode, controller: OntologyExplorerControllerContext): string {
  if (isDetailContext(controller)) {
    return "return to the entry list";
  }
  return isAtExitDepth(mode, controller, 0)
    ? mode.kind === "compose"
      ? "leave the filter composer"
      : "return from the explorer"
    : "return to the previous level";
}

export function getFilterExplorerInteractionActions(
  mode: FilterExplorerMode,
  controller: OntologyExplorerControllerContext,
): TerminalInteractionAction[] {
  if (mode.kind === "compose") {
    if (controller.layoutMode === "detail-only") {
      return [
        { id: "scroll" },
        { id: "jump" },
        { id: "page" },
        { id: "edge" },
        { id: "cycle" },
        { id: "layout", label: "split-view" },
        { id: "back" },
        { id: "search" },
        { id: "help" },
        { id: "quit", label: "back" },
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
        { id: "back" },
        { id: "search" },
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
      { id: "back" },
      { id: "search" },
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "layout", label: "split-view" },
      { id: "back" },
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
      { id: "back" },
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
    { id: "back" },
    { id: "search" },
    { id: "commands" },
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

function buildSelectionEntrySegments(
  field: string,
  selection: FilterExplorerSelectionMap[string],
): DerivedTagTerminalSegment[] {
  const segments: DerivedTagTerminalSegment[] = [{ text: `${field}: ` }];
  let first = true;

  for (const state of ["any", "all", "exclude"] as const) {
    const values = selection[state];
    if (values.length === 0) {
      continue;
    }

    if (!first) {
      segments.push({ text: " | ", tone: "dim" });
    }
    segments.push(...buildFilterExplorerPolicyBadgeSegments(state));
    segments.push({ text: ` ${values.join(", ")}` });
    first = false;
  }

  return segments;
}

export function buildFilterExplorerComposeDetailLines(args: {
  mode: FilterExplorerComposeMode;
  selection: FilterExplorerSelectionMap;
  currentNodeLabel?: string;
  selectedTarget?: FilterExplorerControllerContext["selectedTarget"];
  selectedPolicyState?: FilterExplorerControllerContext["selectedPolicyState"];
  baseDetailLines: readonly DerivedTagTerminalLine[];
}): DerivedTagTerminalLine[] {
  const selectionEntries = Object.entries(args.selection).filter(
    ([, fieldSelection]) =>
      fieldSelection.any.length > 0 || fieldSelection.all.length > 0 || fieldSelection.exclude.length > 0,
  );
  const lines: DerivedTagTerminalLine[] = [
    ...args.baseDetailLines,
    { text: "" },
    { text: args.mode.focusedSelectionTitle ?? "Focused selection", tone: "section" },
  ];

  if (args.selectedTarget) {
    lines.push({
      text: `${args.selectedTarget.fieldLabel}: ${args.selectedTarget.valueLabel ?? args.currentNodeLabel ?? args.selectedTarget.value}`,
    });
    lines.push({
      text: `Focused policy: ${getFilterExplorerPolicyPresentation(args.selectedPolicyState).label}`,
      segments: [
        { text: "Focused policy: ", tone: "accent" },
        ...buildFilterExplorerPolicyLabelSegments(args.selectedPolicyState),
      ],
    });
  } else {
    lines.push({ text: "Focused node is not selectable.", tone: "dim" });
  }

  lines.push({ text: "" });
  lines.push({ text: args.mode.selectedSelectionsTitle ?? "Selected fields", tone: "section" });
  if (selectionEntries.length === 0) {
    lines.push({ text: args.mode.emptySelectionText ?? "No filter values selected yet.", tone: "dim" });
    return lines;
  }

  for (const [field, fieldSelection] of selectionEntries) {
    lines.push({
      text: `${field}: any=${fieldSelection.any.join(", ")}`,
      segments: buildSelectionEntrySegments(field, fieldSelection),
    });
  }

  return lines;
}

function buildComposeListSegments(
  label: string,
  controller: FilterExplorerControllerContext,
  nodeId: string,
  isSelected: boolean,
): DerivedTagTerminalSegment[] | undefined {
  if (controller.mode.kind !== "compose") {
    return undefined;
  }

  const node = controller.ontology.selection.currentNodes.find((candidate) => candidate.id === nodeId);
  const target = controller.mode.resolveSelectionTarget(node);
  if (!target) {
    return undefined;
  }

  return [
    ...buildFilterExplorerPolicyBadgeSegments(getFilterExplorerTargetState(target, controller.selection)),
    { text: " ", tone: "default" },
    { text: label, tone: isSelected ? "selected" : "default" },
  ];
}

export function buildFilterExplorerListLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  return buildOntologyBrowserListRows(
    controller.model,
    controller.ontology.effectiveState,
    controller.ontology.bodyHeight,
    (node, isSelected) => {
      const label = node.listLabel ?? node.label;
      const segments = buildComposeListSegments(label, controller, node.id, isSelected);
      return segments
        ? {
            text: label,
            segments,
            noWrap: true,
          }
        : {
            text: label,
            tone: isSelected ? "selected" : "default",
            noWrap: true,
          };
    },
  ).map((row) => row.line);
}

export function buildFilterExplorerCommandEntries(
  controller: FilterExplorerControllerContext,
): DerivedTagTerminalCommandOption<"openQuery">[] {
  if (controller.mode.kind !== "inspect-and-open" || !controller.mode.onOpenQuery || !controller.selectedQuery) {
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

export function buildFilterExplorerHelpLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  const interactionActions = getFilterExplorerInteractionActions(controller.mode, controller.ontology);
  const commandEntries = buildFilterExplorerCommandEntries(controller);
  const actionActions = interactionActions
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "open"
          ? "drill into the focused node or open its query"
          : action.id === "cycle"
            ? "cycle the focused filter through off, ∪ include-any, ∩ require-all, and ¬ exclude"
            : action.id === "focus"
              ? "switch focus between list and detail"
              : action.id === "layout"
                ? "toggle split and detail-only layouts"
                : action.id === "cancel"
                  ? "clear the current filter without leaving this level"
                  : action.id === "back"
                    ? getBackHelpText(controller.mode, controller.ontology)
                    : action.id === "search"
                      ? "start live filtering"
                      : action.id === "commands"
                        ? "open the explorer command palette"
                        : action.id === "help"
                          ? "show this help"
                          : "leave the explorer",
      label: action.id === "focus" ? "toggle pane" : action.label,
    }));

  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        {
          id:
            controller.ontology.state.activePane === "list" && controller.ontology.layoutMode !== "detail-only"
              ? "move"
              : "scroll",
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
    ...(controller.mode.kind === "inspect-and-open"
      ? [
          {
            title: "Commands",
            commands: commandEntries.map((command) => ({
              label: command.label,
              description: command.description ?? "No additional details.",
              aliases: command.aliases,
            })),
            lines:
              commandEntries.length === 0
                ? [{ text: "No additional palette commands are available for the current node.", tone: "dim" as const }]
                : [],
          },
        ]
      : []),
  ]);
}

function buildComposeStatus(controller: FilterExplorerControllerContext): string {
  const activeFieldCount = Object.values(controller.selection).filter(
    (fieldSelection) =>
      fieldSelection.any.length > 0 || fieldSelection.all.length > 0 || fieldSelection.exclude.length > 0,
  ).length;
  const focusedPolicy = getFilterExplorerPolicyPresentation(controller.selectedPolicyState).label;
  return `Focused ${focusedPolicy} | ${activeFieldCount} field${activeFieldCount === 1 ? "" : "s"} selected`;
}

function buildInspectStatus(controller: FilterExplorerControllerContext): string {
  return controller.selectedQuery ? "query ready" : "browse only";
}

export function buildFilterExplorerScreenModel(
  controller: FilterExplorerControllerContext,
): { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps } | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
  const interactionActions = getFilterExplorerInteractionActions(controller.mode, controller.ontology);
  const leftLines = buildFilterExplorerListLines(controller);
  const statusSuffix =
    controller.mode.kind === "compose" ? buildComposeStatus(controller) : buildInspectStatus(controller);

  if (controller.ontology.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: controller.screenTitle,
        subtitle: `${controller.ontology.breadcrumb}${controller.ontology.searchIndicator}`,
        pane: {
          title: `[FOCUSED DETAIL] ${controller.ontology.detailTitle}`,
          lines: controller.ontology.visibleDetailLines,
          active: true,
        },
        footer: [
          {
            text: controller.ontology.state.searchMode
              ? TERMINAL_LIVE_FILTER_FOOTER
              : formatTerminalInteractionFooter(interactionActions),
            tone: "dim",
          },
          {
            text: controller.ontology.state.searchMode
              ? `Search /${controller.ontology.state.searchInput}`
              : `detail focus | focused detail view | ${statusSuffix} | Detail scroll ${controller.ontology.effectiveState.detailScroll}/${controller.ontology.maxDetailScroll}`,
            tone: "accent",
          },
        ],
      },
    };
  }

  return {
    kind: "two-pane",
    props: {
      title: controller.screenTitle,
      subtitle: `${controller.ontology.breadcrumb}${controller.ontology.searchIndicator}`,
      left: {
        title: controller.ontology.state.activePane === "list" ? "[LIST] Explorer Entries" : "Explorer Entries",
        lines: leftLines,
        active: controller.ontology.state.activePane === "list",
      },
      right: {
        title:
          controller.ontology.state.activePane === "detail"
            ? `[DETAIL] ${controller.ontology.detailTitle}`
            : controller.ontology.detailTitle,
        lines: controller.ontology.visibleDetailLines,
        active: controller.ontology.state.activePane === "detail",
      },
      footer: [
        {
          text: controller.ontology.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
            : formatTerminalInteractionFooter(interactionActions),
          tone: "dim",
        },
        {
          text: controller.ontology.state.searchMode
            ? `Search /${controller.ontology.state.searchInput}`
            : `${controller.ontology.state.activePane} focus | ${controller.ontology.layoutMode} layout | ${statusSuffix} | Detail scroll ${controller.ontology.effectiveState.detailScroll}/${controller.ontology.maxDetailScroll}`,
          tone: "accent",
        },
      ],
      leftWidth: 46,
    },
  };
}
