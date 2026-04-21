import React from "react";

import type { MetadataFilterNode } from "../../domain/metadata-types.js";
import {
  isMetadataPredicate,
} from "../search/query-core.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQuerySubcategory,
} from "../search/service.js";
import { createEmptyStringPolicy } from "../search/policies.js";
import { formatFilterExplorerPolicyCycleCopy } from "../framework/policy-presentation.js";
import {
  buildMetadataNodeFromPolicy,
  buildPolicyFromPredicate,
} from "./metadata-clause-translation.js";
import { promptNumericScalarClause } from "./scalar-editor.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace-action-types.js";

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
    onApply: (nextNode: MetadataFilterNode | null) => void,
    onReturn?: () => void,
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
      onApply: (nextNode: MetadataFilterNode | null) => void,
      onReturn?: () => void,
    ): Promise<boolean> => {
      if (fieldOption.editor !== "sharedExplorer") {
        return false;
      }

      return openFilterExplorer({
        queryOverride: query,
        fieldOptions: [fieldOption],
        initialDraft: user.search.createFilterExplorerDraftFromMetadataNode(currentNode, [fieldOption.value]),
        onReturn,
        singleFieldBehavior: "directValues",
        onApply: (draft) => onApply(user.search.buildFilterExplorerMetadataNode(draft)),
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
        initialDraft: user.search.createFilterExplorerDraftFromMetadataNode(
          null,
          fieldOptions.map((fieldOption) => fieldOption.value),
        ),
        singleFieldBehavior: fieldOptions.length === 1 ? "directValues" : "list",
        onApply: (draft) => onApply(user.search.buildFilterExplorerMetadataNode(draft)),
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
        const currentPolicy =
          currentNode && isMetadataPredicate(currentNode)
            ? (buildPolicyFromPredicate(currentNode) ?? createEmptyStringPolicy())
            : createEmptyStringPolicy();
        const selected = await prompts.promptPolicySelectOption({
          title: `${fieldOption.label} Clause`,
          prompt: `Cycle field values through ${formatFilterExplorerPolicyCycleCopy(
            fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
          )}. Press Esc or Left when finished.`,
          allowedStates: fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
          entries: user.search
            .getFacetValueOptions(metadataField, queryCategory, querySubcategory)
            .map((option) => ({
              value: option.value,
              label: option.label,
              description: option.description,
            })),
          selectedValues: currentPolicy,
        });

        return buildMetadataNodeFromPolicy(fieldOption, selected);
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

        if (result.kind === "cancelled") {
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
