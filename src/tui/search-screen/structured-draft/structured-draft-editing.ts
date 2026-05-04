import React from "react";

import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../search/service.js";
import type { SearchStructuredEditorSession } from "../query-field-builder/query-field-builder-session.js";
import { useSearchStructuredDraftMetadataActions } from "./structured-draft-metadata-actions.js";
import type { SearchStructuredDraftState, StructuredDraftResumeTarget } from "./structured-draft-state.js";
import { buildStructuredQuerySummaryLines } from "../workspace/workspace.js";
import { buildSearchQuerySummary } from "../workspace/query-summary.js";
import type { DerivedTagTerminalActionTargetOption } from "../../action-target.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

export function useSearchStructuredDraftEditing({
  appendStructuredDraftMetadataNode,
  clearStructuredDraftMoveSource,
  editFieldClause,
  enterStructuredDraftMoveMode,
  finishStructuredDraftSession,
  getScopedFieldOptions,
  moveStructuredDraftSelection,
  openFilterExplorer,
  prompts,
  replaceStructuredDraftProjection,
  setStructuredDraftResumeTarget,
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
  openFilterExplorer: OpenSearchFilterExplorer;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  structuredDraftEntries: SearchStructuredDraftEntry[];
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  structuredDraftState: SearchStructuredDraftState | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  user: SearchWorkspaceUser;
}): SearchStructuredEditorSession | null {
  const { editStructuredDraftMetadata, getStructuredDraftEntryActions, runStructuredDraftEntryAction } =
    useSearchStructuredDraftMetadataActions({
      appendStructuredDraftMetadataNode,
      clearStructuredDraftMoveSource,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      moveSourcePath: structuredDraftState?.moveSourcePath ?? null,
      openFilterExplorer,
      prompts,
      replaceStructuredDraftProjection,
      setStructuredDraftResumeTarget,
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
    void editStructuredDraftMetadata(selectedEntry);
  }, [
    clearStructuredDraftMoveSource,
    editStructuredDraftMetadata,
    enterStructuredDraftMoveMode,
    structuredDraftEntries,
    structuredDraftState,
  ]);

  const selectedStructuredDraftEntry = React.useMemo(() => {
    if (!structuredDraftState) {
      return null;
    }
    return (
      structuredDraftEntries[
        clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length)
      ] ?? null
    );
  }, [structuredDraftEntries, structuredDraftState]);

  return React.useMemo<SearchStructuredEditorSession | null>(() => {
    if (!structuredDraftState) {
      return null;
    }

    const selectedActionEntries: DerivedTagTerminalActionTargetOption<string>[] =
      getStructuredDraftEntryActions(selectedStructuredDraftEntry);

    return {
      title: "Structured Query Editor",
      subtitle: "Structured Query Editor | Edit the live structured query tree",
      leftTitle: "[QUERY TREE]",
      rightTitle: "Live Summary & Detail",
      statusText: structuredDraftState.moveSourcePath
        ? "Move mode: select a visible destination slot, Enter confirms, Left/Esc cancels the move."
        : "Left/Esc returns to the main editor. Query-tree changes apply immediately.",
      activePartCount: structuredDraftQuery
        ? buildSearchQuerySummary(structuredDraftQuery).activeStructuredPartCount
        : 0,
      summaryLines: structuredDraftQuery
        ? buildStructuredQuerySummaryLines(structuredDraftQuery, {
            packLabelResolver: user.search.getPackLabel,
          })
        : undefined,
      items: structuredDraftEntries.map((entry) => ({
        kind: "treeEntry" as const,
        label: entry.menuLabel ?? entry.label,
      })),
      actionEntries: selectedActionEntries,
      runAction: (actionId) => {
        if (!selectedStructuredDraftEntry) {
          return;
        }
        void runStructuredDraftEntryAction(
          selectedStructuredDraftEntry,
          actionId as Parameters<typeof runStructuredDraftEntryAction>[1],
        );
      },
      selectedIndex: clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length),
      moveSelection: moveStructuredDraftSelection,
      selectCurrent: selectCurrentStructuredDraftEntry,
      finish: finishStructuredDraftSession,
      cancel: structuredDraftState.moveSourcePath ? clearStructuredDraftMoveSource : finishStructuredDraftSession,
      helpTitle: "Structured Query Editor Help",
      helpBody: [
        { text: "Edit the live structured query tree directly.", tone: "section" },
        { text: "The summary stays visible while you move focus so active query changes do not disappear." },
        {
          text: structuredDraftState.moveSourcePath
            ? "Use Left or Esc to cancel move mode and keep the current tree unchanged."
            : "Use Left or Esc to return to the main editor.",
        },
        { text: "Grouping, wrapping, and clause edits update the live query as soon as they land." },
      ],
    };
  }, [
    clearStructuredDraftMoveSource,
    finishStructuredDraftSession,
    getStructuredDraftEntryActions,
    moveStructuredDraftSelection,
    runStructuredDraftEntryAction,
    selectedStructuredDraftEntry,
    selectCurrentStructuredDraftEntry,
    structuredDraftEntries,
    structuredDraftQuery,
    structuredDraftState,
  ]);
}
