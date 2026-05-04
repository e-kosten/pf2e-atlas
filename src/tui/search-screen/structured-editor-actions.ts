import type { Pf2eTerminalSearchQuery } from "../search/service.js";
import { useSearchQueryFieldEditing } from "./query-field-editing.js";
import type { SearchStructuredEditorSession } from "./query-field-builder/query-field-builder-session.js";
import { useSearchStructuredDraftActions } from "./structured-draft/structured-draft-actions.js";
import { useSearchStructuredDraftEditing } from "./structured-draft/structured-draft-editing.js";
import type { SearchStructuredDraftState } from "./structured-draft/structured-draft-state.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace/workspace-action-types.js";

export function useSearchStructuredEditorActions({
  applyQueryUpdate,
  currentQuery,
  openFilterExplorer,
  prompts,
  terminal,
  user,
}: {
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  currentQuery: Pf2eTerminalSearchQuery;
  openFilterExplorer: OpenSearchFilterExplorer;
  prompts: SearchWorkspacePromptAdapters;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
  openStructuredDraftSession: (
    anchor: SearchStructuredDraftState["anchor"],
    query?: Pf2eTerminalSearchQuery,
  ) => void;
  structuredEditorSession: SearchStructuredEditorSession | null;
} {
  const {
    editFieldClause,
    getScopedFieldOptions,
    openOntologyFieldEditor,
  } = useSearchQueryFieldEditing({
    openFilterExplorer,
    prompts,
    terminal,
    user,
  });

  const structuredDraftActions = useSearchStructuredDraftActions({
    applyQueryUpdate,
    currentQuery,
    user,
  });

  const structuredEditorSession = useSearchStructuredDraftEditing({
    appendStructuredDraftMetadataNode: structuredDraftActions.appendStructuredDraftMetadataNode,
    cancelStructuredDraftSession: structuredDraftActions.cancelStructuredDraftSession,
    clearStructuredDraftMoveSource: structuredDraftActions.clearStructuredDraftMoveSource,
    editFieldClause,
    enterStructuredDraftMoveMode: structuredDraftActions.enterStructuredDraftMoveMode,
    finishStructuredDraftSession: structuredDraftActions.finishStructuredDraftSession,
    getScopedFieldOptions,
    moveStructuredDraftSelection: structuredDraftActions.moveStructuredDraftSelection,
    openFilterExplorer,
    openOntologyFieldEditor,
    prompts,
    replaceStructuredDraftProjection: structuredDraftActions.replaceStructuredDraftProjection,
    setStructuredDraftResumeTarget: structuredDraftActions.setStructuredDraftResumeTarget,
    structuredDraftEntries: structuredDraftActions.structuredDraftEntries,
    structuredDraftQuery: structuredDraftActions.structuredDraftQuery,
    structuredDraftState: structuredDraftActions.structuredDraftState,
    terminal,
    updateStructuredDraftMetadataNode: structuredDraftActions.updateStructuredDraftMetadataNode,
    user,
  });

  return {
    openStructuredDraftSession: structuredDraftActions.openStructuredDraftSession,
    structuredEditorSession,
  };
}
