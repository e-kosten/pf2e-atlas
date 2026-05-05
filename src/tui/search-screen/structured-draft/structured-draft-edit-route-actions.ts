import React from "react";

import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import { getSearchFilterNodeAtPath } from "../../search/query-core.js";
import { canonicalFilterToMetadataNode, metadataFilterNodeToCanonicalFilter } from "../../search/query-parts.js";
import { replaceSearchQueryRootScope } from "../../search/query-state.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
} from "../workspace/workspace-action-types.js";
import type { StructuredDraftHostMutation } from "./structured-draft-continuation.js";
import { applyStructuredDraftHostMutationToQuery } from "./structured-draft-host-mutations.js";
import type { StructuredDraftResumeTarget } from "./structured-draft-state.js";
import {
  createStructuredDraftGroupResumeTarget,
  createStructuredDraftResumeTargetForContainingGroup,
} from "./structured-draft-state.js";
import type {
  ClauseKind,
  SearchFilterNodeEditorResult,
} from "./structured-draft-prompt-actions.js";
import type { StructuredDraftEditRoute, StructuredDraftLeafKind } from "./structured-draft-edit-routes.js";

type ClauseApplyResult = "applied" | "back" | "cancelled";

function getClauseKindForLeafRoute(leafKind: StructuredDraftLeafKind): ClauseKind | null {
  switch (leafKind) {
    case "scope":
      return "scope";
    case "level":
      return "level";
    case "price":
      return "price";
    case "metadataScalar":
    case "metadataBoolean":
    case "metadataText":
      return "field";
    case "metric":
      return "metric";
    case "metricCompare":
      return "metricCompare";
    case "linksTo":
    case "linkedFrom":
      return null;
  }
}

function buildRouteBucketEntry(route: Extract<StructuredDraftEditRoute, { kind: "groupField" }>): SearchStructuredDraftEntry {
  return {
    kind: "queryFieldBucket",
    key: `route:${route.source}:${route.groupPath.join(".")}:${route.field}`,
    label: route.fieldOption.label,
    description: route.fieldOption.description,
    groupPath: route.groupPath,
    field: route.field,
    fieldOperator: "include",
    memberPaths: route.memberPaths,
    fieldMemberPaths: route.memberPaths,
    indent: route.groupPath.length + 1,
    menuLabel: route.fieldOption.label,
  };
}

function applyLeafReplacement({
  nextNode,
  path,
  query,
  replaceStructuredDraftProjection,
}: {
  nextNode: SearchFilterNode | null;
  path: number[];
  query: Pf2eTerminalSearchQuery;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
}): void {
  const application = applyStructuredDraftHostMutationToQuery(
    query,
    {
      kind: "replaceNode",
      node: nextNode,
    } satisfies StructuredDraftHostMutation,
    {
      kind: "replaceNode",
      path,
    },
  );
  if (!application) {
    return;
  }
  replaceStructuredDraftProjection(() => application.nextQuery, {
    resumeTarget: application.resumeTarget,
  });
}

function applyRootScopeReplacement({
  nextNode,
  query,
  replaceStructuredDraftProjection,
}: {
  nextNode: SearchFilterNode | null;
  query: Pf2eTerminalSearchQuery;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
}): void {
  if (nextNode && nextNode.kind !== "scope") {
    return;
  }
  const nextQuery = replaceSearchQueryRootScope(query, nextNode);
  replaceStructuredDraftProjection(() => nextQuery, {
    resumeTarget: nextQuery.filter
      ? createStructuredDraftResumeTargetForContainingGroup(nextQuery.filter, [])
      : createStructuredDraftGroupResumeTarget([]),
  });
}

async function promptForRecordLinkLeaf({
  prompts,
  route,
  currentNode,
}: {
  prompts: SearchWorkspacePromptAdapters;
  route: Extract<StructuredDraftEditRoute, { kind: "leaf" }>;
  currentNode: SearchFilterNode | undefined;
}): Promise<SearchFilterNodeEditorResult> {
  if (route.leafKind !== "linksTo" && route.leafKind !== "linkedFrom") {
    return { kind: "cancel" };
  }
  const currentValue =
    route.leafKind === "linksTo" && currentNode?.kind === "linksTo"
      ? currentNode.target
      : route.leafKind === "linkedFrom" && currentNode?.kind === "linkedFrom"
        ? currentNode.source
        : "";
  const value = await prompts.promptTextInput({
    title: route.leafKind === "linksTo" ? "Links To" : "Linked From",
    prompt: "Enter a canonical record key. Leave blank to clear this clause.",
    defaultValue: currentValue,
  });
  if (value === undefined) {
    return { kind: "cancel" };
  }
  const trimmed = value.trim();
  if (!trimmed) {
    return { kind: "apply", value: null };
  }
  return {
    kind: "apply",
    value: route.leafKind === "linksTo" ? { kind: "linksTo", target: trimmed } : { kind: "linkedFrom", source: trimmed },
  };
}

export function useStructuredDraftEditRouteActions({
  editFieldClause,
  openLiveExplorerGroupedField,
  promptForClauseNode,
  prompts,
  replaceStructuredDraftProjection,
  terminal,
}: {
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  openLiveExplorerGroupedField: (
    query: Pf2eTerminalSearchQuery,
    entry: SearchStructuredDraftEntry,
  ) => Promise<void>;
  promptForClauseNode: (
    promptSession: SearchWorkspacePromptAdapters,
    query: Pf2eTerminalSearchQuery,
    clauseKind: ClauseKind,
    currentNode?: SearchFilterNode,
  ) => Promise<SearchFilterNodeEditorResult>;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  terminal: SearchWorkspaceTerminal;
}) {
  const executeStructuredDraftEditRoute = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      route: StructuredDraftEditRoute,
    ): Promise<ClauseApplyResult> => {
      if (route.kind === "unsupported") {
        await terminal.pauseForAnyKey(route.reason);
        return "cancelled";
      }

      if (route.kind === "groupField") {
        await openLiveExplorerGroupedField(query, buildRouteBucketEntry(route));
        return "applied";
      }

      const currentNode = route.path ? (getSearchFilterNodeAtPath(query.filter, route.path) ?? undefined) : undefined;
      if (route.path && (route.leafKind === "linksTo" || route.leafKind === "linkedFrom")) {
        const nextNode = await terminal.runPromptSession((session) =>
          promptForRecordLinkLeaf({ prompts: session, route, currentNode }),
        );
        if (nextNode.kind !== "apply" || Array.isArray(nextNode.value)) {
          return "cancelled";
        }
        applyLeafReplacement({ nextNode: nextNode.value, path: route.path, query, replaceStructuredDraftProjection });
        return "applied";
      }

      const clauseKind = getClauseKindForLeafRoute(route.leafKind);
      if (!clauseKind) {
        await terminal.pauseForAnyKey("That clause cannot be edited through the current canonical editor set.");
        return "cancelled";
      }

      if (
        route.path &&
        route.fieldOption &&
        (route.leafKind === "metadataScalar" ||
          route.leafKind === "metadataBoolean" ||
          route.leafKind === "metadataText")
      ) {
        const currentMetadataNode = currentNode ? canonicalFilterToMetadataNode(currentNode) : null;
        const nextMetadataNode = await editFieldClause(query, route.fieldOption, currentMetadataNode);
        if (nextMetadataNode === undefined) {
          return "cancelled";
        }
        const nextNode = nextMetadataNode ? (metadataFilterNodeToCanonicalFilter(nextMetadataNode) ?? null) : null;
        applyLeafReplacement({ nextNode, path: route.path, query, replaceStructuredDraftProjection });
        return "applied";
      }

      const nextNode = await terminal.runPromptSession((session) =>
        promptForClauseNode(session, query, clauseKind, currentNode),
      );
      if (nextNode.kind === "back") {
        return "back";
      }
      if (nextNode.kind !== "apply" || Array.isArray(nextNode.value)) {
        return "cancelled";
      }
      if (route.placement === "rootSingleton" && route.leafKind === "scope") {
        applyRootScopeReplacement({ nextNode: nextNode.value, query, replaceStructuredDraftProjection });
        return "applied";
      }
      if (!route.path) {
        return "cancelled";
      }
      applyLeafReplacement({ nextNode: nextNode.value, path: route.path, query, replaceStructuredDraftProjection });
      return "applied";
    },
    [
      editFieldClause,
      openLiveExplorerGroupedField,
      promptForClauseNode,
      replaceStructuredDraftProjection,
      terminal,
    ],
  );

  return {
    executeStructuredDraftEditRoute,
    prompts,
  };
}
