import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../../domain/categories.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import {
  type SearchNumericMatch,
  type SearchFilterNode,
  type SearchScopeSubcategoryMatch,
} from "../../../domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type { SearchCategory } from "../../../domain/search-types.js";
import {
  appendSearchFilterNodeAtPath,
  appendSearchFilterNodesAtPath,
  canLiftSearchFilterNodeAtPath,
  canUnwrapSearchFilterNodeAtPath,
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  liftSearchFilterNodeAtPath,
  moveSearchFilterNodeToGroupPath,
  reshapeSearchFilterBooleanGroupAtPath,
  toggleSearchFilterRootGroupOperator,
  unwrapSearchFilterNodeAtPath,
  updateSearchFilterNodeAtPath,
  wrapSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import { metadataFilterNodeToCanonicalFilter, canonicalFilterToMetadataNode } from "../../search/query-parts.js";
import {
  buildSearchFilterPackSelectionNode,
  getSearchQueryCategory,
  getSearchQueryMetadataTree,
  getSearchQueryPackSelection,
  getSearchQueryRootOperator,
  getSearchQuerySubcategory,
  setSearchQueryPackSelection,
  setSearchQueryMetadataTree,
} from "../../search/query-state.js";
import type {
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import {
  buildStructuredDraftEntries,
  findStructuredDraftGroupedFieldBucketForPath,
} from "./structured-draft-support.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftNodeResumeTarget,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";
import { promptLevelRangeDraft, promptNumericScalarClause } from "../../filter-explorer/scalar-editor.js";
import type { FilterExplorerComposeTarget, FilterExplorerSelectTargetOutcome } from "../../filter-explorer/types.js";
import type { DerivedTagTerminalActionTargetOption } from "../../action-target.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";
import {
  buildSearchFilterExplorerComposeDraft,
  buildSearchFilterExplorerFieldState,
  type SearchFilterExplorerFieldState,
} from "../filter-explorer-field-state.js";
import {
  runStructuredDraftExplorerContinuation,
  type StructuredDraftExplorerContinuationChange,
} from "./structured-draft-continuation.js";

type ClauseKind = "field" | "metric" | "metricCompare" | "pack" | "scope" | "level" | "price" | "rarity" | "actionCost";
type MetricFieldFamily = "actorMetric" | "itemMetric";
type MetricCompareOperator = Extract<Extract<SearchFilterNode, { kind: "metricCompare" }>["op"], string>;
type MetricKeySelection = { value: string; discoveryMode: SearchFilterDiscoveryMode };
const CLAUSE_BACK = Symbol("search-structured-query-clause-back");
type ClausePromptBackResult = typeof CLAUSE_BACK;
type PromptStepResult<T> = T | ClausePromptBackResult | undefined;
type SearchFilterNodeEditorResult = SearchFilterNode | SearchFilterNode[] | ClausePromptBackResult | null | undefined;
type ClausePromptResult = ClauseKind | ClausePromptBackResult | null;
type ClauseApplyResult = "applied" | "back" | "cancelled";
type StructuredDraftEntryActionId =
  | "addClause"
  | "addAndGroup"
  | "addOrGroup"
  | "addNotGroup"
  | "moveHere"
  | "toggleRoot"
  | "edit"
  | "wrapNot"
  | "wrapAnd"
  | "wrapOr"
  | "move"
  | "lift"
  | "remove"
  | "unwrap"
  | "toggleGroup";

function formatFriendlyGroupLabel(kind: "allOf" | "anyOf" | "not"): string {
  switch (kind) {
    case "allOf":
      return "All of";
    case "anyOf":
      return "Any of";
    case "not":
      return "Exclude";
  }
}

function groupPathsEqual(left: number[] | undefined, right: number[]): boolean {
  return Boolean(left) && JSON.stringify(left) === JSON.stringify(right);
}

function buildExplorerOnlyFieldOption(
  field: Pf2eTerminalQueryFieldOption["value"],
  label: string,
  description: string,
  fieldType: Pf2eTerminalQueryFieldOption["fieldType"],
): Pf2eTerminalQueryFieldOption {
  return {
    value: field,
    label,
    description,
    fieldType,
    editor: "sharedExplorer",
  };
}

function searchFilterExplorerFieldStatesEqual(
  left: SearchFilterExplorerFieldState | undefined,
  right: SearchFilterExplorerFieldState | undefined,
): boolean {
  if (!left || !right) {
    return left === right;
  }

  return (
    JSON.stringify(buildSearchFilterExplorerComposeDraft(left)) ===
    JSON.stringify(buildSearchFilterExplorerComposeDraft(right))
  );
}

function getFirstGroupedFieldMemberPath(node: SearchFilterNode, path: number[], field: string): number[] | null {
  if (field === "rarity" || field === "actionCost") {
    if (node.kind === field && node.match.kind === "eq") {
      return path;
    }
    if (node.kind === "anyOf") {
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
  if (node.kind === "anyOf") {
    for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
      const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
      if (childPath) {
        return childPath;
      }
    }
  }
  return null;
}

function getContainingBooleanGroupPath(filter: SearchFilterNode | undefined, path: number[]): number[] {
  for (let depth = path.length - 1; depth >= 0; depth -= 1) {
    const candidatePath = path.slice(0, depth);
    const candidateNode = getSearchFilterNodeAtPath(filter, candidatePath);
    if (candidateNode && isSearchFilterBooleanGroup(candidateNode)) {
      return candidatePath;
    }
  }

  return [];
}

function toSearchFilterNodeEditorResult(
  result: Pf2eTerminalFilterExplorerInsertionResult,
): SearchFilterNode | SearchFilterNode[] | null {
  if (result.kind === "insert") {
    return result.nodes
      .map((node) => metadataFilterNodeToCanonicalFilter(node))
      .filter((node): node is SearchFilterNode => Boolean(node));
  }

  return metadataFilterNodeToCanonicalFilter(result.node) ?? null;
}

function buildGroupedFieldSeedDiscreteClauses(
  node: SearchFilterNode | undefined,
  field: string | undefined,
  operator: "include" | "exclude" = "include",
): Pf2eTerminalFilterExplorerDraft["discreteClauses"] {
  if (!node || !field) {
    return [];
  }

  if (field === "rarity" || field === "actionCost") {
    if (node.kind === field && node.match.kind === "eq") {
      return [{ field, value: String(node.match.value), operator }];
    }
    if (node.kind === "anyOf") {
      return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
    }
    if (node.kind === "not") {
      return buildGroupedFieldSeedDiscreteClauses(node.child, field, "exclude");
    }
    return [];
  }

  if (node.kind === "metadataPredicate" && node.predicate.field === field && "value" in node.predicate) {
    return [{ field, value: String(node.predicate.value), operator }];
  }
  if (node.kind === "anyOf") {
    return node.children.flatMap((child) => buildGroupedFieldSeedDiscreteClauses(child, field, operator));
  }
  if (node.kind === "not") {
    return buildGroupedFieldSeedDiscreteClauses(node.child, field, "exclude");
  }

  return [];
}

function buildGroupedFieldSeedGroupNode(
  groupNode: Extract<SearchFilterNode, { kind: "allOf" } | { kind: "anyOf" }>,
  removedChildIndexes: ReadonlySet<number>,
): SearchFilterNode | undefined {
  const remainingChildren = groupNode.children.filter((_child, childIndex) => !removedChildIndexes.has(childIndex));
  if (remainingChildren.length === 0) {
    return undefined;
  }
  if (remainingChildren.length === 1) {
    return remainingChildren[0];
  }
  return {
    kind: groupNode.kind,
    children: remainingChildren,
  };
}

function getGroupedFieldChildIndexes(groupPath: number[], fieldMemberPaths: readonly number[][]): number[] {
  const childIndexes = new Set<number>();

  for (const memberPath of fieldMemberPaths) {
    if (memberPath.length <= groupPath.length) {
      continue;
    }
    const isInGroup = groupPath.every((segment, index) => memberPath[index] === segment);
    if (!isInGroup) {
      continue;
    }

    const childIndex = memberPath[groupPath.length];
    if (childIndex !== undefined) {
      childIndexes.add(childIndex);
    }
  }

  return [...childIndexes].sort((left, right) => left - right);
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

function buildGroupedFieldReplacementNodes(
  searchUser: SearchWorkspaceUser["search"],
  query: Pf2eTerminalSearchQuery,
  fieldState: SearchFilterExplorerFieldState,
  fieldOption: Pf2eTerminalQueryFieldOption,
  options?: { preserveFlatSetClauses?: boolean },
): SearchFilterNode[] {
  const field = fieldOption.value;
  const selection = fieldState.discreteSelections[field] ?? { include: [], exclude: [] };
  const draft = buildSearchFilterExplorerComposeDraft(fieldState);
  const discreteClauses = draft.discreteClauses.filter((clause) => clause.field === field);

  if (field === "rarity") {
    return discreteClauses.map((clause) =>
      clause.operator === "include"
        ? ({ kind: "rarity", match: { kind: "eq", value: clause.value } } satisfies SearchFilterNode)
        : ({
            kind: "not",
            child: { kind: "rarity", match: { kind: "eq", value: clause.value } },
          } satisfies SearchFilterNode),
    );
  }

  if (field === "actionCost") {
    const replacementNodes: SearchFilterNode[] = [];
    for (const clause of discreteClauses) {
      const numericValue = Number.parseInt(clause.value, 10);
      if (!Number.isFinite(numericValue)) {
        continue;
      }
      replacementNodes.push(
        clause.operator === "include"
          ? ({ kind: "actionCost", match: { kind: "eq", value: numericValue } } satisfies SearchFilterNode)
          : ({
              kind: "not",
              child: { kind: "actionCost", match: { kind: "eq", value: numericValue } },
            } satisfies SearchFilterNode),
      );
    }
    return replacementNodes;
  }

  if (fieldOption.fieldType === "set" && options?.preserveFlatSetClauses) {
    return discreteClauses.flatMap((clause) => {
      const selection =
        clause.operator === "include"
          ? { include: [clause.value], exclude: [] }
          : { include: [], exclude: [clause.value] };
      const replacementNode = metadataFilterNodeToCanonicalFilter(
        getSearchQueryMetadataTree(
          searchUser.applyDiscoverableQueryFieldSelections(
            setSearchQueryMetadataTree(query, null),
            { [field]: selection },
            [field],
          ),
        ),
      );
      return replacementNode ? [replacementNode] : [];
    });
  }

  const replacementNode = metadataFilterNodeToCanonicalFilter(
    getSearchQueryMetadataTree(
      searchUser.applyDiscoverableQueryFieldSelections(
        setSearchQueryMetadataTree(query, null),
        { [field]: selection },
        [field],
      ),
    ),
  );
  return replacementNode ? [replacementNode] : [];
}

export function buildGroupedFieldSeedState(
  query: Pf2eTerminalSearchQuery,
  groupPath: number[],
  options?: {
    field?: string;
    fieldMemberPaths?: number[][];
  },
): {
  seedGroupPath: number[];
  seedQuery: Pf2eTerminalSearchQuery;
  initialFieldState: SearchFilterExplorerFieldState;
  preservedMetadata: MetadataFilterNode | null;
} {
  const groupNode =
    groupPath.length === 0 ? query.filter : (getSearchFilterNodeAtPath(query.filter, groupPath) ?? undefined);
  if (!groupNode) {
    return {
      seedGroupPath: [],
      seedQuery: query,
      initialFieldState: buildSearchFilterExplorerFieldState({
        discreteClauses: [],
        scalarClauses: {},
      }),
      preservedMetadata: getSearchQueryMetadataTree(query),
    };
  }

  const fieldChildIndexes = new Set<number>(getGroupedFieldChildIndexes(groupPath, options?.fieldMemberPaths ?? []));

  const initialDraft: Pf2eTerminalFilterExplorerDraft = {
    discreteClauses: [...fieldChildIndexes]
      .sort((left, right) => left - right)
      .flatMap((childIndex) =>
        buildGroupedFieldSeedDiscreteClauses(
          isSearchFilterBooleanGroup(groupNode) ? groupNode.children[childIndex] : undefined,
          options?.field,
        ),
      ),
    scalarClauses: {},
  };

  const seedGroupNode =
    isSearchFilterBooleanGroup(groupNode) && fieldChildIndexes.size > 0
      ? buildGroupedFieldSeedGroupNode(groupNode, fieldChildIndexes)
      : groupNode;

  const seedQuery = {
    ...query,
    filter:
      groupPath.length === 0
        ? seedGroupNode
        : updateSearchFilterNodeAtPath(query.filter, groupPath, () => seedGroupNode),
  };
  return {
    seedGroupPath: groupPath,
    seedQuery,
    initialFieldState: buildSearchFilterExplorerFieldState(initialDraft),
    preservedMetadata: getSearchQueryMetadataTree(seedQuery),
  };
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

function buildInsertionActionEntries(
  moveMode: boolean,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  if (moveMode) {
    return [
      {
        id: "moveHere",
        label: "Move Here",
        description: "Append the anchored node into this visible insertion slot.",
      },
    ];
  }

  return [
    { id: "addClause", label: "Add Clause", description: "Insert one canonical filter clause into this group." },
    {
      id: "addAndGroup",
      label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
      description: "Insert a nested group where every child must match, starting with its first child.",
    },
    {
      id: "addOrGroup",
      label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
      description: "Insert a nested group where any child may match, starting with its first child.",
    },
    {
      id: "addNotGroup",
      label: `Add ${formatFriendlyGroupLabel("not")} Group`,
      description: "Insert a nested excluded group with its first child.",
    },
  ];
}

function buildRootActionEntries(
  query: Pf2eTerminalSearchQuery,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  return [
    { id: "addClause", label: "Add Clause", description: "Append a new top-level clause." },
    {
      id: "addAndGroup",
      label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
      description: "Append a nested group where every child must match.",
    },
    {
      id: "addOrGroup",
      label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
      description: "Append a nested group where any child may match.",
    },
    {
      id: "addNotGroup",
      label: `Add ${formatFriendlyGroupLabel("not")} Group`,
      description: "Append a nested excluded group.",
    },
    ...(query.filter
      ? [
          {
            id: "toggleRoot" as const,
            label:
              getSearchQueryRootOperator(query) === "anyOf"
                ? `Change Root To ${formatFriendlyGroupLabel("allOf")}`
                : `Change Root To ${formatFriendlyGroupLabel("anyOf")}`,
            description: "Reshape the visible root group without changing its current children.",
          },
        ]
      : []),
  ];
}

function isMetricFieldOptionValue(value: Pf2eTerminalQueryFieldOption["value"]): boolean {
  return value === "actorMetric" || value === "itemMetric";
}

function getQueryFieldValueForNode(node: SearchFilterNode): Pf2eTerminalQueryFieldOption["value"] | null {
  switch (node.kind) {
    case "metadataPredicate":
      return node.predicate.field;
    case "metric":
      return inferMetricFieldFamily(node.metric);
    case "pack":
    case "scope":
    case "level":
    case "price":
    case "rarity":
    case "actionCost":
    case "linksTo":
    case "linkedFrom":
    case "metricCompare":
    case "anyOf":
    case "allOf":
    case "not":
      return null;
  }
}

function getMetadataFilterNodeFieldValue(
  node: MetadataFilterNode | null,
): Pf2eTerminalQueryFieldOption["value"] | null {
  if (!node || "and" in node || "or" in node || "not" in node) {
    return null;
  }
  if (node.field === "actorMetricCompare" || node.field === "itemMetricCompare") {
    return null;
  }
  return node.field;
}

function inferMetricFieldFamily(
  metric: string,
  category: ReturnType<typeof getSearchQueryCategory> = null,
): MetricFieldFamily {
  const actorValueType = inferActorMetricValueType(metric);
  const itemValueType = inferItemMetricValueType(metric);

  if (actorValueType && !itemValueType) {
    return "actorMetric";
  }
  if (itemValueType && !actorValueType) {
    return "itemMetric";
  }

  return category === "equipment" ? "itemMetric" : "actorMetric";
}

function formatNumericMatch(match: SearchNumericMatch): string {
  switch (match.kind) {
    case "eq":
      return String(match.value);
    case "gt":
      return `>${match.value}`;
    case "gte":
      return `>=${match.value}`;
    case "lt":
      return `<${match.value}`;
    case "lte":
      return `<=${match.value}`;
    case "between":
      return `${match.min}-${match.max}`;
  }
}

function buildPackClauseFieldOption(): Pf2eTerminalQueryFieldOption {
  return buildExplorerOnlyFieldOption(
    "pack",
    "Pack",
    "Browse live packs for the current scope and stage canonical pack clauses.",
    "enumString",
  );
}

export function buildMetricSelectionTargetResolver(
  family: MetricFieldFamily,
  fieldLabel: string,
  options: { numericOnly?: boolean } = {},
): (
  node: import("../../../domain/ontology-types.js").OntologyNode | undefined,
) => FilterExplorerComposeTarget | undefined {
  return (node) => {
    if (node?.kind !== "metric") {
      return undefined;
    }

    const metricKey = node.id.split(":").at(-1);
    if (!metricKey) {
      return undefined;
    }

    const valueType =
      family === "actorMetric" ? inferActorMetricValueType(metricKey) : inferItemMetricValueType(metricKey);
    if (!valueType) {
      return undefined;
    }
    if (options.numericOnly && valueType !== "number") {
      return undefined;
    }

    return {
      kind: "scalar",
      key: `${family}:${metricKey}`,
      fieldLabel,
      subjectLabel: node.label,
      valueType,
      editorLabel: `${fieldLabel} / ${node.label}`,
    };
  };
}

function extractMetricKeyFromSelectTargetOutcome(
  outcome: FilterExplorerSelectTargetOutcome,
  family: MetricFieldFamily,
): string | null {
  const target = outcome.result.target;
  if (!target || target.kind !== "scalar") {
    return null;
  }

  const prefix = `${family}:`;
  return target.key.startsWith(prefix) ? target.key.slice(prefix.length) : null;
}

export function useSearchStructuredDraftMetadataActions({
  appendStructuredDraftMetadataNode: _appendStructuredDraftMetadataNode,
  clearStructuredDraftMoveSource,
  editFieldClause,
  enterStructuredDraftMoveMode,
  getScopedFieldOptions,
  moveSourcePath,
  openFilterExplorer,
  prompts,
  replaceStructuredDraftProjection,
  setStructuredDraftResumeTarget,
  structuredDraftQuery,
  terminal,
  updateStructuredDraftMetadataNode,
  user,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  clearStructuredDraftMoveSource: () => void;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openFilterExplorer: OpenSearchFilterExplorer;
  moveSourcePath: number[] | null;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  user: SearchWorkspaceUser;
}): {
  editStructuredDraftMetadata: (entry: SearchStructuredDraftEntry) => Promise<void>;
  getStructuredDraftEntryActions: (
    entry: SearchStructuredDraftEntry | null | undefined,
  ) => DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[];
  runStructuredDraftEntryAction: (
    entry: SearchStructuredDraftEntry | null | undefined,
    actionId: StructuredDraftEntryActionId,
  ) => Promise<void>;
} {
  const applyNextTree = React.useCallback(
    (nextFilter: SearchFilterNode | undefined) => {
      replaceStructuredDraftProjection((draftQuery) => ({
        ...draftQuery,
        filter: nextFilter,
      }));
    },
    [replaceStructuredDraftProjection],
  );

  const openStructuredDraftExplorerContinuation = React.useCallback(
    async ({
      currentNode,
      fieldOption,
      initialFieldState,
      onHostChange,
      preservedMetadata,
      query,
    }: {
      currentNode: MetadataFilterNode | null;
      fieldOption: Pf2eTerminalQueryFieldOption;
      initialFieldState?: SearchFilterExplorerFieldState;
      onHostChange?: (change: StructuredDraftExplorerContinuationChange) => void;
      preservedMetadata?: MetadataFilterNode | null;
      query: Pf2eTerminalSearchQuery;
    }) =>
      runStructuredDraftExplorerContinuation({
        currentNode,
        fieldOption,
        initialFieldState,
        onHostChange,
        openFilterExplorer,
        preservedMetadata,
        query,
        user,
      }),
    [openFilterExplorer, user],
  );

  const openLiveExplorerFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
    ) => {
      const liveChangeState = { saw: false };
      const applyChange = ({ result }: StructuredDraftExplorerContinuationChange) => {
        if (result.kind !== "replace") {
          return;
        }
        liveChangeState.saw = true;

        updateStructuredDraftMetadataNode(path, () => result.node);
      };
      setStructuredDraftResumeTarget(createStructuredDraftNodeResumeTarget(path));
      const continuation = await openStructuredDraftExplorerContinuation({
        query,
        fieldOption,
        currentNode,
        onHostChange: applyChange,
      });
      if (
        continuation.kind !== "notOpened" &&
        continuation.change &&
        !liveChangeState.saw &&
        continuation.change.result.kind === "replace" &&
        JSON.stringify(continuation.change.result.node) !== JSON.stringify(currentNode)
      ) {
        applyChange(continuation.change);
      }
    },
    [openStructuredDraftExplorerContinuation, setStructuredDraftResumeTarget, updateStructuredDraftMetadataNode],
  );

  const openLiveExplorerGroupedField = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, entry: SearchStructuredDraftEntry) => {
      const groupPath = entry.groupPath ?? [];
      const fieldMemberPaths = entry.fieldMemberPaths ?? entry.memberPaths ?? [];
      const field = entry.field;
      if (!field) {
        return;
      }

      const fieldOption =
        field === "rarity"
          ? buildExplorerOnlyFieldOption(
              "rarity",
              "Rarity",
              "Browse live rarities for the current group and stage canonical rarity clauses.",
              "enumString",
            )
          : field === "actionCost"
            ? buildExplorerOnlyFieldOption(
                "actionCost",
                "Action Cost",
                "Browse live action costs for the current group and stage canonical action-cost clauses.",
                "number",
              )
            : (getScopedFieldOptions(query).find((candidate) => candidate.value === field) ?? null);
      if (!fieldOption || fieldOption.editor !== "sharedExplorer") {
        await terminal.pauseForAnyKey("That grouped row cannot be edited through the shared explorer.");
        return;
      }

      const { seedQuery, initialFieldState, preservedMetadata } = buildGroupedFieldSeedState(query, groupPath, {
        field,
        fieldMemberPaths,
      });
      const liveChangeState = { saw: false };
      const applyChange = ({ fieldState }: StructuredDraftExplorerContinuationChange) => {
        liveChangeState.saw = true;
        const replacementNodes = buildGroupedFieldReplacementNodes(user.search, query, fieldState, fieldOption, {
          preserveFlatSetClauses: fieldMemberPaths.length > 0,
        });
        const { nextQuery, nextFocusPath } = applyGroupedFieldReplacementToQuery(
          query,
          groupPath,
          field,
          fieldMemberPaths,
          replacementNodes,
        );
        replaceStructuredDraftProjection(() => nextQuery, {
          resumeTarget: createStructuredDraftGroupResumeTarget(nextFocusPath ?? groupPath),
        });
      };
      const continuation = await openStructuredDraftExplorerContinuation({
        query: seedQuery,
        fieldOption,
        currentNode: null,
        initialFieldState,
        preservedMetadata,
        onHostChange: applyChange,
      });
      if (
        continuation.kind !== "notOpened" &&
        continuation.change &&
        !liveChangeState.saw &&
        !searchFilterExplorerFieldStatesEqual(continuation.change.fieldState, initialFieldState)
      ) {
        applyChange(continuation.change);
      }
    },
    [
      getScopedFieldOptions,
      openStructuredDraftExplorerContinuation,
      replaceStructuredDraftProjection,
      terminal,
      user.search,
    ],
  );

  const openLiveExplorerGroupFieldByName = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, groupPath: number[], fieldOption: Pf2eTerminalQueryFieldOption) => {
      const groupedFieldValues = new Set(
        getScopedFieldOptions(query)
          .filter((candidate) => candidate.editor === "sharedExplorer")
          .map((candidate) => candidate.value),
      );
      const existingBucket = buildStructuredDraftEntries(query, createStructuredDraftGroupResumeTarget(groupPath), {
        groupedFieldValues,
      }).find(
        (entry) =>
          entry.kind === "queryFieldBucket" &&
          groupPathsEqual(entry.groupPath, groupPath) &&
          entry.field === fieldOption.value,
      );

      await openLiveExplorerGroupedField(query, {
        kind: "queryFieldBucket",
        key: `synthetic:${groupPath.join(".")}:${fieldOption.value}`,
        label: fieldOption.label,
        description: fieldOption.description,
        groupPath,
        field: fieldOption.value,
        fieldOperator: existingBucket?.fieldOperator ?? "include",
        memberPaths: existingBucket?.memberPaths ?? [],
        fieldMemberPaths: existingBucket?.fieldMemberPaths ?? [],
        indent: groupPath.length + 1,
        menuLabel: fieldOption.label,
      });
    },
    [getScopedFieldOptions, openLiveExplorerGroupedField],
  );

  const openLiveExplorerCanonicalFieldMember = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], fieldOption: Pf2eTerminalQueryFieldOption) => {
      if (path.length === 0) {
        const initialFieldState = buildSearchFilterExplorerFieldState({
          discreteClauses: buildGroupedFieldSeedDiscreteClauses(query.filter, fieldOption.value),
          scalarClauses: {},
        });
        const seedQuery = { ...query, filter: undefined };
        const liveChangeState = { saw: false };
        const applyChange = ({ fieldState }: StructuredDraftExplorerContinuationChange) => {
          liveChangeState.saw = true;
          const replacementNodes = buildGroupedFieldReplacementNodes(user.search, query, fieldState, fieldOption);
          const nextFilter =
            replacementNodes.length === 0
              ? undefined
              : replacementNodes.length === 1
                ? replacementNodes[0]
                : ({ kind: getSearchQueryRootOperator(query), children: replacementNodes } satisfies SearchFilterNode);
          replaceStructuredDraftProjection(
            () => ({
              ...query,
              filter: nextFilter,
            }),
            { resumeTarget: createStructuredDraftGroupResumeTarget([]) },
          );
        };
        const continuation = await openStructuredDraftExplorerContinuation({
          query: seedQuery,
          fieldOption,
          currentNode: null,
          initialFieldState,
          preservedMetadata: getSearchQueryMetadataTree(seedQuery),
          onHostChange: applyChange,
        });
        if (
          continuation.kind !== "notOpened" &&
          continuation.change &&
          !liveChangeState.saw &&
          !searchFilterExplorerFieldStatesEqual(continuation.change.fieldState, initialFieldState)
        ) {
          applyChange(continuation.change);
        }
        return;
      }

      const groupPath = getContainingBooleanGroupPath(query.filter, path);
      await openLiveExplorerGroupedField(query, {
        kind: "queryFieldBucket",
        key: `synthetic:${groupPath.join(".")}:${fieldOption.value}:${path.join(".")}`,
        label: fieldOption.label,
        description: fieldOption.description,
        groupPath,
        field: fieldOption.value,
        fieldOperator: "include",
        memberPaths: [path],
        fieldMemberPaths: [path],
        indent: groupPath.length + 1,
        menuLabel: fieldOption.label,
      });
    },
    [
      openLiveExplorerGroupedField,
      openStructuredDraftExplorerContinuation,
      replaceStructuredDraftProjection,
      user.search,
    ],
  );

  const promptForFieldClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: "field" | "metric",
      currentNode: MetadataFilterNode | null = null,
    ): Promise<SearchFilterNodeEditorResult> => {
      const fieldOptions = getScopedFieldOptions(query).filter((fieldOption) =>
        family === "metric"
          ? isMetricFieldOptionValue(fieldOption.value)
          : !isMetricFieldOptionValue(fieldOption.value),
      );
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey(
          family === "metric"
            ? "No scoped metric filters are available for the current query."
            : "No scoped field filters are available for the current query.",
        );
        return undefined;
      }

      let preferredFieldValue = getMetadataFilterNodeFieldValue(currentNode) ?? fieldOptions[0]!.value;
      for (;;) {
        const selection = await promptSession.promptSelectOption({
          title: family === "metric" ? "Metric" : "Metadata",
          prompt:
            family === "metric"
              ? "Choose the metric family for the next clause"
              : "Choose the metadata field for the next clause",
          entries: fieldOptions.map((fieldOption) => ({
            value: fieldOption.value,
            label: fieldOption.label,
            description: fieldOption.description,
          })),
          selectedValue: fieldOptions.some((fieldOption) => fieldOption.value === preferredFieldValue)
            ? preferredFieldValue
            : fieldOptions[0]!.value,
        });
        if (selection.kind === "back") {
          return CLAUSE_BACK;
        }
        if (selection.kind !== "selected") {
          return undefined;
        }

        preferredFieldValue = selection.value;
        const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
        if (!fieldOption) {
          return undefined;
        }

        if (fieldOption.editor === "sharedExplorer") {
          const continuation = await openStructuredDraftExplorerContinuation({
            query,
            fieldOption,
            currentNode,
          });
          if (continuation.kind === "cancel" || continuation.kind === "notOpened") {
            return undefined;
          }
          if (!continuation.change) {
            return CLAUSE_BACK;
          }

          return toSearchFilterNodeEditorResult(continuation.change.result);
        }

        const nextNode = await editFieldClause(query, fieldOption, currentNode);
        return nextNode === undefined
          ? undefined
          : nextNode
            ? (metadataFilterNodeToCanonicalFilter(nextNode) ?? null)
            : null;
      }
    },
    [editFieldClause, getScopedFieldOptions, openStructuredDraftExplorerContinuation, prompts, terminal],
  );

  const getAvailableMetricFamilies = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, options: { numericOnly?: boolean } = {}): Promise<MetricFieldFamily[]> => {
      const [actorMetricOptions, itemMetricOptions] = await Promise.all([
        user.search.loadMetricKeyOptions(query, "actorMetric", "matching", options),
        user.search.loadMetricKeyOptions(query, "itemMetric", "matching", options),
      ]);
      const families: MetricFieldFamily[] = [];
      if (actorMetricOptions.length > 0) {
        families.push("actorMetric");
      }
      if (itemMetricOptions.length > 0) {
        families.push("itemMetric");
      }
      return families;
    },
    [user.search],
  );

  const promptForMetricFamily = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      availableFamilies: MetricFieldFamily[],
      currentFamily: MetricFieldFamily | null,
      title: string,
      prompt: string,
    ): Promise<PromptStepResult<MetricFieldFamily>> => {
      const metricFieldOptions = getScopedFieldOptions(query).filter((fieldOption) =>
        isMetricFieldOptionValue(fieldOption.value),
      );
      const entries = availableFamilies
        .map((family) => metricFieldOptions.find((fieldOption) => fieldOption.value === family))
        .filter((fieldOption): fieldOption is Pf2eTerminalQueryFieldOption => Boolean(fieldOption))
        .map((fieldOption) => ({
          value: fieldOption.value as MetricFieldFamily,
          label: fieldOption.label,
          description: fieldOption.description,
        }));
      if (entries.length === 0) {
        return undefined;
      }

      const selection = await promptSession.promptSelectOption({
        title,
        prompt,
        entries,
        selectedValue:
          currentFamily && entries.some((entry) => entry.value === currentFamily) ? currentFamily : entries[0]!.value,
      });
      if (selection.kind === "back") {
        return CLAUSE_BACK;
      }
      return selection.kind === "selected" ? selection.value : undefined;
    },
    [getScopedFieldOptions, prompts],
  );

  const promptForMetricKey = React.useCallback(
    async (
      _promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: MetricFieldFamily,
      title: string,
      _prompt: string,
      _currentMetric?: string,
      options: { numericOnly?: boolean } = {},
      initialDiscoveryMode: SearchFilterDiscoveryMode = "matching",
    ): Promise<PromptStepResult<MetricKeySelection>> => {
      const fieldOption = getScopedFieldOptions(query).find((candidate) => candidate.value === family);
      if (!fieldOption) {
        return undefined;
      }

      const continuation = await runStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption,
        initialDiscoveryMode,
        openFilterExplorer,
        query,
        resolveSelectionTarget: buildMetricSelectionTargetResolver(family, fieldOption.label, options),
        singleFieldBehavior: "list",
        title,
        user,
      });

      switch (continuation.kind) {
        case "selectTarget": {
          const metricKey = extractMetricKeyFromSelectTargetOutcome(continuation.outcome, family);
          return metricKey
            ? {
                value: metricKey,
                discoveryMode: continuation.discoveryMode,
              }
            : undefined;
        }
        case "resumeHost":
          return CLAUSE_BACK;
        case "cancel":
        case "notOpened":
          return undefined;
      }
    },
    [getScopedFieldOptions, openFilterExplorer, user],
  );

  const promptForMetricCompareClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "metricCompare" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "metricCompare" }> | ClausePromptBackResult | undefined> => {
      const availableFamilies = await getAvailableMetricFamilies(query, { numericOnly: true });
      if (availableFamilies.length === 0) {
        await terminal.pauseForAnyKey("No numeric metric comparisons are available for the current query.");
        return undefined;
      }

      const currentFamily: MetricFieldFamily | null = currentNode
        ? inferMetricFieldFamily(currentNode.leftMetric, getSearchQueryCategory(query))
        : null;
      let selectedFamily = currentFamily;
      let metricDiscoveryMode: SearchFilterDiscoveryMode = "matching";

      for (;;) {
        const family = await promptForMetricFamily(
          promptSession,
          query,
          availableFamilies,
          selectedFamily,
          "Metric comparison",
          "Choose the metric family for this comparison clause",
        );
        if (family === CLAUSE_BACK) {
          return CLAUSE_BACK;
        }
        if (family === undefined) {
          return undefined;
        }
        selectedFamily = family;

        for (;;) {
          const leftMetric = await promptForMetricKey(
            promptSession,
            query,
            selectedFamily,
            "Left Metric",
            "Choose the left-hand metric for this comparison clause",
            currentNode && currentFamily === selectedFamily ? currentNode.leftMetric : undefined,
            { numericOnly: true },
            metricDiscoveryMode,
          );
          if (leftMetric === CLAUSE_BACK) {
            break;
          }
          if (leftMetric === undefined) {
            return undefined;
          }
          metricDiscoveryMode = leftMetric.discoveryMode;

          for (;;) {
            const operatorSelection = await promptSession.promptSelectOption({
              title: "Comparison Operator",
              prompt: "Choose how the left metric should compare to the right metric",
              entries: [
                { value: "eq", label: "Equals", description: "Require both metrics to be equal." },
                { value: "notEq", label: "Does Not Equal", description: "Require both metrics to differ." },
                { value: "gt", label: "Greater Than", description: "Require the left metric to be greater." },
                {
                  value: "gte",
                  label: "Greater Or Equal",
                  description: "Require the left metric to be greater or equal.",
                },
                { value: "lt", label: "Less Than", description: "Require the left metric to be less." },
                { value: "lte", label: "Less Or Equal", description: "Require the left metric to be less or equal." },
              ],
              selectedValue: currentNode?.op ?? "gte",
            });
            if (operatorSelection.kind === "back") {
              break;
            }
            if (operatorSelection.kind !== "selected") {
              return undefined;
            }

            const operator = operatorSelection.value as MetricCompareOperator;
            const rightMetric = await promptForMetricKey(
              promptSession,
              query,
              selectedFamily,
              "Right Metric",
              "Choose the right-hand metric for this comparison clause",
              currentNode && currentFamily === selectedFamily ? currentNode.rightMetric : leftMetric.value,
              { numericOnly: true },
              metricDiscoveryMode,
            );
            if (rightMetric === CLAUSE_BACK) {
              continue;
            }
            if (!rightMetric) {
              return undefined;
            }

            return {
              kind: "metricCompare",
              leftMetric: leftMetric.value,
              op: operator,
              rightMetric: rightMetric.value,
            };
          }
        }
      }
    },
    [getAvailableMetricFamilies, promptForMetricFamily, promptForMetricKey, prompts, terminal],
  );

  const promptForPackClause = React.useCallback(
    async (
      _promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "pack" }>,
    ): Promise<SearchFilterNodeEditorResult> => {
      const baseQuery = setSearchQueryPackSelection(query, { include: [], exclude: [] });
      const seededQuery = currentNode
        ? setSearchQueryPackSelection(baseQuery, { include: [currentNode.value], exclude: [] })
        : baseQuery;
      const initialFieldState = buildSearchFilterExplorerFieldState(
        user.search.prepareFilterExplorerDraft(seededQuery, ["pack"]).draft,
      );

      const continuation = await runStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption: buildPackClauseFieldOption(),
        initialFieldState,
        openFilterExplorer,
        query: baseQuery,
        title: "Pack",
        user,
      });

      if (continuation.kind === "cancel" || continuation.kind === "notOpened") {
        return undefined;
      }
      if (continuation.kind !== "resumeHost") {
        return undefined;
      }

      const selection = continuation.change?.fieldState.discreteSelections.pack
        ? {
            include: [...continuation.change.fieldState.discreteSelections.pack.include],
            exclude: [...continuation.change.fieldState.discreteSelections.pack.exclude],
          }
        : getSearchQueryPackSelection(baseQuery);
      const hasSelection = selection.include.length > 0 || selection.exclude.length > 0;
      if (!hasSelection) {
        return currentNode ? null : CLAUSE_BACK;
      }

      return buildSearchFilterPackSelectionNode(selection);
    },
    [openFilterExplorer, user.search],
  );

  const promptForScopeClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "scope" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "scope" }> | ClausePromptBackResult | undefined> => {
      const categoryOptions = user.search
        .getCategoryOptions()
        .filter(
          (option): option is { value: SearchCategory; label: string; description: string } => option.value !== null,
        );
      if (categoryOptions.length === 0) {
        await terminal.pauseForAnyKey("No categories are available for the current query.");
        return undefined;
      }

      for (;;) {
        const categorySelection = await promptSession.promptSelectOption({
          title: "Scope",
          prompt: "Choose the category for this scope clause",
          entries: categoryOptions.map((option) => ({
            value: option.value,
            label: option.label,
            description: option.description,
          })),
          selectedValue: currentNode?.category ?? categoryOptions[0]!.value,
        });
        if (categorySelection.kind === "back") {
          return CLAUSE_BACK;
        }
        if (categorySelection.kind !== "selected") {
          return undefined;
        }

        const category = normalizeSearchCategory(categorySelection.value);
        if (!category) {
          return undefined;
        }

        const subcategoryOptions = user.search
          .getSubcategoryOptions(category)
          .filter((option) => option.value !== null);
        const matchingCurrentNode = currentNode && currentNode.category === category ? currentNode : null;
        const currentMode =
          matchingCurrentNode?.subcategory.kind === "eq"
            ? "specific"
            : matchingCurrentNode?.subcategory.kind === "isNull"
              ? "none"
              : "any";

        for (;;) {
          const modeSelection = await promptSession.promptSelectOption({
            title: "Subcategory Mode",
            prompt: "Choose how this scope clause should treat subcategories",
            entries: [
              {
                value: "any",
                label: "Any subcategory",
                description: "Match any subcategory inside the selected category.",
              },
              {
                value: "specific",
                label: "Specific subcategory",
                description: "Choose one exact subcategory inside the selected category.",
              },
              { value: "none", label: "No subcategory", description: "Match only records without a subcategory." },
            ],
            selectedValue: currentMode,
          });
          if (modeSelection.kind === "back") {
            break;
          }
          if (modeSelection.kind !== "selected") {
            return undefined;
          }

          let subcategory: SearchScopeSubcategoryMatch = { kind: "any" };
          if (modeSelection.value === "none") {
            subcategory = { kind: "isNull" };
          } else if (modeSelection.value === "specific") {
            if (subcategoryOptions.length === 0) {
              await terminal.pauseForAnyKey("No subcategories are available for the selected category.");
              return undefined;
            }
            const currentSubcategoryValue =
              matchingCurrentNode?.subcategory.kind === "eq" ? matchingCurrentNode.subcategory.value : null;
            const subcategorySelection = await promptSession.promptSelectOption({
              title: "Specific Subcategory",
              prompt: "Choose the exact subcategory for this scope clause",
              entries: subcategoryOptions.map((option) => ({
                value: option.value,
                label: option.label,
                description: option.description,
              })),
              selectedValue:
                currentSubcategoryValue && subcategoryOptions.some((option) => option.value === currentSubcategoryValue)
                  ? currentSubcategoryValue
                  : subcategoryOptions[0]!.value,
            });
            if (subcategorySelection.kind === "back") {
              continue;
            }
            if (subcategorySelection.kind !== "selected") {
              return undefined;
            }
            const normalizedSubcategory = normalizeSearchSubcategory(subcategorySelection.value) ?? null;
            if (!normalizedSubcategory) {
              return undefined;
            }
            subcategory = { kind: "eq", value: normalizedSubcategory };
          }

          return {
            kind: "scope",
            category,
            subcategory,
          };
        }
      }
    },
    [prompts, terminal, user.search],
  );

  const promptForNumericMatchClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      nodeKind: "level" | "price" | "actionCost",
      node?:
        | Extract<SearchFilterNode, { kind: "level" }>
        | Extract<SearchFilterNode, { kind: "price" }>
        | Extract<SearchFilterNode, { kind: "actionCost" }>,
    ): Promise<
      | Extract<SearchFilterNode, { kind: "level" }>
      | Extract<SearchFilterNode, { kind: "price" }>
      | Extract<SearchFilterNode, { kind: "actionCost" }>
      | null
      | undefined
    > => {
      if (node?.kind === "actionCost" && (node.match.kind === "isNull" || node.match.kind === "isNotNull")) {
        await terminal.pauseForAnyKey("Null action-cost clauses cannot be edited through the numeric matcher.");
        return undefined;
      }
      let currentNumericMatch: SearchNumericMatch | null = null;
      if (node?.kind === "actionCost") {
        switch (node.match.kind) {
          case "eq":
          case "gt":
          case "gte":
          case "lt":
          case "lte":
          case "between":
            currentNumericMatch = node.match;
            break;
          case "isNull":
          case "isNotNull":
            currentNumericMatch = null;
            break;
        }
      } else if (node) {
        currentNumericMatch = node.match;
      }
      if (nodeKind === "level") {
        const parsed = await promptLevelRangeDraft(promptSession, terminal, {
          defaultValue: currentNumericMatch ? formatNumericMatch(currentNumericMatch) : "",
        });
        if (parsed === undefined) {
          return undefined;
        }
        if (parsed === null) {
          return null;
        }
        return {
          kind: nodeKind,
          match:
            parsed.kind === "between"
              ? {
                  kind: "between",
                  min: Math.min(parsed.min, parsed.max),
                  max: Math.max(parsed.min, parsed.max),
                }
              : parsed,
        };
      }

      const parsed = await promptNumericScalarClause(promptSession, terminal, {
        title: nodeKind === "price" ? "Price Matcher" : "Action Cost Matcher",
        currentClause:
          currentNumericMatch?.kind === "between"
            ? { op: "between", min: currentNumericMatch.min, max: currentNumericMatch.max }
            : currentNumericMatch?.kind === "eq" ||
                currentNumericMatch?.kind === "gt" ||
                currentNumericMatch?.kind === "gte" ||
                currentNumericMatch?.kind === "lt" ||
                currentNumericMatch?.kind === "lte"
              ? { op: currentNumericMatch.kind, value: currentNumericMatch.value }
              : null,
      });
      if (parsed === undefined) {
        return undefined;
      }
      if (parsed === null) {
        return null;
      }
      if (parsed.op === "neq") {
        await terminal.pauseForAnyKey(
          "`!=` is not supported for this matcher. Use an exact, minimum, maximum, or range value.",
        );
        return undefined;
      }

      return {
        kind: nodeKind,
        match:
          parsed.op === "between"
            ? {
                kind: "between",
                min: Math.min(parsed.min, parsed.max),
                max: Math.max(parsed.min, parsed.max),
              }
            : {
                kind: parsed.op,
                value: parsed.value,
              },
      };
    },
    [terminal],
  );

  const promptForRarityClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      node?: Extract<SearchFilterNode, { kind: "rarity" }>,
    ): Promise<SearchFilterNodeEditorResult> => {
      const continuation = await openStructuredDraftExplorerContinuation({
        query,
        fieldOption: buildExplorerOnlyFieldOption(
          "rarity",
          "Rarity",
          "Browse live rarities for the current scope and stage canonical rarity clauses.",
          "enumString",
        ),
        currentNode: node ? canonicalFilterToMetadataNode(node) : null,
      });
      if (continuation.kind === "notOpened") {
        return undefined;
      }
      if (continuation.kind === "cancel") {
        return undefined;
      }
      if (!continuation.change) {
        return CLAUSE_BACK;
      }

      return toSearchFilterNodeEditorResult(continuation.change.result);
    },
    [openStructuredDraftExplorerContinuation],
  );

  const promptForClauseNode = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      clauseKind: ClauseKind,
      currentNode?: SearchFilterNode,
    ): Promise<SearchFilterNodeEditorResult> => {
      switch (clauseKind) {
        case "field":
          return promptForFieldClause(
            promptSession,
            query,
            "field",
            currentNode ? canonicalFilterToMetadataNode(currentNode) : null,
          );
        case "metric":
          return promptForFieldClause(
            promptSession,
            query,
            "metric",
            currentNode ? canonicalFilterToMetadataNode(currentNode) : null,
          );
        case "metricCompare":
          return promptForMetricCompareClause(
            promptSession,
            query,
            currentNode?.kind === "metricCompare" ? currentNode : undefined,
          );
        case "pack":
          return promptForPackClause(promptSession, query, currentNode?.kind === "pack" ? currentNode : undefined);
        case "scope":
          return promptForScopeClause(promptSession, query, currentNode?.kind === "scope" ? currentNode : undefined);
        case "level":
          return promptForNumericMatchClause(
            promptSession,
            "level",
            currentNode?.kind === "level" ? currentNode : undefined,
          );
        case "price":
          return promptForNumericMatchClause(
            promptSession,
            "price",
            currentNode?.kind === "price" ? currentNode : undefined,
          );
        case "rarity":
          return promptForRarityClause(query, currentNode?.kind === "rarity" ? currentNode : undefined);
        case "actionCost":
          return promptForNumericMatchClause(
            promptSession,
            "actionCost",
            currentNode?.kind === "actionCost" ? currentNode : undefined,
          );
      }
    },
    [
      promptForFieldClause,
      promptForMetricCompareClause,
      promptForNumericMatchClause,
      promptForPackClause,
      promptForRarityClause,
      promptForScopeClause,
    ],
  );

  const promptForClauseKind = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
    ): Promise<ClausePromptResult> => {
      const fieldOptions = getScopedFieldOptions(query);
      const hasFieldClauses = fieldOptions.some((fieldOption) => !isMetricFieldOptionValue(fieldOption.value));
      const hasMetricClauses = fieldOptions.some((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const hasMetricCompareClauses = hasMetricClauses;
      const hasPrice = fieldOptions.some((fieldOption) => fieldOption.value === "priceCp");
      const hasActionCost =
        user.search.getActionCostOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)).length > 0;
      const entryByValue = new Map<ClauseKind, { value: ClauseKind; label: string; description: string }>();
      entryByValue.set("scope", {
        value: "scope",
        label: "Scope",
        description: "Add a category and subcategory scope clause.",
      });
      if (hasFieldClauses) {
        entryByValue.set("field", {
          value: "field",
          label: "Metadata",
          description: "Filter on a metadata field such as traits or other categorical fields.",
        });
      }
      if (hasMetricClauses) {
        entryByValue.set("metric", {
          value: "metric",
          label: "Metric",
          description: "Filter on one discovered metric key.",
        });
      }
      if (hasMetricCompareClauses) {
        entryByValue.set("metricCompare", {
          value: "metricCompare",
          label: "Metric comparison",
          description: "Compare two numeric metrics from the current scoped discovery families.",
        });
      }
      entryByValue.set("pack", {
        value: "pack",
        label: "Pack",
        description: "Restrict results to one or more selected packs without waiting on preflight discovery checks.",
      });
      entryByValue.set("level", {
        value: "level",
        label: "Level",
        description: "Add a level matcher such as 1, >=5, <=10, or 1-5.",
      });
      if (hasPrice) {
        entryByValue.set("price", {
          value: "price",
          label: "Price",
          description: "Add a price matcher such as 100, >=500, <=1000, or 100-500.",
        });
      }
      entryByValue.set("rarity", {
        value: "rarity",
        label: "Rarity",
        description: "Add one rarity clause using the shared categorical picker family.",
      });
      if (hasActionCost) {
        entryByValue.set("actionCost", {
          value: "actionCost",
          label: "Action Cost",
          description: "Add an action-cost matcher such as 1, >=2, <=3, or 1-2.",
        });
      }
      const entries = (
        ["scope", "field", "metric", "metricCompare", "pack", "level", "price", "rarity", "actionCost"] as const
      )
        .map((value) => entryByValue.get(value))
        .filter((entry): entry is { value: ClauseKind; label: string; description: string } => Boolean(entry));
      const result = await promptSession.promptSelectOption({
        title: "Add Clause",
        prompt: "Choose the clause kind to insert into the current group",
        entries,
        selectedValue: entries[0]?.value ?? "scope",
      });
      if (result.kind === "back") {
        return CLAUSE_BACK;
      }
      return result.kind === "selected" ? result.value : null;
    },
    [getScopedFieldOptions, user.search],
  );

  const addQueryClauseAtPath = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[] = [],
      wrapper?: "allOf" | "anyOf" | "not",
    ): Promise<ClauseApplyResult> => {
      return terminal.runPromptSession(async (session) => {
        const workingQuery = query;
        for (;;) {
          const clauseKind = await promptForClauseKind(session, workingQuery);
          if (clauseKind === CLAUSE_BACK) {
            return "back";
          }
          if (!clauseKind) {
            return "cancelled";
          }
          if (clauseKind === "field" && wrapper === undefined) {
            const fieldOptions = getScopedFieldOptions(workingQuery).filter(
              (fieldOption) => !isMetricFieldOptionValue(fieldOption.value) && fieldOption.editor === "sharedExplorer",
            );
            if (fieldOptions.length > 0) {
              const selection = await session.promptSelectOption({
                title: "Metadata",
                prompt: "Choose the metadata field for the next clause",
                entries: fieldOptions.map((fieldOption) => ({
                  value: fieldOption.value,
                  label: fieldOption.label,
                  description: fieldOption.description,
                })),
                selectedValue: fieldOptions[0]!.value,
              });
              if (selection.kind === "back") {
                continue;
              }
              if (selection.kind !== "selected") {
                return "cancelled";
              }
              const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
              if (!fieldOption) {
                return "cancelled";
              }

              await openLiveExplorerGroupFieldByName(workingQuery, path, fieldOption);
              return "applied";
            }
          }
          const nextNode = await promptForClauseNode(session, workingQuery, clauseKind);
          if (nextNode === CLAUSE_BACK) {
            continue;
          }
          if (!nextNode) {
            return "cancelled";
          }
          const wrappedNode =
            wrapper === "allOf" || wrapper === "anyOf"
              ? ({
                  kind: wrapper,
                  children: Array.isArray(nextNode) ? nextNode : [nextNode],
                } as SearchFilterNode)
              : wrapper === "not"
                ? ({
                    kind: "not",
                    child: Array.isArray(nextNode)
                      ? ({ kind: "allOf", children: nextNode } as SearchFilterNode)
                      : nextNode,
                  } as SearchFilterNode)
                : nextNode;
          const nextFilter = Array.isArray(wrappedNode)
            ? appendSearchFilterNodesAtPath(
                workingQuery.filter,
                path,
                wrappedNode,
                getSearchQueryRootOperator(workingQuery),
              )
            : appendSearchFilterNodeAtPath(
                workingQuery.filter,
                path,
                wrappedNode,
                getSearchQueryRootOperator(workingQuery),
              );
          applyNextTree(nextFilter);
          return "applied";
        }
      });
    },
    [
      applyNextTree,
      getScopedFieldOptions,
      openLiveExplorerGroupFieldByName,
      promptForClauseKind,
      promptForClauseNode,
      terminal,
    ],
  );

  const runInsertionAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], actionId: StructuredDraftEntryActionId) => {
      if (actionId === "moveHere") {
        if (!moveSourcePath) {
          return;
        }
        applyNextTree(
          moveSearchFilterNodeToGroupPath(query.filter, moveSourcePath, path, getSearchQueryRootOperator(query)),
        );
        clearStructuredDraftMoveSource();
        return;
      }

      if (actionId === "addClause") {
        return addQueryClauseAtPath(query, path);
      }

      if (actionId === "addAndGroup" || actionId === "addOrGroup" || actionId === "addNotGroup") {
        return addQueryClauseAtPath(
          query,
          path,
          actionId === "addAndGroup" ? "allOf" : actionId === "addOrGroup" ? "anyOf" : "not",
        );
      }
      return "cancelled";
    },
    [addQueryClauseAtPath, applyNextTree, clearStructuredDraftMoveSource, moveSourcePath],
  );

  const promptForInsertionAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[]) => {
      for (;;) {
        const entries = buildInsertionActionEntries(Boolean(moveSourcePath));
        const result = await prompts.promptSelectOption({
          title: "Insertion Slot",
          prompt: moveSourcePath
            ? "Choose where to move the selected node"
            : "Choose what to add at this insertion slot",
          entries: entries.map((entry) => ({
            value: entry.id,
            label: entry.label,
            description: entry.description,
          })),
          selectedValue: entries[0]?.id ?? "addClause",
        });
        if (result.kind !== "selected") {
          return;
        }
        const insertionResult = await runInsertionAction(query, path, result.value);
        if (insertionResult !== "back") {
          return;
        }
      }
    },
    [moveSourcePath, runInsertionAction],
  );

  const runRootAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, actionId: StructuredDraftEntryActionId) => {
      if (actionId === "toggleRoot") {
        applyNextTree(toggleSearchFilterRootGroupOperator(query.filter));
        return;
      }

      await runInsertionAction(query, [], actionId);
    },
    [applyNextTree, runInsertionAction],
  );

  const promptForRootAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery) => {
      const entries = buildRootActionEntries(query);
      const result = await prompts.promptSelectOption({
        title: "Root Group",
        prompt: "Choose how to update the visible root group",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runRootAction(query, result.value);
    },
    [runRootAction],
  );

  const getLeafActionEntries = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: SearchFilterNode,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const fieldOptions = getScopedFieldOptions(query);
      const fieldOptionValue = getQueryFieldValueForNode(node);
      const fieldOption = fieldOptionValue
        ? (fieldOptions.find((candidate) => candidate.value === fieldOptionValue) ?? null)
        : null;
      const editableMetadataNode = canonicalFilterToMetadataNode(node);
      const editableClauseKind: ClauseKind | null =
        node.kind === "scope"
          ? "scope"
          : node.kind === "level"
            ? "level"
            : node.kind === "price"
              ? "price"
              : node.kind === "pack"
                ? "pack"
                : node.kind === "metricCompare"
                  ? "metricCompare"
                  : node.kind === "rarity"
                    ? "rarity"
                    : node.kind === "actionCost"
                      ? "actionCost"
                      : fieldOption && editableMetadataNode
                        ? isMetricFieldOptionValue(fieldOption.value)
                          ? "metric"
                          : "field"
                        : null;
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);

      return [
        ...(editableClauseKind
          ? [
              {
                id: "edit" as const,
                label: "Edit Clause",
                description: "Change this canonical clause without leaving the tree editor.",
              },
            ]
          : []),
        {
          id: "wrapNot",
          label: `Wrap In ${formatFriendlyGroupLabel("not")}`,
          description: "Exclude this clause without changing its content.",
        },
        {
          id: "wrapAnd",
          label: `Wrap In ${formatFriendlyGroupLabel("allOf")}`,
          description: "Place this clause inside a new group where every child must match.",
        },
        {
          id: "wrapOr",
          label: `Wrap In ${formatFriendlyGroupLabel("anyOf")}`,
          description: "Place this clause inside a new group where any child may match.",
        },
        { id: "move", label: "Move Node", description: "Move this clause to another visible insertion slot." },
        ...(canLift
          ? [
              {
                id: "lift" as const,
                label: "Lift Node",
                description: "Lift this clause out of its current boolean group.",
              },
            ]
          : []),
        { id: "remove", label: "Remove Clause", description: "Delete this clause from the live query tree." },
      ];
    },
    [getScopedFieldOptions],
  );

  const runLeafAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: SearchFilterNode,
      actionId: StructuredDraftEntryActionId,
    ) => {
      const fieldOptions = getScopedFieldOptions(query);
      const fieldOptionValue = getQueryFieldValueForNode(node);
      const fieldOption = fieldOptionValue
        ? (fieldOptions.find((candidate) => candidate.value === fieldOptionValue) ?? null)
        : null;
      const editableMetadataNode = canonicalFilterToMetadataNode(node);
      const editableClauseKind: ClauseKind | null =
        node.kind === "scope"
          ? "scope"
          : node.kind === "level"
            ? "level"
            : node.kind === "price"
              ? "price"
              : node.kind === "pack"
                ? "pack"
                : node.kind === "metricCompare"
                  ? "metricCompare"
                  : node.kind === "rarity"
                    ? "rarity"
                    : node.kind === "actionCost"
                      ? "actionCost"
                      : fieldOption && editableMetadataNode
                        ? isMetricFieldOptionValue(fieldOption.value)
                          ? "metric"
                          : "field"
                        : null;

      if (actionId === "edit") {
        if (!editableClauseKind) {
          await terminal.pauseForAnyKey("That clause cannot be edited through the current canonical editor set.");
          return;
        }
        if (editableClauseKind === "rarity") {
          await openLiveExplorerCanonicalFieldMember(
            query,
            path,
            buildExplorerOnlyFieldOption(
              "rarity",
              "Rarity",
              "Browse live rarities for the current scope and stage canonical rarity clauses.",
              "enumString",
            ),
          );
          return;
        }
        if (editableClauseKind === "actionCost") {
          await openLiveExplorerCanonicalFieldMember(
            query,
            path,
            buildExplorerOnlyFieldOption(
              "actionCost",
              "Action Cost",
              "Browse live action costs for the current scope and stage canonical action-cost clauses.",
              "number",
            ),
          );
          return;
        }
        if (
          (editableClauseKind === "field" || editableClauseKind === "metric") &&
          fieldOption &&
          editableMetadataNode
        ) {
          if (fieldOption.editor === "sharedExplorer") {
            const groupedFieldValues = new Set(
              fieldOptions
                .filter((candidate) => candidate.editor === "sharedExplorer")
                .map((candidate) => candidate.value),
            );
            const groupedFieldBucket = findStructuredDraftGroupedFieldBucketForPath(query, path, groupedFieldValues);
            if (groupedFieldBucket) {
              await openLiveExplorerGroupedField(query, groupedFieldBucket);
              return;
            }
            await openLiveExplorerFieldClause(query, path, fieldOption, editableMetadataNode);
            return;
          }
          const nextNode = await editFieldClause(query, fieldOption, editableMetadataNode);
          if (nextNode !== undefined) {
            updateStructuredDraftMetadataNode(path, () => nextNode);
          }
          return;
        }

        const nextNode = await terminal.runPromptSession((session) =>
          promptForClauseNode(session, query, editableClauseKind, node),
        );
        if (nextNode === CLAUSE_BACK || nextNode === undefined || Array.isArray(nextNode)) {
          return;
        }
        if (nextNode === null) {
          applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
          return;
        }
        applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => nextNode));
        return;
      }

      if (actionId === "wrapNot" || actionId === "wrapAnd" || actionId === "wrapOr") {
        applyNextTree(
          wrapSearchFilterNodeAtPath(
            query.filter,
            path,
            actionId === "wrapNot" ? "not" : actionId === "wrapAnd" ? "allOf" : "anyOf",
          ),
        );
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(
          path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined),
        );
      }
    },
    [
      applyNextTree,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      openLiveExplorerGroupedField,
      openLiveExplorerCanonicalFieldMember,
      openLiveExplorerFieldClause,
      promptForClauseNode,
      terminal,
      updateStructuredDraftMetadataNode,
    ],
  );

  const promptForLeafAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode) => {
      const entries = getLeafActionEntries(query, path, node);
      const result = await prompts.promptSelectOption({
        title: "Query Clause",
        prompt: "Choose how to update this live query clause",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "wrapNot",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runLeafAction(query, path, node, result.value);
    },
    [getLeafActionEntries, runLeafAction],
  );

  const getNotActionEntries = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      path: number[],
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      return [
        {
          id: "unwrap",
          label: `Remove ${formatFriendlyGroupLabel("not")}`,
          description: "Keep the child clause and remove the exclusion.",
        },
        {
          id: "move",
          label: "Move Node",
          description: "Move this excluded group to another visible insertion slot.",
        },
        ...(canLift
          ? [
              {
                id: "lift" as const,
                label: "Lift Node",
                description: "Lift this excluded group out of its current parent group.",
              },
            ]
          : []),
        { id: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
      ];
    },
    [],
  );

  const runNotAction = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "not" }>,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (actionId === "unwrap") {
        applyNextTree(
          path.length === 0 ? node.child : updateSearchFilterNodeAtPath(query.filter, path, () => node.child),
        );
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(
          path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined),
        );
      }
    },
    [applyNextTree, enterStructuredDraftMoveMode],
  );

  const promptForNotAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: Extract<SearchFilterNode, { kind: "not" }>) => {
      const entries = getNotActionEntries(query, path);
      const result = await prompts.promptSelectOption({
        title: `${formatFriendlyGroupLabel("not")} Group`,
        prompt: "Choose how to update this negated live clause",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "unwrap",
      });
      if (result.kind !== "selected") {
        return;
      }
      runNotAction(query, path, node, result.value);
    },
    [getNotActionEntries, runNotAction],
  );

  const getGroupActionEntries = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<
        SearchFilterNode,
        { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }
      >,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const canUnwrap = canUnwrapSearchFilterNodeAtPath(query.filter, path);
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      return [
        { id: "addClause", label: "Add Clause", description: "Append a new canonical clause at this group bottom." },
        {
          id: "addAndGroup",
          label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
          description: "Append a nested group where every child must match.",
        },
        {
          id: "addOrGroup",
          label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
          description: "Append a nested group where any child may match.",
        },
        {
          id: "addNotGroup",
          label: `Add ${formatFriendlyGroupLabel("not")} Group`,
          description: "Append a nested excluded group.",
        },
        {
          id: "toggleGroup",
          label:
            node.kind === "allOf"
              ? `Change To ${formatFriendlyGroupLabel("anyOf")}`
              : `Change To ${formatFriendlyGroupLabel("allOf")}`,
          description: "Reshape this group without changing its current children.",
        },
        {
          id: "wrapNot",
          label: `Wrap In ${formatFriendlyGroupLabel("not")}`,
          description: "Wrap this group in an excluded group.",
        },
        { id: "move", label: "Move Node", description: "Move this group to another visible insertion slot." },
        ...(canUnwrap
          ? [
              {
                id: "unwrap" as const,
                label: "Unwrap Group",
                description: "Replace this group with its current children.",
              },
            ]
          : []),
        ...(canLift
          ? [
              {
                id: "lift" as const,
                label: "Lift Node",
                description: "Lift this group out of its current parent group.",
              },
            ]
          : []),
        { id: "remove", label: "Remove Group", description: "Delete this group and all of its children." },
      ];
    },
    [],
  );

  const runGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<
        SearchFilterNode,
        { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }
      >,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (
        actionId === "addClause" ||
        actionId === "addAndGroup" ||
        actionId === "addOrGroup" ||
        actionId === "addNotGroup"
      ) {
        await runInsertionAction(query, path, actionId);
        return;
      }
      if (actionId === "toggleGroup") {
        applyNextTree(
          reshapeSearchFilterBooleanGroupAtPath(query.filter, path, node.kind === "allOf" ? "anyOf" : "allOf"),
        );
        return;
      }
      if (actionId === "wrapNot") {
        applyNextTree(wrapSearchFilterNodeAtPath(query.filter, path, "not"));
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "unwrap") {
        applyNextTree(unwrapSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(
          path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined),
        );
      }
    },
    [applyNextTree, enterStructuredDraftMoveMode, runInsertionAction],
  );

  const promptForGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<
        SearchFilterNode,
        { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }
      >,
    ) => {
      const entries = getGroupActionEntries(query, path, node);
      const result = await prompts.promptSelectOption({
        title: "Boolean Group",
        prompt: "Choose how to update this live boolean group",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runGroupAction(query, path, node, result.value);
    },
    [getGroupActionEntries, runGroupAction],
  );

  const editStructuredDraftMetadata = React.useCallback(
    async (entry: SearchStructuredDraftEntry) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await promptForGroupAction(draftQuery, rootPath, rootNode);
            return;
          }
        }
        await promptForRootAction(draftQuery);
        return;
      }
      if (entry.kind === "queryInsertionSlot") {
        if (moveSourcePath) {
          applyNextTree(
            moveSearchFilterNodeToGroupPath(
              draftQuery.filter,
              moveSourcePath,
              entry.insertionPath ?? [],
              getSearchQueryRootOperator(draftQuery),
            ),
          );
          clearStructuredDraftMoveSource();
          return;
        }
        await runInsertionAction(draftQuery, entry.insertionPath ?? [], "addClause");
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        await openLiveExplorerGroupedField(draftQuery, entry);
        return;
      }

      const node = getSearchFilterNodeAtPath(draftQuery.filter, entry.treePath ?? []);
      if (!node) {
        return;
      }
      if (node.kind === "not") {
        await promptForNotAction(draftQuery, entry.treePath ?? [], node);
        return;
      }
      if (isSearchFilterBooleanGroup(node)) {
        await promptForGroupAction(draftQuery, entry.treePath ?? [], node);
        return;
      }
      await promptForLeafAction(draftQuery, entry.treePath ?? [], node);
    },
    [
      applyNextTree,
      clearStructuredDraftMoveSource,
      moveSourcePath,
      promptForGroupAction,
      promptForInsertionAction,
      promptForLeafAction,
      promptForNotAction,
      promptForRootAction,
      structuredDraftQuery,
    ],
  );

  const getStructuredDraftEntryActions = React.useCallback(
    (
      entry: SearchStructuredDraftEntry | null | undefined,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return [];
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            return getGroupActionEntries(draftQuery, rootPath, rootNode);
          }
        }
        return buildRootActionEntries(draftQuery);
      }
      if (entry.kind === "queryInsertionSlot") {
        return buildInsertionActionEntries(Boolean(moveSourcePath));
      }
      if (entry.kind === "queryFieldBucket") {
        return [
          {
            id: "edit",
            label: "Edit Clause",
            description: "Edit this current-group field bucket through the shared explorer.",
          },
        ];
      }

      const node = getSearchFilterNodeAtPath(draftQuery.filter, entry.treePath ?? []);
      if (!node) {
        return [];
      }
      if (node.kind === "not") {
        return getNotActionEntries(draftQuery, entry.treePath ?? []);
      }
      if (isSearchFilterBooleanGroup(node)) {
        return getGroupActionEntries(draftQuery, entry.treePath ?? [], node);
      }
      return getLeafActionEntries(draftQuery, entry.treePath ?? [], node);
    },
    [getGroupActionEntries, getLeafActionEntries, getNotActionEntries, moveSourcePath, structuredDraftQuery],
  );

  const runStructuredDraftEntryAction = React.useCallback(
    async (entry: SearchStructuredDraftEntry | null | undefined, actionId: StructuredDraftEntryActionId) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return;
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await runGroupAction(draftQuery, rootPath, rootNode, actionId);
            return;
          }
        }
        await runRootAction(draftQuery, actionId);
        return;
      }
      if (entry.kind === "queryInsertionSlot") {
        await runInsertionAction(draftQuery, entry.insertionPath ?? [], actionId);
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        if (actionId === "edit") {
          await openLiveExplorerGroupedField(draftQuery, entry);
        }
        return;
      }

      const path = entry.treePath ?? [];
      const node = getSearchFilterNodeAtPath(draftQuery.filter, path);
      if (!node) {
        return;
      }
      if (node.kind === "not") {
        runNotAction(draftQuery, path, node, actionId);
        return;
      }
      if (isSearchFilterBooleanGroup(node)) {
        await runGroupAction(draftQuery, path, node, actionId);
        return;
      }
      await runLeafAction(draftQuery, path, node, actionId);
    },
    [
      runGroupAction,
      runInsertionAction,
      runLeafAction,
      runNotAction,
      openLiveExplorerGroupedField,
      runRootAction,
      structuredDraftQuery,
    ],
  );

  return {
    editStructuredDraftMetadata,
    getStructuredDraftEntryActions,
    runStructuredDraftEntryAction,
  };
}
