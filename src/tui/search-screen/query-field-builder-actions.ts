import React from "react";

import type { MetadataFilterNode } from "../../domain/metadata-types.js";
import { clampStructuredDraftSelection } from "../search/structured-draft-session.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import type { SearchQueryFieldBuilderSession } from "./query-field-builder-session.js";
import {
  buildQueryFieldBuilderItems,
  buildQueryFieldBuilderPreviewQuery,
  buildQueryFieldBuilderSessionItems,
  compileQueryFieldBuilderDrafts,
  type QueryFieldBuilderState,
} from "./query-field-builder-support.js";
import type { SearchWorkspaceTerminal } from "./workspace-action-types.js";

export function useSearchQueryFieldBuilderActions({
  appendStructuredDraftMetadataNode,
  editFieldClause,
  getScopedFieldOptions,
  openOntologyFieldEditor,
  terminal,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openOntologyFieldEditor: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode: MetadataFilterNode | null,
    onApply: (nextNode: MetadataFilterNode | null) => void,
    onReturn?: () => void,
  ) => Promise<boolean>;
  terminal: SearchWorkspaceTerminal;
}): {
  openQueryFieldBuilder: (query: Pf2eTerminalSearchQuery, path?: number[]) => Promise<boolean>;
  queryFieldBuilderSession: SearchQueryFieldBuilderSession | null;
} {
  const [queryFieldBuilderState, setQueryFieldBuilderState] = React.useState<QueryFieldBuilderState | null>(null);

  const updateQueryFieldBuilderDraft = React.useCallback((field: string, node: MetadataFilterNode | null) => {
    setQueryFieldBuilderState((current) => {
      if (!current) {
        return current;
      }

      return {
        ...current,
        fieldDrafts: {
          ...current.fieldDrafts,
          [field]: node,
        },
      };
    });
  }, []);

  const openQueryFieldBuilder = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = []) => {
      const fieldOptions = getScopedFieldOptions(query);
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
        return false;
      }

      setQueryFieldBuilderState({
        draftQuery: query,
        path,
        items: buildQueryFieldBuilderItems(fieldOptions),
        selectedIndex: 0,
        fieldDrafts: {},
      });
      return true;
    },
    [getScopedFieldOptions, terminal],
  );

  const finishQueryFieldBuilder = React.useCallback(() => {
    const nextNode = compileQueryFieldBuilderDrafts(queryFieldBuilderState?.fieldDrafts ?? {});
    const targetPath = queryFieldBuilderState?.path ?? [];
    setQueryFieldBuilderState(null);
    if (!nextNode) {
      return;
    }
    appendStructuredDraftMetadataNode(targetPath, nextNode);
  }, [appendStructuredDraftMetadataNode, queryFieldBuilderState]);

  const discardQueryFieldBuilder = React.useCallback(() => {
    setQueryFieldBuilderState(null);
  }, []);

  const returnQueryFieldBuilder = React.useCallback(() => {
    finishQueryFieldBuilder();
  }, [finishQueryFieldBuilder]);

  const editQueryFieldBuilderField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const scopeQuery = queryFieldBuilderState?.draftQuery;
      if (!scopeQuery) {
        return;
      }

      const currentNode = queryFieldBuilderState.fieldDrafts[fieldOption.value] ?? null;
      if (fieldOption.editor === "sharedExplorer") {
        await openOntologyFieldEditor(scopeQuery, fieldOption, currentNode, (nextNode) => {
          updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
        });
        return;
      }

      const nextNode = await editFieldClause(scopeQuery, fieldOption, currentNode);
      if (nextNode === undefined) {
        return;
      }

      updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
    },
    [editFieldClause, openOntologyFieldEditor, queryFieldBuilderState, updateQueryFieldBuilderDraft],
  );

  const selectCurrentQueryFieldBuilderItem = React.useCallback(() => {
    const selectedItem =
      queryFieldBuilderState?.items[
        clampStructuredDraftSelection(queryFieldBuilderState.selectedIndex, queryFieldBuilderState.items.length)
      ] ?? null;
    if (!selectedItem) {
      return;
    }
    if (selectedItem.kind === "field") {
      void editQueryFieldBuilderField(selectedItem.fieldOption);
      return;
    }
    if (selectedItem.kind === "finish") {
      returnQueryFieldBuilder();
      return;
    }
    discardQueryFieldBuilder();
  }, [discardQueryFieldBuilder, editQueryFieldBuilderField, queryFieldBuilderState, returnQueryFieldBuilder]);

  const queryFieldBuilderSession = React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!queryFieldBuilderState) {
      return null;
    }

    return {
      kind: "queryFieldBuilder",
      title: "Add Query Part",
      subtitle: "Choose query fields and keep the full staged query visible while you refine it",
      leftTitle: "[QUERY FIELDS]",
      rightTitle: "Staged Summary & Detail",
      statusText: "Field edits stay staged until you finish the structured query editor.",
      draftQuery: buildQueryFieldBuilderPreviewQuery(queryFieldBuilderState),
      items: buildQueryFieldBuilderSessionItems(queryFieldBuilderState),
      selectedIndex: clampStructuredDraftSelection(
        queryFieldBuilderState.selectedIndex,
        queryFieldBuilderState.items.length,
      ),
      fieldDrafts: queryFieldBuilderState.fieldDrafts,
      moveSelection: (delta, itemCount) => {
        setQueryFieldBuilderState((current) => {
          if (!current) {
            return current;
          }

          return {
            ...current,
            selectedIndex: clampStructuredDraftSelection(current.selectedIndex + delta, itemCount),
          };
        });
      },
      selectCurrent: selectCurrentQueryFieldBuilderItem,
      finish: returnQueryFieldBuilder,
      cancel: returnQueryFieldBuilder,
      helpTitle: "Add Query Part Help",
      helpBody: [
        { text: "Choose the next query field to stage into the structured query.", tone: "section" },
        {
          text: "The right pane keeps the full staged query summary visible while you move between fields.",
        },
        {
          text: "Use Left or Esc to return to the staged query with current field edits preserved.",
        },
        {
          text: "Use the discard row only when you want to drop the in-progress field edits from this subpage.",
        },
      ],
    };
  }, [queryFieldBuilderState, returnQueryFieldBuilder, selectCurrentQueryFieldBuilderItem]);

  return {
    openQueryFieldBuilder,
    queryFieldBuilderSession,
  };
}
