import React from "react";

import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import type { MetadataFilterNode } from "../../../search/filters/types.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchQueryFieldBuilderSession } from "../query-field-builder/query-field-builder-session.js";
import type { SearchStructuredDraftState } from "./structured-draft-support.js";
import { useSearchStructuredDraftMetadataActions } from "./structured-draft-metadata-actions.js";
import { useSearchStructuredDraftPartActions } from "./structured-draft-part-actions.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

export function useSearchStructuredDraftEditing({
  appendStructuredDraftMetadataNode,
  cancelStructuredDraftSession,
  chooseQueryField,
  editFieldClause,
  finishStructuredDraftSession,
  getExplorerBackedFieldOptions,
  getScopedFieldOptions,
  moveStructuredDraftSelection,
  openFilterExplorer,
  openOntologyFieldEditor,
  openOntologyFieldExplorer,
  openQueryFieldBuilder,
  prompts,
  replaceStructuredDraftQuery,
  structuredDraftEntries,
  structuredDraftState,
  terminal,
  updateStructuredDraftMetadataNode,
  user,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  cancelStructuredDraftSession: () => void;
  chooseQueryField: (query: Pf2eTerminalSearchQuery) => Promise<Pf2eTerminalQueryFieldOption | null>;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  finishStructuredDraftSession: () => void;
  getExplorerBackedFieldOptions: (fieldOptions: Pf2eTerminalQueryFieldOption[]) => Pf2eTerminalQueryFieldOption[];
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  moveStructuredDraftSelection: (delta: number, itemCount: number) => void;
  openFilterExplorer: OpenSearchFilterExplorer;
  openOntologyFieldEditor: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode: MetadataFilterNode | null,
    onApply: (nextNode: MetadataFilterNode | null) => void,
    onReturn?: () => void,
  ) => Promise<boolean>;
  openOntologyFieldExplorer: (
    query: Pf2eTerminalSearchQuery,
    fieldOptions: Pf2eTerminalQueryFieldOption[],
    onApply: (nextNode: MetadataFilterNode | null) => void,
  ) => Promise<boolean>;
  openQueryFieldBuilder: (query: Pf2eTerminalSearchQuery, path?: number[]) => Promise<boolean>;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftQuery: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftEntries: SearchStructuredDraftEntry[];
  structuredDraftState: SearchStructuredDraftState | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
  user: SearchWorkspaceUser;
}): SearchQueryFieldBuilderSession | null {
  const {
    editStructuredDraftActionCost,
    editStructuredDraftCategory,
    editStructuredDraftLevelRange,
    editStructuredDraftRarity,
    editStructuredDraftSubcategory,
  } = useSearchStructuredDraftPartActions({
    openFilterExplorer,
    prompts,
    replaceStructuredDraftQuery,
    structuredDraftState,
    terminal,
    user,
  });

  const { editStructuredDraftMetadata } = useSearchStructuredDraftMetadataActions({
    appendStructuredDraftMetadataNode,
    chooseQueryField,
    editFieldClause,
    getExplorerBackedFieldOptions,
    getScopedFieldOptions,
    openOntologyFieldEditor,
    openOntologyFieldExplorer,
    openQueryFieldBuilder,
    prompts,
    structuredDraftState,
    terminal,
    updateStructuredDraftMetadataNode,
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
    if (selectedEntry.kind === "category") {
      void editStructuredDraftCategory();
      return;
    }
    if (selectedEntry.kind === "subcategory") {
      void editStructuredDraftSubcategory();
      return;
    }
    if (selectedEntry.kind === "levelRange") {
      void editStructuredDraftLevelRange();
      return;
    }
    if (selectedEntry.kind === "rarity") {
      void editStructuredDraftRarity();
      return;
    }
    if (selectedEntry.kind === "actionCost") {
      void editStructuredDraftActionCost();
      return;
    }
    void editStructuredDraftMetadata(selectedEntry.metadataPath ?? []);
  }, [
    cancelStructuredDraftSession,
    editStructuredDraftActionCost,
    editStructuredDraftCategory,
    editStructuredDraftLevelRange,
    editStructuredDraftMetadata,
    editStructuredDraftRarity,
    editStructuredDraftSubcategory,
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
      statusText: "Left/Esc applies the staged query and returns. Use the discard row to abandon it.",
      draftQuery: structuredDraftState.draftQuery,
      items: structuredDraftEntries.map((entry) =>
        entry.kind === "finish" || entry.kind === "cancel"
          ? { kind: entry.kind, label: entry.label }
          : {
              kind: "workspaceEntry" as const,
              label: `${entry.label} | ${entry.value}`,
              workspaceEntry: {
                action: "addQueryPart",
                label: entry.label,
                value: entry.value,
                description: entry.description,
                disabled: entry.disabled,
                disabledReason: entry.disabledReason,
              },
            },
      ),
      selectedIndex: clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length),
      moveSelection: moveStructuredDraftSelection,
      selectCurrent: selectCurrentStructuredDraftEntry,
      finish: finishStructuredDraftSession,
      cancel: finishStructuredDraftSession,
      helpTitle: "Structured Query Editor Help",
      helpBody: [
        { text: "Stage structured search changes before applying them to the live query.", tone: "section" },
        { text: "The summary stays visible while you move focus so prior staged selections do not disappear." },
        { text: "Use Left or Esc to apply the staged query and return to the top editor." },
        { text: "Use the discard row only when you want to abandon the staged query entirely." },
      ],
    };
  }, [
    finishStructuredDraftSession,
    moveStructuredDraftSelection,
    selectCurrentStructuredDraftEntry,
    structuredDraftEntries,
    structuredDraftState,
  ]);
}
