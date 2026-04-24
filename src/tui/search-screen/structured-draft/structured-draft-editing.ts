import React from "react";

import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalPreparedFilterExplorerContext,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchQueryFieldBuilderSession } from "../query-field-builder/query-field-builder-session.js";
import type { SearchStructuredDraftState } from "./structured-draft-support.js";
import { useSearchStructuredDraftMetadataActions } from "./structured-draft-metadata-actions.js";
import { buildStructuredQuerySummaryLines } from "../workspace/workspace.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

export function useSearchStructuredDraftEditing({
  appendStructuredDraftMetadataNode,
  cancelStructuredDraftSession,
  clearStructuredDraftMoveSource,
  editFieldClause,
  enterStructuredDraftMoveMode,
  finishStructuredDraftSession,
  getScopedFieldOptions,
  moveStructuredDraftSelection,
  openOntologyFieldEditor,
  prompts,
  replaceStructuredDraftProjection,
  structuredDraftEntries,
  structuredDraftQuery,
  structuredDraftState,
  terminal,
  updateStructuredDraftMetadataNode,
  user,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  cancelStructuredDraftSession: () => void;
  clearStructuredDraftMoveSource: () => void;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  finishStructuredDraftSession: () => void;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  moveStructuredDraftSelection: (delta: number, itemCount: number) => void;
  openOntologyFieldEditor: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode: MetadataFilterNode | null,
    onApply: (
      result: Pf2eTerminalFilterExplorerInsertionResult,
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void,
    onReturn?: () => void,
  ) => Promise<boolean>;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftEntries: SearchStructuredDraftEntry[];
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  structuredDraftState: SearchStructuredDraftState | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
  user: SearchWorkspaceUser;
}): SearchQueryFieldBuilderSession | null {
  const { editStructuredDraftMetadata } = useSearchStructuredDraftMetadataActions({
    appendStructuredDraftMetadataNode,
    clearStructuredDraftMoveSource,
    editFieldClause,
    enterStructuredDraftMoveMode,
    getScopedFieldOptions,
    moveSourcePath: structuredDraftState?.moveSourcePath ?? null,
    openOntologyFieldEditor,
    prompts,
    replaceStructuredDraftProjection,
    structuredDraftQuery,
    terminal,
    updateStructuredDraftMetadataNode,
    user,
  });

  const selectCurrentStructuredDraftEntry = React.useCallback(() => {
    if (!structuredDraftState) {
      return;
    }

    const selectedEntry =
      structuredDraftEntries[
        clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length)
      ] ?? null;
    if (!selectedEntry) {
      return;
    }
    if (selectedEntry.kind === "finish") {
      finishStructuredDraftSession();
      return;
    }
    if (selectedEntry.kind === "cancel") {
      cancelStructuredDraftSession();
      return;
    }
    void editStructuredDraftMetadata(selectedEntry);
  }, [
    cancelStructuredDraftSession,
    clearStructuredDraftMoveSource,
    editStructuredDraftMetadata,
    enterStructuredDraftMoveMode,
    finishStructuredDraftSession,
    structuredDraftEntries,
    structuredDraftState,
  ]);

  return React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!structuredDraftState) {
      return null;
    }

    return {
      kind: "structuredEditor",
      title: "Structured Query Editor",
      subtitle: "Stage structured search changes before applying them to the live query",
      leftTitle: "[STAGED QUERY]",
      rightTitle: "Staged Summary & Detail",
      statusText: structuredDraftState.moveSourcePath
        ? "Move mode: select a visible destination slot, Enter confirms, Left/Esc cancels the move."
        : "Left/Esc applies the staged query and returns. Use the discard row to abandon it.",
      projectedQuery: structuredDraftQuery,
      summaryLines: structuredDraftQuery
        ? buildStructuredQuerySummaryLines(structuredDraftQuery, {
            packLabelResolver: user.search.getPackLabel,
          })
        : undefined,
      items: structuredDraftEntries.map((entry) =>
        entry.kind === "finish" || entry.kind === "cancel"
          ? { kind: entry.kind, label: entry.label }
          : {
              kind: "treeEntry" as const,
              label: entry.menuLabel ?? entry.label,
            },
      ),
      selectedIndex: clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length),
      moveSelection: moveStructuredDraftSelection,
      selectCurrent: selectCurrentStructuredDraftEntry,
      finish: finishStructuredDraftSession,
      cancel: structuredDraftState.moveSourcePath ? clearStructuredDraftMoveSource : finishStructuredDraftSession,
      helpTitle: "Structured Query Editor Help",
      helpBody: [
        { text: "Stage structured search changes before applying them to the live query.", tone: "section" },
        { text: "The summary stays visible while you move focus so prior staged selections do not disappear." },
        {
          text: structuredDraftState.moveSourcePath
            ? "Use Left or Esc to cancel move mode and keep the current tree unchanged."
            : "Use Left or Esc to apply the staged query and return to the top editor.",
        },
        { text: "Use the discard row only when you want to abandon the staged query entirely." },
      ],
    };
  }, [
    clearStructuredDraftMoveSource,
    finishStructuredDraftSession,
    moveStructuredDraftSelection,
    selectCurrentStructuredDraftEntry,
    structuredDraftEntries,
    structuredDraftQuery,
    structuredDraftState,
  ]);
}
