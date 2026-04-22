import React from "react";

import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import {
  getMetadataNodeAtPath,
  isMetadataPredicate,
} from "../../search/query-core.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import { getSearchQueryMetadataTree } from "../../search/query-state.js";
import type { SearchStructuredDraftState } from "./structured-draft-support.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
} from "../workspace/workspace-action-types.js";

export function useSearchStructuredDraftMetadataActions({
  appendStructuredDraftMetadataNode,
  chooseQueryField,
  editFieldClause,
  getExplorerBackedFieldOptions,
  getScopedFieldOptions,
  openOntologyFieldEditor,
  openOntologyFieldExplorer,
  openQueryFieldBuilder,
  prompts,
  structuredDraftState,
  terminal,
  updateStructuredDraftMetadataNode,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  chooseQueryField: (query: Pf2eTerminalSearchQuery) => Promise<Pf2eTerminalQueryFieldOption | null>;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
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
  prompts: SearchWorkspacePromptAdapters;
  structuredDraftState: SearchStructuredDraftState | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
  ) => void;
}): {
  editStructuredDraftMetadata: (path: number[]) => Promise<void>;
} {
  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = []) => {
      const fieldOptions = getScopedFieldOptions(query);
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
      const fieldOptions = getScopedFieldOptions(query);
      const explorerFieldOptions = getExplorerBackedFieldOptions(fieldOptions);
      const wrapNode = (nextNode: MetadataFilterNode): MetadataFilterNode =>
        groupKind === "and" ? { and: [nextNode] } : groupKind === "or" ? { or: [nextNode] } : { not: nextNode };

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

  const editStructuredDraftMetadata = React.useCallback(
    async (path: number[]) => {
      const draftQuery = structuredDraftState?.draftQuery;
      if (!draftQuery) {
        return;
      }

      const draftMetadataTree = getSearchQueryMetadataTree(draftQuery);
      const node = path.length === 0 ? draftMetadataTree : getMetadataNodeAtPath(draftMetadataTree, path);
      const fieldOptions = getScopedFieldOptions(draftQuery);

      if (!node) {
        if (fieldOptions.length === 0) {
          await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
          return;
        }

        const explorerFieldOptions = getExplorerBackedFieldOptions(fieldOptions);
        if (explorerFieldOptions.length > 0) {
          if (
            await openOntologyFieldExplorer(draftQuery, explorerFieldOptions, (nextNode) => {
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

        if (fieldOptions.length === 1) {
          const onlyField = fieldOptions[0]!;
          const nextNode = await editFieldClause(draftQuery, onlyField);
          if (nextNode) {
            appendStructuredDraftMetadataNode(path, nextNode);
          }
          return;
        }

        await openQueryFieldBuilder(draftQuery, path);
        return;
      }

      if (isMetadataPredicate(node)) {
        const fieldOption = fieldOptions.find((candidate) => candidate.value === node.field) ?? null;
        const actionEntries = [
          ...(fieldOption
            ? [{ value: "edit", label: "Edit Clause", description: "Change the field operator or value." }]
            : []),
          { value: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
          { value: "remove", label: "Remove Clause", description: "Delete this clause from the staged query." },
        ];
        const result = await prompts.promptSelectOption({
          title: "Query Clause",
          prompt: "Choose how to update this staged query clause",
          entries: actionEntries,
          selectedValue: actionEntries[0]!.value,
        });

        if (result.kind !== "selected") {
          return;
        }

        if (result.value === "edit") {
          if (!fieldOption) {
            await terminal.pauseForAnyKey("That clause came from a richer query path and cannot be edited here yet.");
            return;
          }
          if (fieldOption.editor === "sharedExplorer") {
            await openOntologyFieldEditor(draftQuery, fieldOption, node, (nextNode) => {
              updateStructuredDraftMetadataNode(path, () => nextNode);
            });
            return;
          }
          const nextNode = await editFieldClause(draftQuery, fieldOption, node);
          if (nextNode === undefined) {
            return;
          }
          updateStructuredDraftMetadataNode(path, () => nextNode);
          return;
        }

        if (result.value === "wrapNot") {
          updateStructuredDraftMetadataNode(path, (current) => ({ not: current }));
          return;
        }

        updateStructuredDraftMetadataNode(path, () => null);
        return;
      }

      if ("not" in node) {
        const result = await prompts.promptSelectOption({
          title: "NOT Group",
          prompt: "Choose how to update this negated staged clause",
          entries: [
            { value: "unwrap", label: "Remove NOT", description: "Keep the child clause and remove the negation." },
            { value: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
          ],
          selectedValue: "unwrap",
        });

        if (result.kind !== "selected") {
          return;
        }

        updateStructuredDraftMetadataNode(path, (current) =>
          result.value === "unwrap" && "not" in current ? current.not : null,
        );
        return;
      }

      const isAndGroup = "and" in node;
      const result = await prompts.promptSelectOption({
        title: isAndGroup ? "AND Group" : "OR Group",
        prompt: "Choose how to update this staged logic group",
        entries: [
          { value: "addClause", label: "Add Clause", description: "Add another clause inside this logic group." },
          {
            value: "addAndGroup",
            label: "Add AND Group",
            description: "Add a nested AND group with one initial clause.",
          },
          { value: "addOrGroup", label: "Add OR Group", description: "Add a nested OR group with one initial clause." },
          {
            value: "addNotGroup",
            label: "Add NOT Group",
            description: "Add a nested NOT group with one initial clause.",
          },
          {
            value: "toggle",
            label: isAndGroup ? "Convert To OR" : "Convert To AND",
            description: isAndGroup
              ? "Require any child instead of every child."
              : "Require every child instead of any child.",
          },
          { value: "remove", label: "Remove Group", description: "Delete this logic group from the staged query." },
        ],
        selectedValue: "addClause",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "addClause") {
        await addQueryClauseAtPath(draftQuery, path);
        return;
      }
      if (result.value === "addAndGroup" || result.value === "addOrGroup" || result.value === "addNotGroup") {
        await addQueryGroupAtPath(
          draftQuery,
          path,
          result.value === "addAndGroup" ? "and" : result.value === "addOrGroup" ? "or" : "not",
        );
        return;
      }
      if (result.value === "toggle") {
        updateStructuredDraftMetadataNode(path, (current) =>
          "and" in current ? { or: [...current.and] } : "or" in current ? { and: [...current.or] } : current,
        );
        return;
      }
      updateStructuredDraftMetadataNode(path, () => null);
    },
    [
      addQueryClauseAtPath,
      addQueryGroupAtPath,
      appendStructuredDraftMetadataNode,
      editFieldClause,
      getExplorerBackedFieldOptions,
      getScopedFieldOptions,
      openOntologyFieldEditor,
      openOntologyFieldExplorer,
      openQueryFieldBuilder,
      prompts,
      structuredDraftState?.draftQuery,
      terminal,
      updateStructuredDraftMetadataNode,
    ],
  );

  return {
    editStructuredDraftMetadata,
  };
}
