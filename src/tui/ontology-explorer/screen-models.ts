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
import type { HostedOntologyPickerContract } from "./picker-hosting.js";
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

function getFacetPickerBackAction(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
  options: HostedOntologyPickerContract = {},
): TerminalInteractionAction {
  if (isOntologyExplorerDetailContext(controller)) {
    return { id: "back" };
  }
  const rootDepth = options.rootDepth ?? 0;
  return controller.state.activePane === "list" && controller.effectiveState.depth === rootDepth
    ? { id: "back", label: options.rootBackLabel ?? "return" }
    : { id: "back" };
}

type FacetPickerListContext = {
  focusTarget: string;
  listTarget: string;
  title: string;
};

function resolveFacetPickerListContextFromTitle(title: string): FacetPickerListContext {
  switch (title) {
    case "Query Fields":
      return { title, listTarget: "query field", focusTarget: "query fields" };
    case "Advanced Predicates":
      return { title, listTarget: "advanced predicate", focusTarget: "advanced predicates" };
    case "Derived Tags":
      return { title, listTarget: "derived tag", focusTarget: "derived tags" };
    case "Values":
      return { title, listTarget: "value", focusTarget: "values" };
    case "Tags":
      return { title, listTarget: "tag", focusTarget: "tags" };
    case "Traits":
      return { title, listTarget: "trait", focusTarget: "traits" };
    case "Families":
      return { title, listTarget: "family", focusTarget: "families" };
    case "Categories":
      return { title, listTarget: "category", focusTarget: "categories" };
    case "Subcategories":
      return { title, listTarget: "subcategory", focusTarget: "subcategories" };
    case "Records":
      return { title, listTarget: "record", focusTarget: "records" };
    default:
      return { title, listTarget: title.toLowerCase(), focusTarget: title.toLowerCase() };
  }
}

function resolveFacetPickerListContextFromNodes(nodes: readonly OntologyNode[]): FacetPickerListContext | undefined {
  const kinds = [...new Set(nodes.map((node) => node.kind).filter((kind) => kind.length > 0))];
  if (kinds.length !== 1) {
    return undefined;
  }

  switch (kinds[0]) {
    case undefined:
      return undefined;
    case "field":
      return resolveFacetPickerListContextFromTitle("Query Fields");
    case "family":
      return resolveFacetPickerListContextFromTitle("Families");
    case "tag":
      return resolveFacetPickerListContextFromTitle("Tags");
    case "trait":
      return resolveFacetPickerListContextFromTitle("Traits");
    case "value":
      return resolveFacetPickerListContextFromTitle("Values");
    case "derivedTagValue":
      return resolveFacetPickerListContextFromTitle("Derived Tags");
    case "advancedPredicate":
      return resolveFacetPickerListContextFromTitle("Advanced Predicates");
    case "category":
      return resolveFacetPickerListContextFromTitle("Categories");
    case "subcategory":
      return resolveFacetPickerListContextFromTitle("Subcategories");
    case "record":
      return resolveFacetPickerListContextFromTitle("Records");
    default:
      return undefined;
  }
}

function getFacetPickerListContext(
  controller: Pick<OntologyExplorerControllerContext, "effectiveState" | "selection">,
  options: HostedOntologyPickerContract = {},
): FacetPickerListContext {
  if (controller.effectiveState.depth === (options.rootDepth ?? 0) && options.rootListTitle) {
    return resolveFacetPickerListContextFromTitle(options.rootListTitle);
  }
  return (
    resolveFacetPickerListContextFromNodes(controller.selection.currentNodes) ??
    (controller.effectiveState.depth === (options.rootDepth ?? 0)
      ? resolveFacetPickerListContextFromTitle("Query Fields")
      : { title: "Entries", listTarget: "entry", focusTarget: "entries" })
  );
}

function getFacetPickerBackHelpText(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState" | "selection">,
  options: HostedOntologyPickerContract = {},
): string {
  const rootDepth = options.rootDepth ?? 0;
  const rootListContext = getFacetPickerListContext(controller, options);
  if (isOntologyExplorerDetailContext(controller)) {
    return controller.effectiveState.depth === rootDepth
      ? (options.rootDetailBackHelpText ?? `return to the ${rootListContext.listTarget} list`)
      : "return to the previous level";
  }
  return controller.state.activePane === "list" && controller.effectiveState.depth === rootDepth
    ? (options.rootBackHelpText ?? "return to the query editor")
    : "return to the previous level";
}

function getFacetPickerListTitle(
  controller: Pick<OntologyExplorerControllerContext, "effectiveState" | "selection">,
  options: HostedOntologyPickerContract = {},
): string {
  return getFacetPickerListContext(controller, options).title;
}

function getFacetPickerFocusHelpText(
  controller: Pick<OntologyExplorerControllerContext, "effectiveState" | "selection">,
  options: HostedOntologyPickerContract = {},
): string {
  const listContext = getFacetPickerListContext(controller, options);
  return controller.effectiveState.depth === (options.rootDepth ?? 0)
    ? (options.rootFocusHelpText ?? `switch focus between ${listContext.focusTarget} and detail`)
    : `switch focus between ${listContext.focusTarget} and detail`;
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

export function getFacetPickerInteractionActions(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState">,
  options: HostedOntologyPickerContract = {},
): TerminalInteractionAction[] {
  if (controller.layoutMode === "detail-only") {
    return [
      { id: "scroll" },
      { id: "jump" },
      { id: "page" },
      { id: "edge" },
      { id: "cycle" },
      { id: "layout", label: "split-view" },
      getFacetPickerBackAction(controller, options),
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
      getFacetPickerBackAction(controller, options),
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
    getFacetPickerBackAction(controller, options),
    { id: "search" },
    { id: "help" },
    { id: "quit", label: "return" },
  ];
}

export function buildFacetPickerHelpLines(
  controller: Pick<OntologyExplorerControllerContext, "layoutMode" | "state" | "effectiveState" | "selection">,
  options: HostedOntologyPickerContract = {},
): DerivedTagTerminalLine[] {
  const actionActions = getFacetPickerInteractionActions(controller, options)
    .filter((action) => !["move", "scroll", "jump", "page", "edge"].includes(action.id))
    .map((action) => ({
      ...action,
      helpText:
        action.id === "cycle"
          ? "cycle the focused query field policy through off, any, all, and exclude"
          : action.id === "focus"
            ? getFacetPickerFocusHelpText(controller, options)
            : action.id === "layout"
              ? "toggle split and detail-only layouts"
              : action.id === "cancel"
                ? "clear the current filter without leaving this level"
                : action.id === "back"
                  ? getFacetPickerBackHelpText(controller, options)
                  : action.id === "search"
                    ? "start live filtering"
                    : action.id === "help"
                      ? "show this help"
                      : (options.applyHelpText ?? "apply the current query field selections and return"),
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
  options = {},
}: {
  model: OntologyDomainModel;
  controller: OntologyExplorerControllerContext;
  leftLines: DerivedTagTerminalLine[];
  focusedPolicyLabel: string;
  options?: HostedOntologyPickerContract;
}):
  | { kind: "detail-only"; props: DerivedTagTerminalPaneScreenProps }
  | { kind: "two-pane"; props: DerivedTagTerminalTwoPaneScreenProps } {
  if (controller.layoutMode === "detail-only") {
    return {
      kind: "detail-only",
      props: {
        title: "Selection Picker",
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
              : formatTerminalInteractionFooter(getFacetPickerInteractionActions(controller, options)),
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
      title: "Selection Picker",
      subtitle: `${model.label} | ${controller.breadcrumb}${controller.searchIndicator}`,
      left: {
        title:
          controller.state.activePane === "list"
            ? `[${getFacetPickerListTitle(controller, options).toUpperCase()}]`
            : getFacetPickerListTitle(controller, options),
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
            : formatTerminalInteractionFooter(getFacetPickerInteractionActions(controller, options)),
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
