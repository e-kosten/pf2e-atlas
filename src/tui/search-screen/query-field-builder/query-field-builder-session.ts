import type { OntologyDomainModel } from "../../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { FilterExplorerComposeTarget, FilterExplorerSelectTargetOutcome } from "../../filter-explorer/index.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import {
  buildDerivedTagTerminalActionTargetHelpLines,
  type DerivedTagTerminalActionTargetOption,
} from "../../action-target.js";
import {
  buildTerminalInteractionHelpLines,
  type TerminalInteractionAction,
} from "../../interaction-bindings.js";
import type { SearchWorkspaceEntry } from "../workspace/workspace.js";
import { buildStructuredWorkspaceEntryFocusLines, formatSearchWorkspaceEntryLine } from "../workspace/workspace.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { DerivedTagTerminalLine } from "../../framework/types.js";
import {
  createMergedReturnFooterBinding,
  createSharedReturnInteractionActions,
} from "../../shell-navigation-copy.js";
import type { TerminalMenuScreenInteractions } from "../../shared-screens.js";
import type { SearchFilterExplorerFieldState } from "../filter-explorer-field-state.js";

export type SearchFilterExplorerSession = {
  title?: string;
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  query: Pf2eTerminalSearchQuery;
  refreshOnQueryChange?: boolean;
  initialFieldState?: SearchFilterExplorerFieldState;
  preservedMetadata?: MetadataFilterNode | null;
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[];
  resolveSelectionTarget?: (node: import("../../../domain/ontology-types.js").OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  onQueryChange: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
  onBack?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
  onExitRoot?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
  onCancel?: (query: Pf2eTerminalSearchQuery, fieldState: SearchFilterExplorerFieldState) => void;
  onSelectTarget?: (
    outcome: FilterExplorerSelectTargetOutcome,
    query: Pf2eTerminalSearchQuery,
    fieldState: SearchFilterExplorerFieldState,
    discoveryMode: SearchFilterDiscoveryMode,
  ) => void;
};

export type SearchStructuredEditorItem =
  | {
      kind: "workspaceEntry";
      label: string;
      workspaceEntry: SearchWorkspaceEntry;
      detailLines?: DerivedTagTerminalLine[];
    }
  | {
      kind: "treeEntry";
      label: string;
      detailLines?: DerivedTagTerminalLine[];
    };

export type SearchStructuredEditorSession = {
  items: SearchStructuredEditorItem[];
  selectedIndex: number;
  moveSelection: (delta: number, itemCount: number) => void;
  selectCurrent: () => void;
  actionEntries: DerivedTagTerminalActionTargetOption<string>[];
  runAction: (actionId: string) => void;
  finish?: () => void;
  cancel: () => void;
  title?: string;
  subtitle?: string;
  leftTitle?: string;
  rightTitle?: string;
  statusText?: string;
  activePartCount?: number;
  summaryLines?: DerivedTagTerminalLine[];
  buildFocusedDetailLines?: (item: SearchStructuredEditorItem | undefined) => DerivedTagTerminalLine[];
  helpTitle?: string;
  helpBody?: DerivedTagTerminalLine[];
};

function getSearchStructuredEditorNotes(session: SearchStructuredEditorSession): DerivedTagTerminalLine[] {
  if (session.helpBody && session.helpBody.length > 0) {
    return session.helpBody;
  }

  return [
    { text: "Use this editor to change the live structured query tree directly.", tone: "section" },
    { text: "The right pane keeps the current query summary visible while you move focus on the left." },
    {
      text: "Open a row to edit it. Changes apply immediately, and Left returns to the main query editor.",
    },
  ];
}

export function getSearchStructuredEditorInteractionActions(
  _session: SearchStructuredEditorSession,
): TerminalInteractionAction[] {
  return [
    { id: "move", label: "select" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    { id: "select", label: "open" },
    { id: "actions", label: "focus actions" },
    { id: "help" },
    ...createSharedReturnInteractionActions(),
  ];
}

export function createSearchStructuredEditorInteractions(
  session: SearchStructuredEditorSession,
): TerminalMenuScreenInteractions {
  return {
    actions: getSearchStructuredEditorInteractionActions(session),
    footerBindings: [
      { kind: "action", action: { id: "select", label: "open" } },
      { kind: "action", action: { id: "actions", label: "focus actions" } },
      { kind: "action", action: { id: "help" } },
      createMergedReturnFooterBinding(),
    ],
    help: {
      title: session.helpTitle ?? "Structured Query Editor Help",
      sections: [
        {
          title: "Navigation",
          actions: [
            { id: "move", label: "select", helpText: "move through the live query tree" },
            { id: "jump", helpText: "jump through the live query tree" },
            { id: "page", helpText: "page through the live query tree" },
            { id: "edge", helpText: "jump to the start or end of the live query tree" },
          ],
        },
        {
          title: "Actions",
          actions: getSearchStructuredEditorInteractionActions(session)
            .filter((action) => action.id === "select" || action.id === "actions" || action.id === "help" || action.id === "back" || action.id === "quit")
            .map((action) => ({
              ...action,
              helpText:
                action.id === "select"
                  ? "open the focused row or use its default action"
                  : action.id === "actions"
                    ? "focus the node-action rail for grouping, moving, wrapping, and other restructuring"
                    : action.id === "help"
                      ? "show this help"
                      : "return to the main query editor",
              label: action.id === "back" || action.id === "quit" ? "return" : action.label,
            })),
        },
      ],
      appendix: [
        ...buildDerivedTagTerminalActionTargetHelpLines({
          orientation: "horizontal",
          visibility: "onDemand",
          actions: session.actionEntries,
          contentHelpText: "Use the action rail for add/group/move/wrap flows instead of a hidden command palette.",
        }),
        ...(getSearchStructuredEditorNotes(session).length > 0
          ? [{ text: "" }, ...getSearchStructuredEditorNotes(session)]
          : []),
      ],
    },
  };
}

export function buildSearchStructuredEditorHelpLines(
  session: SearchStructuredEditorSession,
): DerivedTagTerminalLine[] {
  const notes = getSearchStructuredEditorNotes(session);

  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Navigation",
        actions: [
          { id: "move", label: "select", helpText: "move through the live query tree" },
          { id: "jump", helpText: "jump through the live query tree" },
          { id: "page", helpText: "page through the live query tree" },
          { id: "edge", helpText: "jump to the start or end of the live query tree" },
        ],
      },
      {
        title: "Actions",
        actions: getSearchStructuredEditorInteractionActions(session)
          .filter((action) => action.id === "select" || action.id === "actions" || action.id === "help" || action.id === "back" || action.id === "quit")
          .map((action) => ({
            ...action,
            helpText:
              action.id === "select"
                ? "open the focused row"
                : action.id === "actions"
                  ? "focus the node-action rail"
                  : action.id === "help"
                    ? "show this help"
                    : "return to the main query editor",
            label: action.id === "back" || action.id === "quit" ? "return" : action.label,
          })),
      },
    ]),
    { text: "" },
    ...buildDerivedTagTerminalActionTargetHelpLines({
      orientation: "horizontal",
      visibility: "onDemand",
      actions: session.actionEntries,
      contentHelpText: "Use the action rail for add/group/move/wrap flows instead of a hidden command palette.",
    }),
    { text: "" },
    ...notes,
  ];
}

function buildDefaultFocusedDetailLines(
  item: SearchStructuredEditorItem | undefined,
): DerivedTagTerminalLine[] {
  if (!item) {
    return [{ text: "No query-tree entry is selected.", tone: "dim" }];
  }
  if (item.detailLines && item.detailLines.length > 0) {
    return item.detailLines;
  }
  if (item.kind === "workspaceEntry") {
    return buildStructuredWorkspaceEntryFocusLines(item.workspaceEntry);
  }
  return item.detailLines ?? [{ text: "Open this tree row to edit the selected node or insertion slot." }];
}

function getSelectedItem(session: SearchStructuredEditorSession): SearchStructuredEditorItem | undefined {
  return session.items[Math.max(0, Math.min(session.selectedIndex, session.items.length - 1))];
}

function countStructuredSelections(session: SearchStructuredEditorSession): number {
  return session.activePartCount ?? 0;
}

export function buildSearchStructuredEditorMenuItems(session: SearchStructuredEditorSession): Array<{ label: string }> {
  return session.items.map((item) => {
    if (item.kind === "workspaceEntry") {
      return { label: formatSearchWorkspaceEntryLine(item.workspaceEntry) };
    }
    return { label: item.label };
  });
}

export function buildSearchStructuredEditorDetailLines(
  session: SearchStructuredEditorSession,
): DerivedTagTerminalLine[] {
  const summaryLines = session.summaryLines && session.summaryLines.length > 0
    ? session.summaryLines
    : [
        { text: "Live Structured Query", tone: "section" as const },
        { text: "No structured query clauses yet.", tone: "dim" as const },
      ];
  const focusedLines =
    session.buildFocusedDetailLines?.(getSelectedItem(session)) ?? buildDefaultFocusedDetailLines(getSelectedItem(session));

  return [...summaryLines, { text: "" }, ...focusedLines];
}

export function buildSearchStructuredEditorStatusLine(session: SearchStructuredEditorSession): DerivedTagTerminalLine {
  return {
    text:
      session.statusText ??
      `${countStructuredSelections(session)} structured part${countStructuredSelections(session) === 1 ? "" : "s"} | live query updates immediately`,
    tone: "accent",
  };
}
