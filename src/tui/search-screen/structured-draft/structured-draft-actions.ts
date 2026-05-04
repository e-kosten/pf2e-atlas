import React from "react";

import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import {
  appendSearchFilterNodeAtPath,
  getSearchFilterNodeAtPath,
  updateSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import { getSearchQueryCategory, getSearchQuerySubcategory, getSearchQueryRootOperator } from "../../search/query-state.js";
import {
  canonicalFilterToMetadataNode,
  metadataFilterNodeToCanonicalFilter,
} from "../../search/query-parts.js";
import {
  buildStructuredDraftEntries,
} from "./structured-draft-support.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftNodeResumeTarget,
  createStructuredDraftResumeTargetForContainingGroup,
  createStructuredDraftResumeTargetForNodePath,
  createStructuredDraftRootResumeTarget,
  getStructuredDraftSelectionIndex,
  getStructuredDraftSelectionIndexForResumeTarget,
  type SearchStructuredDraftState,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";
import type { SearchWorkspaceUser } from "../workspace/workspace-action-types.js";

type StructuredDraftProjectionOptions = {
  resumeTarget?: StructuredDraftResumeTarget | null;
};

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
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: StructuredDraftProjectionOptions,
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  structuredDraftEntries: ReturnType<typeof buildStructuredDraftEntries>;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  structuredDraftState: SearchStructuredDraftState | null;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    options?: StructuredDraftProjectionOptions,
  ) => void;
} {
  const [structuredDraftState, setStructuredDraftState] = React.useState<SearchStructuredDraftState | null>(null);
  const currentQueryRef = React.useRef(currentQuery);

  React.useEffect(() => {
    currentQueryRef.current = currentQuery;
  }, [currentQuery]);

  const getGroupedFieldValuesForQuery = React.useCallback(
    (query: Pf2eTerminalSearchQuery): ReadonlySet<string> =>
      new Set(
        user.search
          .getQueryFieldOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query))
          .filter((fieldOption) => fieldOption.editor === "sharedExplorer")
          .map((fieldOption) => fieldOption.value),
      ),
    [user.search],
  );

  const buildEntriesForQuery = React.useCallback(
    (query: Pf2eTerminalSearchQuery, options?: { resumeTarget?: StructuredDraftResumeTarget | null; moveSourcePath?: number[] | null }) =>
      buildStructuredDraftEntries(query, options?.resumeTarget ?? null, {
        groupedFieldValues: getGroupedFieldValuesForQuery(query),
        packLabelResolver: user.search.getPackLabel,
        moveSourcePath: options?.moveSourcePath ?? null,
      }),
    [getGroupedFieldValuesForQuery, user.search],
  );

  const replaceStructuredDraftProjection = React.useCallback(
    (
      update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
      options?: StructuredDraftProjectionOptions,
    ) => {
      let appliedQuery = currentQueryRef.current;
      applyQueryUpdate((query) => {
        appliedQuery = user.search.normalizeQuery(update(query));
        return appliedQuery;
      });

      currentQueryRef.current = appliedQuery;
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const resumeTarget = options?.resumeTarget ?? current.resumeTarget;
        const entries = buildEntriesForQuery(appliedQuery, {
          resumeTarget,
          moveSourcePath: current.moveSourcePath,
        });

        return {
          ...current,
          resumeTarget,
          selectedIndex: getStructuredDraftSelectionIndexForResumeTarget(entries, resumeTarget, current.selectedIndex),
        };
      });
    },
    [applyQueryUpdate, buildEntriesForQuery, user.search],
  );

  const openStructuredDraftSession = React.useCallback(
    (anchor: SearchStructuredDraftState["anchor"], query: Pf2eTerminalSearchQuery = currentQueryRef.current) => {
      const liveQuery = user.search.normalizeQuery(query);
      const currentLiveQuery = currentQueryRef.current;
      if (query !== currentLiveQuery) {
        currentQueryRef.current = liveQuery;
        applyQueryUpdate(() => liveQuery);
      }

      const resumeTarget =
        anchor.kind === "queryNode"
          ? createStructuredDraftResumeTargetForNodePath(liveQuery.filter, anchor.path)
          : anchor.kind === "addQueryPart"
            ? createStructuredDraftGroupResumeTarget([])
            : createStructuredDraftRootResumeTarget();
      const entries = buildEntriesForQuery(liveQuery, { resumeTarget });

      setStructuredDraftState({
        anchor,
        resumeTarget,
        moveSourcePath: null,
        selectedIndex: getStructuredDraftSelectionIndex(anchor, entries),
      });
    },
    [applyQueryUpdate, buildEntriesForQuery, user.search],
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
      options?: StructuredDraftProjectionOptions,
    ) => {
      const liveQuery = currentQueryRef.current;
      const currentNode = getSearchFilterNodeAtPath(liveQuery.filter, path);
      const currentMetadataNode = currentNode ? canonicalFilterToMetadataNode(currentNode) : null;
      if (!currentMetadataNode) {
        return;
      }

      const nextMetadataNode = update(currentMetadataNode);
      const nextCanonicalNode = metadataFilterNodeToCanonicalFilter(nextMetadataNode);
      replaceStructuredDraftProjection(
        (draftQuery) => ({
          ...draftQuery,
          filter: updateSearchFilterNodeAtPath(draftQuery.filter, path, () => nextCanonicalNode),
        }),
        {
          resumeTarget:
            options?.resumeTarget ??
            (nextCanonicalNode
              ? createStructuredDraftNodeResumeTarget(path)
              : createStructuredDraftResumeTargetForContainingGroup(liveQuery.filter, path)),
        },
      );
    },
    [replaceStructuredDraftProjection],
  );

  const setStructuredDraftResumeTarget = React.useCallback(
    (target: StructuredDraftResumeTarget | null) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const currentQueryState = currentQueryRef.current;
        const entries = buildEntriesForQuery(currentQueryState, {
          resumeTarget: target,
          moveSourcePath: current.moveSourcePath,
        });

        return {
          ...current,
          resumeTarget: target ?? createStructuredDraftRootResumeTarget(),
          selectedIndex: getStructuredDraftSelectionIndexForResumeTarget(entries, target, current.selectedIndex),
        };
      });
    },
    [buildEntriesForQuery],
  );

  const finishStructuredDraftSession = React.useCallback(() => {
    setStructuredDraftState(null);
  }, []);

  const cancelStructuredDraftSession = React.useCallback(() => {
    setStructuredDraftState((current) => {
      if (!current) {
        return current;
      }

      if (!current.moveSourcePath) {
        return null;
      }

      const currentQueryState = currentQueryRef.current;
      const entries = buildEntriesForQuery(currentQueryState, {
        resumeTarget: current.resumeTarget,
      });
      const selectionIndex = entries.findIndex(
        (entry) =>
          entry.kind === "queryNode" &&
          JSON.stringify(entry.treePath ?? []) === JSON.stringify(current.moveSourcePath),
      );

      return {
        ...current,
        moveSourcePath: null,
        selectedIndex: clampStructuredDraftSelection(selectionIndex >= 0 ? selectionIndex : current.selectedIndex, entries.length),
      };
    });
  }, [buildEntriesForQuery]);

  const clearStructuredDraftMoveSource = React.useCallback(() => {
    setStructuredDraftState((current) => {
      if (!current?.moveSourcePath) {
        return current;
      }

      const currentQueryState = currentQueryRef.current;
      const entries = buildEntriesForQuery(currentQueryState, {
        resumeTarget: current.resumeTarget,
      });
      return {
        ...current,
        moveSourcePath: null,
        selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
      };
    });
  }, [buildEntriesForQuery]);

  const enterStructuredDraftMoveMode = React.useCallback(
    (path: number[]) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const currentQueryState = currentQueryRef.current;
        const entries = buildEntriesForQuery(currentQueryState, {
          resumeTarget: current.resumeTarget,
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
    [buildEntriesForQuery],
  );

  const moveStructuredDraftSelection = React.useCallback(
    (delta: number, itemCount: number) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }

        const currentQueryState = currentQueryRef.current;
        const entries = buildEntriesForQuery(currentQueryState, {
          resumeTarget: current.resumeTarget,
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
    },
    [buildEntriesForQuery],
  );

  const structuredDraftEntries = React.useMemo(
    () =>
      structuredDraftState
        ? buildEntriesForQuery(currentQuery, {
            resumeTarget: structuredDraftState.resumeTarget,
            moveSourcePath: structuredDraftState.moveSourcePath,
          })
        : [],
    [buildEntriesForQuery, currentQuery, structuredDraftState],
  );

  const structuredDraftQuery = React.useMemo(
    () => (structuredDraftState ? currentQuery : null),
    [currentQuery, structuredDraftState],
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
    setStructuredDraftResumeTarget,
    structuredDraftEntries,
    structuredDraftQuery,
    structuredDraftState,
    updateStructuredDraftMetadataNode,
  };
}
