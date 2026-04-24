import React from "react";

import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import type { SearchFilterNode } from "../../../domain/search-request-types.js";
import {
  canLiftSearchFilterNodeAtPath,
  canUnwrapSearchFilterNodeAtPath,
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  liftSearchFilterNodeAtPath,
  moveSearchFilterNodeToGroupPath,
  reshapeSearchFilterBooleanGroupAtPath,
  toggleSearchFilterRootGroupOperator,
  updateSearchFilterNodeAtPath,
  unwrapSearchFilterNodeAtPath,
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

function getQueryFieldValueForNode(node: SearchFilterNode): Pf2eTerminalQueryFieldOption["value"] | null {
  switch (node.kind) {
    case "metadataPredicate":
      return node.predicate.field;
    case "metric":
      return node.metric.startsWith("attributes.") || node.metric.startsWith("speed.") ? "actorMetric" : "itemMetric";
    case "metricCompare":
      return node.leftMetric.startsWith("attributes.") || node.leftMetric.startsWith("speed.")
        ? "actorMetric"
        : "itemMetric";
    default:
      return null;
  }
}

export function useSearchStructuredDraftMetadataActions({
  appendStructuredDraftMetadataNode,
  clearStructuredDraftMoveSource,
  chooseQueryField,
  editFieldClause,
  enterStructuredDraftMoveMode,
  getExplorerBackedFieldOptions,
  getScopedFieldOptions,
  openOntologyFieldEditor,
  openOntologyFieldExplorer,
  openQueryFieldBuilder,
  moveSourcePath,
  prompts,
  replaceStructuredDraftProjection,
  structuredDraftQuery,
  terminal,
  updateStructuredDraftMetadataNode,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  clearStructuredDraftMoveSource: () => void;
  chooseQueryField: (query: Pf2eTerminalSearchQuery) => Promise<Pf2eTerminalQueryFieldOption | null>;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  enterStructuredDraftMoveMode: (path: number[]) => void;
  getExplorerBackedFieldOptions: (fieldOptions: Pf2eTerminalQueryFieldOption[]) => Pf2eTerminalQueryFieldOption[];
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openOntologyFieldEditor: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode: MetadataFilterNode | null,
    onApply: (nextNode: MetadataFilterNode | null) => void,
    onReturn?: () => void,
  ) => Promise<boolean>;
  openOntologyFieldExplorer: (
    query: Pf2eTerminalSearchQuery,
    fieldOptions: Pf2eTerminalQueryFieldOption[],
    onApply: (nextNode: MetadataFilterNode | null) => void,
  ) => Promise<boolean>;
  openQueryFieldBuilder: (query: Pf2eTerminalSearchQuery, path?: number[]) => Promise<boolean>;
  moveSourcePath: number[] | null;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
}): {
  editStructuredDraftMetadata: (entry: SearchStructuredDraftEntry) => Promise<void>;
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

  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = []) => {
      const fieldOptions = getScopedFieldOptions(query);
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
        return;
      }

      const explorerFieldOptions = getExplorerBackedFieldOptions(fieldOptions);
      if (explorerFieldOptions.length > 0) {
        if (
          await openOntologyFieldExplorer(query, explorerFieldOptions, (nextNode) => {
            if (nextNode) {
              appendStructuredDraftMetadataNode(path, nextNode);
            }
          })
        ) {
          return;
        }
        await terminal.pauseForAnyKey("No shared explorer is available for the selected field.");
        return;
      }

      await openQueryFieldBuilder(query, path);
    },
    [
      appendStructuredDraftMetadataNode,
      getExplorerBackedFieldOptions,
      getScopedFieldOptions,
      openOntologyFieldExplorer,
      openQueryFieldBuilder,
      terminal,
    ],
  );

  const addQueryGroupAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], groupKind: "and" | "or" | "not") => {
      const wrapNode = (nextNode: MetadataFilterNode): MetadataFilterNode =>
        groupKind === "and" ? { and: [nextNode] } : groupKind === "or" ? { or: [nextNode] } : { not: nextNode };

      const fieldOptions = getScopedFieldOptions(query);
      const explorerFieldOptions = getExplorerBackedFieldOptions(fieldOptions);
      if (explorerFieldOptions.length > 0) {
        if (
          await openOntologyFieldExplorer(query, explorerFieldOptions, (nextNode) => {
            if (nextNode) {
              appendStructuredDraftMetadataNode(path, wrapNode(nextNode));
            }
          })
        ) {
          return;
        }
        await terminal.pauseForAnyKey("No shared explorer is available for the selected field.");
        return;
      }

      const fieldOption = await chooseQueryField(query);
      if (!fieldOption) {
        return;
      }
      if (fieldOption.editor === "sharedExplorer") {
        await openOntologyFieldEditor(query, fieldOption, null, (nextNode) => {
          if (nextNode) {
            appendStructuredDraftMetadataNode(path, wrapNode(nextNode));
          }
        });
        return;
      }

      const clause = await editFieldClause(query, fieldOption);
      if (clause) {
        appendStructuredDraftMetadataNode(path, wrapNode(clause));
      }
    },
    [
      appendStructuredDraftMetadataNode,
      chooseQueryField,
      editFieldClause,
      getExplorerBackedFieldOptions,
      getScopedFieldOptions,
      openOntologyFieldEditor,
      openOntologyFieldExplorer,
      terminal,
    ],
  );

  const promptForInsertionAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[]) => {
      const result = await prompts.promptSelectOption({
        title: "Insertion Slot",
        prompt: "Choose what to add at this insertion slot",
        entries: [
          { value: "addClause", label: "Add Filter", description: "Append a new filter at this group bottom." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
        ],
        selectedValue: "addClause",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "addClause") {
        await addQueryClauseAtPath(query, path);
        return;
      }

      await addQueryGroupAtPath(
        query,
        path,
        result.value === "addAndGroup" ? "and" : result.value === "addOrGroup" ? "or" : "not",
      );
    },
    [addQueryClauseAtPath, addQueryGroupAtPath, prompts],
  );

  const promptForRootAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery) => {
      const result = await prompts.promptSelectOption({
        title: "Root Group",
        prompt: "Choose how to update the visible root group",
        entries: [
          { value: "addClause", label: "Add Filter", description: "Append a new top-level filter." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
          ...(query.filter
            ? [
                {
                  value: "toggle",
                  label: getSearchQueryRootOperator(query) === "anyOf" ? "Change Root To allOf" : "Change Root To anyOf",
                  description: "Reshape the visible root group without changing its current children.",
                },
              ]
            : []),
        ],
        selectedValue: "addClause",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "addClause") {
        await addQueryClauseAtPath(query, []);
        return;
      }

      if (result.value === "toggle") {
        applyNextTree(toggleSearchFilterRootGroupOperator(query.filter));
        return;
      }

      await addQueryGroupAtPath(
        query,
        [],
        result.value === "addAndGroup" ? "and" : result.value === "addOrGroup" ? "or" : "not",
      );
    },
    [addQueryClauseAtPath, addQueryGroupAtPath, applyNextTree, prompts],
  );

  const promptForLeafAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode) => {
      const fieldOptions = getScopedFieldOptions(query);
      const fieldOptionValue = getQueryFieldValueForNode(node);
      const fieldOption = fieldOptionValue
        ? fieldOptions.find((candidate) => candidate.value === fieldOptionValue) ?? null
        : null;
      const editableMetadataNode = canonicalFilterToMetadataNode(node);
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      const result = await prompts.promptSelectOption({
        title: "Query Clause",
        prompt: "Choose how to update this staged query clause",
        entries: [
          ...(fieldOption && editableMetadataNode
            ? [{ value: "edit", label: "Edit Clause", description: "Change the field operator or value." }]
            : []),
          { value: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
          { value: "wrapAnd", label: "Wrap In allOf", description: "Place this clause inside a new allOf group." },
          { value: "wrapOr", label: "Wrap In anyOf", description: "Place this clause inside a new anyOf group." },
          { value: "move", label: "Move Node", description: "Move this clause to another visible insertion slot." },
          ...(canLift
            ? [{ value: "lift", label: "Lift Node", description: "Lift this clause out of its current boolean group." }]
            : []),
          { value: "remove", label: "Remove Clause", description: "Delete this clause from the staged query." },
        ],
        selectedValue: fieldOption && editableMetadataNode ? "edit" : "wrapNot",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "edit") {
        if (!fieldOption || !editableMetadataNode) {
          await terminal.pauseForAnyKey("That clause cannot be edited through the current field-specific editor.");
          return;
        }
        if (fieldOption.editor === "sharedExplorer") {
          await openOntologyFieldEditor(query, fieldOption, editableMetadataNode, (nextNode) => {
            updateStructuredDraftMetadataNode(path, () => nextNode);
          });
          return;
        }
        const nextNode = await editFieldClause(query, fieldOption, editableMetadataNode);
        if (nextNode !== undefined) {
          updateStructuredDraftMetadataNode(path, () => nextNode);
        }
        return;
      }

      if (result.value === "wrapNot" || result.value === "wrapAnd" || result.value === "wrapOr") {
        applyNextTree(
          wrapSearchFilterNodeAtPath(query.filter, path, result.value === "wrapNot" ? "not" : result.value === "wrapAnd" ? "allOf" : "anyOf"),
        );
        return;
      }

      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }

      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }

      applyNextTree(
        path.length === 0
          ? undefined
          : updateSearchFilterNodeAtPath(query.filter, path, () => undefined),
      );
    },
    [
      applyNextTree,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      openOntologyFieldEditor,
      prompts,
      terminal,
      updateStructuredDraftMetadataNode,
    ],
  );

  const promptForNotAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: Extract<SearchFilterNode, { kind: "not" }>) => {
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      const result = await prompts.promptSelectOption({
        title: "NOT Group",
        prompt: "Choose how to update this negated staged clause",
        entries: [
          { value: "unwrap", label: "Remove NOT", description: "Keep the child clause and remove the negation." },
          { value: "move", label: "Move Node", description: "Move this NOT group to another visible insertion slot." },
          ...(canLift
            ? [{ value: "lift", label: "Lift Node", description: "Lift this NOT group out of its current parent group." }]
            : []),
          { value: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
        ],
        selectedValue: "unwrap",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "unwrap") {
        applyNextTree(path.length === 0 ? node.child : updateSearchFilterNodeAtPath(query.filter, path, () => node.child));
        return;
      }
      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
    },
    [applyNextTree, enterStructuredDraftMoveMode, prompts],
  );

  const promptForGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
    ) => {
      const canUnwrap = canUnwrapSearchFilterNodeAtPath(query.filter, path);
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      const result = await prompts.promptSelectOption({
        title: "Boolean Group",
        prompt: "Choose how to update this staged boolean group",
        entries: [
          { value: "addClause", label: "Add Filter", description: "Append a new filter at this group bottom." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
          {
            value: "toggle",
            label: node.kind === "allOf" ? "Change To anyOf" : "Change To allOf",
            description: "Reshape this group without changing its current children.",
          },
          { value: "wrapNot", label: "Wrap In NOT", description: "Wrap this group in a NOT node." },
          { value: "move", label: "Move Node", description: "Move this group to another visible insertion slot." },
          ...(canUnwrap
            ? [{ value: "unwrap", label: "Unwrap Group", description: "Replace this group with its current children." }]
            : []),
          ...(canLift
            ? [{ value: "lift", label: "Lift Node", description: "Lift this group out of its current parent group." }]
            : []),
          { value: "remove", label: "Remove Group", description: "Delete this group and all of its children." },
        ],
        selectedValue: "addClause",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "addClause") {
        await addQueryClauseAtPath(query, path);
        return;
      }
      if (result.value === "addAndGroup" || result.value === "addOrGroup" || result.value === "addNotGroup") {
        await addQueryGroupAtPath(
          query,
          path,
          result.value === "addAndGroup" ? "and" : result.value === "addOrGroup" ? "or" : "not",
        );
        return;
      }
      if (result.value === "toggle") {
        applyNextTree(reshapeSearchFilterBooleanGroupAtPath(query.filter, path, node.kind === "allOf" ? "anyOf" : "allOf"));
        return;
      }
      if (result.value === "wrapNot") {
        applyNextTree(wrapSearchFilterNodeAtPath(query.filter, path, "not"));
        return;
      }
      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (result.value === "unwrap") {
        applyNextTree(unwrapSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
    },
    [addQueryClauseAtPath, addQueryGroupAtPath, applyNextTree, enterStructuredDraftMoveMode, prompts],
  );

  const editStructuredDraftMetadata = React.useCallback(
    async (entry: SearchStructuredDraftEntry) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }

      if (entry.kind === "queryTreeRoot") {
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
        await promptForInsertionAction(draftQuery, entry.insertionPath ?? []);
        return;
      }

      if (entry.kind !== "queryNode") {
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
      promptForGroupAction,
      promptForInsertionAction,
      promptForLeafAction,
      promptForNotAction,
      promptForRootAction,
      moveSourcePath,
      structuredDraftQuery,
    ],
  );

  return {
    editStructuredDraftMetadata,
  };
}
