import React from "react";

import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import type { DerivedTagTerminalActionTargetOption } from "../../action-target.js";
import {
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
import { canonicalFilterToMetadataNode } from "../../search/query-parts.js";
import { getSearchQueryRootOperator } from "../../search/query-state.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
} from "../workspace/workspace-action-types.js";
import { applyStructuredDraftHostMutationToQuery } from "./structured-draft-host-mutations.js";
import {
  classifyStructuredDraftAddFieldRoute,
  classifyStructuredDraftNodeEditRoute,
  getStructuredDraftQueryFieldValueForNode,
  getStructuredDraftSyntheticFieldOption,
  resolveStructuredDraftFieldOption,
  type StructuredDraftEditRoute,
} from "./structured-draft-edit-routes.js";
import {
  isMetricFieldOptionValue,
  type ClauseKind,
  type ClausePromptResult,
  type SearchFilterNodeEditorResult,
  type SharedExplorerFieldOptionPromptResult,
} from "./structured-draft-prompt-actions.js";
import {
  createStructuredDraftGroupResumeTarget,
  type StructuredDraftResumeTarget,
} from "./structured-draft-state.js";

type ClauseApplyResult = "applied" | "back" | "cancelled";

export type StructuredDraftEntryActionId =
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

type BooleanGroupNode = Extract<
  SearchFilterNode,
  { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }
>;

export function formatFriendlyGroupLabel(kind: "allOf" | "anyOf" | "not"): string {
  switch (kind) {
    case "allOf":
      return "All of";
    case "anyOf":
      return "Any of";
    case "not":
      return "Exclude";
  }
}

export function buildInsertionActionEntries(
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

export function buildRootActionEntries(
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

export function getStructuredDraftLeafActionEntries(
  query: Pf2eTerminalSearchQuery,
  path: number[],
  node: SearchFilterNode,
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[],
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  const editableClauseKind = getEditableClauseKind(query, path, node, getScopedFieldOptions);
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
}

export function getStructuredDraftNotActionEntries(
  query: Pf2eTerminalSearchQuery,
  path: number[],
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
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
}

export function getStructuredDraftGroupActionEntries(
  query: Pf2eTerminalSearchQuery,
  path: number[],
  node: BooleanGroupNode,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
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
}

function getEditableClauseKind(
  query: Pf2eTerminalSearchQuery,
  path: number[],
  node: SearchFilterNode,
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[],
): ClauseKind | null {
  const fieldOptions = getScopedFieldOptions(query);
  const fieldOptionValue = getStructuredDraftQueryFieldValueForNode(node);
  const fieldOption = fieldOptionValue
    ? resolveStructuredDraftFieldOption(fieldOptionValue, fieldOptions)
    : null;
  const editableMetadataNode = canonicalFilterToMetadataNode(node);
  const route = classifyStructuredDraftNodeEditRoute({ query, path, fieldOptions });
  if (route.kind === "unsupported") {
    return null;
  }
  return route.kind === "groupField"
    ? "field"
    : node.kind === "scope"
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
}

export function useStructuredDraftStructuralActions({
  clearStructuredDraftMoveSource,
  enterStructuredDraftMoveMode,
  getScopedFieldOptions,
  moveSourcePath,
  executeStructuredDraftEditRoute,
  promptForClauseKind,
  promptForClauseNode,
  promptForSharedExplorerFieldOption,
  prompts,
  replaceStructuredDraftProjection,
  setStructuredDraftResumeTarget,
  terminal,
}: {
  clearStructuredDraftMoveSource: () => void;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  moveSourcePath: number[] | null;
  executeStructuredDraftEditRoute: (
    query: Pf2eTerminalSearchQuery,
    route: StructuredDraftEditRoute,
  ) => Promise<ClauseApplyResult>;
  promptForClauseKind: (
    promptSession: SearchWorkspacePromptAdapters,
    query: Pf2eTerminalSearchQuery,
  ) => Promise<ClausePromptResult>;
  promptForClauseNode: (
    promptSession: SearchWorkspacePromptAdapters,
    query: Pf2eTerminalSearchQuery,
    clauseKind: ClauseKind,
    currentNode?: SearchFilterNode,
  ) => Promise<SearchFilterNodeEditorResult>;
  promptForSharedExplorerFieldOption: (
    promptSession: SearchWorkspacePromptAdapters,
    query: Pf2eTerminalSearchQuery,
  ) => Promise<SharedExplorerFieldOptionPromptResult>;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { resumeTarget?: StructuredDraftResumeTarget | null },
  ) => void;
  setStructuredDraftResumeTarget: (target: StructuredDraftResumeTarget | null) => void;
  terminal: SearchWorkspaceTerminal;
}) {
  const applyNextTree = React.useCallback(
    (nextFilter: SearchFilterNode | undefined) => {
      replaceStructuredDraftProjection((draftQuery) => ({
        ...draftQuery,
        filter: nextFilter,
      }));
    },
    [replaceStructuredDraftProjection],
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
          if (clauseKind.kind === "back") {
            return "back";
          }
          if (clauseKind.kind === "cancel") {
            return "cancelled";
          }
          if (clauseKind.value === "field" && wrapper === undefined) {
            const fieldOption = await promptForSharedExplorerFieldOption(session, workingQuery);
            switch (fieldOption.kind) {
              case "apply":
                if (fieldOption.value) {
                  return executeStructuredDraftEditRoute(
                    workingQuery,
                    classifyStructuredDraftAddFieldRoute({ fieldOption: fieldOption.value, groupPath: path, query: workingQuery }),
                  );
                }
                break;
              case "back":
                continue;
              case "cancel":
                return "cancelled";
            }
          }
          if (
            wrapper === undefined &&
            (clauseKind.value === "pack" || clauseKind.value === "rarity" || clauseKind.value === "actionCost")
          ) {
            const fieldOption = getStructuredDraftSyntheticFieldOption(clauseKind.value);
            if (fieldOption) {
              return executeStructuredDraftEditRoute(
                workingQuery,
                classifyStructuredDraftAddFieldRoute({ fieldOption, groupPath: path, query: workingQuery }),
              );
            }
          }
          if (
            wrapper !== undefined &&
            (clauseKind.value === "pack" || clauseKind.value === "rarity" || clauseKind.value === "actionCost")
          ) {
            await terminal.pauseForAnyKey(
              "Grouped query fields must be added directly to an existing group before structural wrapping.",
            );
            return "cancelled";
          }
          const nextNode = await promptForClauseNode(session, workingQuery, clauseKind.value);
          if (nextNode.kind === "back") {
            continue;
          }
          if (nextNode.kind === "cancel") {
            return "cancelled";
          }
          const nextNodeValue = nextNode.value;
          if (nextNodeValue === null) {
            return "cancelled";
          }
          const wrappedGroupChildren =
            wrapper === "allOf" || wrapper === "anyOf"
              ? Array.isArray(nextNodeValue)
                ? nextNodeValue
                : nextNodeValue.kind === wrapper
                  ? nextNodeValue.children
                  : [nextNodeValue]
              : [];
          const wrappedNode =
            wrapper === "allOf" || wrapper === "anyOf"
              ? ({
                  kind: wrapper,
                  children: wrappedGroupChildren,
                } as SearchFilterNode)
              : wrapper === "not"
                ? ({
                    kind: "not",
                    child: Array.isArray(nextNodeValue)
                      ? ({ kind: "allOf", children: nextNodeValue } as SearchFilterNode)
                      : nextNodeValue,
                  } as SearchFilterNode)
                : nextNodeValue;
          const application = applyStructuredDraftHostMutationToQuery(
            workingQuery,
            {
              kind: "appendNodes",
              nodes: Array.isArray(wrappedNode) ? wrappedNode : [wrappedNode],
            },
            {
              kind: "appendNodes",
              groupPath: path,
              flattenMatchingBooleanGroup: wrapper === undefined,
            },
          );
          if (application) {
            replaceStructuredDraftProjection(() => application.nextQuery, {
              resumeTarget: application.resumeTarget,
            });
          }
          return "applied";
        }
      });
    },
    [
      executeStructuredDraftEditRoute,
      promptForClauseKind,
      promptForClauseNode,
      promptForSharedExplorerFieldOption,
      replaceStructuredDraftProjection,
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
          setStructuredDraftResumeTarget(createStructuredDraftGroupResumeTarget(path));
          return;
        }
        const insertionResult = await runInsertionAction(query, path, result.value);
        if (insertionResult !== "back") {
          return;
        }
        setStructuredDraftResumeTarget(createStructuredDraftGroupResumeTarget(path));
      }
    },
    [moveSourcePath, prompts, runInsertionAction, setStructuredDraftResumeTarget],
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
    [prompts, runRootAction],
  );

  const getLeafActionEntries = React.useCallback(
    (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode) =>
      getStructuredDraftLeafActionEntries(query, path, node, getScopedFieldOptions),
    [getScopedFieldOptions],
  );

  const runLeafAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: SearchFilterNode,
      actionId: StructuredDraftEntryActionId,
    ) => {
      const editableClauseKind = getEditableClauseKind(query, path, node, getScopedFieldOptions);

      if (actionId === "edit") {
        if (!editableClauseKind) {
          await terminal.pauseForAnyKey("That clause cannot be edited through the current canonical editor set.");
          return;
        }
        await executeStructuredDraftEditRoute(
          query,
          classifyStructuredDraftNodeEditRoute({
            fieldOptions: getScopedFieldOptions(query),
            path,
            query,
          }),
        );
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
      executeStructuredDraftEditRoute,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      terminal,
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
    [getLeafActionEntries, prompts, runLeafAction],
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
      const entries = getStructuredDraftNotActionEntries(query, path);
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
    [prompts, runNotAction],
  );

  const getGroupActionEntries = React.useCallback(
    (query: Pf2eTerminalSearchQuery, path: number[], node: BooleanGroupNode) =>
      getStructuredDraftGroupActionEntries(query, path, node),
    [],
  );

  const runGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: BooleanGroupNode,
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
    async (query: Pf2eTerminalSearchQuery, path: number[], node: BooleanGroupNode) => {
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
    [getGroupActionEntries, prompts, runGroupAction],
  );

  const editStructuredDraftStructuralEntry = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, entry: SearchStructuredDraftEntry) => {
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(query.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await promptForGroupAction(query, rootPath, rootNode);
            return true;
          }
        }
        await promptForRootAction(query);
        return true;
      }
      if (entry.kind === "queryInsertionSlot") {
        if (moveSourcePath) {
          applyNextTree(
            moveSearchFilterNodeToGroupPath(
              query.filter,
              moveSourcePath,
              entry.insertionPath ?? [],
              getSearchQueryRootOperator(query),
            ),
          );
          clearStructuredDraftMoveSource();
          return true;
        }
        await runInsertionAction(query, entry.insertionPath ?? [], "addClause");
        return true;
      }
      if (entry.kind === "queryFieldBucket") {
        return false;
      }

      const node = getSearchFilterNodeAtPath(query.filter, entry.treePath ?? []);
      if (!node) {
        return true;
      }
      if (node.kind === "not") {
        await promptForNotAction(query, entry.treePath ?? [], node);
        return true;
      }
      if (isSearchFilterBooleanGroup(node)) {
        await promptForGroupAction(query, entry.treePath ?? [], node);
        return true;
      }
      await promptForLeafAction(query, entry.treePath ?? [], node);
      return true;
    },
    [
      applyNextTree,
      clearStructuredDraftMoveSource,
      moveSourcePath,
      promptForGroupAction,
      promptForLeafAction,
      promptForNotAction,
      promptForRootAction,
      runInsertionAction,
    ],
  );

  const getStructuredDraftStructuralEntryActions = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      entry: SearchStructuredDraftEntry | null | undefined,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] | null => {
      if (!entry) {
        return [];
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(query.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            return getGroupActionEntries(query, rootPath, rootNode);
          }
        }
        return buildRootActionEntries(query);
      }
      if (entry.kind === "queryInsertionSlot") {
        return buildInsertionActionEntries(Boolean(moveSourcePath));
      }
      if (entry.kind === "queryFieldBucket") {
        return null;
      }

      const node = getSearchFilterNodeAtPath(query.filter, entry.treePath ?? []);
      if (!node) {
        return [];
      }
      if (node.kind === "not") {
        return getStructuredDraftNotActionEntries(query, entry.treePath ?? []);
      }
      if (isSearchFilterBooleanGroup(node)) {
        return getGroupActionEntries(query, entry.treePath ?? [], node);
      }
      return getLeafActionEntries(query, entry.treePath ?? [], node);
    },
    [getGroupActionEntries, getLeafActionEntries, moveSourcePath],
  );

  const runStructuredDraftStructuralEntryAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      entry: SearchStructuredDraftEntry | null | undefined,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (!entry) {
        return true;
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(query.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await runGroupAction(query, rootPath, rootNode, actionId);
            return true;
          }
        }
        await runRootAction(query, actionId);
        return true;
      }
      if (entry.kind === "queryInsertionSlot") {
        await runInsertionAction(query, entry.insertionPath ?? [], actionId);
        return true;
      }
      if (entry.kind === "queryFieldBucket") {
        return false;
      }

      const path = entry.treePath ?? [];
      const node = getSearchFilterNodeAtPath(query.filter, path);
      if (!node) {
        return true;
      }
      if (node.kind === "not") {
        runNotAction(query, path, node, actionId);
        return true;
      }
      if (isSearchFilterBooleanGroup(node)) {
        await runGroupAction(query, path, node, actionId);
        return true;
      }
      await runLeafAction(query, path, node, actionId);
      return true;
    },
    [runGroupAction, runInsertionAction, runLeafAction, runNotAction, runRootAction],
  );

  return {
    addQueryClauseAtPath,
    editStructuredDraftStructuralEntry,
    getGroupActionEntries,
    getLeafActionEntries,
    getStructuredDraftStructuralEntryActions,
    promptForGroupAction,
    promptForInsertionAction,
    promptForLeafAction,
    promptForNotAction,
    promptForRootAction,
    runGroupAction,
    runInsertionAction,
    runLeafAction,
    runNotAction,
    runRootAction,
    runStructuredDraftStructuralEntryAction,
  };
}
