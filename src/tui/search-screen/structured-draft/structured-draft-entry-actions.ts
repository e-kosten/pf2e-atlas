import React from "react";

import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import type { DerivedTagTerminalActionTargetOption } from "../../action-target.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";
import { useStructuredDraftExplorerActions } from "./structured-draft-explorer-actions.js";
import { useStructuredDraftPromptActions } from "./structured-draft-prompt-actions.js";
import { useStructuredDraftEditRouteActions } from "./structured-draft-edit-route-actions.js";
import {
  useStructuredDraftStructuralActions,
  type StructuredDraftEntryActionId,
} from "./structured-draft-structural-actions.js";
import type { StructuredDraftResumeTarget } from "./structured-draft-state.js";

export function useSearchStructuredDraftEntryActions({
  clearStructuredDraftMoveSource,
  editFieldClause,
  enterStructuredDraftMoveMode,
  getScopedFieldOptions,
  moveSourcePath,
  openFilterExplorer,
  prompts,
  replaceStructuredDraftProjection,
  setStructuredDraftResumeTarget,
  structuredDraftQuery,
  terminal,
  user,
}: {
  clearStructuredDraftMoveSource: () => void;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openFilterExplorer: OpenSearchFilterExplorer;
  moveSourcePath: number[] | null;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
  editStructuredDraftMetadata: (entry: SearchStructuredDraftEntry) => Promise<void>;
  getStructuredDraftEntryActions: (
    entry: SearchStructuredDraftEntry | null | undefined,
  ) => DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[];
  runStructuredDraftEntryAction: (
    entry: SearchStructuredDraftEntry | null | undefined,
    actionId: StructuredDraftEntryActionId,
  ) => Promise<void>;
} {
  const {
    openLiveExplorerGroupedField,
    openPromptFieldClause,
    selectPromptMetricKey,
  } = useStructuredDraftExplorerActions({
    getScopedFieldOptions,
    openFilterExplorer,
    replaceStructuredDraftProjection,
    terminal,
    user,
  });

  const { promptForClauseKind, promptForClauseNode, promptForSharedExplorerFieldOption } =
    useStructuredDraftPromptActions({
    editFieldClause,
    getScopedFieldOptions,
    openPromptFieldClause,
    selectPromptMetricKey,
    terminal,
    user,
  });

  const { executeStructuredDraftEditRoute } = useStructuredDraftEditRouteActions({
    editFieldClause,
    openLiveExplorerGroupedField,
    promptForClauseNode,
    prompts,
    replaceStructuredDraftProjection,
    terminal,
  });

  const {
    editStructuredDraftStructuralEntry,
    getStructuredDraftStructuralEntryActions,
    runStructuredDraftStructuralEntryAction,
  } = useStructuredDraftStructuralActions({
    clearStructuredDraftMoveSource,
    enterStructuredDraftMoveMode,
    getScopedFieldOptions,
    moveSourcePath,
    executeStructuredDraftEditRoute,
    promptForClauseKind,
    promptForClauseNode,
    promptForSharedExplorerFieldOption,
    prompts,
    replaceStructuredDraftProjection,
    setStructuredDraftResumeTarget,
    terminal,
  });

  const editStructuredDraftMetadata = React.useCallback(
    async (entry: SearchStructuredDraftEntry) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        await openLiveExplorerGroupedField(draftQuery, entry);
        return;
      }
      await editStructuredDraftStructuralEntry(draftQuery, entry);
    },
    [editStructuredDraftStructuralEntry, openLiveExplorerGroupedField, structuredDraftQuery],
  );

  const getStructuredDraftEntryActions = React.useCallback(
    (
      entry: SearchStructuredDraftEntry | null | undefined,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return [];
      }
      if (entry.kind === "queryFieldBucket") {
        return [
          {
            id: "edit",
            label: "Edit Clause",
            description: "Edit this current-group field bucket through the shared explorer.",
          },
        ];
      }
      return getStructuredDraftStructuralEntryActions(draftQuery, entry) ?? [];
    },
    [getStructuredDraftStructuralEntryActions, structuredDraftQuery],
  );

  const runStructuredDraftEntryAction = React.useCallback(
    async (entry: SearchStructuredDraftEntry | null | undefined, actionId: StructuredDraftEntryActionId) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        if (actionId === "edit") {
          await openLiveExplorerGroupedField(draftQuery, entry);
        }
        return;
      }
      await runStructuredDraftStructuralEntryAction(draftQuery, entry, actionId);
    },
    [openLiveExplorerGroupedField, runStructuredDraftStructuralEntryAction, structuredDraftQuery],
  );

  return {
    editStructuredDraftMetadata,
    getStructuredDraftEntryActions,
    runStructuredDraftEntryAction,
  };
}
