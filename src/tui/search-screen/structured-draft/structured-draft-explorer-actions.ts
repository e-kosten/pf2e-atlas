import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type { OntologyNode } from "../../../domain/ontology-types.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import { getSearchQueryCategory } from "../../search/query-state.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
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
  runStructuredDraftExplorerChildSurface,
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
  buildGroupedFieldSeedState,
} from "./structured-draft-grouped-field.js";
import { applyStructuredDraftHostMutationToQuery } from "./structured-draft-host-mutations.js";
import type { StructuredDraftResumeTarget } from "./structured-draft-state.js";
import {
  getStructuredDraftSyntheticFieldOption,
  inferStructuredDraftMetricFieldFamily,
  isStructuredDraftGroupFieldOption,
  structuredDraftSearchFilterNodeContainsFieldValue,
} from "./structured-draft-edit-routes.js";
import { metadataFilterNodeToCanonicalFilter } from "../../search/query-parts.js";

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

  return structuredDraftSearchFilterNodeContainsFieldValue(node, field);
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

export function inferMetricFieldFamily(
  metric: string,
  category: ReturnType<typeof getSearchQueryCategory> = null,
): MetricFieldFamily {
  return inferStructuredDraftMetricFieldFamily(metric, category);
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
  terminal,
  user,
}: {
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openFilterExplorer: OpenSearchFilterExplorer;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}) {
  const openStructuredDraftExplorerContinuation = React.useCallback(
    async ({
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
      fieldOption: Pf2eTerminalQueryFieldOption;
      buildHostMutation: (fieldState: SearchFilterExplorerFieldState) => StructuredDraftHostMutation;
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

  const openStructuredDraftExplorerChildSurface = React.useCallback(
    async ({
      fieldOption,
      initialDiscoveryMode,
      initialFieldState,
      preservedMetadata,
      query,
      resolveSelectionTarget,
      singleFieldBehavior,
      title,
    }: {
      fieldOption: Pf2eTerminalQueryFieldOption;
      initialDiscoveryMode?: SearchFilterDiscoveryMode;
      initialFieldState?: SearchFilterExplorerFieldState;
      preservedMetadata?: MetadataFilterNode | null;
      query: Pf2eTerminalSearchQuery;
      resolveSelectionTarget?: (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined;
      singleFieldBehavior?: "list" | "directValues";
      title?: string;
    }) =>
      runStructuredDraftExplorerChildSurface({
        fieldOption,
        initialDiscoveryMode,
        initialFieldState,
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

  const openPromptFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
    ): Promise<StructuredDraftExplorerPromptNodeResult> => {
      if (isStructuredDraftGroupFieldOption(fieldOption)) {
        await terminal.pauseForAnyKey("That grouped field must be edited through the structured field route.");
        return structuredDraftPromptCancel();
      }
      const initialFieldState = buildSearchFilterExplorerFieldState(
        user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).draft,
      );
      const continuation = await openStructuredDraftExplorerChildSurface({
        query,
        fieldOption,
        initialFieldState,
      });
      if (continuation.kind === "cancel" || continuation.kind === "notOpened") {
        return structuredDraftPromptCancel();
      }
      const insertionResult = user.search.buildFilterExplorerInsertionResult(
        buildSearchFilterExplorerComposeDraft(continuation.fieldState),
        {
          preservedMetadata: user.search.prepareFilterExplorerDraft(query, [fieldOption.value]).preservedMetadata,
          preferReplace: currentNode !== null,
        },
      );
      const mutation =
        insertionResult.kind === "insert"
          ? ({
              kind: "appendNodes",
              nodes: insertionResult.nodes
                .map((node) => metadataFilterNodeToCanonicalFilter(node))
                .filter((node): node is SearchFilterNode => Boolean(node)),
            } satisfies StructuredDraftHostMutation)
          : ({
              kind: "replaceNode",
              node: insertionResult.node ? (metadataFilterNodeToCanonicalFilter(insertionResult.node) ?? null) : null,
            } satisfies StructuredDraftHostMutation);
      const addedQueryFieldNodes = getAddedSearchFilterNodesForFieldValue(
        query.filter,
        continuation.query.filter,
        fieldOption.value,
      );
      if (
        !structuredDraftHostMutationContainsFieldValue(mutation, fieldOption.value) &&
        addedQueryFieldNodes.length > 0
      ) {
        return structuredDraftPromptApply(
          addedQueryFieldNodes.length === 1 ? addedQueryFieldNodes[0]! : addedQueryFieldNodes,
        );
      }
      if (
        searchFilterExplorerFieldStatesEqual(continuation.fieldState, initialFieldState) &&
        !structuredDraftHostMutationContainsFieldValue(mutation, fieldOption.value)
      ) {
        return structuredDraftPromptBack();
      }

      return structuredDraftPromptApply(getSearchFilterNodeEditorValue(mutation));
    },
    [openStructuredDraftExplorerChildSurface, terminal, user.search],
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
      const continuation = await openStructuredDraftExplorerChildSurface({
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
    [openStructuredDraftExplorerChildSurface],
  );

  const openLiveExplorerGroupedField = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, entry: SearchStructuredDraftEntry) => {
      const groupPath = entry.groupPath ?? [];
      const fieldMemberPaths = entry.fieldMemberPaths ?? entry.memberPaths ?? [];
      const field = entry.field as Pf2eTerminalQueryFieldOption["value"] | undefined;
      if (!field) {
        return;
      }

      const fieldOption =
        getStructuredDraftSyntheticFieldOption(field) ??
        getScopedFieldOptions(query).find((candidate) => candidate.value === field) ??
        null;
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

  return {
    openLiveExplorerGroupedField,
    openPromptFieldClause,
    openStructuredDraftExplorerContinuation,
    selectPromptMetricKey,
  };
}
