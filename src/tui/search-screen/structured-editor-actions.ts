import type { Pf2eTerminalSearchQuery } from "../search/service.js";
import { useSearchQueryFieldEditing } from "./query-field-editing.js";
import { useSearchQueryFieldBuilderActions } from "./query-field-builder/query-field-builder-actions.js";
import type { SearchQueryFieldBuilderSession } from "./query-field-builder/query-field-builder-session.js";
import { useSearchStructuredDraftActions } from "./structured-draft/structured-draft-actions.js";
import { useSearchStructuredDraftEditing } from "./structured-draft/structured-draft-editing.js";
import type { SearchStructuredDraftState } from "./structured-draft/structured-draft-support.js";
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
  structuredEditorSession: SearchQueryFieldBuilderSession | null;
} {
  const {
    chooseQueryField,
    editFieldClause,
    getExplorerBackedFieldOptions,
    getScopedFieldOptions,
    openOntologyFieldEditor,
    openOntologyFieldExplorer,
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

  const queryFieldBuilderActions = useSearchQueryFieldBuilderActions({
    appendStructuredDraftMetadataNode: structuredDraftActions.appendStructuredDraftMetadataNode,
    editFieldClause,
    getScopedFieldOptions,
    openOntologyFieldEditor,
    terminal,
  });

  const structuredEditorSession = useSearchStructuredDraftEditing({
    appendStructuredDraftMetadataNode: structuredDraftActions.appendStructuredDraftMetadataNode,
    cancelStructuredDraftSession: structuredDraftActions.cancelStructuredDraftSession,
    chooseQueryField,
    editFieldClause,
    finishStructuredDraftSession: structuredDraftActions.finishStructuredDraftSession,
    getExplorerBackedFieldOptions,
    getScopedFieldOptions,
    moveStructuredDraftSelection: structuredDraftActions.moveStructuredDraftSelection,
    openFilterExplorer,
    openOntologyFieldEditor,
    openOntologyFieldExplorer,
    openQueryFieldBuilder: queryFieldBuilderActions.openQueryFieldBuilder,
    prompts,
    replaceStructuredDraftQuery: structuredDraftActions.replaceStructuredDraftQuery,
    structuredDraftEntries: structuredDraftActions.structuredDraftEntries,
    structuredDraftState: structuredDraftActions.structuredDraftState,
    terminal,
    updateStructuredDraftMetadataNode: structuredDraftActions.updateStructuredDraftMetadataNode,
    user,
  });

  return {
    openStructuredDraftSession: structuredDraftActions.openStructuredDraftSession,
    structuredEditorSession: queryFieldBuilderActions.queryFieldBuilderSession ?? structuredEditorSession,
  };
}
