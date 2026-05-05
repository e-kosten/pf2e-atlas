import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  formatSearchFilterNodePresentationAlias,
  getSearchFilterNodeAtPath,
  getSearchFilterNodeChildren,
  isSearchFilterBooleanGroup,
  isValidSearchFilterMoveTargetGroupPath,
} from "../../search/query-core.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { getSearchQueryCategory, getSearchQueryRootOperator } from "../../search/query-state.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import { projectSearchQueryFilter, type Pf2eTerminalSearchQueryBase } from "../../search/query-projection.js";
import {
  canonicalizeStructuredDraftResumeTarget,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";

export function buildStructuredDraftQuery(state: {
  baseQuery: Pf2eTerminalSearchQueryBase;
  draftFilter: Pf2eTerminalSearchQuery["filter"];
}): Pf2eTerminalSearchQuery {
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
    activeGroupPath: number[] | null;
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
      description:
        "The dedicated filter builder always presents a visible root boolean group for top-level query clauses.",
      treePath: options.activeGroupPath ?? [],
      menuLabel: formatSearchFilterNodePresentationAlias(
        options.rootOperator === "anyOf" ? { kind: "anyOf", children: [] } : { kind: "allOf", children: [] },
        { style: "tree" },
      ),
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
    isLast: boolean,
  ): void => {
    const isGroup =
      isSearchFilterBooleanGroup(current) || (current.kind === "not" && isSearchFilterBooleanGroup(current.child));
    const menuLabel = `${formatTreePrefix(ancestorContinuations, isLast)}${formatSearchFilterNodePresentationAlias(
      current,
      {
        category: options.category,
        packLabelResolver: options.packLabelResolver,
        style: "tree",
      },
    )}`;

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
    const childAncestorContinuations = [...ancestorContinuations, !isLast];
    const showInsertionSlot =
      isSearchFilterBooleanGroup(current) &&
      (moveSourcePath === null || isValidSearchFilterMoveTargetGroupPath(node, moveSourcePath, path));
    children.forEach((child, childIndex) => {
      const childIsLast = childIndex === children.length - 1 && !showInsertionSlot;
      appendNode(child, [...path, childIndex], depth + 1, childAncestorContinuations, childIsLast);
    });

    if (showInsertionSlot) {
      entries.push({
        kind: "queryInsertionSlot",
        key: `querySlot:${path.join(".")}`,
        label: moveSourcePath ? "[move here]" : "[+ add here]",
        description: moveSourcePath
          ? "Confirm the move and append the anchored node at the bottom of this group."
          : "Add a new clause at the bottom of this group. Use the action rail for group or NOT variants.",
        insertionPath: path,
        indent: depth + 1,
        menuLabel: `${formatTreePrefix(childAncestorContinuations, true)}${moveSourcePath ? "[move here]" : "[+ add here]"}`,
      });
    }
  };

  const showRootInsertionSlot =
    moveSourcePath === null || isValidSearchFilterMoveTargetGroupPath(node, moveSourcePath, []);
  displayNodes.forEach((displayNode, displayIndex) => {
    const isLast = displayIndex === displayNodes.length - 1 && !showRootInsertionSlot;
    appendNode(displayNode.node, displayNode.path, 1, [], isLast);
  });

  if (showRootInsertionSlot) {
    entries.push({
      kind: "queryInsertionSlot",
      key: "querySlot:root",
      label: moveSourcePath ? "[move here]" : "[+ add here]",
      description: moveSourcePath
        ? `Confirm the move and append the anchored node at the root ${options.rootOperator} group.`
        : "Add a new top-level clause. Use the action rail for group or NOT variants.",
      insertionPath: [],
      indent: 1,
      menuLabel: `${formatTreePrefix([], true)}${moveSourcePath ? "[move here]" : "[+ add here]"}`,
    });
  }

  return entries;
}

export function buildStructuredDraftEntries(
  draftQuery: Pf2eTerminalSearchQuery,
  resumeTarget: StructuredDraftResumeTarget | null,
  options: {
    groupedFieldValues?: ReadonlySet<string>;
    packLabelResolver?: (packValue: string) => string | null | undefined;
    moveSourcePath?: number[] | null;
  },
): SearchStructuredDraftEntry[] {
  const draftCategory = getSearchQueryCategory(draftQuery);
  const canonicalResumeTarget = resumeTarget
    ? canonicalizeStructuredDraftResumeTarget(draftQuery.filter, resumeTarget)
    : null;
  const activeGroupPath = canonicalResumeTarget?.kind === "group" ? canonicalResumeTarget.groupPath : null;
  const nodeFocusPath = canonicalResumeTarget?.kind === "node" ? canonicalResumeTarget.path : null;
  const entries: SearchStructuredDraftEntry[] = [
    ...buildFilterTreeEntries(draftQuery.filter, {
      category: draftCategory,
      activeGroupPath,
      packLabelResolver: options.packLabelResolver,
      moveSourcePath: options.moveSourcePath ?? null,
      rootOperator: getSearchQueryRootOperator(draftQuery),
    }),
  ];

  if (nodeFocusPath) {
    const focusedNode = getSearchFilterNodeAtPath(draftQuery.filter, nodeFocusPath);
    if (!focusedNode) {
      return entries;
    }
  }

  return entries;
}
