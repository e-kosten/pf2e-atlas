import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import { canonicalFilterToMetadataNode } from "../../search/query-parts.js";
import {
  buildSearchFilterPackSelectionNode,
  buildSearchFilterValueSelectionNode,
  getSearchQueryCategory,
  getSearchQueryMetadataTree,
  getSearchQueryPackSelection,
  getSearchQueryRaritySelection,
  getSearchQueryRootOperator,
  setSearchQueryPackSelection,
  setSearchQueryRaritySelection,
} from "../../search/query-state.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { appendSearchFilterNodesAtPath, getSearchFilterNodeAtPath, updateSearchFilterNodeAtPath } from "../../search/query-core.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import type { FilterExplorerComposeTarget, FilterExplorerSelectTargetOutcome } from "../../filter-explorer/types.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";
import {
  buildSearchFilterExplorerComposeDraft,
  buildSearchFilterExplorerFieldState,
  type SearchFilterExplorerFieldState,
} from "../filter-explorer-field-state.js";
import {
  buildStructuredDraftGroupedFieldMutation,
  runStructuredDraftExplorerContinuation,
  structuredDraftPromptApply,
  structuredDraftPromptBack,
  structuredDraftPromptCancel,
  type StructuredDraftContinuationChange,
  type StructuredDraftHostMutation,
  type StructuredDraftPromptFlowResult,
} from "./structured-draft-continuation.js";
import {
  buildGroupedFieldReplacementNodes,
  buildGroupedFieldSeedDiscreteClauses,
  buildGroupedFieldSeedState,
} from "./structured-draft-grouped-field.js";
import {
  applyStructuredDraftHostMutationToQuery,
  getContainingBooleanGroupPath,
} from "./structured-draft-host-mutations.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftNodeResumeTarget,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";
import { buildStructuredDraftEntries } from "./structured-draft-support.js";

export type MetricFieldFamily = "actorMetric" | "itemMetric";
export type StructuredDraftExplorerMetricKeySelection = {
  value: string;
  discoveryMode: SearchFilterDiscoveryMode;
};
export type StructuredDraftExplorerPromptNodeValue = SearchFilterNode | SearchFilterNode[] | null;
export type StructuredDraftExplorerPromptNodeResult =
  StructuredDraftPromptFlowResult<StructuredDraftExplorerPromptNodeValue>;
export type StructuredDraftExplorerMetricKeyResult =
  StructuredDraftPromptFlowResult<StructuredDraftExplorerMetricKeySelection>;

function groupPathsEqual(left: number[] | undefined, right: number[]): boolean {
  return Boolean(left) && JSON.stringify(left) === JSON.stringify(right);
}

export function buildExplorerOnlyFieldOption(
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

export function searchFilterExplorerFieldStatesEqual(
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

export function searchFilterNodeContainsFieldValue(
  node: SearchFilterNode | null | undefined,
  field: Pf2eTerminalQueryFieldOption["value"],
): boolean {
  if (!node) {
    return false;
  }

  if (node.kind === "metadataPredicate") {
    return node.predicate.field === field;
  }

  if (node.kind === "rarity" || node.kind === "actionCost" || node.kind === "pack") {
    return node.kind === field;
  }

  if (node.kind === "metric") {
    return inferMetricFieldFamily(node.metric) === field;
  }

  if (node.kind === "allOf" || node.kind === "anyOf") {
    return node.children.some((child) => searchFilterNodeContainsFieldValue(child, field));
  }

  if (node.kind === "not") {
    return searchFilterNodeContainsFieldValue(node.child, field);
  }

  return false;
}

function collectSearchFilterNodesForFieldValue(
  node: SearchFilterNode | null | undefined,
  field: Pf2eTerminalQueryFieldOption["value"],
): SearchFilterNode[] {
  if (!node) {
    return [];
  }

  if (searchFilterNodeContainsFieldValue(node, field)) {
    if (node.kind === "allOf" || node.kind === "anyOf") {
      return node.children.flatMap((child) => collectSearchFilterNodesForFieldValue(child, field));
    }
    if (node.kind === "not") {
      return collectSearchFilterNodesForFieldValue(node.child, field).map(
        (child) => ({ kind: "not", child }) satisfies SearchFilterNode,
      );
    }
    return [node];
  }

  return [];
}

export function getAddedSearchFilterNodesForFieldValue(
  previousFilter: SearchFilterNode | undefined,
  nextFilter: SearchFilterNode | undefined,
  field: Pf2eTerminalQueryFieldOption["value"],
): SearchFilterNode[] {
  const previousCounts = new Map<string, number>();
  for (const node of collectSearchFilterNodesForFieldValue(previousFilter, field)) {
    const key = JSON.stringify(node);
    previousCounts.set(key, (previousCounts.get(key) ?? 0) + 1);
  }

  const addedNodes: SearchFilterNode[] = [];
  for (const node of collectSearchFilterNodesForFieldValue(nextFilter, field)) {
    const key = JSON.stringify(node);
    const previousCount = previousCounts.get(key) ?? 0;
    if (previousCount > 0) {
      previousCounts.set(key, previousCount - 1);
    } else {
      addedNodes.push(node);
    }
  }

  return addedNodes;
}

export function structuredDraftHostMutationContainsFieldValue(
  mutation: StructuredDraftHostMutation,
  field: Pf2eTerminalQueryFieldOption["value"],
): boolean {
  if (mutation.kind === "replaceGroupedField") {
    return mutation.field === field;
  }

  if (mutation.kind === "replaceNode") {
    return searchFilterNodeContainsFieldValue(mutation.node, field);
  }

  return mutation.nodes.some((node) => searchFilterNodeContainsFieldValue(node, field));
}

function getSearchFilterNodeEditorValue(mutation: StructuredDraftHostMutation): StructuredDraftExplorerPromptNodeValue {
  if (mutation.kind === "appendNodes") {
    return mutation.nodes;
  }

  if (mutation.kind === "replaceNode") {
    return mutation.node;
  }

  return null;
}

export function getQueryFieldValueForNode(node: SearchFilterNode): Pf2eTerminalQueryFieldOption["value"] | null {
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

export function getMetadataFilterNodeFieldValue(
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

export function canOpenStructuredDraftExactNodeFieldFallback({
  currentNode,
  fieldOption,
  path,
  query,
}: {
  currentNode: MetadataFilterNode | null;
  fieldOption: Pf2eTerminalQueryFieldOption;
  path: number[];
  query: Pf2eTerminalSearchQuery;
}): boolean {
  if (fieldOption.editor !== "sharedExplorer" || path.length === 0 || !currentNode) {
    return false;
  }

  const canonicalNode = getSearchFilterNodeAtPath(query.filter, path);
  if (!canonicalNode || getQueryFieldValueForNode(canonicalNode) !== fieldOption.value) {
    return false;
  }

  return JSON.stringify(canonicalFilterToMetadataNode(canonicalNode)) === JSON.stringify(currentNode);
}

export function inferMetricFieldFamily(
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

export function buildMetricSelectionTargetResolver(
  family: MetricFieldFamily,
  fieldLabel: string,
  options: { numericOnly?: boolean } = {},
): (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined {
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

export function extractMetricKeyFromSelectTargetOutcome(
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

export function useStructuredDraftExplorerActions({
  getScopedFieldOptions,
  openFilterExplorer,
  replaceStructuredDraftProjection,
  setStructuredDraftResumeTarget,
  terminal,
  user,
}: {
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openFilterExplorer: OpenSearchFilterExplorer;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}) {
  const openStructuredDraftExplorerContinuation = React.useCallback(
    async ({
      currentNode,
      fieldOption,
      buildHostMutation,
      initialDiscoveryMode,
      initialFieldState,
      onHostChange,
      preservedMetadata,
      query,
      resolveSelectionTarget,
      singleFieldBehavior,
      title,
    }: {
      currentNode: MetadataFilterNode | null;
      fieldOption: Pf2eTerminalQueryFieldOption;
      buildHostMutation?: (fieldState: SearchFilterExplorerFieldState) => StructuredDraftHostMutation;
      initialDiscoveryMode?: SearchFilterDiscoveryMode;
      initialFieldState?: SearchFilterExplorerFieldState;
      onHostChange?: (change: StructuredDraftContinuationChange) => void;
      preservedMetadata?: MetadataFilterNode | null;
      query: Pf2eTerminalSearchQuery;
      resolveSelectionTarget?: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
      singleFieldBehavior?: "list" | "directValues";
      title?: string;
    }) =>
      runStructuredDraftExplorerContinuation({
        currentNode,
        fieldOption,
        buildHostMutation,
        initialDiscoveryMode,
        initialFieldState,
        onHostChange,
        openFilterExplorer,
        preservedMetadata,
        query,
        resolveSelectionTarget,
        singleFieldBehavior,
        title,
        user,
      }),
    [openFilterExplorer, user],
  );

  const openLiveExplorerExactNodeFieldClauseFallback = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
    ) => {
      if (
        !canOpenStructuredDraftExactNodeFieldFallback({
          currentNode,
          fieldOption,
          path,
          query,
        })
      ) {
        return;
      }

      const liveChangeState = { saw: false };
      const applyChange = ({ mutation }: StructuredDraftContinuationChange) => {
        if (mutation.kind !== "replaceNode") {
          return;
        }
        if (mutation.node?.kind === "allOf" && path.length > 0 && mutation.node.children.length > 1) {
          const [firstNode, ...additionalNodes] = mutation.node.children;
          const groupPath = path.slice(0, -1);
          const replacedFilter = updateSearchFilterNodeAtPath(query.filter, path, () => firstNode);
          const nextFilter = appendSearchFilterNodesAtPath(
            replacedFilter,
            groupPath,
            additionalNodes,
            getSearchQueryRootOperator(query),
          );
          liveChangeState.saw = true;
          replaceStructuredDraftProjection(
            () => ({
              ...query,
              filter: nextFilter,
            }),
            { resumeTarget: createStructuredDraftGroupResumeTarget(groupPath) },
          );
          return;
        }
        const application = applyStructuredDraftHostMutationToQuery(query, mutation, {
          kind: "replaceNode",
          path,
        });
        if (!application) {
          return;
        }
        liveChangeState.saw = true;
        replaceStructuredDraftProjection(() => application.nextQuery, {
          resumeTarget: application.resumeTarget,
        });
      };
      setStructuredDraftResumeTarget(createStructuredDraftNodeResumeTarget(path));
      const preparedDraft = user.search.prepareFilterExplorerDraftFromMetadataNode(currentNode, [fieldOption.value]);
      const continuation = await openStructuredDraftExplorerContinuation({
        query,
        fieldOption,
        currentNode,
        initialFieldState: buildSearchFilterExplorerFieldState(preparedDraft.draft),
        preservedMetadata: preparedDraft.preservedMetadata,
        onHostChange: applyChange,
      });
      if (
        continuation.kind !== "notOpened" &&
        continuation.change &&
        !liveChangeState.saw &&
        continuation.change.mutation.kind === "replaceNode" &&
        JSON.stringify(canonicalFilterToMetadataNode(continuation.change.mutation.node ?? undefined)) !==
          JSON.stringify(currentNode)
      ) {
        applyChange(continuation.change);
      }
    },
    [openStructuredDraftExplorerContinuation, replaceStructuredDraftProjection, setStructuredDraftResumeTarget, user.search],
  );

  const openPromptFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
    ): Promise<StructuredDraftExplorerPromptNodeResult> => {
      const initialFieldState = buildSearchFilterExplorerFieldState(
        user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).draft,
      );
      const continuation = await openStructuredDraftExplorerContinuation({
        query,
        fieldOption,
        currentNode,
        initialFieldState,
      });
      if (continuation.kind === "cancel" || continuation.kind === "notOpened") {
        return structuredDraftPromptCancel();
      }
      const addedQueryFieldNodes = continuation.change
        ? getAddedSearchFilterNodesForFieldValue(query.filter, continuation.change.query.filter, fieldOption.value)
        : [];
      if (
        continuation.change &&
        !structuredDraftHostMutationContainsFieldValue(continuation.change.mutation, fieldOption.value) &&
        addedQueryFieldNodes.length > 0
      ) {
        return structuredDraftPromptApply(
          addedQueryFieldNodes.length === 1 ? addedQueryFieldNodes[0]! : addedQueryFieldNodes,
        );
      }
      if (
        !continuation.change ||
        (searchFilterExplorerFieldStatesEqual(continuation.change.fieldState, initialFieldState) &&
          !structuredDraftHostMutationContainsFieldValue(continuation.change.mutation, fieldOption.value))
      ) {
        return structuredDraftPromptBack();
      }

      return structuredDraftPromptApply(getSearchFilterNodeEditorValue(continuation.change.mutation));
    },
    [openStructuredDraftExplorerContinuation, user.search],
  );

  const selectPromptMetricKey = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      family: MetricFieldFamily,
      fieldOption: Pf2eTerminalQueryFieldOption,
      title: string,
      options: { numericOnly?: boolean } = {},
      initialDiscoveryMode: SearchFilterDiscoveryMode = "matching",
    ): Promise<StructuredDraftExplorerMetricKeyResult> => {
      const continuation = await openStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption,
        initialDiscoveryMode,
        query,
        resolveSelectionTarget: buildMetricSelectionTargetResolver(family, fieldOption.label, options),
        singleFieldBehavior: "list",
        title,
      });

      switch (continuation.kind) {
        case "selectTarget": {
          const metricKey = extractMetricKeyFromSelectTargetOutcome(continuation.outcome, family);
          return metricKey
            ? structuredDraftPromptApply({
                value: metricKey,
                discoveryMode: continuation.discoveryMode,
              })
            : structuredDraftPromptCancel();
        }
        case "resumeHost":
          return structuredDraftPromptBack();
        case "cancel":
        case "notOpened":
          return structuredDraftPromptCancel();
      }
    },
    [openStructuredDraftExplorerContinuation],
  );

  const openPromptPackClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "pack" }>,
    ): Promise<StructuredDraftExplorerPromptNodeResult> => {
      const baseQuery = setSearchQueryPackSelection(query, { include: [], exclude: [] });
      const seededQuery = currentNode
        ? setSearchQueryPackSelection(baseQuery, { include: [currentNode.value], exclude: [] })
        : baseQuery;
      const initialFieldState = buildSearchFilterExplorerFieldState(
        user.search.prepareFilterExplorerDraft(seededQuery, ["pack"]).draft,
      );

      const continuation = await openStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption: buildExplorerOnlyFieldOption(
          "pack",
          "Pack",
          "Browse live packs for the current scope and stage canonical pack clauses.",
          "enumString",
        ),
        initialFieldState,
        query: baseQuery,
        title: "Pack",
      });

      if (continuation.kind === "notOpened") {
        return structuredDraftPromptCancel();
      }
      if (continuation.kind !== "resumeHost" && continuation.kind !== "cancel") {
        return structuredDraftPromptCancel();
      }
      if (!continuation.change) {
        return currentNode ? structuredDraftPromptApply(null) : structuredDraftPromptBack();
      }

      const selection = continuation.change.fieldState.discreteSelections.pack
        ? {
            include: [...continuation.change.fieldState.discreteSelections.pack.include],
            exclude: [...continuation.change.fieldState.discreteSelections.pack.exclude],
          }
        : getSearchQueryPackSelection(baseQuery);
      const hasSelection = selection.include.length > 0 || selection.exclude.length > 0;
      if (!hasSelection) {
        return currentNode ? structuredDraftPromptApply(null) : structuredDraftPromptBack();
      }

      return structuredDraftPromptApply(buildSearchFilterPackSelectionNode(selection));
    },
    [openStructuredDraftExplorerContinuation, user.search],
  );

  const openPromptRarityClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      node?: Extract<SearchFilterNode, { kind: "rarity" }>,
    ): Promise<StructuredDraftExplorerPromptNodeResult> => {
      const baseQuery = setSearchQueryRaritySelection(query, { include: [], exclude: [] });
      const currentValue = node?.match.kind === "eq" ? node.match.value : null;
      const seededQuery = currentValue
        ? setSearchQueryRaritySelection(baseQuery, { include: [currentValue], exclude: [] })
        : baseQuery;
      const initialFieldState = buildSearchFilterExplorerFieldState(
        user.search.prepareFilterExplorerDraft(seededQuery, ["rarity"]).draft,
      );
      const continuation = await openStructuredDraftExplorerContinuation({
        currentNode: null,
        fieldOption: buildExplorerOnlyFieldOption(
          "rarity",
          "Rarity",
          "Browse live rarities for the current scope and stage canonical rarity clauses.",
          "enumString",
        ),
        initialFieldState,
        query: baseQuery,
      });
      if (continuation.kind === "notOpened") {
        return structuredDraftPromptCancel();
      }
      if (continuation.kind !== "resumeHost" && continuation.kind !== "cancel") {
        return structuredDraftPromptCancel();
      }
      if (!continuation.change) {
        return node ? structuredDraftPromptApply(null) : structuredDraftPromptBack();
      }

      const nextSelection = continuation.change.fieldState.discreteSelections.rarity
        ? {
            include: [...continuation.change.fieldState.discreteSelections.rarity.include],
            exclude: [...continuation.change.fieldState.discreteSelections.rarity.exclude],
          }
        : getSearchQueryRaritySelection(baseQuery);
      const hasSelection = nextSelection.include.length > 0 || nextSelection.exclude.length > 0;
      if (!hasSelection) {
        return node ? structuredDraftPromptApply(null) : structuredDraftPromptBack();
      }

      const nextNode = buildSearchFilterValueSelectionNode("rarity", nextSelection);
      return nextNode
        ? structuredDraftPromptApply(nextNode)
        : node
          ? structuredDraftPromptApply(null)
          : structuredDraftPromptBack();
    },
    [openStructuredDraftExplorerContinuation, user.search],
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
      const buildHostMutation = (fieldState: SearchFilterExplorerFieldState) =>
        buildStructuredDraftGroupedFieldMutation({ fieldOption, fieldState });
      const applyChange = ({ mutation }: StructuredDraftContinuationChange) => {
        if (mutation.kind !== "replaceGroupedField") {
          return;
        }
        liveChangeState.saw = true;
        const replacementNodes = buildGroupedFieldReplacementNodes(user.search, query, mutation.fieldState, fieldOption);
        const application = applyStructuredDraftHostMutationToQuery(query, mutation, {
          kind: "replaceGroupedField",
          groupPath,
          field,
          fieldMemberPaths,
          replacementNodes,
        });
        if (!application) {
          return;
        }
        replaceStructuredDraftProjection(() => application.nextQuery, {
          resumeTarget: application.resumeTarget,
        });
      };
      const continuation = await openStructuredDraftExplorerContinuation({
        query: seedQuery,
        fieldOption,
        currentNode: null,
        buildHostMutation,
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
        const buildHostMutation = (fieldState: SearchFilterExplorerFieldState) =>
          buildStructuredDraftGroupedFieldMutation({ fieldOption, fieldState });
        const applyChange = ({ mutation }: StructuredDraftContinuationChange) => {
          if (mutation.kind !== "replaceGroupedField") {
            return;
          }
          liveChangeState.saw = true;
          const replacementNodes = buildGroupedFieldReplacementNodes(user.search, query, mutation.fieldState, fieldOption);
          const application = applyStructuredDraftHostMutationToQuery(query, mutation, {
            kind: "replaceGroupedField",
            groupPath: [],
            field: fieldOption.value,
            fieldMemberPaths: [],
            replacementNodes,
            replaceRoot: true,
          });
          if (!application) {
            return;
          }
          replaceStructuredDraftProjection(() => application.nextQuery, {
            resumeTarget: application.resumeTarget,
          });
        };
        const continuation = await openStructuredDraftExplorerContinuation({
          query: seedQuery,
          fieldOption,
          currentNode: null,
          buildHostMutation,
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

  return {
    openLiveExplorerCanonicalFieldMember,
    openLiveExplorerExactNodeFieldClauseFallback,
    openLiveExplorerGroupedField,
    openLiveExplorerGroupFieldByName,
    openPromptFieldClause,
    openPromptPackClause,
    openPromptRarityClause,
    openStructuredDraftExplorerContinuation,
    selectPromptMetricKey,
  };
}
