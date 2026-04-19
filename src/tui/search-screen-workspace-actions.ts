import React from "react";

import type { MetadataFilterNode, MetadataPredicate } from "../types.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { SearchTerminalPromptAdapters } from "./interaction-context-adapters.js";
import type { SearchScreenAction, SearchScreenState } from "./search-screen-state.js";
import type { SearchQueryFieldBuilderSession } from "./search-query-field-builder-session.js";
import type {
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./search-service.js";
import {
  getSearchQueryActionCostPolicy,
  getSearchQueryCategory,
  getSearchQueryLevelRange,
  getSearchQueryMetadataTree,
  getSearchQueryPart,
  getSearchQueryRarityPolicy,
  getSearchQuerySubcategory,
  removeSearchQueryPart,
  setSearchQueryCategory,
  setSearchQueryMetadataTree,
  setSearchQueryPart,
} from "./search-service.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import type { DerivedTagTerminalApp } from "./terminal-ui.js";
import {
  buildEditorCommandPaletteEntries,
  buildResultCommandPaletteEntries,
  decodeQueryPartAction,
  decodeQueryNodeActionPath,
  formatFilterPolicy,
  formatLevelRange,
  formatSearchCategory,
  formatSearchSubcategory,
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

function createEmptyStringPolicy(): Pf2eTerminalFilterValuePolicy<string> {
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
  const policy = createEmptyStringPolicy();
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

function buildSelectionMap(
  field: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): Pf2eTerminalQueryFieldSelectionMap {
  return {
    [field]: {
      any: [...policy.any],
      all: [...policy.all],
      exclude: [...policy.exclude],
    },
  };
}

type QueryFieldBuilderState = {
  path: number[];
  items: SearchQueryFieldBuilderSession["items"];
  selectedIndex: number;
  fieldDrafts: Record<string, MetadataFilterNode | null>;
};

function buildPolicyFromMetadataNode(node: MetadataFilterNode | null): Pf2eTerminalFilterValuePolicy<string> {
  if (!node) {
    return createEmptyStringPolicy();
  }

  if ("and" in node) {
    return node.and.reduce((policy, child) => {
      const childPolicy = buildPolicyFromMetadataNode(child);
      return {
        any: [...policy.any, ...childPolicy.any],
        all: [...policy.all, ...childPolicy.all],
        exclude: [...policy.exclude, ...childPolicy.exclude],
      };
    }, createEmptyStringPolicy());
  }

  if ("or" in node || "not" in node) {
    return createEmptyStringPolicy();
  }

  return buildPolicyFromPredicate(node) ?? createEmptyStringPolicy();
}

function buildQueryFieldBuilderItems(fieldOptions: Pf2eTerminalQueryFieldOption[]): SearchQueryFieldBuilderSession["items"] {
  return [
    ...fieldOptions.map((fieldOption) => ({
      kind: "field" as const,
      fieldOption,
      label: fieldOption.label,
    })),
    { kind: "finish" as const, label: "Finish Clause Builder" },
    { kind: "cancel" as const, label: "Cancel" },
  ];
}

function compileQueryFieldBuilderDrafts(fieldDrafts: Record<string, MetadataFilterNode | null>): MetadataFilterNode | null {
  const nodes = Object.values(fieldDrafts)
    .map((node) => normalizeMetadataNode(node))
    .filter((node): node is MetadataFilterNode => Boolean(node));

  if (nodes.length === 0) {
    return null;
  }
  if (nodes.length === 1) {
    return nodes[0]!;
  }
  return { and: nodes };
}

type AddQueryPartCommand =
  | "category"
  | "subcategory"
  | "levelRange"
  | "rarity"
  | "actionCost"
  | "clause"
  | "andGroup"
  | "orGroup"
  | "notGroup";

export function useSearchWorkspaceActions({
  applyQueryUpdate,
  dispatch,
  executeRequest,
  exitSearchScreen,
  jumpToResultPosition,
  maxDetailScroll,
  openQueryFieldPicker,
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
  openQueryFieldPicker: (options: {
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    initialSelections?: Pf2eTerminalQueryFieldSelectionMap;
    onApply: (selection: Pf2eTerminalQueryFieldSelectionMap) => void;
    onReturn?: () => void;
    singleFieldBehavior?: "list" | "directValues";
  }) => Promise<boolean>;
  origin: SearchScreenOrigin;
  prompts: Pick<
    SearchTerminalPromptAdapters,
    | "promptCommandPalette"
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
  queryFieldBuilderSession: SearchQueryFieldBuilderSession | null;
} {
  const category = getSearchQueryCategory(state.query);
  const subcategory = getSearchQuerySubcategory(state.query);
  const levelRange = getSearchQueryLevelRange(state.query);
  const rarityPolicy = getSearchQueryRarityPolicy(state.query);
  const actionCostPolicy = getSearchQueryActionCostPolicy(state.query);
  const metadataTree = getSearchQueryMetadataTree(state.query);
  const [queryFieldBuilderState, setQueryFieldBuilderState] = React.useState<QueryFieldBuilderState | null>(null);

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
      selectedValue: category,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyQueryUpdate((request) => setSearchQueryCategory(request, result.kind === "all" ? null : result.value));
  }, [applyQueryUpdate, prompts, category, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const [allSubcategoryOption, ...subcategoryEntries] = user.search.getSubcategoryOptions(category);
    if (subcategoryEntries.length === 0) {
      await terminal.pauseForAnyKey("No subcategories are available for the current category.");
      return;
    }
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
      selectedValue: subcategory,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyQueryUpdate((request) =>
      result.kind === "all"
        ? removeSearchQueryPart(request, "subcategory")
        : result.value
          ? setSearchQueryPart(request, { kind: "subcategory", subcategory: result.value })
          : request,
    );
  }, [applyQueryUpdate, category, prompts, subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(category, subcategory);
    const selected = await prompts.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: rarityPolicy,
    });

    applyQueryUpdate((request) =>
      selected.any.length === 0 && selected.exclude.length === 0
        ? removeSearchQueryPart(request, "rarityPolicy")
        : setSearchQueryPart(request, {
            kind: "rarityPolicy",
            policy: {
              any: selected.any,
              all: [],
              exclude: selected.exclude,
            },
          }),
    );
  }, [applyQueryUpdate, category, prompts, rarityPolicy, subcategory, user.search]);

  const chooseActionCostFilter = React.useCallback(async () => {
    const options = user.search.getActionCostOptions(category, subcategory);
    if (options.length === 0) {
      await terminal.pauseForAnyKey("No action-cost filters are available for the current query.");
      return;
    }

    const selected = await prompts.promptPolicySelectOption({
      title: "Action Cost Filter",
      prompt: "Cycle action costs through include or exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: {
        any: actionCostPolicy.any.map(String),
        all: [],
        exclude: actionCostPolicy.exclude.map(String),
      },
    });

    const nextPolicy = {
      any: selected.any.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
      all: [] as number[],
      exclude: selected.exclude.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
    };

    applyQueryUpdate((request) =>
      nextPolicy.any.length === 0 && nextPolicy.exclude.length === 0
        ? removeSearchQueryPart(request, "actionCostPolicy")
        : setSearchQueryPart(request, { kind: "actionCostPolicy", policy: nextPolicy }),
    );
  }, [actionCostPolicy.any, actionCostPolicy.exclude, applyQueryUpdate, category, prompts, subcategory, terminal, user.search]);

  const editLevelRange = React.useCallback(async () => {
    const input = await prompts.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue:
        levelRange.levelMin === null && levelRange.levelMax === null
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

    applyQueryUpdate((request) =>
      parsed.levelMin === null && parsed.levelMax === null
        ? removeSearchQueryPart(request, "levelRange")
        : setSearchQueryPart(request, {
            kind: "levelRange",
            levelMin: parsed.levelMin,
            levelMax: parsed.levelMax,
          }),
    );
  }, [applyQueryUpdate, levelRange.levelMax, levelRange.levelMin, prompts, state.query, terminal]);

  const hasSelectableSubcategories = React.useCallback(
    (category: Pf2eTerminalSearchQuery["filters"]["category"]): boolean => {
      if (!category) {
        return false;
      }
      return user.search.getSubcategoryOptions(category).length > 1;
    },
    [user.search],
  );

  const chooseQueryField = React.useCallback(async (): Promise<Pf2eTerminalQueryFieldOption | null> => {
    const fieldOptions = user.search.getQueryFieldOptions(category, subcategory);
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
  }, [category, prompts, subcategory, terminal, user.search]);

  const openOntologyFieldEditor = React.useCallback(
    async (
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentPolicy: Pf2eTerminalFilterValuePolicy<string>,
      onApply: (nextNode: MetadataFilterNode | null) => void,
    ): Promise<boolean> => {
      if (fieldOption.editor !== "ontologyPicker") {
        return false;
      }

      return openQueryFieldPicker({
        fieldOptions: [fieldOption],
        initialSelections: buildSelectionMap(fieldOption.value, currentPolicy),
        singleFieldBehavior: "directValues",
        onApply: (selection) => {
          const nextNode =
            buildMetadataNodeFromPolicy(fieldOption, selection[fieldOption.value] ?? createEmptyStringPolicy());
          onApply(nextNode);
        },
      });
    },
    [openQueryFieldPicker],
  );

  const editFieldClause = React.useCallback(
    async (
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null = null,
    ): Promise<MetadataFilterNode | null | undefined> => {
      if (fieldOption.fieldType === "set" || fieldOption.fieldType === "enumString") {
        const currentPolicy =
          currentNode && isMetadataPredicate(currentNode)
            ? (buildPolicyFromPredicate(currentNode) ?? createEmptyStringPolicy())
            : createEmptyStringPolicy();
        const selected = await prompts.promptPolicySelectOption({
          title: `${fieldOption.label} Clause`,
          prompt: "Cycle field values through include, require-all, or exclude. Press Esc or Left when finished.",
          allowedStates: fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
          entries: user.search
            .getFacetValueOptions(fieldOption.value, category, subcategory)
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
    [category, prompts, subcategory, terminal, user.search],
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

  const editQueryFieldBuilderField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const currentNode = queryFieldBuilderState?.fieldDrafts[fieldOption.value] ?? null;
      if (fieldOption.editor === "ontologyPicker") {
        await openOntologyFieldEditor(fieldOption, buildPolicyFromMetadataNode(currentNode), (nextNode) => {
          updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
        });
        return;
      }

      const nextNode = await editFieldClause(fieldOption, currentNode);
      if (nextNode === undefined) {
        return;
      }
      updateQueryFieldBuilderDraft(fieldOption.value, nextNode);
    },
    [editFieldClause, openOntologyFieldEditor, queryFieldBuilderState?.fieldDrafts, updateQueryFieldBuilderDraft],
  );

  const openQueryFieldBuilder = React.useCallback(
    async (path: number[] = []) => {
      const fieldOptions = user.search.getQueryFieldOptions(category, subcategory);
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
        return;
      }

      setQueryFieldBuilderState({
        path,
        items: buildQueryFieldBuilderItems(fieldOptions),
        selectedIndex: 0,
        fieldDrafts: {},
      });

      if (fieldOptions.length === 1 && fieldOptions[0]!.editor === "ontologyPicker") {
        const onlyFieldOption = fieldOptions[0]!;
        setTimeout(() => {
          void editQueryFieldBuilderField(onlyFieldOption);
        }, 0);
      }
    },
    [category, editQueryFieldBuilderField, subcategory, terminal, user.search],
  );

  const finishQueryFieldBuilder = React.useCallback(() => {
    const nextNode = compileQueryFieldBuilderDrafts(queryFieldBuilderState?.fieldDrafts ?? {});
    const targetPath = queryFieldBuilderState?.path ?? [];
    setQueryFieldBuilderState(null);
    if (!nextNode) {
      return;
    }
    applyQueryUpdate((request) =>
      setSearchQueryMetadataTree(request, appendMetadataNodeAtPath(getSearchQueryMetadataTree(request), targetPath, nextNode)),
    );
  }, [applyQueryUpdate, queryFieldBuilderState]);

  const cancelQueryFieldBuilder = React.useCallback(() => {
    setQueryFieldBuilderState(null);
  }, []);

  const selectCurrentQueryFieldBuilderItem = React.useCallback(() => {
    const selectedItem =
      queryFieldBuilderState?.items[Math.max(0, Math.min(queryFieldBuilderState.selectedIndex, queryFieldBuilderState.items.length - 1))];
    if (!selectedItem) {
      return;
    }
    if (selectedItem.kind === "field") {
      void editQueryFieldBuilderField(selectedItem.fieldOption);
      return;
    }
    if (selectedItem.kind === "finish") {
      finishQueryFieldBuilder();
      return;
    }
    cancelQueryFieldBuilder();
  }, [cancelQueryFieldBuilder, editQueryFieldBuilderField, finishQueryFieldBuilder, queryFieldBuilderState]);

  const queryFieldBuilderSession = React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!queryFieldBuilderState) {
      return null;
    }
    return {
      items: queryFieldBuilderState.items.map((item) =>
        item.kind === "field"
          ? {
              ...item,
              label: queryFieldBuilderState.fieldDrafts[item.fieldOption.value] ? `${item.fieldOption.label} | staged` : item.fieldOption.label,
            }
          : item,
      ),
      selectedIndex: queryFieldBuilderState.selectedIndex,
      fieldDrafts: queryFieldBuilderState.fieldDrafts,
      moveSelection: (delta, itemCount) => {
        setQueryFieldBuilderState((current) => {
          if (!current) {
            return current;
          }
          const nextIndex = Math.max(0, Math.min(itemCount - 1, current.selectedIndex + delta));
          return {
            ...current,
            selectedIndex: nextIndex,
          };
        });
      },
      selectCurrent: selectCurrentQueryFieldBuilderItem,
      finish: finishQueryFieldBuilder,
      cancel: cancelQueryFieldBuilder,
    };
  }, [cancelQueryFieldBuilder, finishQueryFieldBuilder, queryFieldBuilderState, selectCurrentQueryFieldBuilderItem]);

  const addQueryClauseAtPath = React.useCallback(
    async (path: number[] = []) => {
      await openQueryFieldBuilder(path);
    },
    [openQueryFieldBuilder],
  );

  const addQueryGroupAtPath = React.useCallback(
    async (path: number[], groupKind: "and" | "or" | "not") => {
      const fieldOption = await chooseQueryField();
      if (!fieldOption) {
        return;
      }
      if (fieldOption.editor === "ontologyPicker") {
        await openOntologyFieldEditor(fieldOption, createEmptyStringPolicy(), (nextNode) => {
          if (!nextNode) {
            return;
          }
          const group: MetadataFilterNode =
            groupKind === "and" ? { and: [nextNode] } : groupKind === "or" ? { or: [nextNode] } : { not: nextNode };
          applyQueryUpdate((request) =>
            setSearchQueryMetadataTree(request, appendMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, group)),
          );
        });
        return;
      }
      const clause = await editFieldClause(fieldOption);
      if (!clause) {
        return;
      }
      const group: MetadataFilterNode =
        groupKind === "and" ? { and: [clause] } : groupKind === "or" ? { or: [clause] } : { not: clause };
      applyQueryUpdate((request) =>
        setSearchQueryMetadataTree(request, appendMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, group)),
      );
    },
    [applyQueryUpdate, chooseQueryField, editFieldClause, openOntologyFieldEditor],
  );

  const openAddQueryPart = React.useCallback(async () => {
    const hasCategory = Boolean(category);
    const hasSubcategory = Boolean(getSearchQueryPart(state.query, "subcategory"));
    const hasLevelRange = Boolean(getSearchQueryPart(state.query, "levelRange"));
    const hasRarity = Boolean(getSearchQueryPart(state.query, "rarityPolicy"));
    const hasActionCost = Boolean(getSearchQueryPart(state.query, "actionCostPolicy"));
    const hasActionCostOptions = user.search.getActionCostOptions(category, subcategory).length > 0;
    const entries: Array<{ value: AddQueryPartCommand; label: string; description: string }> = [
      ...(!hasCategory
        ? [
            {
              value: "category" as const,
              label: "Add Category",
              description: `Current: ${formatSearchCategory(category)}.`,
            },
          ]
        : []),
      ...(!hasSubcategory && hasSelectableSubcategories(category)
        ? [
            {
              value: "subcategory" as const,
              label: "Add Subcategory",
              description: `Current: ${formatSearchSubcategory(subcategory)}.`,
            },
          ]
        : []),
      ...(!hasLevelRange
        ? [
            {
              value: "levelRange" as const,
              label: "Add Level Range",
              description: `Current: ${formatLevelRange(state.query)}.`,
            },
          ]
        : []),
      ...(!hasRarity
        ? [
            {
              value: "rarity" as const,
              label: "Add Rarity",
              description: `Current: ${formatFilterPolicy(rarityPolicy)}.`,
            },
          ]
        : []),
      ...(!hasActionCost && hasActionCostOptions
        ? [
            {
              value: "actionCost" as const,
              label: "Add Action Cost",
              description: `Current: ${formatFilterPolicy(actionCostPolicy)}.`,
            },
          ]
        : []),
      ...(hasCategory
        ? [
            {
              value: "clause" as const,
              label: "Add Query Clause",
              description: "Add a metadata clause at the root of the unified query.",
            },
            {
              value: "andGroup" as const,
              label: "Add AND Group",
              description: "Add an AND logic group with one initial clause.",
            },
            {
              value: "orGroup" as const,
              label: "Add OR Group",
              description: "Add an OR logic group with one initial clause.",
            },
            {
              value: "notGroup" as const,
              label: "Add NOT Group",
              description: "Add a NOT logic group with one initial clause.",
            },
          ]
        : []),
    ];

    if (entries.length === 0) {
      await terminal.pauseForAnyKey("No additional query parts are available from the current root scope.");
      return;
    }

    const result = await prompts.promptSelectOption({
      title: "Add Query Part",
      prompt: "Choose a root query part to add",
      entries,
      selectedValue: entries[0]!.value,
    });
    if (result.kind !== "selected") {
      return;
    }

    switch (result.value) {
      case "category":
        await chooseCategoryFilter();
        return;
      case "subcategory":
        await chooseSubcategoryFilter();
        return;
      case "levelRange":
        await editLevelRange();
        return;
      case "rarity":
        await chooseRarityFilter();
        return;
      case "actionCost":
        await chooseActionCostFilter();
        return;
      case "clause":
        await addQueryClauseAtPath();
        return;
      case "andGroup":
      case "orGroup":
      case "notGroup":
        await addQueryGroupAtPath([], result.value === "andGroup" ? "and" : result.value === "orGroup" ? "or" : "not");
        return;
    }
  }, [
    addQueryClauseAtPath,
    addQueryGroupAtPath,
    chooseCategoryFilter,
    chooseActionCostFilter,
    chooseRarityFilter,
    chooseSubcategoryFilter,
    editLevelRange,
    hasSelectableSubcategories,
    prompts,
    actionCostPolicy,
    category,
    levelRange.levelMax,
    levelRange.levelMin,
    rarityPolicy,
    state.query,
    subcategory,
    terminal,
    user.search,
  ]);

  const editQueryNode = React.useCallback(
    async (path: number[]) => {
      const node = getMetadataNodeAtPath(metadataTree, path);
      if (!node) {
        return;
      }

      if (isMetadataPredicate(node)) {
        const scopedFields = user.search.getQueryFieldOptions(category, subcategory);
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
          if (fieldOption.editor === "ontologyPicker") {
            const currentPolicy = buildPolicyFromPredicate(node) ?? createEmptyStringPolicy();
            await openOntologyFieldEditor(fieldOption, currentPolicy, (nextNode) => {
              applyQueryUpdate((request) =>
                setSearchQueryMetadataTree(request, updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, () => nextNode)),
              );
            });
            return;
          }
          const nextNode = await editFieldClause(fieldOption, node);
          if (nextNode === undefined) {
            return;
          }
          applyQueryUpdate((request) =>
            setSearchQueryMetadataTree(request, updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, () => nextNode)),
          );
          return;
        }

        if (result.value === "wrapNot") {
          applyQueryUpdate((request) =>
            setSearchQueryMetadataTree(request, updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, (current) => ({ not: current }))),
          );
          return;
        }

        applyQueryUpdate((request) =>
          setSearchQueryMetadataTree(request, updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, () => null)),
        );
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

        applyQueryUpdate((request) =>
          setSearchQueryMetadataTree(
            request,
            updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, (current) =>
              result.value === "unwrap" && "not" in current ? current.not : null,
            ),
          ),
        );
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
        applyQueryUpdate((request) =>
          setSearchQueryMetadataTree(
            request,
            updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, (current) =>
              "and" in current ? { or: [...current.and] } : "or" in current ? { and: [...current.or] } : current,
            ),
          ),
        );
        return;
      }
      applyQueryUpdate((request) =>
        setSearchQueryMetadataTree(request, updateMetadataNodeAtPath(getSearchQueryMetadataTree(request), path, () => null)),
      );
    },
    [
      addQueryClauseAtPath,
      addQueryGroupAtPath,
      applyQueryUpdate,
      category,
      editFieldClause,
      metadataTree,
      openOntologyFieldEditor,
      prompts,
      subcategory,
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

      if (action === "execute") {
        void executeRequest(state.query);
        return;
      }
      if (action === "mode") {
        void chooseMode();
        return;
      }
      if (action === "query") {
        void editQueryText();
        return;
      }
      if (action === "profile") {
        void chooseSearchProfile();
        return;
      }
      if (action === "addQueryPart") {
        void openAddQueryPart();
        return;
      }
      if (action === "clearClauses") {
        applyQueryUpdate((request) => setSearchQueryMetadataTree(request, null));
        return;
      }
      if (action === "reset") {
        resetQueryEditor();
        return;
      }
      if (action === "clearResults") {
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
        switch (part) {
          case "category":
            void chooseCategoryFilter();
            return;
          case "subcategory":
            void chooseSubcategoryFilter();
            return;
          case "levelRange":
            void editLevelRange();
            return;
          case "rarity":
            void chooseRarityFilter();
            return;
          case "actionCost":
            void chooseActionCostFilter();
            return;
        }
      }
    },
    [
      applyQueryUpdate,
      chooseActionCostFilter,
      chooseCategoryFilter,
      chooseMode,
      chooseRarityFilter,
      chooseSearchProfile,
      chooseSubcategoryFilter,
      dispatch,
      editQueryNode,
      editLevelRange,
      editQueryText,
      executeRequest,
      openAddQueryPart,
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
    queryFieldBuilderSession,
  };
}
