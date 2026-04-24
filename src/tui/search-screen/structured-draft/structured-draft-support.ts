import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  formatSearchFilterNodePresentationAlias,
  getSearchFilterNodeAtPath,
  getSearchFilterNodeChildren,
  isSearchFilterBooleanGroup,
  isValidSearchFilterMoveTargetGroupPath,
} from "../../search/query-core.js";
import { clampStructuredDraftSelection } from "../../search/structured-draft-session.js";
import type {
  SearchStructuredDraftAnchor,
  SearchStructuredDraftEntry,
  SearchStructuredDraftEntryKind,
} from "../../search/structured-draft-session.js";
import { getSearchQueryCategory, getSearchQueryRootOperator } from "../../search/query-state.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import {
  projectSearchQueryFilter,
  type Pf2eTerminalSearchQueryBase,
} from "../../search/query-projection.js";

export type SearchStructuredDraftState = {
  anchor: SearchStructuredDraftAnchor;
  baseQuery: Pf2eTerminalSearchQueryBase;
  draftFilter: Pf2eTerminalSearchQuery["filter"];
  metadataFocusPath: number[] | null;
  moveSourcePath: number[] | null;
  selectedIndex: number;
};

export function buildStructuredDraftQuery(state: Pick<SearchStructuredDraftState, "baseQuery" | "draftFilter">): Pf2eTerminalSearchQuery {
  return projectSearchQueryFilter(state.baseQuery, state.draftFilter);
}

type FilterDisplayNode = {
  node: SearchFilterNode;
  path: number[];
};

function getFilterRootDisplayNodes(node: SearchFilterNode | undefined): FilterDisplayNode[] {
  if (!node) {
    return [];
  }

  if (node.kind === "allOf" || node.kind === "anyOf") {
    return node.children.map((child, index) => ({ node: child, path: [index] }));
  }

  return [{ node, path: [] }];
}

function formatTreePrefix(ancestorContinuations: boolean[], isLast: boolean): string {
  const stem = ancestorContinuations.map((continuation) => (continuation ? "│  " : "   ")).join("");
  return `${stem}${isLast ? "└─ " : "├─ "}`;
}

function buildFilterTreeEntries(
  node: SearchFilterNode | undefined,
  options: {
    category: ReturnType<typeof getSearchQueryCategory>;
    packLabelResolver?: (packValue: string) => string | null | undefined;
    moveSourcePath: number[] | null;
    rootOperator: "allOf" | "anyOf";
  },
): SearchStructuredDraftEntry[] {
  const entries: SearchStructuredDraftEntry[] = [
    {
      kind: "queryTreeRoot",
      key: "queryTree:root",
      label: "Filter Tree",
      description: "The dedicated filter builder always presents a visible root boolean group for top-level query clauses.",
      menuLabel: options.rootOperator === "anyOf" ? "anyOf" : "allOf",
    },
  ];

  const displayNodes = getFilterRootDisplayNodes(node);
  const moveSourcePath = options.moveSourcePath;
  const canSelectNode = (path: number[]): boolean =>
    moveSourcePath === null || path.length === 0 || moveSourcePath.every((segment, index) => path[index] === segment);

  const appendNode = (
    current: SearchFilterNode,
    path: number[],
    depth: number,
    ancestorContinuations: boolean[],
  ): void => {
    const isGroup = isSearchFilterBooleanGroup(current) || current.kind === "not";
    const menuLabel = `${formatTreePrefix(ancestorContinuations, false)}${formatSearchFilterNodePresentationAlias(current, {
      category: options.category,
      packLabelResolver: options.packLabelResolver,
      style: "tree",
    })}`;

    entries.push({
      kind: "queryNode",
      key: `queryNode:${path.length > 0 ? path.join(".") : "rootNode"}`,
      label: formatSearchFilterNodePresentationAlias(current, {
        category: options.category,
        packLabelResolver: options.packLabelResolver,
        style: "compact",
      }),
      description: "Open node actions for wrapping, regrouping, moving, or removing the selected filter node.",
      disabled: !canSelectNode(path),
      disabledReason: moveSourcePath ? "Move mode only allows the anchored node or destination slots." : undefined,
      treePath: path,
      indent: depth,
      menuLabel,
    });

    if (!isGroup) {
      return;
    }

    const children = getSearchFilterNodeChildren(current);
    children.forEach((child, childIndex) => {
      appendNode(child, [...path, childIndex], depth + 1, [...ancestorContinuations, true]);
    });

    if (isSearchFilterBooleanGroup(current)) {
      const isValidMoveTarget =
        moveSourcePath === null || isValidSearchFilterMoveTargetGroupPath(node, moveSourcePath, path);
      if (isValidMoveTarget) {
        entries.push({
          kind: "queryInsertionSlot",
          key: `querySlot:${path.join(".")}`,
          label: moveSourcePath ? "[move here]" : "[+ add here]",
          description: moveSourcePath
            ? "Confirm the move and append the anchored node at the bottom of this group."
            : "Add a new filter or nested group at the bottom of this group.",
          insertionPath: path,
          indent: depth + 1,
          menuLabel: `${formatTreePrefix([...ancestorContinuations, false], true)}${moveSourcePath ? "[move here]" : "[+ add here]"}`,
        });
      }
    }
  };

  displayNodes.forEach((displayNode) => {
    appendNode(displayNode.node, displayNode.path, 1, []);
  });

  if (moveSourcePath === null || isValidSearchFilterMoveTargetGroupPath(node, moveSourcePath, [])) {
    entries.push({
      kind: "queryInsertionSlot",
      key: "querySlot:root",
      label: moveSourcePath ? "[move here]" : "[+ add here]",
      description: moveSourcePath
        ? `Confirm the move and append the anchored node at the root ${options.rootOperator} group.`
        : "Add a new top-level filter or nested group.",
      insertionPath: [],
      indent: 1,
      menuLabel: `${formatTreePrefix([], true)}${moveSourcePath ? "[move here]" : "[+ add here]"}`,
    });
  }

  return entries;
}

export function buildStructuredDraftEntries(
  draftQuery: Pf2eTerminalSearchQuery,
  metadataFocusPath: number[] | null,
  options: {
    packLabelResolver?: (packValue: string) => string | null | undefined;
    moveSourcePath?: number[] | null;
  },
): SearchStructuredDraftEntry[] {
  const draftCategory = getSearchQueryCategory(draftQuery);
  const entries: SearchStructuredDraftEntry[] = [
    ...buildFilterTreeEntries(draftQuery.filter, {
      category: draftCategory,
      packLabelResolver: options.packLabelResolver,
      moveSourcePath: options.moveSourcePath ?? null,
      rootOperator: getSearchQueryRootOperator(draftQuery),
    }),
    {
      kind: "finish",
      key: "finish",
      label: "Apply Structured Edit",
      value: "Apply staged query",
      description: "Commit the staged structured query back into the live editor.",
    },
    {
      kind: "cancel",
      key: "cancel",
      label: "Discard Structured Edit",
      value: "Discard staged query",
      description: "Discard the staged structured query and keep the live query unchanged.",
    },
  ];

  if (metadataFocusPath) {
    const focusedNode = getSearchFilterNodeAtPath(draftQuery.filter, metadataFocusPath);
    if (!focusedNode) {
      return entries;
    }
  }

  return entries;
}

function getStructuredDraftAnchorKind(
  anchor: SearchStructuredDraftAnchor,
): SearchStructuredDraftEntryKind {
  if (anchor.kind === "addQueryPart") {
    return "queryInsertionSlot";
  }

  if (anchor.kind === "queryTreeRoot") {
    return "queryTreeRoot";
  }

  return "queryNode";
}

export function getStructuredDraftSelectionIndex(
  anchor: SearchStructuredDraftAnchor,
  entries: SearchStructuredDraftEntry[],
): number {
  const preferredKind = getStructuredDraftAnchorKind(anchor);

  if (anchor.kind === "queryNode") {
    const matchIndex = entries.findIndex(
      (entry) => entry.kind === "queryNode" && JSON.stringify(entry.treePath ?? []) === JSON.stringify(anchor.path),
    );
    return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : 0, entries.length);
  }

  if (anchor.kind === "addQueryPart") {
    const matchIndex = entries.findIndex(
      (entry) => entry.kind === "queryInsertionSlot" && (entry.insertionPath?.length ?? 0) === 0,
    );
    return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : 0, entries.length);
  }

  if (anchor.kind === "queryTreeRoot") {
    const matchIndex = entries.findIndex((entry) => entry.kind === "queryTreeRoot");
    return clampStructuredDraftSelection(matchIndex >= 0 ? matchIndex : 0, entries.length);
  }

  const entryIndex = entries.findIndex((entry) => entry.kind === preferredKind);
  return clampStructuredDraftSelection(entryIndex >= 0 ? entryIndex : 0, entries.length);
}
