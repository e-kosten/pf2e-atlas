import React from "react";

import type { MetadataFilterNode } from "../search/metadata-filter-draft.js";
import { isMetadataPredicate } from "../search/query-core.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import { promptNumericScalarClause } from "../filter-explorer/scalar-editor.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace/workspace-action-types.js";

export function useSearchQueryFieldEditing({
  openFilterExplorer,
  prompts,
  terminal,
  user,
}: {
  openFilterExplorer: OpenSearchFilterExplorer;
  prompts: SearchWorkspacePromptAdapters;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
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
    onApply: (
      result: Pf2eTerminalFilterExplorerInsertionResult,
      nextQuery: Pf2eTerminalSearchQuery,
    ) => void,
    onReturn?: () => void,
    onQueryChange?: (
      result: Pf2eTerminalFilterExplorerInsertionResult,
      nextQuery: Pf2eTerminalSearchQuery,
    ) => void,
    options?: {
      onBack?: () => void;
      onExitRoot?: () => void;
      onCancel?: () => void;
      initialDraft?: Pf2eTerminalFilterExplorerDraft;
    },
  ) => Promise<boolean>;
  openOntologyFieldExplorer: (
    query: Pf2eTerminalSearchQuery,
    fieldOptions: Pf2eTerminalQueryFieldOption[],
    onApply: (nextNode: MetadataFilterNode | null) => void,
  ) => Promise<boolean>;
} {
  const getScopedFieldOptions = React.useCallback(
    (query: Pf2eTerminalSearchQuery): Pf2eTerminalQueryFieldOption[] =>
      user.search.getQueryFieldOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)),
    [user.search],
  );

  const chooseQueryField = React.useCallback(
    async (query: Pf2eTerminalSearchQuery): Promise<Pf2eTerminalQueryFieldOption | null> => {
      const fieldOptions = getScopedFieldOptions(query);
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
        return null;
      }

      const result = await prompts.promptSelectOption({
        title: "Query Field",
        prompt: "Choose the field for the next metadata clause",
        entries: fieldOptions.map((fieldOption) => ({
          value: fieldOption.value,
          label: fieldOption.label,
          description: fieldOption.description,
        })),
        selectedValue: fieldOptions[0]!.value,
      });

      if (result.kind !== "selected") {
        return null;
      }

      return fieldOptions.find((fieldOption) => fieldOption.value === result.value) ?? null;
    },
    [getScopedFieldOptions, prompts, terminal],
  );

  const getExplorerBackedFieldOptions = React.useCallback(
    (fieldOptions: Pf2eTerminalQueryFieldOption[]) =>
      fieldOptions.filter((fieldOption) => fieldOption.editor === "sharedExplorer"),
    [],
  );

  const openOntologyFieldEditor = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
      onApply: (
        result: Pf2eTerminalFilterExplorerInsertionResult,
        nextQuery: Pf2eTerminalSearchQuery,
      ) => void,
      onReturn?: () => void,
      onQueryChange?: (
        result: Pf2eTerminalFilterExplorerInsertionResult,
        nextQuery: Pf2eTerminalSearchQuery,
      ) => void,
      options?: {
        onBack?: () => void;
        onExitRoot?: () => void;
        onCancel?: () => void;
        initialDraft?: Pf2eTerminalFilterExplorerDraft;
      },
    ): Promise<boolean> => {
      if (fieldOption.editor !== "sharedExplorer") {
        return false;
      }

      const buildResultForQuery = (nextQuery: Pf2eTerminalSearchQuery): Pf2eTerminalFilterExplorerInsertionResult => {
        const preparedDraft = user.search.prepareFilterExplorerDraft(nextQuery, [fieldOption.value]);
        return user.search.buildFilterExplorerInsertionResult(preparedDraft.draft, {
          preservedMetadata: preparedDraft.preservedMetadata,
          preferReplace: currentNode !== null,
        });
      };
      const shouldApplyOnClose = !onQueryChange;

      return openFilterExplorer({
        queryOverride: query,
        initialDraft: options?.initialDraft,
        fieldOptions: [fieldOption],
        onQueryChange: onQueryChange
          ? (nextQuery) => {
              onQueryChange(buildResultForQuery(nextQuery), nextQuery);
            }
          : undefined,
        onBack: (nextQuery) => {
          if (options?.onBack) {
            options.onBack();
          } else if (shouldApplyOnClose) {
            onApply(buildResultForQuery(nextQuery), nextQuery);
          }
          onReturn?.();
        },
        onExitRoot: (nextQuery) => {
          if (options?.onExitRoot) {
            options.onExitRoot();
            return;
          }
          if (shouldApplyOnClose) {
            onApply(buildResultForQuery(nextQuery), nextQuery);
          }
        },
        onCancel: options?.onCancel ? () => options.onCancel?.() : undefined,
        singleFieldBehavior: "directValues",
      });
    },
    [openFilterExplorer, user.search],
  );

  const openOntologyFieldExplorer = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOptions: Pf2eTerminalQueryFieldOption[],
      onApply: (nextNode: MetadataFilterNode | null) => void,
    ): Promise<boolean> => {
      if (fieldOptions.length === 0 || fieldOptions.some((fieldOption) => fieldOption.editor !== "sharedExplorer")) {
        return false;
      }

      return openFilterExplorer({
        queryOverride: query,
        fieldOptions,
        singleFieldBehavior: fieldOptions.length === 1 ? "directValues" : "list",
        onBack: (nextQuery) => {
          const preparedDraft = user.search.prepareFilterExplorerDraft(
            nextQuery,
            fieldOptions.map((fieldOption) => fieldOption.value),
          );
          onApply(user.search.buildFilterExplorerMetadataNode(preparedDraft.draft, { preservedMetadata: preparedDraft.preservedMetadata }));
        },
        onExitRoot: (nextQuery) => {
          const preparedDraft = user.search.prepareFilterExplorerDraft(
            nextQuery,
            fieldOptions.map((fieldOption) => fieldOption.value),
          );
          onApply(user.search.buildFilterExplorerMetadataNode(preparedDraft.draft, { preservedMetadata: preparedDraft.preservedMetadata }));
        },
      });
    },
    [openFilterExplorer, user.search],
  );

  const editFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null = null,
    ): Promise<MetadataFilterNode | null | undefined> => {
      const queryCategory = getSearchQueryCategory(query);
      const querySubcategory = getSearchQuerySubcategory(query);
      if (fieldOption.editor === "sharedExplorer") {
        return currentNode;
      }
      const metadataField = fieldOption.value as Pf2eTerminalFacetField;

      if (fieldOption.fieldType === "set" || fieldOption.fieldType === "enumString") {
        await terminal.pauseForAnyKey("This field should be edited through the shared explorer path.");
        return undefined;
      }

      if (fieldOption.fieldType === "boolean") {
        const currentValue =
          currentNode && isMetadataPredicate(currentNode) && "value" in currentNode && currentNode.op === "eq"
            ? String(currentNode.value)
            : null;
        const result = await prompts.promptOptionalSelectOption({
          title: `${fieldOption.label} Clause`,
          prompt: "Choose the boolean condition for this clause",
          allOption: {
            label: "Clear Clause",
            description: "Remove this boolean clause.",
          },
          entries: [
            { value: "true", label: "Is True", description: "Require the field to be true." },
            { value: "false", label: "Is False", description: "Require the field to be false." },
          ],
          selectedValue: currentValue,
        });

        if (result.kind === "cancelled" || result.kind === "back") {
          return undefined;
        }
        if (result.kind === "all") {
          return null;
        }
        return { field: metadataField, op: "eq", value: result.value === "true" } as MetadataFilterNode;
      }

      if (fieldOption.fieldType === "text") {
        const currentText =
          currentNode && isMetadataPredicate(currentNode) && "value" in currentNode ? String(currentNode.value) : "";
        const currentOp =
          currentNode &&
          isMetadataPredicate(currentNode) &&
          "op" in currentNode &&
          ["contains", "notContains"].includes(currentNode.op)
            ? currentNode.op
            : "contains";
        const opResult = await prompts.promptSelectOption({
          title: `${fieldOption.label} Clause`,
          prompt: "Choose how this text field should match",
          entries: [
            { value: "contains", label: "Contains", description: "Match records whose field contains the text." },
            {
              value: "notContains",
              label: "Does Not Contain",
              description: "Exclude records whose field contains the text.",
            },
          ],
          selectedValue: currentOp,
        });

        if (opResult.kind !== "selected") {
          return undefined;
        }

        const value = await prompts.promptTextInput({
          title: `${fieldOption.label} Text`,
          prompt: "Enter the text value for this clause. Leave blank to clear.",
          defaultValue: currentText,
        });

        if (value === undefined) {
          return undefined;
        }
        if (!value.trim()) {
          return null;
        }
        return { field: metadataField, op: opResult.value, value: value.trim() } as MetadataFilterNode;
      }

      const currentNumericOp =
        currentNode &&
        isMetadataPredicate(currentNode) &&
        (currentNode.op === "eq" ||
          currentNode.op === "gte" ||
          currentNode.op === "lte" ||
          currentNode.op === "between")
          ? currentNode.op
          : null;
      const currentClause =
        currentNumericOp === "between" && currentNode && "min" in currentNode && "max" in currentNode
          ? { op: "between" as const, min: currentNode.min, max: currentNode.max }
          : currentNumericOp &&
              currentNumericOp !== "between" &&
              currentNode &&
              "value" in currentNode &&
              typeof currentNode.value === "number"
            ? { op: currentNumericOp, value: currentNode.value }
            : null;
      const nextClause = await promptNumericScalarClause(prompts, terminal, {
        title: `${fieldOption.label} Clause`,
        currentClause,
      });

      if (nextClause === undefined) {
        return undefined;
      }

      if (!nextClause) {
        return null;
      }
      return nextClause.op === "between"
        ? ({ field: metadataField, op: "between", min: nextClause.min, max: nextClause.max } as MetadataFilterNode)
        : ({ field: metadataField, op: nextClause.op, value: nextClause.value } as MetadataFilterNode);
    },
    [prompts, terminal, user.search],
  );

  return {
    chooseQueryField,
    editFieldClause,
    getExplorerBackedFieldOptions,
    getScopedFieldOptions,
    openOntologyFieldEditor,
    openOntologyFieldExplorer,
  };
}
