import {
  buildDerivedTagTerminalActionTargetHelpLines,
  buildDerivedTagTerminalActionTargetLine,
  getDerivedTagTerminalActionTargetInteractionActions,
  shouldRenderDerivedTagTerminalActionTarget,
} from "../action-target.js";
import {
  TERMINAL_LIVE_FILTER_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import { formatOntologySearchVocabularyLabel } from "../../domain/presentation-vocabulary.js";
import type { DerivedTagTerminalLine, DerivedTagTerminalSegment } from "../framework/types.js";
import { buildTerminalListDetailScreenModel, type TerminalListDetailScreenModel } from "../list-detail-presentation.js";
import { buildFilterExplorerListRows } from "./browser.js";
import { describeFilterExplorerHostNode } from "./host-adapter.js";
import { FILTER_EXPLORER_LAUNCH_INTENT } from "./types.js";
import type { TerminalDebugTraceSnapshot, TerminalDebugTraceSpanSnapshot } from "../debug-trace.js";
import type {
  FilterExplorerActionEntry,
  FilterExplorerActivationStyle,
  FilterExplorerDiscoveryState,
  FilterExplorerComposeMode,
  FilterExplorerBrowserContext,
  FilterExplorerControllerContext,
  FilterExplorerDiscreteClauseOperator,
  FilterExplorerInspectResult,
  FilterExplorerMode,
  FilterExplorerScalarClause,
  FilterExplorerStateBadge,
} from "./types.js";

function isDetailContext(controller: FilterExplorerBrowserContext): boolean {
  return controller.layoutMode === "detail-only" || controller.state.activePane === "detail";
}

function isAtExitDepth(mode: FilterExplorerMode, controller: FilterExplorerBrowserContext, rootDepth: number): boolean {
  return controller.effectiveState.depth === rootDepth;
}

function getBackHelpText(mode: FilterExplorerMode, controller: FilterExplorerBrowserContext): string {
  if (controller.detailInteractionState.kind === "target") {
    return "leave target mode and return to section navigation";
  }
  if (isDetailContext(controller)) {
    return "return to the entry list";
  }
  return isAtExitDepth(mode, controller, 0)
    ? mode.kind === "compose"
      ? "leave the filter composer"
      : "return from the explorer"
    : "return to the previous level";
}

function shouldShowActionRail(hasActionEntries: boolean): boolean {
  return hasActionEntries;
}

const FILTER_EXPLORER_NAVIGATION_ACTION_IDS = new Set([
  "move",
  "viewportScrollSmall",
  "viewportScrollLarge",
  "viewportPage",
  "viewportEdge",
  "scroll",
  "jump",
  "page",
  "edge",
]);

function isSelectionExplorer(controller: FilterExplorerControllerContext): boolean {
  return controller.mode.kind === "compose" || Boolean(controller.host.selectionPresentation);
}

function getCurrentActivationStyle(controller: FilterExplorerControllerContext): FilterExplorerActivationStyle {
  return (
    describeFilterExplorerHostNode({
      host: controller.host,
      node: controller.browser.selection.currentNode,
      isFocused: true,
      controller,
    })?.activationStyle ?? "none"
  );
}

export function getFilterExplorerInteractionActions(
  controller: FilterExplorerControllerContext,
  hasActionEntries = false,
): TerminalInteractionAction[] {
  const { browser } = controller;
  const pageDetailState = browser.detailInteractionState;
  if (isSelectionExplorer(controller)) {
    const composeActionLabel = getCurrentActivationStyle(controller) === "edit" ? "edit" : "cycle";

    if (browser.layoutMode === "detail-only") {
      return [
        { id: "viewportScrollSmall" },
        { id: "viewportScrollLarge" },
        { id: "viewportPage" },
        { id: "viewportEdge" },
        { id: "cycle", label: composeActionLabel },
        { id: "layout", label: "split-view" },
        { id: "back" },
        { id: "search" },
        ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
        { id: "help" },
        { id: "quit", label: "back" },
      ];
    }

    if (browser.state.activePane === "list") {
      return [
        { id: "move", label: "select" },
        { id: "jump" },
        { id: "page" },
        { id: "edge" },
        { id: "cycle", label: composeActionLabel },
        { id: "focus", label: "pane" },
        { id: "layout", label: "detail-only" },
        ...(browser.effectiveState.filter ? [{ id: "cancel" as const, label: "clear filter" }] : []),
        { id: "back" },
        { id: "search" },
        ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
        { id: "help" },
        { id: "quit", label: "back" },
      ];
    }

    return [
      { id: "viewportScrollSmall" },
      { id: "viewportScrollLarge" },
      { id: "viewportPage" },
      { id: "viewportEdge" },
      { id: "focus", label: "pane" },
      { id: "layout", label: "detail-only" },
      { id: "back" },
      { id: "search" },
      ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  const inspectActionLabel = getCurrentActivationStyle(controller) === "edit" ? "edit" : "open";

  if (pageDetailState.kind === "section" || pageDetailState.kind === "target") {
    const pageActions: TerminalInteractionAction[] = [
      { id: "move", label: pageDetailState.kind === "target" ? "target" : "section" },
      { id: "viewportScrollSmall" },
      { id: "viewportScrollLarge" },
      { id: "viewportPage" },
      { id: "viewportEdge" },
      ...(pageDetailState.kind === "section" && pageDetailState.canEnterTargets
        ? [{ id: "select", label: "targets" } as const]
        : []),
      ...(pageDetailState.kind === "target" && browser.detailTargetActionId
        ? [{ id: browser.detailTargetActionId } as const]
        : []),
    ];

    if (browser.layoutMode === "detail-only") {
      return [
        ...pageActions,
        { id: "layout", label: "split-view" },
        { id: "back" },
        { id: "search" },
        ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
        { id: "help" },
        { id: "quit", label: "back" },
      ];
    }

    if (browser.state.activePane === "detail") {
      return [
        ...pageActions,
        { id: "focus", label: "pane" },
        { id: "layout", label: "detail-only" },
        { id: "back" },
        { id: "search" },
        ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
        { id: "help" },
        { id: "quit", label: "back" },
      ];
    }
  }

  if (browser.layoutMode === "detail-only") {
    return [
      { id: "viewportScrollSmall" },
      { id: "viewportScrollLarge" },
      { id: "viewportPage" },
      { id: "viewportEdge" },
      { id: "layout", label: "split-view" },
      { id: "back" },
      { id: "search" },
      ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  if (browser.state.activePane === "list") {
    return [
      { id: "move" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "open", label: inspectActionLabel },
      { id: "focus", label: "pane" },
      { id: "layout", label: "detail-only" },
      ...(browser.effectiveState.filter ? [{ id: "cancel" as const, label: "clear filter" }] : []),
      { id: "back" },
      { id: "search" },
      ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
      { id: "help" },
      { id: "quit", label: "back" },
    ];
  }

  return [
    { id: "viewportScrollSmall" },
    { id: "viewportScrollLarge" },
    { id: "viewportPage" },
    { id: "viewportEdge" },
    { id: "focus", label: "pane" },
    { id: "layout", label: "detail-only" },
    { id: "back" },
    { id: "search" },
    ...(shouldShowActionRail(hasActionEntries) ? [{ id: "actions" as const }] : []),
    { id: "help" },
    { id: "quit", label: "back" },
  ];
}

function getStateBadgePresentation(badge: FilterExplorerStateBadge | undefined): {
  text: string;
  label: string;
  tone: DerivedTagTerminalSegment["tone"];
  bracketed: boolean;
} {
  switch (badge?.kind) {
    case "include":
      return { text: "\u2713", label: "include", tone: "success", bracketed: true };
    case "exclude":
      return { text: "x", label: "exclude", tone: "danger", bracketed: true };
    case "custom":
      return { text: badge.text, label: badge.text, tone: badge.tone ?? "accent", bracketed: false };
    case "off":
    case undefined:
      return { text: ".", label: "off", tone: "dim", bracketed: true };
  }
}

function getDiscreteOperatorStateBadge(operator: FilterExplorerDiscreteClauseOperator): FilterExplorerStateBadge {
  return operator === "include" ? { kind: "include" } : { kind: "exclude" };
}

function buildStateBadgeSegments(badge: FilterExplorerStateBadge | undefined): DerivedTagTerminalSegment[] {
  const presentation = getStateBadgePresentation(badge);
  if (!presentation.bracketed) {
    return [{ text: presentation.text, tone: presentation.tone }];
  }

  return [
    { text: "[", tone: "dim" },
    { text: presentation.text, tone: presentation.tone },
    { text: "]", tone: "dim" },
  ];
}

function buildStateBadgeLabelSegments(badge: FilterExplorerStateBadge | undefined): DerivedTagTerminalSegment[] {
  const presentation = getStateBadgePresentation(badge);
  return [...buildStateBadgeSegments(badge), { text: ` ${presentation.label}`, tone: presentation.tone }];
}

function buildGroupedDiscreteClauseEntrySegments(
  field: string,
  operator: FilterExplorerDiscreteClauseOperator,
  values: string[],
): DerivedTagTerminalSegment[] {
  return [
    { text: `${formatOntologySearchVocabularyLabel(field)}: ` },
    ...buildStateBadgeLabelSegments(getDiscreteOperatorStateBadge(operator)),
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
    gt: ">",
    gte: ">=",
    lt: "<",
    lte: "<=",
  }[clause.operator];

  return `${operatorLabel} ${clause.valueLabel ?? formatFilterExplorerScalarValue(clause.value)}`;
}

function buildScalarClauseEntrySegments(
  label: string,
  clause: FilterExplorerScalarClause,
): DerivedTagTerminalSegment[] {
  const summaryLabel = formatOntologySearchVocabularyLabel(label);
  return [{ text: `${summaryLabel}: ` }, { text: buildFilterExplorerScalarClauseSummary(clause), tone: "accent" }];
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
  const groupedDiscreteClauses = new Map<
    string,
    { field: string; operator: FilterExplorerDiscreteClauseOperator; values: string[] }
  >();
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
      text: `Focused clause: ${
        getStateBadgePresentation(
          args.selectedDiscreteClause?.operator === "include"
            ? { kind: "include" }
            : args.selectedDiscreteClause?.operator === "exclude"
              ? { kind: "exclude" }
              : { kind: "off" },
        ).label
      }`,
      segments: [
        { text: "Focused clause: ", tone: "accent" },
        ...buildStateBadgeLabelSegments(
          args.selectedDiscreteClause?.operator === "include"
            ? { kind: "include" }
            : args.selectedDiscreteClause?.operator === "exclude"
              ? { kind: "exclude" }
              : { kind: "off" },
        ),
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

  for (const entry of [...groupedDiscreteClauses.values()].sort(
    (left, right) => left.field.localeCompare(right.field) || left.operator.localeCompare(right.operator),
  )) {
    const values = [...new Set(entry.values)].sort((left, right) => left.localeCompare(right));
    lines.push({
      text: `${formatOntologySearchVocabularyLabel(entry.field)}: ${
        getStateBadgePresentation(getDiscreteOperatorStateBadge(entry.operator)).label
      } ${values.join(", ")}`,
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
  const node = controller.browser.selection.currentNodes.find((candidate) => candidate.id === nodeId);
  if (!node) {
    return undefined;
  }

  const presentation = describeFilterExplorerHostNode({
    host: controller.host,
    node,
    isFocused: isSelected,
    controller,
  });
  if (!presentation || (!presentation.stateBadge && !presentation.suffixText)) {
    return undefined;
  }

  return [
    ...(presentation.stateBadge
      ? [...buildStateBadgeSegments(presentation.stateBadge), { text: " ", tone: "default" as const }]
      : []),
    { text: label, tone: isSelected ? "selected" : (presentation.tone ?? "default") },
    ...(presentation.suffixText
      ? [{ text: "  ", tone: "dim" } as const, { text: presentation.suffixText, tone: "accent" as const }]
      : []),
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

function buildDiscoveryModeActionEntries(
  discovery: FilterExplorerDiscoveryState | undefined,
): FilterExplorerActionEntry[] {
  if (!discovery?.onModeChange || discovery.modes.length === 0) {
    return [];
  }

  const hiddenMode =
    discovery.isRefreshing && discovery.pendingMode && discovery.pendingMode !== discovery.mode
      ? discovery.pendingMode
      : discovery.mode;

  return discovery.modes
    .filter((mode) => mode.value !== hiddenMode)
    .map((mode) => ({
      id: `setMode:${mode.value}` as const,
      label: `Use ${mode.label}`,
      description: mode.description,
      action: {
        kind: "setMode" as const,
        mode: mode.value,
      },
    }));
}

export function buildFilterExplorerActionEntries(
  controller: FilterExplorerControllerContext,
): FilterExplorerControllerContext["actionEntries"] {
  const actions = buildDiscoveryModeActionEntries(controller.discovery);

  if (controller.mode.kind !== "inspect-and-open" || !controller.selectedInspectResult) {
    return actions;
  }

  const result = controller.selectedInspectResult;
  const request = result.query.request;
  const targetLabel = buildInspectTargetLabel(result);
  const resultsDescription = buildInspectCommandDescription(result);
  const queryDescription = targetLabel
    ? `Seed the browse/search editor from the focused target: ${targetLabel}.`
    : "Seed the browse/search editor from the focused selection.";

  return [
    ...actions,
    ...(request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
      ? [
          {
            id: "selectTarget:default" as const,
            label: "Open Results Page",
            description: resultsDescription,
            action: {
              kind: "selectTarget" as const,
              selection: "default" as const,
            },
          },
        ]
      : []),
    {
      id:
        request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
          ? "selectTarget:query"
          : "selectTarget:default",
      label:
        request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
          ? "Open Search Query"
          : "Open Query",
      description: queryDescription,
      action: {
        kind: "selectTarget" as const,
        selection:
          request.mode === "browse" && result.launchIntent === FILTER_EXPLORER_LAUNCH_INTENT.RESULTS
            ? "query"
            : "default",
      },
    },
  ];
}

export function buildFilterExplorerHelpLines(controller: FilterExplorerControllerContext): DerivedTagTerminalLine[] {
  const interactionActions = getFilterExplorerInteractionActions(controller, controller.actionEntries.length > 0);
  const pageDetailState = controller.browser.detailInteractionState;
  const actionActions = interactionActions
    .filter((action) => !FILTER_EXPLORER_NAVIGATION_ACTION_IDS.has(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "select"
          ? "enter link targets inside the active section"
          : action.id === "preview"
            ? "preview the focused page target"
            : action.id === "open"
              ? pageDetailState.kind === "target"
                ? "open the focused page target"
                : "drill into the focused node or open the focused selection"
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
                          : action.id === "actions"
                            ? "focus the explorer action rail"
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
            controller.browser.detailInteractionState.kind === "section" ||
            controller.browser.detailInteractionState.kind === "target"
              ? "move"
              : controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
                ? "move"
                : "viewportScrollSmall",
          helpText:
            controller.browser.detailInteractionState.kind === "target"
              ? "move through targets in the active section"
              : controller.browser.detailInteractionState.kind === "section"
                ? "move through sections in the preview"
                : controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
                  ? "move through the active pane"
                  : "scroll the active pane",
        },
        {
          id:
            controller.browser.detailInteractionState.kind === "section" ||
            controller.browser.detailInteractionState.kind === "target"
              ? "viewportScrollLarge"
              : controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
                ? "jump"
                : "viewportScrollLarge",
          helpText: "jump through the active pane",
        },
        {
          id:
            controller.browser.detailInteractionState.kind === "section" ||
            controller.browser.detailInteractionState.kind === "target"
              ? "viewportPage"
              : controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
                ? "page"
                : "viewportPage",
          helpText: "page through the active pane",
        },
        {
          id:
            controller.browser.detailInteractionState.kind === "section" ||
            controller.browser.detailInteractionState.kind === "target"
              ? "viewportEdge"
              : controller.browser.state.activePane === "list" && controller.browser.layoutMode !== "detail-only"
                ? "edge"
                : "viewportEdge",
          helpText: "jump to the start or end of the active pane",
        },
      ],
    },
    {
      title: "Actions",
      actions: actionActions,
    },
    ...(controller.actionEntries.length > 0
      ? [
          {
            title: "Explorer Actions",
            lines: buildDerivedTagTerminalActionTargetHelpLines({
              orientation: "horizontal",
              visibility: "onDemand",
              actions: controller.actionEntries,
              contentHelpText: "Use the shared action rail here instead of a hidden command palette.",
            }),
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

  const focusedClause = getStateBadgePresentation(
    controller.selectedDiscreteClause?.operator === "include"
      ? { kind: "include" }
      : controller.selectedDiscreteClause?.operator === "exclude"
        ? { kind: "exclude" }
        : { kind: "off" },
  ).label;
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

function getDiscoveryModeLabel(
  discovery: NonNullable<FilterExplorerControllerContext["discovery"]>,
  mode: string,
): string {
  return discovery.modes.find((entry) => entry.value === mode)?.label.toLowerCase() ?? mode;
}

function formatDiscoveryStatus(discovery: NonNullable<FilterExplorerControllerContext["discovery"]>): string {
  const activeLabel = getDiscoveryModeLabel(discovery, discovery.mode);
  if (!discovery.isRefreshing) {
    return activeLabel;
  }

  const pendingLabel = discovery.pendingMode ?? discovery.mode;
  const pendingModeLabel = getDiscoveryModeLabel(discovery, pendingLabel);
  return pendingLabel === discovery.mode
    ? `${activeLabel} | refreshing`
    : `${activeLabel} | refreshing ${pendingModeLabel}`;
}

function formatTraceMetadata(metadata: TerminalDebugTraceSpanSnapshot["metadata"]): string {
  const entries = Object.entries(metadata).filter(([, value]) => value.length > 0);
  if (entries.length === 0) {
    return "";
  }

  return ` | ${entries
    .slice(0, 5)
    .map(([key, value]) => `${key}=${value.length > 100 ? `${value.slice(0, 100)}...` : value}`)
    .join(" ")}`;
}

function getLongestRunningTraceSpan(
  snapshot: TerminalDebugTraceSnapshot,
): TerminalDebugTraceSpanSnapshot | undefined {
  return [...snapshot.running].sort((left, right) => right.elapsedMs - left.elapsedMs)[0];
}

function getLatestSlowTraceSpan(snapshot: TerminalDebugTraceSnapshot): TerminalDebugTraceSpanSnapshot | undefined {
  return snapshot.recent.find((span) => span.elapsedMs >= snapshot.slowThresholdMs);
}

function formatDebugTraceLine(snapshot: TerminalDebugTraceSnapshot | undefined): DerivedTagTerminalLine | undefined {
  if (!snapshot?.enabled) {
    return undefined;
  }

  const running = getLongestRunningTraceSpan(snapshot);
  if (running) {
    return {
      text: `debug | running ${running.name} ${Math.round(running.elapsedMs)}ms${formatTraceMetadata(running.metadata)}`,
      tone: running.elapsedMs >= 500 ? "warning" : "dim",
    };
  }

  const recent = getLatestSlowTraceSpan(snapshot);
  if (recent) {
    return {
      text: `debug | last ${recent.name} ${Math.round(recent.elapsedMs)}ms${formatTraceMetadata(recent.metadata)}`,
      tone: recent.elapsedMs >= 500 ? "warning" : "dim",
    };
  }

  return {
    text: "debug | idle",
    tone: "dim",
  };
}

export function buildFilterExplorerScreenModel(
  controller: FilterExplorerControllerContext,
): TerminalListDetailScreenModel {
  const interactionActions = getFilterExplorerInteractionActions(controller, controller.actionEntries.length > 0);
  const leftLines = buildFilterExplorerListLines(controller);
  const statusSuffix = isSelectionExplorer(controller)
    ? buildComposeStatus(controller)
    : buildInspectStatus(controller);
  const childLoadingStatus = controller.browser.state.loadingChildNodeId ? " | loading entries" : "";
  const statusText = controller.discovery
    ? `${statusSuffix} | ${formatDiscoveryStatus(controller.discovery)}${childLoadingStatus}`
    : `${statusSuffix}${childLoadingStatus}`;
  const actionRailVisible =
    controller.actionEntries.length > 0 &&
    shouldRenderDerivedTagTerminalActionTarget(controller.actionTargetState, "onDemand");
  const footerText =
    controller.actionEntries.length > 0 && controller.actionTargetState.activeTarget === "actions"
      ? formatTerminalInteractionFooter([
          ...getDerivedTagTerminalActionTargetInteractionActions(controller.actionTargetState, "horizontal"),
          { id: "help" },
        ])
      : controller.browser.state.searchMode
        ? TERMINAL_LIVE_FILTER_FOOTER
        : formatTerminalInteractionFooter(interactionActions);
  const statusLine = actionRailVisible
    ? buildDerivedTagTerminalActionTargetLine(controller.actionEntries, controller.actionTargetState)
    : {
        text: controller.browser.state.searchMode
          ? `Search /${controller.browser.state.searchInput}`
          : controller.browser.layoutMode === "detail-only"
            ? `detail focus | focused detail view | ${statusText} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`
            : `${controller.browser.state.activePane} focus | ${controller.browser.layoutMode} layout | ${statusText} | Detail scroll ${controller.browser.effectiveState.detailScroll}/${controller.browser.maxDetailScroll}`,
        tone: "accent" as const,
      };
  const debugTraceLine = formatDebugTraceLine(controller.debugSnapshot);

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
        text: footerText,
        tone: "dim",
      },
      statusLine,
      ...(debugTraceLine ? [debugTraceLine] : []),
    ],
    pointerRegions: {
      list: controller.onListPointerEvent
        ? {
            onPointerEvent: controller.onListPointerEvent,
          }
        : undefined,
      detail: controller.onDetailPointerEvent
        ? {
            onPointerEvent: controller.onDetailPointerEvent,
          }
        : undefined,
    },
    notification: controller.notification,
    transitionStatus: controller.transitionStatus,
  });
}
