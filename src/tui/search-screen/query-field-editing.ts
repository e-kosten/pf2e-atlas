import React from "react";

import type { SearchFilterNode } from "../../domain/search-request-types.js";
import type {
  MetadataBooleanField,
  MetadataNumberField,
  MetadataTextStringField,
} from "../../domain/metadata-field-types.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import { getSearchQueryCategory, getSearchQuerySubcategory } from "../search/query-state.js";
import { promptNumericScalarClause } from "../filter-explorer/scalar-editor.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace/workspace-action-types.js";

export function useSearchQueryFieldEditing({
  prompts,
  terminal,
  user,
}: {
  prompts: SearchWorkspacePromptAdapters;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: SearchFilterNode | null,
  ) => Promise<SearchFilterNode | null | undefined>;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
} {
  const getScopedFieldOptions = React.useCallback(
    (query: Pf2eTerminalSearchQuery): Pf2eTerminalQueryFieldOption[] =>
      user.search.getQueryFieldOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)),
    [user.search],
  );

  const editFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: SearchFilterNode | null = null,
    ): Promise<SearchFilterNode | null | undefined> => {
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
          currentNode?.kind === "metadataPredicate" &&
          "value" in currentNode.predicate &&
          currentNode.predicate.op === "eq"
            ? String(currentNode.predicate.value)
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
        return {
          kind: "metadataPredicate",
          predicate: { field: metadataField as MetadataBooleanField, op: "eq", value: result.value === "true" },
        };
      }

      if (fieldOption.fieldType === "text") {
        const currentText =
          currentNode?.kind === "metadataPredicate" && "value" in currentNode.predicate
            ? String(currentNode.predicate.value)
            : "";
        const currentOp =
          currentNode?.kind === "metadataPredicate" &&
          ["contains", "notContains"].includes(currentNode.predicate.op)
            ? currentNode.predicate.op
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
        return {
          kind: "metadataPredicate",
          predicate: {
            field: metadataField as MetadataTextStringField,
            op: opResult.value as "contains" | "notContains",
            value: value.trim(),
          },
        };
      }

      const currentNumericOp =
        currentNode?.kind === "metadataPredicate" &&
        (currentNode.predicate.op === "eq" ||
          currentNode.predicate.op === "gte" ||
          currentNode.predicate.op === "lte" ||
          currentNode.predicate.op === "between")
          ? currentNode.predicate.op
          : null;
      const currentPredicate = currentNode?.kind === "metadataPredicate" ? currentNode.predicate : null;
      const currentClause =
        currentNumericOp === "between" && currentPredicate && "min" in currentPredicate && "max" in currentPredicate
          ? { op: "between" as const, min: currentPredicate.min!, max: currentPredicate.max! }
          : currentNumericOp &&
              currentNumericOp !== "between" &&
              currentPredicate &&
              "value" in currentPredicate &&
              typeof currentPredicate.value === "number"
            ? { op: currentNumericOp, value: currentPredicate.value }
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
      if (nextClause.op === "notEq") {
        return undefined;
      }
      return nextClause.op === "between"
        ? {
            kind: "metadataPredicate",
            predicate: {
              field: metadataField as MetadataNumberField,
              op: "between",
              min: nextClause.min,
              max: nextClause.max,
            },
          }
        : {
            kind: "metadataPredicate",
            predicate: {
              field: metadataField as MetadataNumberField,
              op: nextClause.op,
              value: nextClause.value,
            },
          };
    },
    [prompts, terminal, user.search],
  );

  return {
    editFieldClause,
    getScopedFieldOptions,
  };
}
