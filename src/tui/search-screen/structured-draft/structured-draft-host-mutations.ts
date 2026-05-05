import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  appendSearchFilterNodeAtPath,
  appendSearchFilterNodesAtPath,
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  updateSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import { getSearchQueryRootOperator } from "../../search/query-state.js";
import type { Pf2eTerminalSearchQuery } from "../../search/service.js";
import type { StructuredDraftHostMutation } from "./structured-draft-continuation.js";
import { getGroupedFieldChildIndexes } from "./structured-draft-grouped-paths.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftResumeTargetForContainingGroup,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";

export function getFirstGroupedFieldMemberPath(node: SearchFilterNode, path: number[], field: string): number[] | null {
  if (field === "pack") {
    if (node.kind === "pack") {
      return path;
    }
    if (node.kind === "allOf" || node.kind === "anyOf") {
      for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
        const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
        if (childPath) {
          return childPath;
        }
      }
      return null;
    }
    if (node.kind === "not") {
      return getFirstGroupedFieldMemberPath(node.child, [...path, 0], field);
    }
    return null;
  }

  if (field === "rarity" || field === "actionCost") {
    if (node.kind === field && node.match.kind === "eq") {
      return path;
    }
    if (node.kind === "allOf" || node.kind === "anyOf") {
      for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
        const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
        if (childPath) {
          return childPath;
        }
      }
      return null;
    }
    if (node.kind === "not") {
      return getFirstGroupedFieldMemberPath(node.child, [...path, 0], field);
    }
    return null;
  }

  if (node.kind === "metadataPredicate" && node.predicate.field === field) {
    return path;
  }
  if (node.kind === "not") {
    return getFirstGroupedFieldMemberPath(node.child, [...path, 0], field);
  }
  if (node.kind === "allOf" || node.kind === "anyOf") {
    for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
      const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
      if (childPath) {
        return childPath;
      }
    }
  }
  return null;
}

export function getContainingBooleanGroupPath(filter: SearchFilterNode | undefined, path: number[]): number[] {
  for (let depth = path.length - 1; depth >= 0; depth -= 1) {
    const candidatePath = path.slice(0, depth);
    const candidateNode = getSearchFilterNodeAtPath(filter, candidatePath);
    if (candidateNode && isSearchFilterBooleanGroup(candidateNode)) {
      return candidatePath;
    }
  }

  return [];
}

function flattenReplacementNodesForGroup(
  groupKind: "allOf" | "anyOf" | "not",
  replacementNodes: readonly SearchFilterNode[],
): SearchFilterNode[] {
  if (groupKind === "not") {
    return [...replacementNodes];
  }
  if (replacementNodes.length !== 1) {
    return [...replacementNodes];
  }

  const [node] = replacementNodes;
  if (!node || node.kind !== groupKind) {
    return [...replacementNodes];
  }

  return [...node.children];
}

export function applyGroupedFieldReplacementToQuery(
  query: Pf2eTerminalSearchQuery,
  groupPath: number[],
  field: string,
  fieldMemberPaths: readonly number[][],
  replacementNodes: readonly SearchFilterNode[],
): { nextQuery: Pf2eTerminalSearchQuery; nextFocusPath: number[] | null } {
  const groupNode =
    groupPath.length === 0 ? query.filter : (getSearchFilterNodeAtPath(query.filter, groupPath) ?? undefined);
  const flattenedRootReplacementNodes = flattenReplacementNodesForGroup(
    getSearchQueryRootOperator(query),
    replacementNodes,
  );
  if (groupPath.length === 0 && replacementNodes.length > 0 && (!groupNode || !isSearchFilterBooleanGroup(groupNode))) {
    const nextFilter =
      flattenedRootReplacementNodes.length > 1
        ? appendSearchFilterNodesAtPath(
            query.filter,
            [],
            flattenedRootReplacementNodes,
            getSearchQueryRootOperator(query),
          )
        : appendSearchFilterNodeAtPath(
            query.filter,
            [],
            flattenedRootReplacementNodes[0]!,
            getSearchQueryRootOperator(query),
          );
    const nextFocusPath = nextFilter ? getFirstGroupedFieldMemberPath(nextFilter, [], field) : null;
    return {
      nextQuery: {
        ...query,
        filter: nextFilter,
      },
      nextFocusPath,
    };
  }

  if (!groupNode || !isSearchFilterBooleanGroup(groupNode)) {
    return { nextQuery: query, nextFocusPath: groupPath.length > 0 ? groupPath : null };
  }

  const fieldChildIndexes = new Set<number>(getGroupedFieldChildIndexes(groupPath, fieldMemberPaths));
  const flattenedReplacementNodes = flattenReplacementNodesForGroup(groupNode.kind, replacementNodes);
  const firstReplacementIndex = groupNode.children.findIndex((_, childIndex) => fieldChildIndexes.has(childIndex));
  const nextChildren =
    firstReplacementIndex >= 0
      ? groupNode.children.flatMap((child, childIndex) =>
          fieldChildIndexes.has(childIndex)
            ? childIndex === firstReplacementIndex
              ? flattenedReplacementNodes
              : []
            : [child],
        )
      : [...groupNode.children, ...flattenedReplacementNodes];

  const nextGroupNode =
    nextChildren.length === 0
      ? undefined
      : nextChildren.length === 1
        ? nextChildren[0]
        : {
            kind: groupNode.kind,
            children: nextChildren,
          };
  const nextFilter =
    groupPath.length === 0 ? nextGroupNode : updateSearchFilterNodeAtPath(query.filter, groupPath, () => nextGroupNode);
  const nextGroupNodeInQuery =
    groupPath.length === 0 ? (nextGroupNode ?? null) : getSearchFilterNodeAtPath(nextFilter, groupPath);
  const nextFocusPath = nextGroupNodeInQuery
    ? (getFirstGroupedFieldMemberPath(nextGroupNodeInQuery, groupPath, field) ??
      (groupPath.length > 0 ? groupPath : null))
    : groupPath.length > 0
      ? groupPath
      : null;

  return {
    nextQuery: {
      ...query,
      filter: nextFilter,
    },
    nextFocusPath,
  };
}

export type StructuredDraftHostMutationTarget =
  | { kind: "appendNodes"; groupPath: number[]; flattenMatchingBooleanGroup?: boolean }
  | { kind: "replaceNode"; path: number[]; splitAllOfReplacementIntoContainingGroup?: boolean }
  | {
      kind: "replaceGroupedField";
      groupPath: number[];
      field: string;
      fieldMemberPaths: readonly number[][];
      replacementNodes: readonly SearchFilterNode[];
      replaceRoot?: boolean;
    };

export type StructuredDraftHostMutationApplication = {
  nextQuery: Pf2eTerminalSearchQuery;
  resumeTarget: StructuredDraftResumeTarget | null;
};

export function applyStructuredDraftHostMutationToQuery(
  query: Pf2eTerminalSearchQuery,
  mutation: StructuredDraftHostMutation,
  target: StructuredDraftHostMutationTarget,
): StructuredDraftHostMutationApplication | null {
  if (mutation.kind !== target.kind) {
    return null;
  }

  if (mutation.kind === "appendNodes" && target.kind === "appendNodes") {
    const targetNode =
      target.groupPath.length === 0 ? query.filter : getSearchFilterNodeAtPath(query.filter, target.groupPath);
    const targetGroupKind =
      targetNode && isSearchFilterBooleanGroup(targetNode)
        ? targetNode.kind
        : target.groupPath.length === 0
          ? getSearchQueryRootOperator(query)
          : null;
    const nodes =
      target.flattenMatchingBooleanGroup && targetGroupKind
        ? mutation.nodes.flatMap((node) => flattenReplacementNodesForGroup(targetGroupKind, [node]))
        : mutation.nodes;
    const nextFilter = appendSearchFilterNodesAtPath(
      query.filter,
      target.groupPath,
      nodes,
      getSearchQueryRootOperator(query),
    );
    return {
      nextQuery: {
        ...query,
        filter: nextFilter,
      },
      resumeTarget: createStructuredDraftGroupResumeTarget(target.groupPath),
    };
  }

  if (mutation.kind === "replaceNode" && target.kind === "replaceNode") {
    if (
      target.splitAllOfReplacementIntoContainingGroup &&
      mutation.node?.kind === "allOf" &&
      target.path.length > 0 &&
      mutation.node.children.length > 1
    ) {
      const [firstNode, ...additionalNodes] = mutation.node.children;
      const groupPath = target.path.slice(0, -1);
      const replacedFilter = updateSearchFilterNodeAtPath(query.filter, target.path, () => firstNode);
      const nextFilter = appendSearchFilterNodesAtPath(
        replacedFilter,
        groupPath,
        additionalNodes,
        getSearchQueryRootOperator(query),
      );
      return {
        nextQuery: {
          ...query,
          filter: nextFilter,
        },
        resumeTarget: createStructuredDraftGroupResumeTarget(groupPath),
      };
    }
    const nextFilter = updateSearchFilterNodeAtPath(query.filter, target.path, () => mutation.node ?? undefined);
    const nextQuery = {
      ...query,
      filter: nextFilter,
    };
    return {
      nextQuery,
      resumeTarget: mutation.node
        ? createStructuredDraftResumeTargetForContainingGroup(nextFilter, target.path)
        : createStructuredDraftGroupResumeTarget(getContainingBooleanGroupPath(query.filter, target.path)),
    };
  }

  if (mutation.kind === "replaceGroupedField" && target.kind === "replaceGroupedField") {
    if (target.replaceRoot) {
      const nextFilter =
        target.replacementNodes.length === 0
          ? undefined
          : target.replacementNodes.length === 1
            ? target.replacementNodes[0]
            : ({
                kind: getSearchQueryRootOperator(query),
                children: [...target.replacementNodes],
              } satisfies SearchFilterNode);
      return {
        nextQuery: {
          ...query,
          filter: nextFilter,
        },
        resumeTarget:
          nextFilter && target.replacementNodes.length > 0
            ? createStructuredDraftResumeTargetForContainingGroup(
                nextFilter,
                getFirstGroupedFieldMemberPath(nextFilter, [], target.field) ?? [],
              )
            : createStructuredDraftGroupResumeTarget([]),
      };
    }
    const { nextFocusPath, nextQuery } = applyGroupedFieldReplacementToQuery(
      query,
      target.groupPath,
      target.field,
      target.fieldMemberPaths,
      target.replacementNodes,
    );
    return {
      nextQuery,
      resumeTarget: nextFocusPath
        ? createStructuredDraftResumeTargetForContainingGroup(nextQuery.filter, nextFocusPath)
        : createStructuredDraftGroupResumeTarget(target.groupPath),
    };
  }

  return null;
}
