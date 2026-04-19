import React from "react";

import type { MetadataFilterNode, MetadataPredicate } from "../types.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { SearchTerminalPromptAdapters } from "./interaction-context-adapters.js";
import type { SearchScreenAction, SearchScreenState } from "./search-screen-state.js";
import type { SearchQueryFieldBuilderSession } from "./search-query-field-builder-session.js";
import type { SearchStructuredDraftSession } from "./search-structured-draft-session.js";
import { clampStructuredDraftSelection } from "./search-structured-draft-session.js";
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
  if ("value" in node && node.op === "eq") {
    policy.any = [String(node.value)];
    return policy;
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

type SearchStructuredDraftState = {
  anchor: SearchStructuredDraftSession["anchor"];
  draftQuery: Pf2eTerminalSearchQuery;
  metadataFocusPath: number[] | null;
  selectedIndex: number;
};

type QueryFieldBuilderState = {
  draftQuery: Pf2eTerminalSearchQuery;
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

type StructuredDraftEntryKind = SearchStructuredDraftSession["entries"][number]["kind"];

function countMetadataPredicates(node: MetadataFilterNode | null): number {
  if (!node) {
    return 0;
  }
  if (isMetadataPredicate(node)) {
    return 1;
  }
  return getMetadataNodeChildren(node).reduce((total, child) => total + countMetadataPredicates(child), 0);
}

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
  structuredEditorSession: SearchQueryFieldBuilderSession | null;
} {
  const [structuredDraftState, setStructuredDraftState] = React.useState<SearchStructuredDraftState | null>(null);
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

  const hasSelectableSubcategories = React.useCallback(
    (category: Pf2eTerminalSearchQuery["filters"]["category"]): boolean => {
      if (!category) {
        return false;
      }
      return user.search.getSubcategoryOptions(category).length > 1;
    },
    [user.search],
  );

  const buildStructuredDraftEntries = React.useCallback(
    (draftQuery: Pf2eTerminalSearchQuery, metadataFocusPath: number[] | null): SearchStructuredDraftSession["entries"] => {
      const draftCategory = getSearchQueryCategory(draftQuery);
      const draftSubcategory = getSearchQuerySubcategory(draftQuery);
      const draftRarityPolicy = getSearchQueryRarityPolicy(draftQuery);
      const draftActionCostPolicy = getSearchQueryActionCostPolicy(draftQuery);
      const draftMetadataTree = getSearchQueryMetadataTree(draftQuery);
      const entries: SearchStructuredDraftSession["entries"] = [
        {
          kind: "category",
          key: "category",
          label: "Category",
          value: formatSearchCategory(draftCategory),
          description: "Choose the category boundary for the staged structured query.",
        },
      ];

      if (hasSelectableSubcategories(draftCategory)) {
        entries.push({
          kind: "subcategory",
          key: "subcategory",
          label: "Subcategory",
          value: formatSearchSubcategory(draftSubcategory),
          description: "Choose the staged subcategory boundary within the current category.",
        });
      }

      entries.push(
        {
          kind: "levelRange",
          key: "levelRange",
          label: "Level Range",
          value: formatLevelRange(draftQuery),
          description: "Constrain the staged query by minimum or maximum level.",
        },
        {
          kind: "rarity",
          key: "rarity",
          label: "Rarity",
          value: formatFilterPolicy(draftRarityPolicy),
          description: "Stage include or exclude rarity filters.",
        },
      );

      if (user.search.getActionCostOptions(draftCategory, draftSubcategory).length > 0) {
        entries.push({
          kind: "actionCost",
          key: "actionCost",
          label: "Action Cost",
          value: formatFilterPolicy(draftActionCostPolicy),
          description: "Stage include or exclude action-cost filters.",
        });
      }

      if (draftCategory) {
        const focusedNode = metadataFocusPath ? getMetadataNodeAtPath(draftMetadataTree, metadataFocusPath) : null;
        const predicateCount = countMetadataPredicates(draftMetadataTree);
        entries.push({
          kind: "metadata",
          key: "metadata",
          label: "Query Logic",
          value:
            predicateCount > 0
              ? `${predicateCount} staged clause${predicateCount === 1 ? "" : "s"}`
              : "No staged clauses",
          description: focusedNode
            ? `Resume staged metadata editing at ${metadataFocusPath?.length === 0 ? "the root query node" : `path ${metadataFocusPath?.join(".")}`}.`
            : "Stage metadata clauses and logic groups for the structured query.",
          metadataPath: metadataFocusPath ?? [],
        });
      }

      entries.push(
        {
          kind: "finish",
          key: "finish",
          label: "Finish Structured Edit",
          value: "Apply draft",
          description: "Commit the staged structured query back into the live editor.",
        },
        {
          kind: "cancel",
          key: "cancel",
          label: "Cancel Structured Edit",
          value: "Discard draft",
          description: "Discard the staged structured query and keep the live query unchanged.",
        },
      );

      return entries;
    },
    [hasSelectableSubcategories, user.search],
  );

  const getStructuredDraftAnchorKind = React.useCallback(
    (
      anchor: SearchStructuredDraftSession["anchor"],
      entries: SearchStructuredDraftSession["entries"],
    ): StructuredDraftEntryKind => {
      if (anchor.kind === "queryPart") {
        return anchor.part;
      }
      if (anchor.kind === "queryNode") {
        return "metadata";
      }

      const preferredKinds: StructuredDraftEntryKind[] = [
        "category",
        "subcategory",
        "levelRange",
        "rarity",
        "actionCost",
        "metadata",
      ];
      const emptyKinds = new Set<StructuredDraftEntryKind>();
      for (const entry of entries) {
        if (
          entry.kind === "category" ||
          entry.kind === "subcategory" ||
          entry.kind === "levelRange" ||
          entry.kind === "rarity" ||
          entry.kind === "actionCost" ||
          entry.kind === "metadata"
        ) {
          if (
            entry.value === "Any Category" ||
            entry.value === "Any Subcategory" ||
            entry.value === "(any)" ||
            entry.value === "No staged clauses"
          ) {
            emptyKinds.add(entry.kind);
          }
        }
      }
      return preferredKinds.find((kind) => emptyKinds.has(kind) && entries.some((entry) => entry.kind === kind)) ?? "metadata";
    },
    [],
  );

  const getStructuredDraftSelectionIndex = React.useCallback(
    (
      anchor: SearchStructuredDraftSession["anchor"],
      entries: SearchStructuredDraftSession["entries"],
    ): number => {
      const preferredKind = getStructuredDraftAnchorKind(anchor, entries);
      const entryIndex = entries.findIndex((entry) => entry.kind === preferredKind);
      return clampStructuredDraftSelection(entryIndex >= 0 ? entryIndex : 0, entries.length);
    },
    [getStructuredDraftAnchorKind],
  );

  const openStructuredDraftSession = React.useCallback(
    (anchor: SearchStructuredDraftSession["anchor"], query: Pf2eTerminalSearchQuery = state.query) => {
      const draftQuery = user.search.normalizeQuery(query);
      const metadataFocusPath = anchor.kind === "queryNode" ? [...anchor.path] : null;
      const entries = buildStructuredDraftEntries(draftQuery, metadataFocusPath);
      setStructuredDraftState({
        anchor,
        draftQuery,
        metadataFocusPath,
        selectedIndex: getStructuredDraftSelectionIndex(anchor, entries),
      });
    },
    [buildStructuredDraftEntries, getStructuredDraftSelectionIndex, state.query, user.search],
  );

  const replaceStructuredDraftQuery = React.useCallback(
    (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => {
      setStructuredDraftState((current) => {
        if (!current) {
          return current;
        }
        const nextDraftQuery = user.search.normalizeQuery(update(current.draftQuery));
        const entries = buildStructuredDraftEntries(nextDraftQuery, current.metadataFocusPath);
        return {
          ...current,
          draftQuery: nextDraftQuery,
          selectedIndex: clampStructuredDraftSelection(current.selectedIndex, entries.length),
        };
      });
    },
    [buildStructuredDraftEntries, user.search],
  );

  const chooseQueryField = React.useCallback(
    async (query: Pf2eTerminalSearchQuery): Promise<Pf2eTerminalQueryFieldOption | null> => {
      const queryCategory = getSearchQueryCategory(query);
      const querySubcategory = getSearchQuerySubcategory(query);
      const fieldOptions = user.search.getQueryFieldOptions(queryCategory, querySubcategory);
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
    [prompts, terminal, user.search],
  );

  const openOntologyFieldEditor = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentPolicy: Pf2eTerminalFilterValuePolicy<string>,
      onApply: (nextNode: MetadataFilterNode | null) => void,
      onReturn?: () => void,
    ): Promise<boolean> => {
      if (fieldOption.editor !== "ontologyPicker") {
        return false;
      }

      return openQueryFieldPicker({
        fieldOptions: [fieldOption],
        initialSelections: buildSelectionMap(fieldOption.value, currentPolicy),
        onReturn,
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
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null = null,
    ): Promise<MetadataFilterNode | null | undefined> => {
      const queryCategory = getSearchQueryCategory(query);
      const querySubcategory = getSearchQuerySubcategory(query);

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
            .getFacetValueOptions(fieldOption.value, queryCategory, querySubcategory)
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
    [prompts, terminal, user.search],
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
      const queryCategory = getSearchQueryCategory(query);
      const querySubcategory = getSearchQuerySubcategory(query);
      const fieldOptions = user.search.getQueryFieldOptions(queryCategory, querySubcategory);
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
    [terminal, user.search],
  );

  const finishQueryFieldBuilder = React.useCallback(() => {
    const nextNode = compileQueryFieldBuilderDrafts(queryFieldBuilderState?.fieldDrafts ?? {});
    const targetPath = queryFieldBuilderState?.path ?? [];
    setQueryFieldBuilderState(null);
    if (!nextNode) {
      return;
    }
    replaceStructuredDraftQuery((draftQuery) =>
      setSearchQueryMetadataTree(
        draftQuery,
        appendMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), targetPath, nextNode),
      ),
    );
  }, [queryFieldBuilderState, replaceStructuredDraftQuery]);

  const cancelQueryFieldBuilder = React.useCallback(() => {
    setQueryFieldBuilderState(null);
  }, []);

  const editQueryFieldBuilderField = React.useCallback(
    async (fieldOption: Pf2eTerminalQueryFieldOption) => {
      const scopeQuery = queryFieldBuilderState?.draftQuery;
      if (!scopeQuery) {
        return;
      }

      const currentNode = queryFieldBuilderState.fieldDrafts[fieldOption.value] ?? null;
      if (fieldOption.editor === "ontologyPicker") {
        await openOntologyFieldEditor(scopeQuery, fieldOption, buildPolicyFromMetadataNode(currentNode), (nextNode) => {
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
      finishQueryFieldBuilder();
      return;
    }
    cancelQueryFieldBuilder();
  }, [cancelQueryFieldBuilder, editQueryFieldBuilderField, finishQueryFieldBuilder, queryFieldBuilderState]);

  const queryFieldBuilderSession = React.useMemo<SearchQueryFieldBuilderSession | null>(() => {
    if (!queryFieldBuilderState) {
      return null;
    }
    const stagedNode = compileQueryFieldBuilderDrafts(queryFieldBuilderState.fieldDrafts);
    const previewQuery = stagedNode
      ? setSearchQueryMetadataTree(
          queryFieldBuilderState.draftQuery,
          appendMetadataNodeAtPath(
            getSearchQueryMetadataTree(queryFieldBuilderState.draftQuery),
            queryFieldBuilderState.path,
            stagedNode,
          ),
        )
      : queryFieldBuilderState.draftQuery;
    return {
      kind: "queryFieldBuilder",
      title: "Add Query Part",
      subtitle: "Choose query fields and keep the full staged query visible while you refine it",
      leftTitle: "[QUERY FIELDS]",
      rightTitle: "Staged Summary & Detail",
      statusText: "Field edits stay staged until you finish the structured query editor.",
      draftQuery: previewQuery,
      items: queryFieldBuilderState.items.map((item) =>
        item.kind === "field"
          ? {
              ...item,
              label: queryFieldBuilderState.fieldDrafts[item.fieldOption.value]
                ? `${item.fieldOption.label} | staged`
                : item.fieldOption.label,
            }
          : item,
      ),
      selectedIndex: clampStructuredDraftSelection(queryFieldBuilderState.selectedIndex, queryFieldBuilderState.items.length),
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
      finish: finishQueryFieldBuilder,
      cancel: cancelQueryFieldBuilder,
      helpTitle: "Add Query Part Help",
      helpBody: [
        { text: "Choose the next query field to stage into the structured draft.", tone: "section" },
        {
          text: "The right pane keeps the full staged query summary visible while you move between fields.",
        },
        {
          text: "Concrete selections accumulate in the staged summary immediately, even before you finish the parent draft.",
        },
      ],
    };
  }, [
    cancelQueryFieldBuilder,
    finishQueryFieldBuilder,
    queryFieldBuilderState,
    selectCurrentQueryFieldBuilderItem,
  ]);

  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = []) => {
      await openQueryFieldBuilder(query, path);
    },
    [openQueryFieldBuilder],
  );

  const addQueryGroupAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], groupKind: "and" | "or" | "not") => {
      const fieldOption = await chooseQueryField(query);
      if (!fieldOption) {
        return;
      }
      if (fieldOption.editor === "ontologyPicker") {
        await openOntologyFieldEditor(query, fieldOption, createEmptyStringPolicy(), (nextNode) => {
          if (!nextNode) {
            return;
          }
          const group: MetadataFilterNode =
            groupKind === "and" ? { and: [nextNode] } : groupKind === "or" ? { or: [nextNode] } : { not: nextNode };
          replaceStructuredDraftQuery((draftQuery) =>
            setSearchQueryMetadataTree(
              draftQuery,
              appendMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, group),
            ),
          );
        });
        return;
      }
      const clause = await editFieldClause(query, fieldOption);
      if (!clause) {
        return;
      }
      const group: MetadataFilterNode =
        groupKind === "and" ? { and: [clause] } : groupKind === "or" ? { or: [clause] } : { not: clause };
      replaceStructuredDraftQuery((draftQuery) =>
        setSearchQueryMetadataTree(
          draftQuery,
          appendMetadataNodeAtPath(getSearchQueryMetadataTree(draftQuery), path, group),
        ),
      );
    },
    [chooseQueryField, editFieldClause, openOntologyFieldEditor, replaceStructuredDraftQuery],
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
      structuredDraftState ? buildStructuredDraftEntries(structuredDraftState.draftQuery, structuredDraftState.metadataFocusPath) : [],
    [buildStructuredDraftEntries, structuredDraftState],
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
    const input = await prompts.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue:
        draftLevelRange.levelMin === null && draftLevelRange.levelMax === null
          ? ""
          : formatLevelRange(draftQuery).replaceAll("L", "").replace("<= ", "<="),
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

  const editStructuredDraftRarity = React.useCallback(async () => {
    const draftQuery = structuredDraftState?.draftQuery;
    if (!draftQuery) {
      return;
    }

    const draftCategory = getSearchQueryCategory(draftQuery);
    const draftSubcategory = getSearchQuerySubcategory(draftQuery);
    const draftRarityPolicy = getSearchQueryRarityPolicy(draftQuery);
    const options = user.search.getRarityOptions(draftCategory, draftSubcategory);
    const selected = await prompts.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: draftRarityPolicy,
    });

    replaceStructuredDraftQuery((query) =>
      selected.any.length === 0 && selected.exclude.length === 0
        ? removeSearchQueryPart(query, "rarityPolicy")
        : setSearchQueryPart(query, {
            kind: "rarityPolicy",
            policy: {
              any: selected.any,
              all: [],
              exclude: selected.exclude,
            },
          }),
    );
  }, [prompts, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, user.search]);

  const editStructuredDraftActionCost = React.useCallback(async () => {
    const draftQuery = structuredDraftState?.draftQuery;
    if (!draftQuery) {
      return;
    }

    const draftCategory = getSearchQueryCategory(draftQuery);
    const draftSubcategory = getSearchQuerySubcategory(draftQuery);
    const draftActionCostPolicy = getSearchQueryActionCostPolicy(draftQuery);
    const options = user.search.getActionCostOptions(draftCategory, draftSubcategory);
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
        any: draftActionCostPolicy.any.map(String),
        all: [],
        exclude: draftActionCostPolicy.exclude.map(String),
      },
    });

    const nextPolicy = {
      any: selected.any.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
      all: [] as number[],
      exclude: selected.exclude.map((value) => Number.parseInt(value, 10)).filter((value) => Number.isFinite(value)),
    };

    replaceStructuredDraftQuery((query) =>
      nextPolicy.any.length === 0 && nextPolicy.exclude.length === 0
        ? removeSearchQueryPart(query, "actionCostPolicy")
        : setSearchQueryPart(query, { kind: "actionCostPolicy", policy: nextPolicy }),
    );
  }, [prompts, replaceStructuredDraftQuery, structuredDraftState?.draftQuery, terminal, user.search]);

  const editStructuredDraftMetadata = React.useCallback(
    async (path: number[]) => {
      const draftQuery = structuredDraftState?.draftQuery;
      if (!draftQuery) {
        return;
      }

      const draftMetadataTree = getSearchQueryMetadataTree(draftQuery);
      const node = path.length === 0 ? draftMetadataTree : getMetadataNodeAtPath(draftMetadataTree, path);
      const queryCategory = getSearchQueryCategory(draftQuery);
      const querySubcategory = getSearchQuerySubcategory(draftQuery);

      if (!node) {
        const fieldOptions = user.search.getQueryFieldOptions(queryCategory, querySubcategory);
        if (fieldOptions.length === 0) {
          await terminal.pauseForAnyKey("No scoped metadata fields are available for the current query.");
          return;
        }

        if (fieldOptions.length === 1) {
          const onlyField = fieldOptions[0]!;
          if (onlyField.editor === "ontologyPicker") {
            await openOntologyFieldEditor(draftQuery, onlyField, createEmptyStringPolicy(), (nextNode) => {
              if (!nextNode) {
                return;
              }
              replaceStructuredDraftQuery((query) =>
                setSearchQueryMetadataTree(query, appendMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, nextNode)),
              );
            });
            return;
          }

          const nextNode = await editFieldClause(draftQuery, onlyField);
          if (!nextNode) {
            return;
          }
          replaceStructuredDraftQuery((query) =>
            setSearchQueryMetadataTree(query, appendMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, nextNode)),
          );
          return;
        }

        await openQueryFieldBuilder(draftQuery, path);
        return;
      }

      if (isMetadataPredicate(node)) {
        const scopedFields = user.search.getQueryFieldOptions(queryCategory, querySubcategory);
        const fieldOption = scopedFields.find((candidate) => candidate.value === node.field) ?? null;
        const actionEntries = [
          ...(fieldOption ? [{ value: "edit", label: "Edit Clause", description: "Change the field operator or value." }] : []),
          { value: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
          { value: "remove", label: "Remove Clause", description: "Delete this clause from the draft query." },
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
          if (fieldOption.editor === "ontologyPicker") {
            const currentPolicy = buildPolicyFromPredicate(node) ?? createEmptyStringPolicy();
            await openOntologyFieldEditor(draftQuery, fieldOption, currentPolicy, (nextNode) => {
              replaceStructuredDraftQuery((query) =>
                setSearchQueryMetadataTree(
                  query,
                  updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, () => nextNode),
                ),
              );
            });
            return;
          }
          const nextNode = await editFieldClause(draftQuery, fieldOption, node);
          if (nextNode === undefined) {
            return;
          }
          replaceStructuredDraftQuery((query) =>
            setSearchQueryMetadataTree(
              query,
              updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, () => nextNode),
            ),
          );
          return;
        }

        if (result.value === "wrapNot") {
          replaceStructuredDraftQuery((query) =>
            setSearchQueryMetadataTree(
              query,
              updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, (current) => ({ not: current })),
            ),
          );
          return;
        }

        replaceStructuredDraftQuery((query) =>
          setSearchQueryMetadataTree(query, updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, () => null)),
        );
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

        replaceStructuredDraftQuery((query) =>
          setSearchQueryMetadataTree(
            query,
            updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, (current) =>
              result.value === "unwrap" && "not" in current ? current.not : null,
            ),
          ),
        );
        return;
      }

      const isAndGroup = "and" in node;
      const result = await prompts.promptSelectOption({
        title: isAndGroup ? "AND Group" : "OR Group",
        prompt: "Choose how to update this staged logic group",
        entries: [
          { value: "addClause", label: "Add Clause", description: "Add another clause inside this logic group." },
          { value: "addAndGroup", label: "Add AND Group", description: "Add a nested AND group with one initial clause." },
          { value: "addOrGroup", label: "Add OR Group", description: "Add a nested OR group with one initial clause." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Add a nested NOT group with one initial clause." },
          {
            value: "toggle",
            label: isAndGroup ? "Convert To OR" : "Convert To AND",
            description: isAndGroup
              ? "Require any child instead of every child."
              : "Require every child instead of any child.",
          },
          { value: "remove", label: "Remove Group", description: "Delete this logic group from the draft query." },
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
        replaceStructuredDraftQuery((query) =>
          setSearchQueryMetadataTree(
            query,
            updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, (current) =>
              "and" in current ? { or: [...current.and] } : "or" in current ? { and: [...current.or] } : current,
            ),
          ),
        );
        return;
      }
      replaceStructuredDraftQuery((query) =>
        setSearchQueryMetadataTree(query, updateMetadataNodeAtPath(getSearchQueryMetadataTree(query), path, () => null)),
      );
    },
    [
      addQueryClauseAtPath,
      addQueryGroupAtPath,
      editFieldClause,
      openOntologyFieldEditor,
      openQueryFieldBuilder,
      prompts,
      replaceStructuredDraftQuery,
      structuredDraftState?.draftQuery,
      terminal,
      user.search,
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
      statusText: "Structured changes stay in draft form until you finish.",
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
      cancel: cancelStructuredDraftSession,
      helpTitle: "Structured Query Editor Help",
      helpBody: [
        { text: "Stage structured search changes before applying them to the live query.", tone: "section" },
        { text: "The summary stays visible while you move focus so prior staged selections do not disappear." },
        { text: "Open a row to edit it, stage more changes, then finish when the draft looks correct." },
      ],
    };
  }, [
    cancelStructuredDraftSession,
    finishStructuredDraftSession,
    selectCurrentStructuredDraftEntry,
    structuredDraftEntries,
    structuredDraftState,
  ]);

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
        openStructuredDraftSession({ kind: "addQueryPart" });
        return;
      }
      if (action === "clearClauses") {
        openStructuredDraftSession({ kind: "queryNode", path: [] }, setSearchQueryMetadataTree(state.query, null));
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
        openStructuredDraftSession({ kind: "queryNode", path });
        return;
      }

      if (isQueryPartAction(action)) {
        const part = decodeQueryPartAction(action);
        if (!part) {
          return;
        }
        openStructuredDraftSession({ kind: "queryPart", part });
        return;
      }
    },
    [
      chooseMode,
      chooseSearchProfile,
      dispatch,
      editQueryText,
      executeRequest,
      openStructuredDraftSession,
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
    structuredEditorSession: queryFieldBuilderSession ?? structuredEditorSession,
  };
}
