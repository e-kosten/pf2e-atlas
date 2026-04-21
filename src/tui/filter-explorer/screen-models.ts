import {
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import {
  appendRouteTransitionFooterLine,
} from "../route-transition-status.js";
import type {
  DerivedTagTerminalCommandOption,
  DerivedTagTerminalLine,
  DerivedTagTerminalPaneScreenProps,
  DerivedTagTerminalSegment,
  DerivedTagTerminalTwoPaneScreenProps,
} from "../framework/types.js";
import { buildFilterExplorerListRows } from "./browser.js";
import {
  getFilterExplorerScalarClause,
  getFilterExplorerTargetState,
  isFilterExplorerScalarTarget,
} from "./compose-state.js";
import {
  buildFilterExplorerPolicyBadgeSegments,
  buildFilterExplorerPolicyLabelSegments,
  getFilterExplorerPolicyPresentation,
} from "./policy-presentation.js";
import { FILTER_EXPLORER_LAUNCH_INTENT } from "./types.js";
import type {
  FilterExplorerComposeMode,
  FilterExplorerBrowserContext,
  FilterExplorerControllerContext,
  FilterExplorerInspectResult,
  FilterExplorerMode,
  FilterExplorerScalarClause,
} from "./types.js";

function isDetailContext(controller: FilterExplorerBrowserContext): boolean {
  return controller.layoutMode === "detail-only" || controller.state.activePane === "detail";
}

function isAtExitDepth(mode: FilterExplorerMode, controller: FilterExplorerBrowserContext, rootDepth: number): boolean {
  return mode.kind === "compose" && controller.effectiveState.depth === rootDepth;
}

function getBackHelpText(mode: FilterExplorerMode, controller: FilterExplorerBrowserContext): string {
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
  controller: FilterExplorerBrowserContext,
): TerminalInteractionAction[] {
  if (mode.kind === "compose") {
    const focusedTarget = mode.resolveSelectionTarget(controller.selection.currentNode);
    const composeActionLabel = focusedTarget?.kind === "scalar" ? "edit" : "cycle";

    if (controller.layoutMode === "detail-only") {
      return [
        { id: "scroll" },
        { id: "jump" },
        { id: "page" },
        { id: "edge" },
        { id: "cycle", label: composeActionLabel },
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
        { id: "cycle", label: composeActionLabel },
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
  selection: FilterExplorerControllerContext["selection"][string],
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
    segments.push(...buildFilterExplorerPolicyBadgeSegments(state, { fallback: "discoverable" }));
    segments.push({ text: ` ${values.join(", ")}` });
    first = false;
  }

  return segments;
}

function formatFilterExplorerScalarValue(value: string | number | boolean): string {
  return typeof value === "string" ? value : String(value);
}

function buildFilterExplorerScalarClauseSummary(clause: FilterExplorerScalarClause): string {
  if (clause.summaryLabel) {
    return clause.summaryLabel;
  }

  if (clause.operator === "between") {
    return `between ${clause.min} and ${clause.max}`;
  }

  const operatorLabel = {
    eq: "=",
    neq: "!=",
    gte: ">=",
    lte: "<=",
  }[clause.operator];

  return `${operatorLabel} ${clause.valueLabel ?? formatFilterExplorerScalarValue(clause.value)}`;
}

function buildScalarClauseEntrySegments(label: string, clause: FilterExplorerScalarClause): DerivedTagTerminalSegment[] {
  return [
    { text: `${label}: ` },
    { text: buildFilterExplorerScalarClauseSummary(clause), tone: "accent" },
  ];
}

export function buildFilterExplorerComposeDetailLines(args: {
  mode: FilterExplorerComposeMode;
  draft: FilterExplorerControllerContext["draft"];
  currentNodeLabel?: string;
  selectedTarget?: FilterExplorerControllerContext["selectedTarget"];
  selectedPolicyState?: FilterExplorerControllerContext["selectedPolicyState"];
  selectedScalarClause?: FilterExplorerControllerContext["selectedScalarClause"];
  baseDetailLines: readonly DerivedTagTerminalLine[];
}): DerivedTagTerminalLine[] {
  const selectionEntries = Object.entries(args.draft.selection).filter(
    ([, fieldSelection]) =>
      fieldSelection.any.length > 0 || fieldSelection.all.length > 0 || fieldSelection.exclude.length > 0,
  );
  const scalarEntries = Object.entries(args.draft.scalarClauses);
  const lines: DerivedTagTerminalLine[] = [
    ...args.baseDetailLines,
    { text: "" },
    { text: args.mode.focusedSelectionTitle ?? "Focused selection", tone: "section" },
  ];

  if (args.selectedTarget && args.selectedTarget.kind !== "scalar") {
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
  } else if (args.selectedTarget?.kind === "scalar") {
    lines.push({
      text: `${args.selectedTarget.fieldLabel}: ${args.selectedTarget.subjectLabel}`,
    });
    lines.push({
      text: args.selectedScalarClause
        ? `Focused clause: ${buildFilterExplorerScalarClauseSummary(args.selectedScalarClause)}`
        : `Focused clause: ${args.selectedTarget.editorLabel ?? "Open editor"}`,
      segments: args.selectedScalarClause
        ? [
            { text: "Focused clause: ", tone: "accent" },
            { text: buildFilterExplorerScalarClauseSummary(args.selectedScalarClause), tone: "accent" },
          ]
        : [
            { text: "Focused clause: ", tone: "accent" },
            { text: args.selectedTarget.editorLabel ?? "Open editor", tone: "dim" },
          ],
    });
  } else {
    lines.push({ text: "Focused node is not selectable.", tone: "dim" });
  }

  lines.push({ text: "" });
  lines.push({ text: args.mode.selectedSelectionsTitle ?? "Selected fields", tone: "section" });
  if (selectionEntries.length === 0 && scalarEntries.length === 0) {
    lines.push({ text: args.mode.emptySelectionText ?? "No filter values selected yet.", tone: "dim" });
    return lines;
  }

  for (const [field, fieldSelection] of selectionEntries) {
    lines.push({
      text: `${field}: any=${fieldSelection.any.join(", ")}`,
      segments: buildSelectionEntrySegments(field, fieldSelection),
    });
  }

  for (const [key, clause] of scalarEntries) {
    lines.push({
      text: `${key}: ${buildFilterExplorerScalarClauseSummary(clause)}`,
      segments: buildScalarClauseEntrySegments(key, clause),
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

  const node = controller.browser.selection.currentNodes.find((candidate) => candidate.id === nodeId);
  const target = controller.mode.resolveSelectionTarget(node);
  if (!target) {
    return undefined;
  }

  if (isFilterExplorerScalarTarget(target)) {
    const clause = getFilterExplorerScalarClause(target, controller.draft);
    return [
      { text: clause ? "ƒ" : "·", tone: clause ? "accent" : "dim" },
      { text: " ", tone: "default" },
      { text: label, tone: isSelected ? "selected" : "default" },
      ...(clause
        ? [
            { text: "  ", tone: "dim" } as const,
            { text: buildFilterExplorerScalarClauseSummary(clause), tone: "accent" as const },
          ]
        : []),
    ];
  }

  return [
    ...buildFilterExplorerPolicyBadgeSegments(getFilterExplorerTargetState(target, controller.selection)),
    { text: " ", tone: "default" },
    { text: label, tone: isSelected ? "selected" : "default" },
  ];
}

export function buildFilterExplorerListLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  return buildFilterExplorerListRows(
    controller.model,
    controller.browser.effectiveState,
    controller.browser.bodyHeight,
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
): DerivedTagTerminalCommandOption<"openSelection" | "openQuery" | "openResults">[] {
  if (
    controller.mode.kind !== "inspect-and-open" ||
    (!controller.mode.onOpenInspectResult && !controller.mode.onOpenQuery) ||
    !controller.selectedInspectResult
  ) {
    return [];
  }

  const result = controller.selectedInspectResult;
  const targetLabel = buildInspectTargetLabel(result);
  const resultsDescription = buildInspectCommandDescription(result);
  const queryDescription = targetLabel
    ? `Seed the browse/search editor from the focused target: ${targetLabel}.`
    : "Seed the browse/search editor from the focused selection.";

  return [
    ...(result.query.kind === "listRecords" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? [
          {
            value: "openResults" as const,
            label: "Open Results Page",
            description: resultsDescription,
            aliases: ["Open Selection"],
            keywords: ["results", "reader", "records", "open"],
          },
        ]
      : []),
    {
      value:
        result.query.kind === "listRecords" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
          ? "openQuery"
          : "openSelection",
      label:
        result.query.kind === "listRecords" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
          ? "Open Search Query"
          : "Open Query",
      description: queryDescription,
      aliases:
        result.query.kind === "listRecords" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
          ? ["Open Query"]
          : undefined,
      keywords: ["search", "browse", "editor", "query", "open"],
    },
  ];
}

export function buildFilterExplorerHelpLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  const interactionActions = getFilterExplorerInteractionActions(controller.mode, controller.browser);
  const commandEntries = buildFilterExplorerCommandEntries(controller);
  const actionActions = interactionActions
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "open"
          ? "drill into the focused node or open the focused selection"
          : action.id === "cycle"
            ? controller.selectedTarget?.kind === "scalar"
              ? "open the focused scalar filter editor"
              : "cycle the focused filter through off, ∪ include-any, ∩ require-all, and ¬ exclude"
            : action.id === "focus"
              ? "switch focus between list and detail"
              : action.id === "layout"
                ? "toggle split and detail-only layouts"
                : action.id === "cancel"
                  ? "clear the current filter without leaving this level"
                  : action.id === "back"
                    ? getBackHelpText(controller.mode, controller.browser)
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
            controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
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
  const activeFieldCount = Object.values(controller.draft.selection).filter(
    (fieldSelection) =>
      fieldSelection.any.length > 0 || fieldSelection.all.length > 0 || fieldSelection.exclude.length > 0,
  ).length;
  const scalarClauseCount = Object.keys(controller.draft.scalarClauses).length;
  const totalCount = activeFieldCount + scalarClauseCount;

  if (controller.selectedTarget?.kind === "scalar") {
    const focusedSummary = controller.selectedScalarClause
      ? buildFilterExplorerScalarClauseSummary(controller.selectedScalarClause)
      : "editor ready";
    return `Focused ${focusedSummary} | ${totalCount} filter${totalCount === 1 ? "" : "s"} selected`;
  }

  const focusedPolicy = getFilterExplorerPolicyPresentation(controller.selectedPolicyState).label;
  return `Focused ${focusedPolicy} | ${totalCount} filter${totalCount === 1 ? "" : "s"} selected`;
}

function buildInspectTargetLabel(result: FilterExplorerInspectResult): string | null {
  if (!result.target) {
    return null;
  }

  return result.target.kind === "scalar"
    ? `${result.target.fieldLabel} / ${result.target.subjectLabel}`
    : `${result.target.fieldLabel}: ${result.target.valueLabel ?? result.target.value}`;
}

function buildInspectCommandDescription(result: FilterExplorerInspectResult): string {
  const openLabel =
    result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? "Open the focused selection in results."
      : result.query.kind === "lookup"
        ? "Open the focused selection in lookup."
        : result.query.kind === "search"
          ? "Open the focused selection in search."
          : "Open the focused selection in browse.";

  const targetLabel = buildInspectTargetLabel(result);
  return targetLabel ? `${openLabel} Focused target: ${targetLabel}.` : openLabel;
}

function buildInspectStatus(controller: FilterExplorerControllerContext): string {
  const result = controller.selectedInspectResult;
  if (!result) {
    return "browse only";
  }

  const targetLabel = buildInspectTargetLabel(result);
  const openLabel =
    result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? "open results"
      : result.query.kind === "lookup"
        ? "open lookup"
        : result.query.kind === "search"
          ? "open search"
          : "open browse";

  return targetLabel ? `Focused ${targetLabel} | ${openLabel}` : openLabel;
}

export function buildFilterExplorerScreenModel(
  controller: FilterExplorerControllerContext,
): { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps } | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
  const interactionActions = getFilterExplorerInteractionActions(controller.mode, controller.browser);
  const leftLines = buildFilterExplorerListLines(controller);
  const statusSuffix =
    controller.mode.kind === "compose" ? buildComposeStatus(controller) : buildInspectStatus(controller);

  if (controller.browser.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: controller.screenTitle,
        subtitle: `${controller.browser.breadcrumb}${controller.browser.searchIndicator}`,
        pane: {
          title: `[FOCUSED DETAIL] ${controller.browser.detailTitle}`,
          lines: controller.browser.visibleDetailLines,
          active: true,
        },
        footer: appendRouteTransitionFooterLine([
          {
            text: controller.browser.state.searchMode
              ? TERMINAL_LIVE_FILTER_FOOTER
              : formatTerminalInteractionFooter(interactionActions),
            tone: "dim",
          },
          {
            text: controller.browser.state.searchMode
              ? `Search /${controller.browser.state.searchInput}`
              : `detail focus | focused detail view | ${statusSuffix} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`,
            tone: "accent",
          },
        ], controller.transitionStatus),
      },
    };
  }

  return {
    kind: "two-pane",
    props: {
      title: controller.screenTitle,
      subtitle: `${controller.browser.breadcrumb}${controller.browser.searchIndicator}`,
      left: {
        title: controller.browser.state.activePane === "list" ? "[LIST] Explorer Entries" : "Explorer Entries",
        lines: leftLines,
        active: controller.browser.state.activePane === "list",
      },
      right: {
        title:
          controller.browser.state.activePane === "detail"
            ? `[DETAIL] ${controller.browser.detailTitle}`
            : controller.browser.detailTitle,
        lines: controller.browser.visibleDetailLines,
        active: controller.browser.state.activePane === "detail",
      },
      footer: appendRouteTransitionFooterLine([
        {
          text: controller.browser.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
            : formatTerminalInteractionFooter(interactionActions),
          tone: "dim",
        },
        {
          text: controller.browser.state.searchMode
            ? `Search /${controller.browser.state.searchInput}`
            : `${controller.browser.state.activePane} focus | ${controller.browser.layoutMode} layout | ${statusSuffix} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`,
          tone: "accent",
        },
      ], controller.transitionStatus),
      leftWidth: 46,
    },
  };
}
