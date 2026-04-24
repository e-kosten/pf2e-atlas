import React from "react";

import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import {
  appendSearchFilterNodeAtPath,
  getSearchFilterNodeAtPath,
  updateSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import {
  projectSearchQueryFilter,
  stripSearchQueryFilter,
} from "../../search/query-projection.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import { getSearchQueryRootOperator } from "../../search/query-state.js";
import {
  canonicalFilterToMetadataNode,
  metadataFilterNodeToCanonicalFilter,
} from "../../search/query-parts.js";
import {
  buildStructuredDraftQuery,
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
  clearStructuredDraftMoveSource: () => void;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  finishStructuredDraftSession: () => void;
  moveStructuredDraftSelection: (delta: number, itemCount: number) => void;
  openStructuredDraftSession: (
    anchor: SearchStructuredDraftState["anchor"],
    query?: Pf2eTerminalSearchQuery,
  ) => void;
  replaceStructuredDraftProjection: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftEntries: ReturnType<typeof buildStructuredDraftEntries>;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  structuredDraftState: SearchStructuredDraftState | null;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
} {
  const [structuredDraftState, setStructuredDraftState] = React.useState<SearchStructuredDraftState | null>(null);

  const openStructuredDraftSession = React.useCallback(
    (anchor: SearchStructuredDraftState["anchor"], query: Pf2eTerminalSearchQuery = currentQuery) => {
      const draftQuery = user.search.normalizeQuery(query);
      const metadataFocusPath = anchor.kind === "queryNode" ? [...anchor.path] : null;
      const entries = buildStructuredDraftEntries(draftQuery, metadataFocusPath, {
        packLabelResolver: user.search.getPackLabel,
      });

      setStructuredDraftState({
        anchor,
        baseQuery: stripSearchQueryFilter(draftQuery),
        draftFilter: draftQuery.filter,
        metadataFocusPath,
        moveSourcePath: null,
        selectedIndex: getStructuredDraftSelectionIndex(anchor, entries),
      });
    },
    [currentQuery, user.search],
  );

  const replaceStructuredDraftProjection = React.useCallback(
    (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const nextDraftQuery = user.search.normalizeQuery(update(buildStructuredDraftQuery(current)));
        const entries = buildStructuredDraftEntries(nextDraftQuery, current.metadataFocusPath, {
          packLabelResolver: user.search.getPackLabel,
          moveSourcePath: current.moveSourcePath,
        });

        return {
          ...current,
          baseQuery: stripSearchQueryFilter(nextDraftQuery),
          draftFilter: nextDraftQuery.filter,
          selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
        };
      });
    },
    [user.search],
  );

  const appendStructuredDraftMetadataNode = React.useCallback(
    (path: number[], nextNode: MetadataFilterNode) => {
      const nextFilterNode = metadataFilterNodeToCanonicalFilter(nextNode);
      if (!nextFilterNode) {
        return;
      }
      replaceStructuredDraftProjection((draftQuery) => ({
        ...draftQuery,
        filter: appendSearchFilterNodeAtPath(
          draftQuery.filter,
          path,
          nextFilterNode,
          getSearchQueryRootOperator(draftQuery),
        ),
      }));
    },
    [replaceStructuredDraftProjection],
  );

  const updateStructuredDraftMetadataNode = React.useCallback(
    (
      path: number[],
      update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    ) => {
      replaceStructuredDraftProjection((draftQuery) => {
        const currentNode = getSearchFilterNodeAtPath(draftQuery.filter, path);
        const currentMetadataNode = currentNode ? canonicalFilterToMetadataNode(currentNode) : null;
        if (!currentMetadataNode) {
          return draftQuery;
        }

        const nextMetadataNode = update(currentMetadataNode);
        const nextCanonicalNode = metadataFilterNodeToCanonicalFilter(nextMetadataNode);
        return {
          ...draftQuery,
          filter: updateSearchFilterNodeAtPath(draftQuery.filter, path, () => nextCanonicalNode),
        };
      });
    },
    [replaceStructuredDraftProjection],
  );

  const finishStructuredDraftSession = React.useCallback(() => {
    if (!structuredDraftState) {
      return;
    }

    const nextQuery = buildStructuredDraftQuery(structuredDraftState);
    setStructuredDraftState(null);
    applyQueryUpdate(() => nextQuery);
  }, [applyQueryUpdate, structuredDraftState]);

  const cancelStructuredDraftSession = React.useCallback(() => {
    setStructuredDraftState((current) => {
      if (!current?.moveSourcePath) {
        return null;
      }

      const currentQuery = buildStructuredDraftQuery(current);
      const entries = buildStructuredDraftEntries(currentQuery, current.metadataFocusPath, {
        packLabelResolver: user.search.getPackLabel,
      });
      const selectionIndex = current.moveSourcePath
        ? entries.findIndex(
            (entry) =>
              entry.kind === "queryNode" &&
              JSON.stringify(entry.treePath ?? []) === JSON.stringify(current.moveSourcePath),
          )
        : current.selectedIndex;

      return {
        ...current,
        moveSourcePath: null,
        selectedIndex: clampStructuredDraftSelection(selectionIndex >= 0 ? selectionIndex : current.selectedIndex, entries.length),
      };
    });
  }, [user.search]);

  const clearStructuredDraftMoveSource = React.useCallback(() => {
    setStructuredDraftState((current) => {
      if (!current?.moveSourcePath) {
        return current;
      }

      const currentQuery = buildStructuredDraftQuery(current);
      const entries = buildStructuredDraftEntries(currentQuery, current.metadataFocusPath, {
        packLabelResolver: user.search.getPackLabel,
      });
      return {
        ...current,
        moveSourcePath: null,
        selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
      };
    });
  }, [user.search]);

  const enterStructuredDraftMoveMode = React.useCallback(
    (path: number[]) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const currentQuery = buildStructuredDraftQuery(current);
        const entries = buildStructuredDraftEntries(currentQuery, current.metadataFocusPath, {
          packLabelResolver: user.search.getPackLabel,
          moveSourcePath: path,
        });
        const firstSlotIndex = entries.findIndex((entry) => entry.kind === "queryInsertionSlot");

        return {
          ...current,
          moveSourcePath: [...path],
          selectedIndex: clampStructuredDraftSelection(firstSlotIndex >= 0 ? firstSlotIndex : current.selectedIndex, entries.length),
        };
      });
    },
    [user.search],
  );

  const moveStructuredDraftSelection = React.useCallback((delta: number, itemCount: number) => {
    setStructuredDraftState((current) => {
      if (!current) {
        return current;
      }

      const currentQuery = buildStructuredDraftQuery(current);
      const entries = buildStructuredDraftEntries(currentQuery, current.metadataFocusPath, {
        packLabelResolver: user.search.getPackLabel,
        moveSourcePath: current.moveSourcePath,
      });
      const selectableIndexes = entries.flatMap((entry, index) =>
        current.moveSourcePath ? (entry.kind === "queryInsertionSlot" ? [index] : []) : [index],
      );
      const currentSelectableIndex = Math.max(
        0,
        selectableIndexes.findIndex((index) => index >= current.selectedIndex),
      );
      const nextSelectablePosition = Math.max(
        0,
        Math.min(currentSelectableIndex + delta, Math.max(0, selectableIndexes.length - 1)),
      );

      return {
        ...current,
        selectedIndex:
          selectableIndexes.length > 0
            ? selectableIndexes[nextSelectablePosition]!
            : clampStructuredDraftSelection(current.selectedIndex + delta, itemCount),
      };
    });
  }, [user.search]);

  const structuredDraftEntries = React.useMemo(
    () =>
      structuredDraftState
        ? buildStructuredDraftEntries(buildStructuredDraftQuery(structuredDraftState), structuredDraftState.metadataFocusPath, {
            packLabelResolver: user.search.getPackLabel,
            moveSourcePath: structuredDraftState.moveSourcePath,
          })
        : [],
    [structuredDraftState, user.search],
  );

  const structuredDraftQuery = React.useMemo(
    () => (structuredDraftState ? buildStructuredDraftQuery(structuredDraftState) : null),
    [structuredDraftState],
  );

  return {
    appendStructuredDraftMetadataNode,
    cancelStructuredDraftSession,
    clearStructuredDraftMoveSource,
    enterStructuredDraftMoveMode,
    finishStructuredDraftSession,
    moveStructuredDraftSelection,
    openStructuredDraftSession,
    replaceStructuredDraftProjection,
    structuredDraftEntries,
    structuredDraftQuery,
    structuredDraftState,
    updateStructuredDraftMetadataNode,
  };
}
