import type { OntologyDomainModel } from "../../../domain/ontology-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { FilterExplorerComposeTarget } from "../../filter-explorer/index.js";
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
import type { Pf2eTerminalFilterExplorerDraft } from "../../search/service.js";
import type { DerivedTagTerminalLine } from "../../framework/types.js";
import {
  createMergedReturnFooterBinding,
  createSharedReturnInteractionActions,
} from "../../shell-navigation-copy.js";
import type { TerminalMenuScreenInteractions } from "../../shared-screens.js";

export type SearchFilterExplorerSession = {
  title?: string;
  model: OntologyDomainModel;
  initialDiscoveryMode?: SearchFilterDiscoveryMode;
  loadModelForDiscoveryMode?: (mode: SearchFilterDiscoveryMode) => Promise<OntologyDomainModel>;
  draft: Pf2eTerminalFilterExplorerDraft;
  onDraftChange?: (draft: Pf2eTerminalFilterExplorerDraft) => void;
  resolveSelectionTarget: (node: import("../../../domain/ontology-types.js").OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
  onApply: (draft: Pf2eTerminalFilterExplorerDraft) => void;
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
    }
  | {
      kind: "finish" | "cancel";
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
  stagedPartCount?: number;
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
    { text: "Use this editor to stage structured search changes before committing them.", tone: "section" },
    { text: "The right pane keeps the full staged query summary visible while you move focus on the left." },
    {
      text: "Open a row to edit it, then continue staging more changes or finish when the staged query looks correct.",
    },
  ];
}

export function getSearchStructuredEditorInteractionActions(
  session: SearchStructuredEditorSession,
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
            { id: "move", label: "select", helpText: "move through the staged query list" },
            { id: "jump", helpText: "jump through the staged query list" },
            { id: "page", helpText: "page through the staged query list" },
            { id: "edge", helpText: "jump to the start or end of the staged query list" },
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
                      : "apply the staged structured query and return to the live editor",
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
          { id: "move", label: "select", helpText: "move through the staged query list" },
          { id: "jump", helpText: "jump through the staged query list" },
          { id: "page", helpText: "page through the staged query list" },
          { id: "edge", helpText: "jump to the start or end of the staged query list" },
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
                  : "apply the staged structured query and return to the live editor",
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
    return [{ text: "No staged entry is selected.", tone: "dim" }];
  }
  if (item.detailLines && item.detailLines.length > 0) {
    return item.detailLines;
  }
  if (item.kind === "workspaceEntry") {
    return buildStructuredWorkspaceEntryFocusLines(item.workspaceEntry);
  }
  if (item.kind === "treeEntry") {
    return item.detailLines ?? [{ text: "Open this tree row to edit the selected node or insertion slot." }];
  }
  if (item.kind === "finish") {
    return [
      { text: "Focused Entry", tone: "section" },
      { text: "Finish staged changes", tone: "accent" },
      { text: "Apply the full staged structured query to the live query and return to the editor." },
    ];
  }
  return [
    { text: "Focused Entry", tone: "section" },
    { text: "Cancel staged changes", tone: "warning" },
    { text: "Discard the staged structured query and return to the editor." },
  ];
}

function getSelectedItem(session: SearchStructuredEditorSession): SearchStructuredEditorItem | undefined {
  return session.items[Math.max(0, Math.min(session.selectedIndex, session.items.length - 1))];
}

function countStructuredSelections(session: SearchStructuredEditorSession): number {
  return session.stagedPartCount ?? 0;
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
        { text: "Staged Structured Query", tone: "section" as const },
        { text: "No staged structured changes yet.", tone: "dim" as const },
      ];
  const focusedLines =
    session.buildFocusedDetailLines?.(getSelectedItem(session)) ?? buildDefaultFocusedDetailLines(getSelectedItem(session));

  return [...summaryLines, { text: "" }, ...focusedLines];
}

export function buildSearchStructuredEditorStatusLine(session: SearchStructuredEditorSession): DerivedTagTerminalLine {
  return {
    text:
      session.statusText ??
      `${countStructuredSelections(session)} staged structured part${countStructuredSelections(session) === 1 ? "" : "s"} | live query unchanged until finish`,
    tone: "accent",
  };
}
