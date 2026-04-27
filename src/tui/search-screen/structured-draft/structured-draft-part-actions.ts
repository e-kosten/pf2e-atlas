import React from "react";

import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQuerySubcategory,
  setSearchQueryCategory,
  setSearchQueryLevelRange,
  setSearchQuerySubcategory,
} from "../../search/query-state.js";
import { formatLevelRange } from "../model.js";
import { promptLevelRangeDraft } from "../../filter-explorer/scalar-editor.js";
import type { SearchStructuredDraftState } from "./structured-draft-support.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

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

export function useSearchStructuredDraftPartActions({
  openFilterExplorer,
  prompts,
  replaceStructuredDraftProjection,
  structuredDraftQuery,
  terminal,
  user,
}: {
  openFilterExplorer: OpenSearchFilterExplorer;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
  editStructuredDraftActionCost: () => Promise<void>;
  editStructuredDraftCategory: () => Promise<void>;
  editStructuredDraftLevelRange: () => Promise<void>;
  editStructuredDraftRarity: () => Promise<void>;
  editStructuredDraftSubcategory: () => Promise<void>;
} {
  const editStructuredDraftCategory = React.useCallback(async () => {
    const draftQuery = structuredDraftQuery;
    if (!draftQuery) {
      return;
    }

    const draftCategory = getSearchQueryCategory(draftQuery);
    const [allCategoryOption, ...categoryEntries] = user.search.getCategoryOptions();
    const result = await prompts.promptOptionalSelectOption({
      title: "Category Scope",
      prompt: "Choose the staged category boundary",
      allOption: {
        label: allCategoryOption?.label ?? "Any Category",
        description: allCategoryOption?.description,
      },
      entries: categoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: draftCategory,
    });

    if (result.kind === "cancelled" || result.kind === "back") {
      return;
    }

    replaceStructuredDraftProjection((query) => setSearchQueryCategory(query, result.kind === "all" ? null : result.value));
  }, [prompts, replaceStructuredDraftProjection, structuredDraftQuery, user.search]);

  const editStructuredDraftSubcategory = React.useCallback(async () => {
    const draftQuery = structuredDraftQuery;
    if (!draftQuery) {
      return;
    }

    const draftCategory = getSearchQueryCategory(draftQuery);
    const draftSubcategory = getSearchQuerySubcategory(draftQuery);
    if (!draftCategory) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const [allSubcategoryOption, ...subcategoryEntries] = user.search.getSubcategoryOptions(draftCategory);
    if (subcategoryEntries.length === 0) {
      await terminal.pauseForAnyKey("No subcategories are available for the current category.");
      return;
    }

    const result = await prompts.promptOptionalSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the staged subcategory boundary",
      allOption: {
        label: allSubcategoryOption?.label ?? "Any Subcategory",
        description: allSubcategoryOption?.description,
      },
      entries: subcategoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: draftSubcategory,
    });

    if (result.kind === "cancelled" || result.kind === "back") {
      return;
    }

    replaceStructuredDraftProjection((query) =>
      result.kind === "all" ? setSearchQuerySubcategory(query, null) : result.value ? setSearchQuerySubcategory(query, result.value) : query,
    );
  }, [prompts, replaceStructuredDraftProjection, structuredDraftQuery, terminal, user.search]);

  const editStructuredDraftLevelRange = React.useCallback(async () => {
    const draftQuery = structuredDraftQuery;
    if (!draftQuery) {
      return;
    }

    const draftLevelRange = getSearchQueryLevelRange(draftQuery);
    const parsed = await promptLevelRangeDraft(prompts, terminal, {
      defaultValue:
        draftLevelRange.levelMin === null && draftLevelRange.levelMax === null
          ? ""
          : formatLevelRange(draftQuery).replaceAll("L", "").replace("<= ", "<="),
    });

    if (parsed === undefined) {
      return;
    }

    replaceStructuredDraftProjection((query) =>
      setSearchQueryLevelRange(query, {
        levelMin: parsed.levelMin,
        levelMax: parsed.levelMax,
      }),
    );
  }, [prompts, replaceStructuredDraftProjection, structuredDraftQuery, terminal]);

  const editStructuredDraftExplorerField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }

      await openFilterExplorer({
        queryOverride: draftQuery,
        fieldOptions: [fieldOption],
        initialPreparedDraft: user.search.prepareFilterExplorerDraft(draftQuery, [fieldOption.value]),
        singleFieldBehavior: "directValues",
        onDraftChange: (draft, context) => {
          replaceStructuredDraftProjection((query) => user.search.applyFilterExplorerDraft(query, draft, context));
        },
        onApply: () => {},
      });
    },
    [openFilterExplorer, replaceStructuredDraftProjection, structuredDraftQuery, user.search],
  );

  const editStructuredDraftRarity = React.useCallback(async () => {
    await editStructuredDraftExplorerField(
      buildExplorerOnlyFieldOption(
        "rarity",
        "Rarity",
        "Browse live rarities for the current scope and stage include or exclude filters.",
        "enumString",
      ),
    );
  }, [editStructuredDraftExplorerField]);

  const editStructuredDraftActionCost = React.useCallback(async () => {
    await editStructuredDraftExplorerField(
      buildExplorerOnlyFieldOption(
        "actionCost",
        "Action Cost",
        "Browse live action costs for the current scope and stage include or exclude filters.",
        "number",
      ),
    );
  }, [editStructuredDraftExplorerField]);

  return {
    editStructuredDraftActionCost,
    editStructuredDraftCategory,
    editStructuredDraftLevelRange,
    editStructuredDraftRarity,
    editStructuredDraftSubcategory,
  };
}
