import type { MetadataFilterNode } from "../../domain/metadata-types.js";
import type { OntologyDomainModel } from "../../domain/ontology-types.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import type { SearchWorkspaceEntry } from "./workspace.js";
import { flattenMetadataTree, isMetadataPredicate } from "../search/query-core.js";
import {
  buildStructuredQuerySummaryLines,
  buildStructuredWorkspaceEntryFocusLines,
  formatSearchWorkspaceEntryLine,
} from "./workspace.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";

export type SearchQueryFieldPickerSession = {
  model: OntologyDomainModel;
  initialSelections: Record<string, { any: string[]; all: string[]; exclude: string[] }>;
  applySelection: (selection: Record<string, { any: string[]; all: string[]; exclude: string[] }>) => void;
};

export type SearchQueryFieldBuilderStep = "fieldList" | "ontologyPicker";

export type SearchStructuredEditorItem =
  | {
      kind: "workspaceEntry";
      label: string;
      workspaceEntry: SearchWorkspaceEntry;
      detailLines?: DerivedTagTerminalLine[];
    }
  | {
      kind: "field";
      fieldOption: Pf2eTerminalQueryFieldOption;
      label: string;
      detailLines?: DerivedTagTerminalLine[];
    }
  | {
      kind: "finish" | "cancel";
      label: string;
      detailLines?: DerivedTagTerminalLine[];
    };

export type SearchQueryFieldBuilderItem = SearchStructuredEditorItem;

export type SearchQueryFieldBuilderDraft = {
  query: Pf2eTerminalSearchQuery;
  stagedSelections: Pf2eTerminalQueryFieldSelectionMap;
  payload: Record<string, unknown> | null;
};

export type SearchQueryFieldBuilderOutcome =
  | {
      kind: "finish";
      selectedField: Pf2eTerminalQueryFieldOption | null;
      draft: SearchQueryFieldBuilderDraft | null;
    }
  | {
      kind: "cancel";
      selectedField: Pf2eTerminalQueryFieldOption | null;
      draft: SearchQueryFieldBuilderDraft | null;
    };

export type SearchStructuredEditorSession = {
  items: SearchStructuredEditorItem[];
  selectedIndex: number;
  moveSelection: (delta: number, itemCount: number) => void;
  selectCurrent: () => void;
  finish?: () => void;
  cancel: () => void;
  title?: string;
  subtitle?: string;
  leftTitle?: string;
  rightTitle?: string;
  statusText?: string;
  draftQuery?: Pf2eTerminalSearchQuery | null;
  summaryLines?: DerivedTagTerminalLine[];
  buildFocusedDetailLines?: (item: SearchStructuredEditorItem | undefined) => DerivedTagTerminalLine[];
  helpTitle?: string;
  helpBody?: DerivedTagTerminalLine[];
  kind?: "structuredEditor" | "queryFieldBuilder";
  availableFields?: Pf2eTerminalQueryFieldOption[];
  selectedFieldIndex?: number;
  step?: SearchQueryFieldBuilderStep;
  activeChildView?: "none" | "ontologyPicker";
  childSession?: SearchQueryFieldPickerSession | null;
  draft?: SearchQueryFieldBuilderDraft | null;
  fieldDrafts?: Record<string, MetadataFilterNode | null>;
  onFinish?: (outcome: Extract<SearchQueryFieldBuilderOutcome, { kind: "finish" }>) => void;
  onCancel?: (outcome: Extract<SearchQueryFieldBuilderOutcome, { kind: "cancel" }>) => void;
};

export type SearchQueryFieldBuilderSession = SearchStructuredEditorSession;

function getSearchStructuredEditorBackLabel(session: SearchStructuredEditorSession): "return" | "cancel" {
  return session.kind === "structuredEditor" ? "return" : "cancel";
}

function getSearchStructuredEditorNotes(session: SearchStructuredEditorSession): DerivedTagTerminalLine[] {
  if (session.helpBody && session.helpBody.length > 0) {
    return session.helpBody;
  }

  return [
    { text: "Use this editor to stage structured search changes before committing them.", tone: "section" },
    { text: "The right pane keeps the full staged query summary visible while you move focus on the left." },
    {
      text: "Open a row to edit it, then continue staging more changes or finish when the draft looks correct.",
    },
  ];
}

export function getSearchStructuredEditorInteractionActions(
  session: SearchStructuredEditorSession,
): TerminalInteractionAction[] {
  const backLabel = getSearchStructuredEditorBackLabel(session);

  return [
    { id: "move", label: "select" },
    { id: "jump" },
    { id: "page" },
    { id: "edge" },
    { id: "select", label: "open" },
    { id: "help" },
    { id: "back", label: backLabel },
    { id: "quit", label: backLabel },
  ];
}

export function buildSearchStructuredEditorFooterText(session: SearchStructuredEditorSession): string {
  return formatTerminalInteractionFooter(getSearchStructuredEditorInteractionActions(session));
}

export function buildSearchStructuredEditorHelpLines(
  session: SearchStructuredEditorSession,
): DerivedTagTerminalLine[] {
  const backLabel = getSearchStructuredEditorBackLabel(session);
  const notes = getSearchStructuredEditorNotes(session);
  const navigationSubject = session.kind === "queryFieldBuilder" ? "query field list" : "staged query list";
  const backHelpText =
    session.kind === "queryFieldBuilder"
      ? "return to the staged query with current field edits preserved"
      : "apply the staged structured query and return to the live editor";

  return [
    ...buildTerminalInteractionHelpLines([
      {
        title: "Navigation",
        actions: [
          { id: "move", label: "select", helpText: `move through the ${navigationSubject}` },
          { id: "jump", helpText: `jump through the ${navigationSubject}` },
          { id: "page", helpText: `page through the ${navigationSubject}` },
          { id: "edge", helpText: `jump to the start or end of the ${navigationSubject}` },
        ],
      },
      {
        title: "Actions",
        actions: getSearchStructuredEditorInteractionActions(session)
          .filter((action) => action.id === "select" || action.id === "help" || action.id === "back" || action.id === "quit")
          .map((action) => ({
            ...action,
            helpText:
              action.id === "select"
                ? "open the focused row"
                : action.id === "help"
                  ? "show this help"
                  : backHelpText,
            label: action.id === "back" || action.id === "quit" ? backLabel : action.label,
          })),
      },
    ]),
    { text: "" },
    ...notes,
  ];
}

function buildLegacyMetadataNodeLines(node: MetadataFilterNode, indentBase = 0): DerivedTagTerminalLine[] {
  return flattenMetadataTree(node, { rootLabel: "node" }).map((entry) => ({
    text: `${entry.summary.label}: ${entry.summary.value}`,
    indent: indentBase + entry.depth * 2,
    tone: isMetadataPredicate(entry.node) ? "default" : "accent",
  }));
}

function buildLegacyStructuredSummaryLines(session: SearchStructuredEditorSession): DerivedTagTerminalLine[] {
  const stagedFieldItems: Array<[Extract<SearchStructuredEditorItem, { kind: "field" }>, MetadataFilterNode]> = [];

  session.items.forEach((item) => {
    if (item.kind !== "field") {
      return;
    }
    const node = session.fieldDrafts?.[item.fieldOption.value] ?? null;
    if (!node) {
      return;
    }
    stagedFieldItems.push([item, node]);
  });

  if (stagedFieldItems.length === 0) {
    return [
      { text: "Staged Structured Query", tone: "section" },
      { text: "No staged structured changes yet.", tone: "dim" },
    ];
  }

  const lines: DerivedTagTerminalLine[] = [{ text: "Staged Structured Query", tone: "section" }];
  stagedFieldItems.forEach(([item, node], index) => {
    if (index > 0) {
      lines.push({ text: "" });
    }
    lines.push({ text: item.fieldOption.label, tone: "accent" });
    lines.push(...buildLegacyMetadataNodeLines(node, 2));
  });
  return lines;
}

function buildLegacyFieldFocusLines(
  item: Extract<SearchStructuredEditorItem, { kind: "field" }>,
  session: SearchStructuredEditorSession,
): DerivedTagTerminalLine[] {
  const node = session.fieldDrafts?.[item.fieldOption.value] ?? null;
  const lines: DerivedTagTerminalLine[] = [
    { text: "Focused Entry", tone: "section" },
    { text: item.fieldOption.label, tone: "accent" },
    { text: item.fieldOption.description || "No additional field description is available." },
    { text: `Editor: ${item.fieldOption.editor}` },
  ];

  if (node) {
    lines.push({ text: "" });
    lines.push({ text: "Current field draft", tone: "section" });
    lines.push(...buildLegacyMetadataNodeLines(node, 2));
  } else {
    lines.push({ text: "" });
    lines.push({ text: "No staged draft for this field yet.", tone: "dim" });
  }

  lines.push({ text: "" });
  lines.push({ text: `Step: ${session.step ?? "fieldList"}` });
  lines.push({ text: `Child view: ${session.activeChildView ?? "none"}` });
  lines.push({ text: "Selections remain staged until you finish.", tone: "dim" });
  return lines;
}

function buildCompatibilityFocusedDetailLines(
  session: SearchStructuredEditorSession,
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
  if (item.kind === "field") {
    return buildLegacyFieldFocusLines(item, session);
  }
  if (item.kind === "finish") {
    return [
      { text: "Focused Entry", tone: "section" },
      { text: "Finish staged changes", tone: "accent" },
      { text: "Apply the full staged structured draft to the live query and return to the editor." },
    ];
  }
  return [
    { text: "Focused Entry", tone: "section" },
    { text: "Cancel staged changes", tone: "warning" },
    { text: "Discard the staged structured draft and return to the editor." },
  ];
}

function getSelectedItem(session: SearchStructuredEditorSession): SearchStructuredEditorItem | undefined {
  return session.items[Math.max(0, Math.min(session.selectedIndex, session.items.length - 1))];
}

function countStructuredSelections(session: SearchStructuredEditorSession): number {
  if (session.draftQuery) {
    return (session.draftQuery.filters.category ? 1 : 0) + session.draftQuery.filters.parts.length;
  }
  return Object.values(session.fieldDrafts ?? {}).filter((node) => Boolean(node)).length;
}

export function buildSearchStructuredEditorMenuItems(session: SearchStructuredEditorSession): Array<{ label: string }> {
  return session.items.map((item) => {
    if (item.kind === "workspaceEntry") {
      return { label: formatSearchWorkspaceEntryLine(item.workspaceEntry) };
    }
    if (item.kind === "field" && session.fieldDrafts?.[item.fieldOption.value]) {
      return { label: `${item.label} | staged` };
    }
    return { label: item.label };
  });
}

export function buildSearchStructuredEditorDetailLines(
  session: SearchStructuredEditorSession,
): DerivedTagTerminalLine[] {
  const summaryLines =
    session.summaryLines && session.summaryLines.length > 0
      ? session.summaryLines
      : session.draftQuery
        ? buildStructuredQuerySummaryLines(session.draftQuery)
        : buildLegacyStructuredSummaryLines(session);
  const focusedLines =
    session.buildFocusedDetailLines?.(getSelectedItem(session)) ??
    buildCompatibilityFocusedDetailLines(session, getSelectedItem(session));

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

export const buildSearchQueryFieldBuilderDetailLines = buildSearchStructuredEditorDetailLines;
export const buildSearchQueryFieldBuilderStatusLine = buildSearchStructuredEditorStatusLine;
