import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import { formatMetadataFieldLabel } from "../../../domain/presentation-vocabulary.js";
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
  createStructuredDraftNodeResumeTarget,
  getStructuredDraftResumeFocusPath,
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

type ActiveGroupFieldMember = {
  field: string;
  fieldLabel: string;
  operator: "include" | "exclude";
  valueLabel: string;
  path: number[];
};

type ActiveGroupFieldBucket = {
  field: string;
  fieldLabel: string;
  operator: "include" | "exclude";
  values: string[];
  memberPaths: number[][];
  fieldMemberPaths: number[][];
  firstIndex: number;
};

type ActiveGroupChildRenderItem =
  | { kind: "bucket"; bucket: ActiveGroupFieldBucket }
  | { kind: "node"; child: SearchFilterNode; childIndex: number };

function compareActiveGroupFieldBuckets(left: ActiveGroupFieldBucket, right: ActiveGroupFieldBucket): number {
  const fieldCompare = left.fieldLabel.localeCompare(right.fieldLabel);
  if (fieldCompare !== 0) {
    return fieldCompare;
  }

  if (left.operator !== right.operator) {
    return left.operator === "include" ? -1 : 1;
  }

  return left.firstIndex - right.firstIndex;
}

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

function formatStructuredDraftBucketLabel(bucket: ActiveGroupFieldBucket): string {
  if (bucket.operator === "exclude") {
    return `${bucket.fieldLabel}: ${bucket.values.map((value) => `!${value}`).join(", ")}`;
  }

  return `${bucket.fieldLabel}: Include ${bucket.values.join(", ")}`;
}

function collectTopLevelSelectionMembers(
  node: SearchFilterNode,
  path: number[],
  operator: "include" | "exclude",
): ActiveGroupFieldMember[] | null {
  if (node.kind === "metadataPredicate" && "value" in node.predicate) {
    return [
      {
        field: node.predicate.field,
        fieldLabel: formatMetadataFieldLabel(node.predicate.field),
        operator,
        valueLabel: String(node.predicate.value),
        path,
      },
    ];
  }

  if (node.kind === "rarity" && node.match.kind === "eq") {
    return [
      {
        field: "rarity",
        fieldLabel: "Rarity",
        operator,
        valueLabel: node.match.value,
        path,
      },
    ];
  }

  if (node.kind === "actionCost" && node.match.kind === "eq") {
    return [
      {
        field: "actionCost",
        fieldLabel: "Action Cost",
        operator,
        valueLabel: String(node.match.value),
        path,
      },
    ];
  }

  if (node.kind !== "anyOf") {
    return null;
  }

  const members = node.children.flatMap((child) => collectTopLevelSelectionMembers(child, path, operator) ?? []);
  if (members.length !== node.children.length) {
    return null;
  }

  const field = members[0]?.field;
  return field && members.every((member) => member.field === field) ? members : null;
}

function collectActiveGroupFieldMembers(
  node: SearchFilterNode,
  path: number[],
  groupedFieldValues: ReadonlySet<string>,
): ActiveGroupFieldMember[] | null {
  if (node.kind === "metadataPredicate" && groupedFieldValues.has(node.predicate.field)) {
    return [
      {
        field: node.predicate.field,
        fieldLabel: formatMetadataFieldLabel(node.predicate.field),
        operator: "include",
        valueLabel: String(node.predicate.value),
        path,
      },
    ];
  }

  const topLevelMembers = collectTopLevelSelectionMembers(node, path, "include");
  if (topLevelMembers) {
    return topLevelMembers;
  }

  if (node.kind !== "not") {
    return null;
  }

  if (node.child.kind === "metadataPredicate" && groupedFieldValues.has(node.child.predicate.field)) {
    return [
      {
        field: node.child.predicate.field,
        fieldLabel: formatMetadataFieldLabel(node.child.predicate.field),
        operator: "exclude",
        valueLabel: String(node.child.predicate.value),
        path,
      },
    ];
  }

  return collectTopLevelSelectionMembers(node.child, path, "exclude");
}

function buildActiveGroupFieldBuckets(
  children: SearchFilterNode[],
  path: number[],
  groupedFieldValues: ReadonlySet<string>,
): { buckets: ActiveGroupFieldBucket[]; groupedChildIndexes: Set<number> } {
  const bucketsByKey = new Map<string, ActiveGroupFieldBucket>();
  const groupedChildIndexes = new Set<number>();
  const fieldMemberPathsByField = new Map<string, number[][]>();

  children.forEach((child, childIndex) => {
    const childPath = [...path, childIndex];
    const members = collectActiveGroupFieldMembers(child, childPath, groupedFieldValues);
    if (!members || members.length === 0) {
      return;
    }

    groupedChildIndexes.add(childIndex);
    for (const member of members) {
      const fieldMemberPaths = fieldMemberPathsByField.get(member.field) ?? [];
      fieldMemberPaths.push(member.path);
      fieldMemberPathsByField.set(member.field, fieldMemberPaths);
      const bucketKey = `${member.field}\u0000${member.operator}`;
      const bucket = bucketsByKey.get(bucketKey);
      if (bucket) {
        bucket.values.push(member.valueLabel);
        bucket.memberPaths.push(member.path);
        continue;
      }
      bucketsByKey.set(bucketKey, {
        field: member.field,
        fieldLabel: member.fieldLabel,
        operator: member.operator,
        values: [member.valueLabel],
        memberPaths: [member.path],
        fieldMemberPaths: [],
        firstIndex: childIndex,
      });
    }
  });

  const buckets = [...bucketsByKey.values()]
    .map((bucket) => ({
      ...bucket,
      values: [...bucket.values].sort((left, right) => left.localeCompare(right)),
    }))
    .sort(compareActiveGroupFieldBuckets);

  for (const bucket of buckets) {
    bucket.fieldMemberPaths = fieldMemberPathsByField.get(bucket.field) ?? [...bucket.memberPaths];
  }

  return { buckets, groupedChildIndexes };
}

function resolveActiveGroupPath(filter: SearchFilterNode | undefined, focusPath: number[] | null): number[] | null {
  if (!focusPath) {
    return null;
  }

  for (let depth = focusPath.length; depth >= 0; depth -= 1) {
    const candidatePath = focusPath.slice(0, depth);
    const candidateNode = getSearchFilterNodeAtPath(filter, candidatePath);
    if (candidateNode && isSearchFilterBooleanGroup(candidateNode)) {
      return candidatePath;
    }
  }

  return null;
}

function pathsEqual(left: number[] | undefined, right: number[]): boolean {
  return Boolean(left) && JSON.stringify(left) === JSON.stringify(right);
}

function pathContains(descendant: number[], ancestor: number[] | undefined): boolean {
  if (!ancestor || ancestor.length > descendant.length) {
    return false;
  }

  return ancestor.every((segment, index) => descendant[index] === segment);
}

function buildFilterTreeEntries(
  node: SearchFilterNode | undefined,
  options: {
    category: ReturnType<typeof getSearchQueryCategory>;
    activeGroupPath: number[] | null;
    groupedFieldValues: ReadonlySet<string>;
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
    const useActiveGroupProjection =
      current.kind === "allOf" && options.activeGroupPath !== null && pathsEqual(path, options.activeGroupPath);
    const { buckets, groupedChildIndexes } = useActiveGroupProjection
      ? buildActiveGroupFieldBuckets(children, path, options.groupedFieldValues)
      : { buckets: [], groupedChildIndexes: new Set<number>() };

    const childRenderItems: ActiveGroupChildRenderItem[] = [];
    children.forEach((child, childIndex) => {
      if (groupedChildIndexes.has(childIndex)) {
        return;
      }
      childRenderItems.push({ kind: "node", child, childIndex });
    });
    for (const bucket of buckets) {
      childRenderItems.push({ kind: "bucket", bucket });
    }

    childRenderItems.forEach((item, renderIndex) => {
      const childIsLast = renderIndex === childRenderItems.length - 1 && !showInsertionSlot;
      if (item.kind === "bucket") {
        entries.push({
          kind: "queryFieldBucket",
          key: `queryFieldBucket:${path.join(".")}:${item.bucket.field}:${item.bucket.operator}`,
          label: formatStructuredDraftBucketLabel(item.bucket),
          description: `Edit the current-group ${item.bucket.fieldLabel.toLowerCase()} selections through the shared explorer.`,
          groupPath: path,
          field: item.bucket.field,
          fieldOperator: item.bucket.operator,
          memberPaths: item.bucket.memberPaths,
          fieldMemberPaths: item.bucket.fieldMemberPaths,
          indent: depth + 1,
          menuLabel: `${formatTreePrefix(childAncestorContinuations, childIsLast)}${formatStructuredDraftBucketLabel(item.bucket)}`,
        });
        return;
      }

      appendNode(item.child, [...path, item.childIndex], depth + 1, childAncestorContinuations, childIsLast);
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
  const useRootActiveGroupProjection =
    options.activeGroupPath !== null &&
    options.activeGroupPath.length === 0 &&
    node !== undefined &&
    node.kind === "allOf";
  if (useRootActiveGroupProjection) {
    const { buckets, groupedChildIndexes } = buildActiveGroupFieldBuckets(
      node.children,
      [],
      options.groupedFieldValues,
    );
    const rootRenderItems: ActiveGroupChildRenderItem[] = [];
    node.children.forEach((child, childIndex) => {
      if (groupedChildIndexes.has(childIndex)) {
        return;
      }
      rootRenderItems.push({ kind: "node", child, childIndex });
    });
    for (const bucket of buckets) {
      rootRenderItems.push({ kind: "bucket", bucket });
    }

    rootRenderItems.forEach((item, renderIndex) => {
      const isLast = renderIndex === rootRenderItems.length - 1 && !showRootInsertionSlot;
      if (item.kind === "bucket") {
        entries.push({
          kind: "queryFieldBucket",
          key: `queryFieldBucket:root:${item.bucket.field}:${item.bucket.operator}`,
          label: formatStructuredDraftBucketLabel(item.bucket),
          description: `Edit the current-group ${item.bucket.fieldLabel.toLowerCase()} selections through the shared explorer.`,
          groupPath: [],
          field: item.bucket.field,
          fieldOperator: item.bucket.operator,
          memberPaths: item.bucket.memberPaths,
          fieldMemberPaths: item.bucket.fieldMemberPaths,
          indent: 1,
          menuLabel: `${formatTreePrefix([], isLast)}${formatStructuredDraftBucketLabel(item.bucket)}`,
        });
        return;
      }

      appendNode(item.child, [item.childIndex], 1, [], isLast);
    });
  } else {
    displayNodes.forEach((displayNode, displayIndex) => {
      const isLast = displayIndex === displayNodes.length - 1 && !showRootInsertionSlot;
      appendNode(displayNode.node, displayNode.path, 1, [], isLast);
    });
  }

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
  const focusPath = getStructuredDraftResumeFocusPath(resumeTarget);
  const activeGroupPath =
    resumeTarget?.kind === "group" ? resumeTarget.groupPath : resolveActiveGroupPath(draftQuery.filter, focusPath);
  const entries: SearchStructuredDraftEntry[] = [
    ...buildFilterTreeEntries(draftQuery.filter, {
      category: draftCategory,
      activeGroupPath,
      groupedFieldValues: options.groupedFieldValues ?? new Set<string>(),
      packLabelResolver: options.packLabelResolver,
      moveSourcePath: options.moveSourcePath ?? null,
      rootOperator: getSearchQueryRootOperator(draftQuery),
    }),
  ];

  if (focusPath && resumeTarget?.kind === "node") {
    const focusedNode = getSearchFilterNodeAtPath(draftQuery.filter, focusPath);
    if (!focusedNode) {
      return entries;
    }
  }

  return entries;
}

export function findStructuredDraftGroupedFieldBucketForPath(
  draftQuery: Pf2eTerminalSearchQuery,
  focusPath: number[],
  groupedFieldValues: ReadonlySet<string>,
): SearchStructuredDraftEntry | null {
  for (let depth = focusPath.length; depth >= 0; depth -= 1) {
    const scopedFocusPath = focusPath.slice(0, depth);
    const bucket =
      buildStructuredDraftEntries(draftQuery, createStructuredDraftNodeResumeTarget(scopedFocusPath), {
        groupedFieldValues,
      }).find(
        (entry) =>
          entry.kind === "queryFieldBucket" &&
          [...(entry.memberPaths ?? []), ...(entry.fieldMemberPaths ?? [])].some((path) =>
            pathContains(focusPath, path),
          ),
      ) ?? null;
    if (bucket) {
      return bucket;
    }
  }

  return null;
}
