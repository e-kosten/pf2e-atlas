import {
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type {
  DerivedTagTerminalCommandOption,
  DerivedTagTerminalLine,
  DerivedTagTerminalSegment,
} from "../framework/types.js";
import {
  buildTerminalListDetailScreenModel,
  type TerminalListDetailScreenModel,
} from "../list-detail-presentation.js";
import { buildFilterExplorerListRows } from "./browser.js";
import {
  getFilterExplorerDiscreteClauseOperator,
  getFilterExplorerScalarClause,
  isFilterExplorerScalarTarget,
} from "./compose-state.js";
import { FILTER_EXPLORER_LAUNCH_INTENT } from "./types.js";
import type {
  FilterExplorerDiscoveryMode,
  FilterExplorerDiscoveryState,
  FilterExplorerComposeMode,
  FilterExplorerBrowserContext,
  FilterExplorerControllerContext,
  FilterExplorerDiscreteClause,
  FilterExplorerDiscreteClauseOperator,
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

function shouldShowCommandPalette(
  mode: FilterExplorerMode,
  discovery?: FilterExplorerDiscoveryState,
): boolean {
  return mode.kind === "inspect-and-open" || Boolean(discovery?.onModeChange);
}

export function getFilterExplorerInteractionActions(
  mode: FilterExplorerMode,
  controller: FilterExplorerBrowserContext,
  discovery?: FilterExplorerDiscoveryState,
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
        ...(shouldShowCommandPalette(mode, discovery) ? [{ id: "commands" as const }] : []),
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
        ...(shouldShowCommandPalette(mode, discovery) ? [{ id: "commands" as const }] : []),
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
      ...(shouldShowCommandPalette(mode, discovery) ? [{ id: "commands" as const }] : []),
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

function getDiscreteClausePresentation(
  operator: FilterExplorerDiscreteClauseOperator | undefined,
): { badge: string; label: string; tone: DerivedTagTerminalSegment["tone"] } {
  switch (operator) {
    case "include":
      return { badge: "check", label: "include", tone: "success" };
    case "exclude":
      return { badge: "x", label: "exclude", tone: "danger" };
    default:
      return { badge: ".", label: "off", tone: "dim" };
  }
}

function buildDiscreteClauseBadgeSegments(
  operator: FilterExplorerDiscreteClauseOperator | undefined,
): DerivedTagTerminalSegment[] {
  const presentation = getDiscreteClausePresentation(operator);
  return [
    { text: "[", tone: "dim" },
    { text: presentation.badge === "check" ? "\u2713" : presentation.badge, tone: presentation.tone },
    { text: "]", tone: "dim" },
  ];
}

function buildDiscreteClauseLabelSegments(
  operator: FilterExplorerDiscreteClauseOperator | undefined,
): DerivedTagTerminalSegment[] {
  const presentation = getDiscreteClausePresentation(operator);
  return [
    ...buildDiscreteClauseBadgeSegments(operator),
    { text: ` ${presentation.label}`, tone: presentation.tone },
  ];
}

function buildDiscreteClauseEntrySegments(
  clause: FilterExplorerDiscreteClause,
): DerivedTagTerminalSegment[] {
  return [
    { text: `${formatOntologySearchVocabularyLabel(clause.field)}: ` },
    ...buildDiscreteClauseLabelSegments(clause.operator),
    { text: ` ${clause.value}` },
  ];
}

function buildGroupedDiscreteClauseEntrySegments(
  field: string,
  operator: FilterExplorerDiscreteClauseOperator,
  values: string[],
): DerivedTagTerminalSegment[] {
  return [
    { text: `${formatOntologySearchVocabularyLabel(field)}: ` },
    ...buildDiscreteClauseLabelSegments(operator),
    { text: ` ${values.join(", ")}` },
  ];
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
  const summaryLabel = formatOntologySearchVocabularyLabel(label);
  return [
    { text: `${summaryLabel}: ` },
    { text: buildFilterExplorerScalarClauseSummary(clause), tone: "accent" },
  ];
}

export function buildFilterExplorerComposeDetailLines(args: {
  mode: FilterExplorerComposeMode;
  draft: FilterExplorerControllerContext["draft"];
  currentNodeLabel?: string;
  selectedTarget?: FilterExplorerControllerContext["selectedTarget"];
  selectedDiscreteClause?: FilterExplorerControllerContext["selectedDiscreteClause"];
  selectedScalarClause?: FilterExplorerControllerContext["selectedScalarClause"];
  baseDetailLines: readonly DerivedTagTerminalLine[];
}): DerivedTagTerminalLine[] {
  const groupedDiscreteClauses = new Map<string, { field: string; operator: FilterExplorerDiscreteClauseOperator; values: string[] }>();
  for (const clause of args.draft.discreteClauses) {
    const key = `${clause.field}\u0000${clause.operator}`;
    const existing = groupedDiscreteClauses.get(key);
    if (existing) {
      existing.values.push(clause.value);
      continue;
    }
    groupedDiscreteClauses.set(key, {
      field: clause.field,
      operator: clause.operator,
      values: [clause.value],
    });
  }
  const scalarEntries = Object.entries(args.draft.scalarClauses);
  const lines: DerivedTagTerminalLine[] = [
    ...args.baseDetailLines,
    { text: "" },
    { text: args.mode.focusedClauseTitle ?? "Focused clause", tone: "section" },
  ];

  if (args.selectedTarget && args.selectedTarget.kind !== "scalar") {
    lines.push({
      text: `${args.selectedTarget.fieldLabel}: ${args.selectedTarget.valueLabel ?? args.currentNodeLabel ?? args.selectedTarget.value}`,
    });
    lines.push({
      text: `Focused clause: ${getDiscreteClausePresentation(args.selectedDiscreteClause?.operator).label}`,
      segments: [
        { text: "Focused clause: ", tone: "accent" },
        ...buildDiscreteClauseLabelSegments(args.selectedDiscreteClause?.operator),
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
  lines.push({ text: args.mode.stagedClausesTitle ?? "Staged clauses", tone: "section" });
  if (groupedDiscreteClauses.size === 0 && scalarEntries.length === 0) {
    lines.push({ text: args.mode.emptySelectionText ?? "No filter values selected yet.", tone: "dim" });
    return lines;
  }

  for (const entry of [...groupedDiscreteClauses.values()].sort((left, right) =>
    left.field.localeCompare(right.field) || left.operator.localeCompare(right.operator),
  )) {
    const values = [...new Set(entry.values)].sort((left, right) => left.localeCompare(right));
    lines.push({
      text: `${formatOntologySearchVocabularyLabel(entry.field)}: ${getDiscreteClausePresentation(entry.operator).label} ${values.join(", ")}`,
      segments: buildGroupedDiscreteClauseEntrySegments(entry.field, entry.operator, values),
    });
  }

  for (const [key, clause] of scalarEntries) {
    const clauseLabel = formatOntologySearchVocabularyLabel(key);
    lines.push({
      text: `${clauseLabel}: ${buildFilterExplorerScalarClauseSummary(clause)}`,
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

  const operator = getFilterExplorerDiscreteClauseOperator(target, controller.draft);
  return [
    ...buildDiscreteClauseBadgeSegments(operator),
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

type FilterExplorerCommandValue =
  | "openSelection"
  | "openQuery"
  | "openResults"
  | "switchToMatching"
  | "switchToCatalog";

function buildDiscoveryModeCommandEntries(
  discovery: FilterExplorerDiscoveryState | undefined,
): DerivedTagTerminalCommandOption<FilterExplorerCommandValue>[] {
  if (!discovery?.onModeChange) {
    return [];
  }

  const availableModes = discovery.availableModes ?? (["matching", "catalog"] as const);
  const labelByMode: Record<FilterExplorerDiscoveryMode, string> = {
    matching: "Matching",
    catalog: "Catalog",
  };
  const descriptionByMode: Record<FilterExplorerDiscoveryMode, string> = {
    matching: "Show values and counts from the current matching query context.",
    catalog: "Show values and counts from the wider applicability slice only.",
  };

  return availableModes
    .filter((mode) => mode !== discovery.mode)
    .map((mode) => ({
      value: mode === "matching" ? "switchToMatching" : "switchToCatalog",
      label: `Use ${labelByMode[mode]} Counts`,
      description: descriptionByMode[mode],
      keywords: ["discovery", "counts", "mode", labelByMode[mode].toLowerCase()],
    }));
}

export function buildFilterExplorerCommandEntries(
  controller: FilterExplorerControllerContext,
): DerivedTagTerminalCommandOption<FilterExplorerCommandValue>[] {
  const commands = buildDiscoveryModeCommandEntries(controller.discovery);

  if (
    controller.mode.kind !== "inspect-and-open" ||
    (!controller.mode.onOpenInspectResult && !controller.mode.onOpenQueryIntent) ||
    !controller.selectedInspectResult
  ) {
    return commands;
  }

  const result = controller.selectedInspectResult;
  const request = result.query.request;
  const targetLabel = buildInspectTargetLabel(result);
  const resultsDescription = buildInspectCommandDescription(result);
  const queryDescription = targetLabel
    ? `Seed the browse/search editor from the focused target: ${targetLabel}.`
    : "Seed the browse/search editor from the focused selection.";

  return [
    ...commands,
    ...(request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
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
      value: request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
        ? "openQuery"
        : "openSelection",
      label: request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
        ? "Open Search Query"
        : "Open Query",
      description: queryDescription,
      aliases: request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
        ? ["Open Query"]
        : undefined,
      keywords: ["search", "browse", "editor", "query", "open"],
    },
  ];
}

export function buildFilterExplorerHelpLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  const interactionActions = getFilterExplorerInteractionActions(
    controller.mode,
    controller.browser,
    controller.discovery,
  );
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
              : "cycle the focused discrete clause through off and the allowed include/exclude operators"
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
                        ? controller.discovery?.onModeChange
                          ? "open explorer commands, including discovery-mode switching when available"
                          : "open the explorer command palette"
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
  const discreteClauseCount = controller.draft.discreteClauses.length;
  const scalarClauseCount = Object.keys(controller.draft.scalarClauses).length;
  const totalCount = discreteClauseCount + scalarClauseCount;

  if (controller.selectedTarget?.kind === "scalar") {
    const focusedSummary = controller.selectedScalarClause
      ? buildFilterExplorerScalarClauseSummary(controller.selectedScalarClause)
      : "editor ready";
    return `Focused ${focusedSummary} | ${totalCount} filter${totalCount === 1 ? "" : "s"} selected`;
  }

  const focusedClause = getDiscreteClausePresentation(controller.selectedDiscreteClause?.operator).label;
  return `Focused ${focusedClause} | ${totalCount} filter${totalCount === 1 ? "" : "s"} selected`;
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
  const request = result.query.request;
  const openLabel =
    result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? "Open the focused selection in results."
      : request.mode === "lookup"
        ? "Open the focused selection in lookup."
        : request.mode === "search"
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
  const request = result.query.request;
  const openLabel =
    result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? "open results"
      : request.mode === "lookup"
        ? "open lookup"
        : request.mode === "search"
          ? "open search"
          : "open browse";

  return targetLabel ? `Focused ${targetLabel} | ${openLabel}` : openLabel;
}

export function buildFilterExplorerScreenModel(
  controller: FilterExplorerControllerContext,
): TerminalListDetailScreenModel {
  const interactionActions = getFilterExplorerInteractionActions(
    controller.mode,
    controller.browser,
    controller.discovery,
  );
  const leftLines = buildFilterExplorerListLines(controller);
  const statusSuffix =
    controller.mode.kind === "compose" ? buildComposeStatus(controller) : buildInspectStatus(controller);
  const statusText = controller.discovery ? `${statusSuffix} | ${controller.discovery.mode} counts` : statusSuffix;

  return buildTerminalListDetailScreenModel({
    title: controller.screenTitle,
    subtitle: `${controller.browser.breadcrumb}${controller.browser.searchIndicator}`,
    activePane: controller.browser.state.activePane,
    layoutMode: controller.browser.layoutMode,
    leftWidth: 46,
    leftPane: {
      title: controller.browser.state.activePane === "list" ? "[LIST] Explorer Entries" : "Explorer Entries",
      lines: leftLines,
    },
    rightPane: {
      title:
        controller.browser.state.activePane === "detail"
          ? `[DETAIL] ${controller.browser.detailTitle}`
          : controller.browser.detailTitle,
      detailOnlyTitle: `[FOCUSED DETAIL] ${controller.browser.detailTitle}`,
    },
    metrics: {
      visibleDetailLines: controller.browser.visibleDetailLines,
    },
    footer: [
      {
        text: controller.browser.state.searchMode
            ? TERMINAL_LIVE_FILTER_FOOTER
          : formatTerminalInteractionFooter(interactionActions),
        tone: "dim",
      },
      {
        text: controller.browser.state.searchMode
          ? `Search /${controller.browser.state.searchInput}`
          : controller.browser.layoutMode === "detail-only"
            ? `detail focus | focused detail view | ${statusText} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`
            : `${controller.browser.state.activePane} focus | ${controller.browser.layoutMode} layout | ${statusText} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`,
        tone: "accent",
      },
    ],
    notification: controller.notification,
    transitionStatus: controller.transitionStatus,
  });
}
