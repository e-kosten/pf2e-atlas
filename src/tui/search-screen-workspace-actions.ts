import React from "react";

import type { MetadataFilterNode, MetadataPredicate } from "../types.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { SearchTerminalPromptAdapters } from "./interaction-context-adapters.js";
import type { SearchScreenAction, SearchScreenState } from "./search-screen-state.js";
import type {
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "./search-service.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import type { DerivedTagTerminalApp, DerivedTagTerminalCommandOption } from "./terminal-ui.js";
import {
  buildEditorCommandPaletteEntries,
  buildFacetRemovalEntries,
  buildResultCommandPaletteEntries,
  decodeQueryPartAction,
  decodeQueryNodeActionPath,
  formatFilterPolicy,
  formatLevelRange,
  formatSearchCategory,
  formatSearchSubcategory,
  hasFilterPolicy,
  isQueryPartAction,
  isQueryNodeAction,
  parseLevelRangeInput,
  type SearchWorkspaceAction,
} from "./search-screen-model.js";
import type { SearchWorkspaceEntry } from "./search-screen-workspace.js";

function isMetadataPredicate(node: MetadataFilterNode): node is MetadataPredicate {
  return !("and" in node) && !("or" in node) && !("not" in node);
}

function normalizeMetadataNode(node: MetadataFilterNode | null): MetadataFilterNode | null {
  if (!node) {
    return null;
  }
  if ("and" in node) {
    const children = node.and
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { and: children };
  }
  if ("or" in node) {
    const children = node.or
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { or: children };
  }
  if ("not" in node) {
    const child = normalizeMetadataNode(node.not);
    if (!child) {
      return null;
    }
    if ("not" in child) {
      return normalizeMetadataNode(child.not);
    }
    return { not: child };
  }
  return node;
}

function getMetadataNodeChildren(node: MetadataFilterNode): MetadataFilterNode[] {
  if ("and" in node) {
    return node.and;
  }
  if ("or" in node) {
    return node.or;
  }
  if ("not" in node) {
    return [node.not];
  }
  return [];
}

function getMetadataNodeAtPath(node: MetadataFilterNode | null, path: number[]): MetadataFilterNode | null {
  let current = node;
  for (const segment of path) {
    if (!current) {
      return null;
    }
    const children = getMetadataNodeChildren(current);
    current = children[segment] ?? null;
  }
  return current;
}

function updateMetadataNodeAtPath(
  node: MetadataFilterNode | null,
  path: number[],
  update: (current: MetadataFilterNode) => MetadataFilterNode | null,
): MetadataFilterNode | null {
  if (!node) {
    return null;
  }
  if (path.length === 0) {
    return normalizeMetadataNode(update(node));
  }
  const [segment, ...rest] = path;
  if (segment === undefined) {
    return normalizeMetadataNode(node);
  }
  if ("and" in node) {
    const children = [...node.and];
    const updatedChild = updateMetadataNodeAtPath(children[segment] ?? null, rest, update);
    if (updatedChild) {
      children[segment] = updatedChild;
    } else {
      children.splice(segment, 1);
    }
    return normalizeMetadataNode({ and: children });
  }
  if ("or" in node) {
    const children = [...node.or];
    const updatedChild = updateMetadataNodeAtPath(children[segment] ?? null, rest, update);
    if (updatedChild) {
      children[segment] = updatedChild;
    } else {
      children.splice(segment, 1);
    }
    return normalizeMetadataNode({ or: children });
  }
  if ("not" in node) {
    if (segment !== 0) {
      return node;
    }
    const updatedChild = updateMetadataNodeAtPath(node.not, rest, update);
    return normalizeMetadataNode(updatedChild ? { not: updatedChild } : null);
  }
  return node;
}

function appendMetadataNodeAtPath(
  metadata: MetadataFilterNode | null,
  path: number[],
  nextNode: MetadataFilterNode,
): MetadataFilterNode | null {
  if (!metadata) {
    return normalizeMetadataNode(nextNode);
  }
  return updateMetadataNodeAtPath(metadata, path, (current) => {
    if ("and" in current) {
      return { and: [...current.and, nextNode] };
    }
    if ("or" in current) {
      return { or: [...current.or, nextNode] };
    }
    return { and: [current, nextNode] };
  });
}

function createEmptyPolicy(): Pf2eTerminalFilterValuePolicy<string> {
  return { any: [], all: [], exclude: [] };
}

function buildMetadataNodeFromPolicy(
  fieldOption: Pf2eTerminalQueryFieldOption,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): MetadataFilterNode | null {
  const clauses: MetadataFilterNode[] = [];

  if (fieldOption.fieldType === "set") {
    if (policy.any.length > 0) {
      clauses.push({ field: fieldOption.value, op: "includesAny", values: [...policy.any] } as MetadataFilterNode);
    }
    if (policy.all.length > 0) {
      clauses.push({ field: fieldOption.value, op: "includesAll", values: [...policy.all] } as MetadataFilterNode);
    }
    if (policy.exclude.length > 0) {
      clauses.push({ field: fieldOption.value, op: "excludesAny", values: [...policy.exclude] } as MetadataFilterNode);
    }
  }

  if (fieldOption.fieldType === "enumString") {
    if (policy.any.length === 1) {
      clauses.push({ field: fieldOption.value, op: "eq", value: policy.any[0]! } as MetadataFilterNode);
    } else if (policy.any.length > 1) {
      clauses.push({ field: fieldOption.value, op: "in", values: [...policy.any] } as MetadataFilterNode);
    }
    if (policy.exclude.length > 0) {
      clauses.push({ field: fieldOption.value, op: "notIn", values: [...policy.exclude] } as MetadataFilterNode);
    }
  }

  if (clauses.length === 0) {
    return null;
  }
  return clauses.length === 1 ? clauses[0]! : { and: clauses };
}

function buildPolicyFromPredicate(node: MetadataPredicate): Pf2eTerminalFilterValuePolicy<string> | null {
  const policy = createEmptyPolicy();
  if ("values" in node) {
    if (node.op === "includesAny" || node.op === "in") {
      policy.any = [...node.values.map((value) => String(value))];
    } else if (node.op === "includesAll") {
      policy.all = [...node.values.map((value) => String(value))];
    } else {
      policy.exclude = [...node.values.map((value) => String(value))];
    }
    return policy;
  }
  if ("value" in node) {
    if (node.op === "eq") {
      policy.any = [String(node.value)];
      return policy;
    }
  }
  return null;
}

type AddQueryPartCommand =
  | "profile"
  | "category"
  | "subcategory"
  | "levels"
  | "rarity"
  | "facet"
  | "clearFacet"
  | "clause"
  | "andGroup"
  | "orGroup"
  | "notGroup";

type ScopeCommand = "category" | "subcategory" | "clearScope";
type FacetCommand = "editFacet" | "clearFacet";

function countFacetPolicies(query: Pf2eTerminalSearchQuery): number {
  return query.filters.facets.length + (hasFilterPolicy(query.filters.actionCost) ? 1 : 0);
}

export function useSearchWorkspaceActions({
  applyQueryUpdate,
  dispatch,
  executeRequest,
  exitSearchScreen,
  jumpToResultPosition,
  maxDetailScroll,
  onOpenFacetPicker,
  origin,
  prompts,
  resultCount,
  selectedWorkspaceEntry,
  showSearchHelp,
  state,
  terminal,
  user,
  workspaceEntries,
  chooseResultSort,
}: {
  applyQueryUpdate: (update: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  dispatch: React.Dispatch<SearchScreenAction>;
  executeRequest: (query: Pf2eTerminalSearchQuery) => Promise<void>;
  exitSearchScreen: () => void;
  jumpToResultPosition: () => Promise<void>;
  maxDetailScroll: number;
  onOpenFacetPicker: () => Promise<void>;
  origin: SearchScreenOrigin;
  prompts: Pick<
    SearchTerminalPromptAdapters,
    | "promptCommandPalette"
    | "promptMultiSelectOption"
    | "promptOptionalSelectOption"
    | "promptPolicySelectOption"
    | "promptSelectOption"
    | "promptTextInput"
  >;
  resultCount: number;
  selectedWorkspaceEntry?: SearchWorkspaceEntry;
  showSearchHelp: () => void;
  state: SearchScreenState;
  terminal: Pick<DerivedTagTerminalApp, "pauseForAnyKey">;
  user: Pick<Pf2eTerminalAppServices["user"], "search">;
  workspaceEntries: SearchWorkspaceEntry[];
  chooseResultSort: () => Promise<void>;
}): {
  handleIntent: (intent: import("./search-screen-model.js").SearchScreenIntent) => void;
} {
  const editQueryText = React.useCallback(async () => {
    const queryText = await prompts.promptTextInput({
      title: "Query Text",
      prompt:
        state.query.mode === "lookup"
          ? "Enter an exact or near-exact record name"
          : "Enter search text for the current query",
      defaultValue: state.query.queryText,
      hint: state.query.mode === "lookup" ? "Example: Raise Shield" : "Example: ghost ship captain",
    });

    if (queryText === undefined) {
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      queryText,
    }));
  }, [applyQueryUpdate, prompts, state.query.mode, state.query.queryText]);

  const chooseMode = React.useCallback(async () => {
    const result = await prompts.promptSelectOption({
      title: "Query Mode",
      prompt: "Choose how the current query should execute",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.query.mode,
    });

    if (result.kind !== "selected") {
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      mode: result.value,
    }));
  }, [applyQueryUpdate, prompts, state.query.mode, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const result = await prompts.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the current profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.query.searchProfile,
    });

    if (result.kind === "selected") {
      applyQueryUpdate((request) => ({
        ...request,
        searchProfile: result.value,
      }));
    }
  }, [applyQueryUpdate, prompts, state.query.searchProfile, user.search]);

  const promptCommandSelection = React.useCallback(
    async <T extends string>(
      title: string,
      prompt: string,
      entries: DerivedTagTerminalCommandOption<T>[],
    ): Promise<T | undefined> => {
      const selected = await prompts.promptCommandPalette({
        title,
        prompt,
        entries,
      });
      if (!selected) {
        return undefined;
      }
      const selectedEntry = entries.find((entry) => entry.value === selected);
      if (selectedEntry?.disabled) {
        if (selectedEntry.disabledReason) {
          await terminal.pauseForAnyKey(selectedEntry.disabledReason);
        }
        return undefined;
      }
      return selected;
    },
    [prompts, terminal],
  );

  const chooseCategoryFilter = React.useCallback(async () => {
    const [allCategoryOption, ...categoryEntries] = user.search.getCategoryOptions();
    const result = await prompts.promptOptionalSelectOption({
      title: "Category Scope",
      prompt: "Choose the current category boundary",
      allOption: {
        label: allCategoryOption?.label ?? "Any Category",
        description: allCategoryOption?.description,
      },
      entries: categoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.query.filters.category ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        category: result.kind === "all" ? null : result.value,
        subcategory: null,
      },
    }));
  }, [applyQueryUpdate, prompts, state.query.filters.category, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!state.query.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const [allSubcategoryOption, ...subcategoryEntries] = user.search.getSubcategoryOptions(
      state.query.filters.category,
    );
    const result = await prompts.promptOptionalSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the current subcategory boundary",
      allOption: {
        label: allSubcategoryOption?.label ?? "Any Subcategory",
        description: allSubcategoryOption?.description,
      },
      entries: subcategoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.query.filters.subcategory ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        subcategory: result.kind === "all" ? null : result.value,
      },
    }));
  }, [applyQueryUpdate, prompts, state.query.filters.category, state.query.filters.subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(state.query.filters.category, state.query.filters.subcategory);
    const selected = await prompts.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: state.query.filters.rarity,
    });

    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        rarity: {
          any: selected.any,
          all: [],
          exclude: selected.exclude,
        },
      },
    }));
  }, [
    applyQueryUpdate,
    prompts,
    state.query.filters.category,
    state.query.filters.rarity,
    state.query.filters.subcategory,
    user.search,
  ]);

  const editLevelRange = React.useCallback(async () => {
    const input = await prompts.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue:
        state.query.filters.levelMin === null && state.query.filters.levelMax === null
          ? ""
          : formatLevelRange(state.query).replaceAll("L", "").replace("<= ", "<="),
      hint: "Examples: 3-8 or <=5",
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseLevelRangeInput(input);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        levelMin: parsed.levelMin,
        levelMax: parsed.levelMax,
      },
    }));
  }, [applyQueryUpdate, prompts, state.query, terminal]);

  const removeFacetFilter = React.useCallback(async () => {
    if (
      state.query.filters.facets.length === 0 &&
      state.query.filters.actionCost.any.length === 0 &&
      state.query.filters.actionCost.all.length === 0 &&
      state.query.filters.actionCost.exclude.length === 0
    ) {
      await terminal.pauseForAnyKey("There are no facet policies to clear from the current query.");
      return;
    }

    const selected = await prompts.promptMultiSelectOption({
      title: "Clear Facet Filter",
      prompt: "Toggle facet fields to clear. Press Esc or Left when finished.",
      entries: buildFacetRemovalEntries(state.query.filters.facets, state.query.filters.actionCost),
      selectedValues: [],
    });
    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        actionCost: selected.includes("actionCost") ? { any: [], all: [], exclude: [] } : request.filters.actionCost,
        facets: request.filters.facets.filter((facet) => !selected.includes(facet.field)),
      },
    }));
  }, [applyQueryUpdate, prompts, state.query.filters.actionCost, state.query.filters.facets, terminal]);

  const chooseQueryField = React.useCallback(async (): Promise<Pf2eTerminalQueryFieldOption | null> => {
    const fieldOptions = user.search.getQueryFieldOptions(
      state.query.filters.category,
      state.query.filters.subcategory,
    );
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
  }, [prompts, state.query.filters.category, state.query.filters.subcategory, terminal, user.search]);

  const editFieldClause = React.useCallback(
    async (
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null = null,
    ): Promise<MetadataFilterNode | null | undefined> => {
      if (fieldOption.fieldType === "set" || fieldOption.fieldType === "enumString") {
        const currentPolicy =
          currentNode && isMetadataPredicate(currentNode)
            ? (buildPolicyFromPredicate(currentNode) ?? createEmptyPolicy())
            : createEmptyPolicy();
        const selected = await prompts.promptPolicySelectOption({
          title: `${fieldOption.label} Clause`,
          prompt: "Cycle field values through include, require-all, or exclude. Press Esc or Left when finished.",
          allowedStates: fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
          entries: user.search
            .getFacetValueOptions(fieldOption.value, state.query.filters.category, state.query.filters.subcategory)
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
        return { field: fieldOption.value, op: "eq", value: result.value === "true" } as MetadataFilterNode;
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
        return { field: fieldOption.value, op: opResult.value, value: value.trim() } as MetadataFilterNode;
      }

      const currentOp =
        currentNode &&
        isMetadataPredicate(currentNode) &&
        "op" in currentNode &&
        ["eq", "gte", "lte", "between"].includes(currentNode.op)
          ? currentNode.op
          : "eq";
      const opResult = await prompts.promptSelectOption({
        title: `${fieldOption.label} Clause`,
        prompt: "Choose the numeric comparison for this clause",
        entries: [
          { value: "eq", label: "Equals", description: "Match exactly one value." },
          { value: "gte", label: "At Least", description: "Match values greater than or equal to the target." },
          { value: "lte", label: "At Most", description: "Match values less than or equal to the target." },
          { value: "between", label: "Between", description: "Match values inside an inclusive range." },
        ],
        selectedValue: currentOp,
      });

      if (opResult.kind !== "selected") {
        return undefined;
      }

      const currentInput =
        currentNode && isMetadataPredicate(currentNode)
          ? "min" in currentNode && "max" in currentNode
            ? `${currentNode.min}-${currentNode.max}`
            : "value" in currentNode
              ? String(currentNode.value)
              : ""
          : "";
      const value = await prompts.promptTextInput({
        title: `${fieldOption.label} Value`,
        prompt:
          opResult.value === "between"
            ? "Enter an inclusive range such as `3-8`. Leave blank to clear."
            : "Enter a numeric value. Leave blank to clear.",
        defaultValue: currentInput,
      });

      if (value === undefined) {
        return undefined;
      }
      const trimmed = value.trim();
      if (!trimmed) {
        return null;
      }
      if (opResult.value === "between") {
        const match = trimmed.match(/^(-?\d+(?:\.\d+)?)\s*-\s*(-?\d+(?:\.\d+)?)$/);
        if (!match) {
          await terminal.pauseForAnyKey("Use a numeric range such as `3-8`.");
          return undefined;
        }
        return {
          field: fieldOption.value,
          op: "between",
          min: Number.parseFloat(match[1]!),
          max: Number.parseFloat(match[2]!),
        } as MetadataFilterNode;
      }
      const numericValue = Number.parseFloat(trimmed);
      if (!Number.isFinite(numericValue)) {
        await terminal.pauseForAnyKey("Enter a valid number.");
        return undefined;
      }
      return {
        field: fieldOption.value,
        op: opResult.value,
        value: numericValue,
      } as MetadataFilterNode;
    },
    [prompts, state.query.filters.category, state.query.filters.subcategory, terminal, user.search],
  );

  const addQueryClauseAtPath = React.useCallback(
    async (path: number[] = []) => {
      const fieldOption = await chooseQueryField();
      if (!fieldOption) {
        return;
      }
      const clause = await editFieldClause(fieldOption);
      if (!clause) {
        return;
      }
      applyQueryUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          metadata: appendMetadataNodeAtPath(request.filters.metadata, path, clause),
        },
      }));
    },
    [applyQueryUpdate, chooseQueryField, editFieldClause],
  );

  const addQueryGroupAtPath = React.useCallback(
    async (path: number[], groupKind: "and" | "or" | "not") => {
      const fieldOption = await chooseQueryField();
      if (!fieldOption) {
        return;
      }
      const clause = await editFieldClause(fieldOption);
      if (!clause) {
        return;
      }
      const group: MetadataFilterNode =
        groupKind === "and" ? { and: [clause] } : groupKind === "or" ? { or: [clause] } : { not: clause };
      applyQueryUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          metadata: appendMetadataNodeAtPath(request.filters.metadata, path, group),
        },
      }));
    },
    [applyQueryUpdate, chooseQueryField, editFieldClause],
  );

  const openScopeQueryPart = React.useCallback(async () => {
    const selected = await promptCommandSelection<ScopeCommand>("Scope", "Adjust the current query scope", [
      {
        value: "category",
        label: "Set Category",
        description: `Current: ${formatSearchCategory(state.query.filters.category)}.`,
      },
      {
        value: "subcategory",
        label: "Set Subcategory",
        description: `Current: ${formatSearchSubcategory(state.query.filters.subcategory)}.`,
        disabled: !state.query.filters.category,
        disabledReason: "Choose a category before refining to a subcategory.",
      },
      {
        value: "clearScope",
        label: "Clear Scope",
        description: "Remove the current category and subcategory boundary.",
        disabled: !state.query.filters.category && !state.query.filters.subcategory,
        disabledReason: "No category scope is currently applied.",
      },
    ]);

    if (!selected) {
      return;
    }

    if (selected === "category") {
      await chooseCategoryFilter();
      return;
    }
    if (selected === "subcategory") {
      await chooseSubcategoryFilter();
      return;
    }

    applyQueryUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        category: null,
        subcategory: null,
      },
    }));
  }, [
    applyQueryUpdate,
    chooseCategoryFilter,
    chooseSubcategoryFilter,
    promptCommandSelection,
    state.query.filters.category,
    state.query.filters.subcategory,
  ]);

  const openFacetQueryPart = React.useCallback(async () => {
    const facetPolicyCount = countFacetPolicies(state.query);
    const selected = await promptCommandSelection<FacetCommand>("Facet Filters", "Adjust discoverable facet filters", [
      {
        value: "editFacet",
        label: "Edit Facet Filter",
        description: `Current: ${facetPolicyCount} active facet block${facetPolicyCount === 1 ? "" : "s"}.`,
        disabled: !state.query.filters.category,
        disabledReason: "Choose a category before editing discoverable facet filters.",
      },
      {
        value: "clearFacet",
        label: "Clear Facet Filter",
        description: "Remove one or more facet policy blocks from the current query.",
        disabled: facetPolicyCount === 0,
        disabledReason: "No facet policies are currently applied.",
      },
    ]);

    if (!selected) {
      return;
    }

    if (selected === "editFacet") {
      await onOpenFacetPicker();
      return;
    }

    await removeFacetFilter();
  }, [
    onOpenFacetPicker,
    promptCommandSelection,
    removeFacetFilter,
    state.query,
    state.query.filters.category,
  ]);

  const openAddQueryPart = React.useCallback(async () => {
    const facetPolicyCount = countFacetPolicies(state.query);
    const requiresScopedQuery = !state.query.filters.category;
    const entries: DerivedTagTerminalCommandOption<AddQueryPartCommand>[] = [
      ...(state.query.mode === "search"
        ? [
            {
              value: "profile" as const,
              label: "Choose Search Profile",
              description: `Current: ${state.query.searchProfile}.`,
            },
          ]
        : []),
      {
        value: "category",
        label: "Set Category",
        description: `Current: ${formatSearchCategory(state.query.filters.category)}.`,
      },
      {
        value: "subcategory",
        label: "Set Subcategory",
        description: `Current: ${formatSearchSubcategory(state.query.filters.subcategory)}.`,
        disabled: !state.query.filters.category,
        disabledReason: "Choose a category before refining to a subcategory.",
      },
      {
        value: "levels",
        label: "Set Level Range",
        description: `Current: ${formatLevelRange(state.query)}.`,
      },
      {
        value: "rarity",
        label: "Set Rarity",
        description: `Current: ${formatFilterPolicy(state.query.filters.rarity)}.`,
      },
      {
        value: "facet",
        label: "Edit Facet Filter",
        description: `Current: ${facetPolicyCount} active facet block${facetPolicyCount === 1 ? "" : "s"}.`,
        disabled: !state.query.filters.category,
        disabledReason: "Choose a category before editing discoverable facet filters.",
      },
      {
        value: "clearFacet",
        label: "Clear Facet Filter",
        description: "Remove one or more facet policy blocks from the current query.",
        disabled: facetPolicyCount === 0,
        disabledReason: "No facet policies are currently applied.",
      },
      {
        value: "clause",
        label: "Add Query Clause",
        description: "Add a metadata clause at the root of the unified query.",
        disabled: requiresScopedQuery,
        disabledReason: "Choose a category before adding scoped metadata clauses.",
      },
      {
        value: "andGroup",
        label: "Add AND Group",
        description: "Add an AND logic group with one initial clause.",
        disabled: requiresScopedQuery,
        disabledReason: "Choose a category before adding scoped metadata clauses.",
      },
      {
        value: "orGroup",
        label: "Add OR Group",
        description: "Add an OR logic group with one initial clause.",
        disabled: requiresScopedQuery,
        disabledReason: "Choose a category before adding scoped metadata clauses.",
      },
      {
        value: "notGroup",
        label: "Add NOT Group",
        description: "Add a NOT logic group with one initial clause.",
        disabled: requiresScopedQuery,
        disabledReason: "Choose a category before adding scoped metadata clauses.",
      },
    ];

    const selected = await promptCommandSelection("Add Query Part", "Filter query parts to add or adjust", entries);
    if (!selected) {
      return;
    }

    switch (selected) {
      case "profile":
        await chooseSearchProfile();
        return;
      case "category":
        await chooseCategoryFilter();
        return;
      case "subcategory":
        await chooseSubcategoryFilter();
        return;
      case "levels":
        await editLevelRange();
        return;
      case "rarity":
        await chooseRarityFilter();
        return;
      case "facet":
        await onOpenFacetPicker();
        return;
      case "clearFacet":
        await removeFacetFilter();
        return;
      case "clause":
        await addQueryClauseAtPath();
        return;
      case "andGroup":
      case "orGroup":
      case "notGroup":
        await addQueryGroupAtPath([], selected === "andGroup" ? "and" : selected === "orGroup" ? "or" : "not");
        return;
    }
  }, [
    addQueryClauseAtPath,
    addQueryGroupAtPath,
    chooseCategoryFilter,
    chooseRarityFilter,
    chooseSearchProfile,
    chooseSubcategoryFilter,
    editLevelRange,
    onOpenFacetPicker,
    promptCommandSelection,
    removeFacetFilter,
    state.query,
    state.query.filters.category,
    state.query.filters.rarity,
    state.query.filters.subcategory,
    state.query.mode,
    state.query.searchProfile,
  ]);

  const editQueryNode = React.useCallback(
    async (path: number[]) => {
      const node = getMetadataNodeAtPath(state.query.filters.metadata, path);
      if (!node) {
        return;
      }

      if (isMetadataPredicate(node)) {
        const scopedFields = user.search.getQueryFieldOptions(
          state.query.filters.category,
          state.query.filters.subcategory,
        );
        const fieldOption = scopedFields.find((candidate) => candidate.value === node.field) ?? null;
        const actionEntries = [
          ...(fieldOption
            ? [{ value: "edit", label: "Edit Clause", description: "Change the field operator or value." }]
            : []),
          { value: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
          { value: "remove", label: "Remove Clause", description: "Delete this clause from the query." },
        ];
        const result = await prompts.promptSelectOption({
          title: "Query Clause",
          prompt: "Choose how to update this query clause",
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
          const nextNode = await editFieldClause(fieldOption, node);
          if (nextNode === undefined) {
            return;
          }
          applyQueryUpdate((request) => ({
            ...request,
            filters: {
              ...request.filters,
              metadata: updateMetadataNodeAtPath(request.filters.metadata, path, () => nextNode),
            },
          }));
          return;
        }

        if (result.value === "wrapNot") {
          applyQueryUpdate((request) => ({
            ...request,
            filters: {
              ...request.filters,
              metadata: updateMetadataNodeAtPath(request.filters.metadata, path, (current) => ({ not: current })),
            },
          }));
          return;
        }

        applyQueryUpdate((request) => ({
          ...request,
          filters: {
            ...request.filters,
            metadata: updateMetadataNodeAtPath(request.filters.metadata, path, () => null),
          },
        }));
        return;
      }

      if ("not" in node) {
        const result = await prompts.promptSelectOption({
          title: "NOT Group",
          prompt: "Choose how to update this negated clause",
          entries: [
            { value: "unwrap", label: "Remove NOT", description: "Keep the child clause and remove the negation." },
            { value: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
          ],
          selectedValue: "unwrap",
        });

        if (result.kind !== "selected") {
          return;
        }

        applyQueryUpdate((request) => ({
          ...request,
          filters: {
            ...request.filters,
            metadata: updateMetadataNodeAtPath(request.filters.metadata, path, (current) =>
              result.value === "unwrap" && "not" in current ? current.not : null,
            ),
          },
        }));
        return;
      }

      const isAndGroup = "and" in node;
      const result = await prompts.promptSelectOption({
        title: isAndGroup ? "AND Group" : "OR Group",
        prompt: "Choose how to update this logic group",
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
          { value: "remove", label: "Remove Group", description: "Delete this logic group from the query." },
        ],
        selectedValue: "addClause",
      });

      if (result.kind !== "selected") {
        return;
      }

      if (result.value === "addClause") {
        await addQueryClauseAtPath(path);
        return;
      }
      if (result.value === "addAndGroup" || result.value === "addOrGroup" || result.value === "addNotGroup") {
        await addQueryGroupAtPath(
          path,
          result.value === "addAndGroup" ? "and" : result.value === "addOrGroup" ? "or" : "not",
        );
        return;
      }
      if (result.value === "toggle") {
        applyQueryUpdate((request) => ({
          ...request,
          filters: {
            ...request.filters,
            metadata: updateMetadataNodeAtPath(request.filters.metadata, path, (current) =>
              "and" in current ? { or: [...current.and] } : "or" in current ? { and: [...current.or] } : current,
            ),
          },
        }));
        return;
      }
      applyQueryUpdate((request) => ({
        ...request,
        filters: {
          ...request.filters,
          metadata: updateMetadataNodeAtPath(request.filters.metadata, path, () => null),
        },
      }));
    },
    [
      addQueryClauseAtPath,
      addQueryGroupAtPath,
      applyQueryUpdate,
      editFieldClause,
      prompts,
      state.query.filters.category,
      state.query.filters.metadata,
      state.query.filters.subcategory,
      terminal,
      user.search,
    ],
  );

  const resetQueryEditor = React.useCallback(() => {
    const defaultQuery = user.search.createDefaultQuery();
    dispatch({ type: "set_query", query: defaultQuery });
    dispatch({ type: "set_layout", layout: "editor", pane: "list" });
  }, [dispatch, user.search]);

  const runWorkspaceAction = React.useCallback(
    (action: SearchWorkspaceAction) => {
      const entry = workspaceEntries.find((candidate) => candidate.action === action);
      if (entry?.disabled) {
        return;
      }

      switch (action) {
        case "execute":
          void executeRequest(state.query);
          return;
        case "mode":
          void chooseMode();
          return;
        case "query":
          void editQueryText();
          return;
        case "profile":
          void chooseSearchProfile();
          return;
        case "addQueryPart":
          void openAddQueryPart();
          return;
        case "clearClauses":
          applyQueryUpdate((request) => ({
            ...request,
            filters: {
              ...request.filters,
              metadata: null,
            },
          }));
          return;
        case "reset":
          resetQueryEditor();
          return;
        case "clearResults":
          dispatch({ type: "clear_results" });
          return;
      }

      if (isQueryNodeAction(action)) {
        const path = decodeQueryNodeActionPath(action);
        if (!path) {
          return;
        }
        void editQueryNode(path);
        return;
      }

      if (isQueryPartAction(action)) {
        const part = decodeQueryPartAction(action);
        if (!part) {
          return;
        }
        if (part === "scope") {
          void openScopeQueryPart();
          return;
        }
        if (part === "levels") {
          void editLevelRange();
          return;
        }
        if (part === "rarity") {
          void chooseRarityFilter();
          return;
        }
        if (part === "facets") {
          void openFacetQueryPart();
        }
      }
    },
    [
      applyQueryUpdate,
      chooseMode,
      chooseRarityFilter,
      chooseSearchProfile,
      dispatch,
      editQueryNode,
      editLevelRange,
      editQueryText,
      executeRequest,
      openAddQueryPart,
      openFacetQueryPart,
      openScopeQueryPart,
      resetQueryEditor,
      state.query,
      workspaceEntries,
    ],
  );

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }
    runWorkspaceAction(selectedWorkspaceEntry.action);
  }, [runWorkspaceAction, selectedWorkspaceEntry]);

  const openEditorCommandPalette = React.useCallback(async () => {
    const selected = await prompts.promptCommandPalette({
      title: "Query Editor Commands",
      prompt: "Filter query editor commands",
      entries: buildEditorCommandPaletteEntries(workspaceEntries),
    });
    if (!selected) {
      return;
    }
    if (workspaceEntries.find((entry) => entry.action === selected)?.disabled) {
      return;
    }
    runWorkspaceAction(selected);
  }, [prompts, runWorkspaceAction, workspaceEntries]);

  const openResultCommandPalette = React.useCallback(async () => {
    const selected = await prompts.promptCommandPalette({
      title: "Result Commands",
      prompt: "Filter result commands",
      entries: buildResultCommandPaletteEntries(state, origin),
    });
    if (selected === "jumpToResult") {
      void jumpToResultPosition();
      return;
    }
    if (selected === "sortResults") {
      void chooseResultSort();
      return;
    }
    if (selected === "openEditor") {
      dispatch({ type: "set_layout", layout: "editor", pane: "list" });
    }
  }, [chooseResultSort, dispatch, jumpToResultPosition, origin, prompts, state]);

  const handleIntent = React.useCallback(
    (intent: import("./search-screen-model.js").SearchScreenIntent) => {
      switch (intent.type) {
        case "show_help":
          showSearchHelp();
          return;
        case "quit":
          exitSearchScreen();
          return;
        case "edit_query":
          void editQueryText();
          return;
        case "open_editor_commands":
          void openEditorCommandPalette();
          return;
        case "execute":
          void executeRequest(state.query);
          return;
        case "back_to_app":
          exitSearchScreen();
          return;
        case "move_workspace_selection":
          dispatch({
            type: "move_workspace_selection",
            delta: intent.delta,
            itemCount: workspaceEntries.length,
          });
          return;
        case "workspace_selection_boundary":
          dispatch({
            type: "workspace_selection_boundary",
            boundary: intent.boundary,
            itemCount: workspaceEntries.length,
          });
          return;
        case "edit_selected_workspace":
          openSelectedWorkspaceEntry();
          return;
        case "open_result_commands":
          void openResultCommandPalette();
          return;
        case "toggle_pane":
          dispatch({ type: "set_active_pane", pane: state.activePane === "list" ? "detail" : "list" });
          return;
        case "return_to_editor":
          if (origin === "ontology") {
            exitSearchScreen();
            return;
          }
          dispatch({ type: "set_layout", layout: "editor", pane: "list" });
          return;
        case "open_preview":
          dispatch({ type: "set_active_pane", pane: "detail" });
          return;
        case "move_result_selection":
          dispatch({ type: "move_result_selection", delta: intent.delta, itemCount: resultCount });
          return;
        case "result_selection_boundary":
          dispatch({
            type: "result_selection_boundary",
            boundary: intent.boundary,
            itemCount: resultCount,
          });
          return;
        case "return_to_result_list":
          dispatch({ type: "set_active_pane", pane: "list" });
          return;
        case "move_detail":
          dispatch({ type: "move_detail", delta: intent.delta, maxDetailScroll });
          return;
        case "detail_boundary":
          dispatch({ type: "detail_boundary", boundary: intent.boundary, maxDetailScroll });
          return;
      }
    },
    [
      dispatch,
      editQueryText,
      executeRequest,
      exitSearchScreen,
      maxDetailScroll,
      openEditorCommandPalette,
      openResultCommandPalette,
      openSelectedWorkspaceEntry,
      origin,
      resultCount,
      showSearchHelp,
      state.activePane,
      state.query,
      workspaceEntries.length,
    ],
  );

  return {
    handleIntent,
  };
}
