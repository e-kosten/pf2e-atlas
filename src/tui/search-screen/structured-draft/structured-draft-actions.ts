import React from "react";

import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import {
  appendMetadataNodeAtPath,
  updateMetadataNodeAtPath,
} from "../../search/query-core.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQueryMetadataTree,
  setSearchQueryMetadataTree,
} from "../../search/query-state.js";
import {
  buildStructuredDraftEntries,
  getStructuredDraftSelectionIndex,
  type SearchStructuredDraftState,
} from "./structured-draft-support.js";
import type { SearchWorkspaceUser } from "../workspace/workspace-action-types.js";

export function useSearchStructuredDraftActions({
  applyQueryUpdate,
  currentQuery,
  user,
}: {
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  currentQuery: Pf2eTerminalSearchQuery;
  user: SearchWorkspaceUser;
}): {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  cancelStructuredDraftSession: () => void;
  finishStructuredDraftSession: () => void;
  moveStructuredDraftSelection: (delta: number, itemCount: number) => void;
  openStructuredDraftSession: (
    anchor: SearchStructuredDraftState["anchor"],
    query?: Pf2eTerminalSearchQuery,
  ) => void;
  replaceStructuredDraftQuery: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftEntries: ReturnType<typeof buildStructuredDraftEntries>;
  structuredDraftState: SearchStructuredDraftState | null;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
} {
  const [structuredDraftState, setStructuredDraftState] = React.useState<SearchStructuredDraftState | null>(null);

  const hasSelectableSubcategories = React.useCallback(
    (category: ReturnType<typeof getSearchQueryCategory>): boolean => {
      if (!category) {
        return false;
      }
      return user.search.getSubcategoryOptions(category).length > 1;
    },
    [user.search],
  );

  const openStructuredDraftSession = React.useCallback(
    (anchor: SearchStructuredDraftState["anchor"], query: Pf2eTerminalSearchQuery = currentQuery) => {
      const draftQuery = user.search.normalizeQuery(query);
      const metadataFocusPath = anchor.kind === "queryNode" ? [...anchor.path] : null;
      const entries = buildStructuredDraftEntries(draftQuery, metadataFocusPath, {
        hasSelectableSubcategories,
        getActionCostOptions: user.search.getActionCostOptions,
      });

      setStructuredDraftState({
        anchor,
        draftQuery,
        metadataFocusPath,
        selectedIndex: getStructuredDraftSelectionIndex(anchor, entries),
      });
    },
    [currentQuery, hasSelectableSubcategories, user.search],
  );

  const replaceStructuredDraftQuery = React.useCallback(
    (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const nextDraftQuery = user.search.normalizeQuery(update(current.draftQuery));
        const entries = buildStructuredDraftEntries(nextDraftQuery, current.metadataFocusPath, {
          hasSelectableSubcategories,
          getActionCostOptions: user.search.getActionCostOptions,
        });

        return {
          ...current,
          draftQuery: nextDraftQuery,
          selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
        };
      });
    },
    [hasSelectableSubcategories, user.search],
  );

  const appendStructuredDraftMetadataNode = React.useCallback(
    (path: number[], nextNode: MetadataFilterNode) => {
      replaceStructuredDraftQuery((draftQuery) =>
        setSearchQueryMetadataTree(
          draftQuery,
          appendMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, nextNode),
        ),
      );
    },
    [replaceStructuredDraftQuery],
  );

  const updateStructuredDraftMetadataNode = React.useCallback(
    (
      path: number[],
      update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    ) => {
      replaceStructuredDraftQuery((draftQuery) =>
        setSearchQueryMetadataTree(
          draftQuery,
          updateMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, update),
        ),
      );
    },
    [replaceStructuredDraftQuery],
  );

  const finishStructuredDraftSession = React.useCallback(() => {
    if (!structuredDraftState) {
      return;
    }

    const nextQuery = structuredDraftState.draftQuery;
    setStructuredDraftState(null);
    applyQueryUpdate(() => nextQuery);
  }, [applyQueryUpdate, structuredDraftState]);

  const cancelStructuredDraftSession = React.useCallback(() => {
    setStructuredDraftState(null);
  }, []);

  const moveStructuredDraftSelection = React.useCallback((delta: number, itemCount: number) => {
    setStructuredDraftState((current) => {
      if (!current) {
        return current;
      }

      return {
        ...current,
        selectedIndex: clampStructuredDraftSelection(current.selectedIndex + delta, itemCount),
      };
    });
  }, []);

  const structuredDraftEntries = React.useMemo(
    () =>
      structuredDraftState
        ? buildStructuredDraftEntries(structuredDraftState.draftQuery, structuredDraftState.metadataFocusPath, {
            hasSelectableSubcategories,
            getActionCostOptions: user.search.getActionCostOptions,
          })
        : [],
    [hasSelectableSubcategories, structuredDraftState, user.search],
  );

  return {
    appendStructuredDraftMetadataNode,
    cancelStructuredDraftSession,
    finishStructuredDraftSession,
    moveStructuredDraftSelection,
    openStructuredDraftSession,
    replaceStructuredDraftQuery,
    structuredDraftEntries,
    structuredDraftState,
    updateStructuredDraftMetadataNode,
  };
}
