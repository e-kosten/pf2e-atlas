import React from "react";

import type { MetadataFilterNode } from "../../domain/metadata-types.js";
import { clampStructuredDraftSelection } from "../search/structured-draft-session.js";
import {
  appendMetadataNodeAtPath,
  getMetadataNodeAtPath,
  isMetadataPredicate,
  updateMetadataNodeAtPath,
} from "../search/query-core.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../search/service.js";
import {
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQuerySubcategory,
  removeSearchQueryPart,
  setSearchQueryCategory,
  setSearchQueryMetadataTree,
  setSearchQueryPart,
} from "../search/service.js";
import type { SearchQueryFieldBuilderSession } from "./query-field-builder-session.js";
import {
  buildQueryFieldBuilderItems,
  buildQueryFieldBuilderPreviewQuery,
  buildQueryFieldBuilderSessionItems,
  compileQueryFieldBuilderDrafts,
  type QueryFieldBuilderState,
} from "./query-field-builder-support.js";
import {
  buildStructuredDraftEntries,
  getStructuredDraftSelectionIndex,
  type SearchStructuredDraftState,
} from "./structured-draft-support.js";
import { promptLevelRangeDraft } from "./scalar-editor.js";
import { formatLevelRange } from "./model.js";
import { useSearchQueryFieldEditing } from "./query-field-editing.js";
import type {
  OpenSearchFilterExplorer,
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "./workspace-action-types.js";

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

export function useSearchStructuredEditorActions({
  applyQueryUpdate,
  currentQuery,
  openFilterExplorer,
  prompts,
  terminal,
  user,
}: {
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  currentQuery: Pf2eTerminalSearchQuery;
  openFilterExplorer: OpenSearchFilterExplorer;
  prompts: SearchWorkspacePromptAdapters;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}): {
  openStructuredDraftSession: (
    anchor: SearchStructuredDraftState["anchor"],
    query?: Pf2eTerminalSearchQuery,
  ) => void;
  structuredEditorSession: SearchQueryFieldBuilderSession | null;
} {
  const [structuredDraftState, setStructuredDraftState] = React.useState<SearchStructuredDraftState | null>(null);
  const [queryFieldBuilderState, setQueryFieldBuilderState] = React.useState<QueryFieldBuilderState | null>(null);

  const {
    chooseQueryField,
    editFieldClause,
    getExplorerBackedFieldOptions,
    getScopedFieldOptions,
    openOntologyFieldEditor,
    openOntologyFieldExplorer,
  } = useSearchQueryFieldEditing({
    openFilterExplorer,
    prompts,
    terminal,
    user,
  });

  const hasSelectableSubcategories = React.useCallback(
    (category: Pf2eTerminalSearchQuery["filters"]["category"]): boolean => {
      if (!category) {
        return false;
      }
      return user.search.getSubcategoryOptions(category).length > 1;
    },
    [user.search],
  );

  const openStructuredDraftSession = React.useCallback(
    (anchor: SearchStructuredDraftState["anchor"], query: Pf2eTerminalSearchQuery = currentQuery) => {
      const draftQuery = user.search.normalizeQuery(query);
      const metadataFocusPath = anchor.kind === "queryNode" ? [...anchor.path] : null;
      const entries = buildStructuredDraftEntries(draftQuery, metadataFocusPath, {
        hasSelectableSubcategories,
        getActionCostOptions: user.search.getActionCostOptions,
      });
      setStructuredDraftState({
        anchor,
        draftQuery,
        metadataFocusPath,
        selectedIndex: getStructuredDraftSelectionIndex(anchor, entries),
      });
    },
    [currentQuery, hasSelectableSubcategories, user.search],
  );

  const replaceStructuredDraftQuery = React.useCallback(
    (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }
        const nextDraftQuery = user.search.normalizeQuery(update(current.draftQuery));
        const entries = buildStructuredDraftEntries(nextDraftQuery, current.metadataFocusPath, {
          hasSelectableSubcategories,
          getActionCostOptions: user.search.getActionCostOptions,
        });
        return {
          ...current,
          draftQuery: nextDraftQuery,
          selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
        };
      });
    },
    [hasSelectableSubcategories, user.search],
  );

  const appendStructuredDraftMetadataNode = React.useCallback(
    (path: number[], nextNode: MetadataFilterNode) => {
      replaceStructuredDraftQuery((draftQuery) =>
        setSearchQueryMetadataTree(
          draftQuery,
          appendMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, nextNode),
        ),
      );
    },
    [replaceStructuredDraftQuery],
  );

  const updateStructuredDraftMetadataNode = React.useCallback(
    (
      path: number[],
      update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    ) => {
      replaceStructuredDraftQuery((draftQuery) =>
        setSearchQueryMetadataTree(
          draftQuery,
          updateMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, update),
        ),
      );
    },
    [replaceStructuredDraftQuery],
  );

  const updateQueryFieldBuilderDraft = React.useCallback((field: string, node: MetadataFilterNode | null) => {
    setQueryFieldBuilderState((current) => {
      if (!current) {
        return current;
      }
      return {
        ...current,
        fieldDrafts: {
          ...current.fieldDrafts,
          [field]: node,
        },
      };
    });
  }, []);

  const openQueryFieldBuilder = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = []) => {
      const fieldOptions = getScopedFieldOptions(query);
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
        return false;
      }

      setQueryFieldBuilderState({
        draftQuery: query,
        path,
        items: buildQueryFieldBuilderItems(fieldOptions),
        selectedIndex: 0,
        fieldDrafts: {},
      });
      return true;
    },
    [getScopedFieldOptions, terminal],
  );

  const finishQueryFieldBuilder = React.useCallback(() => {
    const nextNode = compileQueryFieldBuilderDrafts(queryFieldBuilderState?.fieldDrafts ?? {});
    const targetPath = queryFieldBuilderState?.path ?? [];
    setQueryFieldBuilderState(null);
    if (!nextNode) {
      return;
    }
    appendStructuredDraftMetadataNode(targetPath, nextNode);
  }, [appendStructuredDraftMetadataNode, queryFieldBuilderState]);

  const discardQueryFieldBuilder = React.useCallback(() => {
    setQueryFieldBuilderState(null);
  }, []);

  const returnQueryFieldBuilder = React.useCallback(() => {
    finishQueryFieldBuilder();
  }, [finishQueryFieldBuilder]);

  const editQueryFieldBuilderField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const scopeQuery = queryFieldBuilderState?.draftQuery;
      if (!scopeQuery) {
        return;
      }

      const currentNode = queryFieldBuilderState.fieldDrafts[fieldOption.value] ?? null;
      if (fieldOption.editor === "sharedExplorer") {
        await openOntologyFieldEditor(scopeQuery, fieldOption, currentNode, (nextNode) => {
          updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
        });
        return;
      }

      const nextNode = await editFieldClause(scopeQuery, fieldOption, currentNode);
      if (nextNode === undefined) {
        return;
      }
      updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
    },
    [editFieldClause, openOntologyFieldEditor, queryFieldBuilderState, updateQueryFieldBuilderDraft],
  );

  const selectCurrentQueryFieldBuilderItem = React.useCallback(() => {
    const selectedItem =
      queryFieldBuilderState?.items[
        clampStructuredDraftSelection(queryFieldBuilderState.selectedIndex, queryFieldBuilderState.items.length)
      ] ?? null;
    if (!selectedItem) {
      return;
    }
    if (selectedItem.kind === "field") {
      void editQueryFieldBuilderField(selectedItem.fieldOption);
      return;
    }
    if (selectedItem.kind === "finish") {
      returnQueryFieldBuilder();
      return;
    }
    discardQueryFieldBuilder();
  }, [discardQueryFieldBuilder, editQueryFieldBuilderField, queryFieldBuilderState, returnQueryFieldBuilder]);

  const queryFieldBuilderSession = React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!queryFieldBuilderState) {
      return null;
    }
    return {
      kind: "queryFieldBuilder",
      title: "Add Query Part",
      subtitle: "Choose query fields and keep the full staged query visible while you refine it",
      leftTitle: "[QUERY FIELDS]",
      rightTitle: "Staged Summary & Detail",
      statusText: "Field edits stay staged until you finish the structured query editor.",
      draftQuery: buildQueryFieldBuilderPreviewQuery(queryFieldBuilderState),
      items: buildQueryFieldBuilderSessionItems(queryFieldBuilderState),
      selectedIndex: clampStructuredDraftSelection(
        queryFieldBuilderState.selectedIndex,
        queryFieldBuilderState.items.length,
      ),
      fieldDrafts: queryFieldBuilderState.fieldDrafts,
      moveSelection: (delta, itemCount) => {
        setQueryFieldBuilderState((current) => {
          if (!current) {
            return current;
          }
          return {
            ...current,
            selectedIndex: clampStructuredDraftSelection(current.selectedIndex + delta, itemCount),
          };
        });
      },
      selectCurrent: selectCurrentQueryFieldBuilderItem,
      finish: returnQueryFieldBuilder,
      cancel: returnQueryFieldBuilder,
      helpTitle: "Add Query Part Help",
      helpBody: [
        { text: "Choose the next query field to stage into the structured query.", tone: "section" },
        {
          text: "The right pane keeps the full staged query summary visible while you move between fields.",
        },
        {
          text: "Use Left or Esc to return to the staged query with current field edits preserved.",
        },
        {
          text: "Use the discard row only when you want to drop the in-progress field edits from this subpage.",
        },
      ],
    };
  }, [queryFieldBuilderState, returnQueryFieldBuilder, selectCurrentQueryFieldBuilderItem]);

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

  const finishStructuredDraftSession = React.useCallback(() => {
    if (!structuredDraftState) {
      return;
    }
    const nextQuery = structuredDraftState.draftQuery;
    setStructuredDraftState(null);
    applyQueryUpdate(() => nextQuery);
  }, [applyQueryUpdate, structuredDraftState]);

  const cancelStructuredDraftSession = React.useCallback(() => {
    setStructuredDraftState(null);
  }, []);

  const structuredDraftEntries = React.useMemo(
    () =>
      structuredDraftState
        ? buildStructuredDraftEntries(structuredDraftState.draftQuery, structuredDraftState.metadataFocusPath, {
            hasSelectableSubcategories,
            getActionCostOptions: user.search.getActionCostOptions,
          })
        : [],
    [hasSelectableSubcategories, structuredDraftState, user.search],
  );

  const editStructuredDraftCategory = React.useCallback(async () => {
    const draftQuery = structuredDraftState?.draftQuery;
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

    if (result.kind === "cancelled") {
      return;
    }

    replaceStructuredDraftQuery((query) => setSearchQueryCategory(query, result.kind === "all" ? null : result.value));
  }, [prompts, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, user.search]);

  const editStructuredDraftSubcategory = React.useCallback(async () => {
    const draftQuery = structuredDraftState?.draftQuery;
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

    if (result.kind === "cancelled") {
      return;
    }

    replaceStructuredDraftQuery((query) =>
      result.kind === "all"
        ? removeSearchQueryPart(query, "subcategory")
        : result.value
          ? setSearchQueryPart(query, { kind: "subcategory", subcategory: result.value })
          : query,
    );
  }, [prompts, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, terminal, user.search]);

  const editStructuredDraftLevelRange = React.useCallback(async () => {
    const draftQuery = structuredDraftState?.draftQuery;
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

    replaceStructuredDraftQuery((query) =>
      parsed.levelMin === null && parsed.levelMax === null
        ? removeSearchQueryPart(query, "levelRange")
        : setSearchQueryPart(query, {
            kind: "levelRange",
            levelMin: parsed.levelMin,
            levelMax: parsed.levelMax,
          }),
    );
  }, [prompts, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, terminal]);

  const editStructuredDraftExplorerField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const draftQuery = structuredDraftState?.draftQuery;
      if (!draftQuery) {
        return;
      }

      await openFilterExplorer({
        queryOverride: draftQuery,
        fieldOptions: [fieldOption],
        initialDraft: user.search.createFilterExplorerDraft(draftQuery, [fieldOption.value]),
        singleFieldBehavior: "directValues",
        onApply: (draft) => {
          replaceStructuredDraftQuery((query) => user.search.applyFilterExplorerDraft(query, draft));
        },
      });
    },
    [openFilterExplorer, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, user.search],
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

  const selectCurrentStructuredDraftEntry = React.useCallback(() => {
    if (!structuredDraftState) {
      return;
    }
    const selectedEntry =
      structuredDraftEntries[
        clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length)
      ] ?? null;
    if (!selectedEntry) {
      return;
    }
    if (selectedEntry.kind === "finish") {
      finishStructuredDraftSession();
      return;
    }
    if (selectedEntry.kind === "cancel") {
      cancelStructuredDraftSession();
      return;
    }
    if (selectedEntry.kind === "category") {
      void editStructuredDraftCategory();
      return;
    }
    if (selectedEntry.kind === "subcategory") {
      void editStructuredDraftSubcategory();
      return;
    }
    if (selectedEntry.kind === "levelRange") {
      void editStructuredDraftLevelRange();
      return;
    }
    if (selectedEntry.kind === "rarity") {
      void editStructuredDraftRarity();
      return;
    }
    if (selectedEntry.kind === "actionCost") {
      void editStructuredDraftActionCost();
      return;
    }
    void editStructuredDraftMetadata(selectedEntry.metadataPath ?? []);
  }, [
    cancelStructuredDraftSession,
    editStructuredDraftActionCost,
    editStructuredDraftCategory,
    editStructuredDraftLevelRange,
    editStructuredDraftMetadata,
    editStructuredDraftRarity,
    editStructuredDraftSubcategory,
    finishStructuredDraftSession,
    structuredDraftEntries,
    structuredDraftState,
  ]);

  const structuredEditorSession = React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!structuredDraftState) {
      return null;
    }
    return {
      kind: "structuredEditor",
      title: "Structured Query Editor",
      subtitle: "Stage structured search changes before applying them to the live query",
      leftTitle: "[STAGED QUERY]",
      rightTitle: "Staged Summary & Detail",
      statusText: "Left/Esc applies the staged query and returns. Use the discard row to abandon it.",
      draftQuery: structuredDraftState.draftQuery,
      items: structuredDraftEntries.map((entry) =>
        entry.kind === "finish" || entry.kind === "cancel"
          ? { kind: entry.kind, label: entry.label }
          : {
              kind: "workspaceEntry" as const,
              label: `${entry.label} | ${entry.value}`,
              workspaceEntry: {
                action: "addQueryPart",
                label: entry.label,
                value: entry.value,
                description: entry.description,
                disabled: entry.disabled,
                disabledReason: entry.disabledReason,
              },
            },
      ),
      selectedIndex: clampStructuredDraftSelection(structuredDraftState.selectedIndex, structuredDraftEntries.length),
      moveSelection: (delta, itemCount) => {
        setStructuredDraftState((current) => {
          if (!current) {
            return current;
          }
          return {
            ...current,
            selectedIndex: clampStructuredDraftSelection(current.selectedIndex + delta, itemCount),
          };
        });
      },
      selectCurrent: selectCurrentStructuredDraftEntry,
      finish: finishStructuredDraftSession,
      cancel: finishStructuredDraftSession,
      helpTitle: "Structured Query Editor Help",
      helpBody: [
        { text: "Stage structured search changes before applying them to the live query.", tone: "section" },
        { text: "The summary stays visible while you move focus so prior staged selections do not disappear." },
        { text: "Use Left or Esc to apply the staged query and return to the top editor." },
        { text: "Use the discard row only when you want to abandon the staged query entirely." },
      ],
    };
  }, [finishStructuredDraftSession, selectCurrentStructuredDraftEntry, structuredDraftEntries, structuredDraftState]);

  return {
    openStructuredDraftSession,
    structuredEditorSession: queryFieldBuilderSession ?? structuredEditorSession,
  };
}
