import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../../domain/categories.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import {
  buildAllOfFilter,
  findSearchScopeFilter,
  type SearchNumericMatch,
  type SearchFilterNode,
  type SearchScopeSubcategoryMatch,
} from "../../../domain/search-request-types.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import {
  appendSearchFilterNodeAtPath,
  appendSearchFilterNodesAtPath,
  canLiftSearchFilterNodeAtPath,
  canUnwrapSearchFilterNodeAtPath,
  getSearchFilterNodeAtPath,
  isSearchFilterBooleanGroup,
  liftSearchFilterNodeAtPath,
  moveSearchFilterNodeToGroupPath,
  reshapeSearchFilterBooleanGroupAtPath,
  toggleSearchFilterRootGroupOperator,
  unwrapSearchFilterNodeAtPath,
  updateSearchFilterNodeAtPath,
  wrapSearchFilterNodeAtPath,
} from "../../search/query-core.js";
import { metadataFilterNodeToCanonicalFilter, canonicalFilterToMetadataNode } from "../../search/query-parts.js";
import { getSearchQueryCategory, getSearchQueryRootOperator, getSearchQuerySubcategory } from "../../search/query-state.js";
import type {
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { promptLevelRangeDraft, promptNumericScalarClause } from "../../filter-explorer/scalar-editor.js";
import type { DerivedTagTerminalActionTargetOption } from "../../action-target.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

type ClauseKind = "field" | "metric" | "metricCompare" | "pack" | "scope" | "level" | "price" | "rarity" | "actionCost";
type MetricFieldFamily = "actorMetric" | "itemMetric";
type MetricCompareOperator = Extract<Extract<SearchFilterNode, { kind: "metricCompare" }>["op"], string>;
type MetricKeySelection = { value: string; discoveryMode: SearchFilterDiscoveryMode };
const CLAUSE_BACK = Symbol("search-structured-draft-clause-back");
const CLAUSE_CONTINUE = Symbol("search-structured-draft-clause-continue");
type ClausePromptBackResult = typeof CLAUSE_BACK;
type ClausePromptContinueResult = {
  kind: typeof CLAUSE_CONTINUE;
  node: SearchFilterNode | SearchFilterNode[];
};
type PromptStepResult<T> = T | ClausePromptBackResult | undefined;
type SearchFilterNodeEditorResult =
  | SearchFilterNode
  | SearchFilterNode[]
  | ClausePromptBackResult
  | ClausePromptContinueResult
  | null
  | undefined;
type ClausePromptResult = ClauseKind | ClausePromptBackResult | null;
type ClauseApplyResult = "applied" | "back" | "cancelled";
type StructuredDraftEntryActionId =
  | "addClause"
  | "addAndGroup"
  | "addOrGroup"
  | "addNotGroup"
  | "moveHere"
  | "toggleRoot"
  | "edit"
  | "wrapNot"
  | "wrapAnd"
  | "wrapOr"
  | "move"
  | "lift"
  | "remove"
  | "unwrap"
  | "toggleGroup";

function isClausePromptContinueResult(result: SearchFilterNodeEditorResult): result is ClausePromptContinueResult {
  return Boolean(result && typeof result === "object" && !Array.isArray(result) && "kind" in result && result.kind === CLAUSE_CONTINUE);
}

function formatFriendlyGroupLabel(kind: "allOf" | "anyOf" | "not"): string {
  switch (kind) {
    case "allOf":
      return "All of";
    case "anyOf":
      return "Any of";
    case "not":
      return "Exclude";
  }
}

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

function getFirstGroupedFieldMemberPath(
  node: SearchFilterNode,
  path: number[],
  field: string,
): number[] | null {
  if (field === "rarity" || field === "actionCost") {
    if (node.kind === field && node.match.kind === "eq") {
      return path;
    }
    if (node.kind === "anyOf") {
      for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
        const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
        if (childPath) {
          return childPath;
        }
      }
      return null;
    }
    if (node.kind === "not") {
      return getFirstGroupedFieldMemberPath(node.child, [...path, 0], field);
    }
    return null;
  }

  if (node.kind === "metadataPredicate" && node.predicate.field === field) {
    return path;
  }
  if (node.kind === "not") {
    return getFirstGroupedFieldMemberPath(node.child, [...path, 0], field);
  }
  if (node.kind === "anyOf") {
    for (let childIndex = 0; childIndex < node.children.length; childIndex += 1) {
      const childPath = getFirstGroupedFieldMemberPath(node.children[childIndex]!, [...path, childIndex], field);
      if (childPath) {
        return childPath;
      }
    }
  }
  return null;
}

function toSearchFilterNodeEditorResult(
  result: Pf2eTerminalFilterExplorerInsertionResult,
): SearchFilterNode | SearchFilterNode[] | null {
  if (result.kind === "insert") {
    return result.nodes
      .map((node) => metadataFilterNodeToCanonicalFilter(node))
      .filter((node): node is SearchFilterNode => Boolean(node));
  }

  return metadataFilterNodeToCanonicalFilter(result.node) ?? null;
}

export function buildGroupedFieldSeedState(
  query: Pf2eTerminalSearchQuery,
  groupPath: number[],
): {
  seedGroupPath: number[];
  seedQuery: Pf2eTerminalSearchQuery;
} {
  const groupNode = groupPath.length === 0 ? query.filter : getSearchFilterNodeAtPath(query.filter, groupPath) ?? undefined;
  if (!groupNode) {
    return {
      seedGroupPath: [],
      seedQuery: query,
    };
  }

  const scopeNode = findSearchScopeFilter(query.filter);
  if (groupPath.length === 0 || !scopeNode) {
    return {
      seedGroupPath: [],
      seedQuery: {
        ...query,
        filter: groupNode,
      },
    };
  }

  return {
    seedGroupPath: [1],
    seedQuery: {
      ...query,
      filter: buildAllOfFilter([scopeNode, groupNode]),
    },
  };
}

export function applyGroupedFieldSeedQueryToQuery(
  query: Pf2eTerminalSearchQuery,
  groupPath: number[],
  seedGroupPath: number[],
  field: string,
  nextSeedQuery: Pf2eTerminalSearchQuery,
): { nextQuery: Pf2eTerminalSearchQuery; nextFocusPath: number[] | null } {
  const nextGroupNode =
    seedGroupPath.length === 0
      ? nextSeedQuery.filter
      : getSearchFilterNodeAtPath(nextSeedQuery.filter, seedGroupPath) ?? undefined;
  const nextFilter =
    groupPath.length === 0
      ? nextGroupNode
      : updateSearchFilterNodeAtPath(query.filter, groupPath, () => nextGroupNode);
  const nextGroupNodeInQuery =
    groupPath.length === 0 ? nextGroupNode ?? null : getSearchFilterNodeAtPath(nextFilter, groupPath);
  const nextFocusPath = nextGroupNodeInQuery
    ? getFirstGroupedFieldMemberPath(nextGroupNodeInQuery, groupPath, field) ?? (groupPath.length > 0 ? groupPath : null)
    : groupPath.length > 0
      ? groupPath
      : null;

  return {
    nextQuery: {
      ...query,
      filter: nextFilter,
    },
    nextFocusPath,
  };
}

function buildInsertionActionEntries(
  moveMode: boolean,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  if (moveMode) {
    return [
      {
        id: "moveHere",
        label: "Move Here",
        description: "Append the anchored node into this visible insertion slot.",
      },
    ];
  }

  return [
    { id: "addClause", label: "Add Clause", description: "Insert one canonical filter clause into this group." },
    {
      id: "addAndGroup",
      label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
      description: "Insert a nested group where every child must match, starting with its first child.",
    },
    {
      id: "addOrGroup",
      label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
      description: "Insert a nested group where any child may match, starting with its first child.",
    },
    {
      id: "addNotGroup",
      label: `Add ${formatFriendlyGroupLabel("not")} Group`,
      description: "Insert a nested excluded group with its first child.",
    },
  ];
}

function buildRootActionEntries(
  query: Pf2eTerminalSearchQuery,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  return [
    { id: "addClause", label: "Add Clause", description: "Append a new top-level clause." },
    {
      id: "addAndGroup",
      label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
      description: "Append a nested group where every child must match.",
    },
    {
      id: "addOrGroup",
      label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
      description: "Append a nested group where any child may match.",
    },
    {
      id: "addNotGroup",
      label: `Add ${formatFriendlyGroupLabel("not")} Group`,
      description: "Append a nested excluded group.",
    },
    ...(query.filter
      ? [
          {
            id: "toggleRoot" as const,
            label:
              getSearchQueryRootOperator(query) === "anyOf"
                ? `Change Root To ${formatFriendlyGroupLabel("allOf")}`
                : `Change Root To ${formatFriendlyGroupLabel("anyOf")}`,
            description: "Reshape the visible root group without changing its current children.",
          },
        ]
      : []),
  ];
}

function isMetricFieldOptionValue(value: Pf2eTerminalQueryFieldOption["value"]): boolean {
  return value === "actorMetric" || value === "itemMetric";
}

function getQueryFieldValueForNode(node: SearchFilterNode): Pf2eTerminalQueryFieldOption["value"] | null {
  switch (node.kind) {
    case "metadataPredicate":
      return node.predicate.field;
    case "metric":
      return inferMetricFieldFamily(node.metric);
    default:
      return null;
  }
}

function inferMetricFieldFamily(metric: string, category: ReturnType<typeof getSearchQueryCategory> = null): MetricFieldFamily {
  const actorValueType = inferActorMetricValueType(metric);
  const itemValueType = inferItemMetricValueType(metric);

  if (actorValueType && !itemValueType) {
    return "actorMetric";
  }
  if (itemValueType && !actorValueType) {
    return "itemMetric";
  }

  return category === "equipment" ? "itemMetric" : "actorMetric";
}

function formatNumericMatch(match: SearchNumericMatch): string {
  switch (match.kind) {
    case "eq":
      return String(match.value);
    case "gte":
      return `>=${match.value}`;
    case "lte":
      return `<=${match.value}`;
    case "between":
      return `${match.min}-${match.max}`;
  }
}

function buildDiscoveryModeSubtitle(discoveryMode: SearchFilterDiscoveryMode): string {
  return discoveryMode === "matching" ? "Matching counts" : "Catalog counts";
}

export function useSearchStructuredDraftMetadataActions({
  appendStructuredDraftMetadataNode,
  clearStructuredDraftMoveSource,
  editFieldClause,
  enterStructuredDraftMoveMode,
  getScopedFieldOptions,
  moveSourcePath,
  openOntologyFieldEditor,
  prompts,
  replaceStructuredDraftProjection,
  setStructuredDraftMetadataFocusPath,
  structuredDraftQuery,
  terminal,
  updateStructuredDraftMetadataNode,
  user,
}: {
  appendStructuredDraftMetadataNode: (path: number[], nextNode: MetadataFilterNode) => void;
  clearStructuredDraftMoveSource: () => void;
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  enterStructuredDraftMoveMode: (path: number[]) => void;
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
    },
  ) => Promise<boolean>;
  moveSourcePath: number[] | null;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (
    update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery,
    options?: { metadataFocusPath?: number[] | null },
  ) => void;
  setStructuredDraftMetadataFocusPath: (path: number[] | null) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
    options?: { metadataFocusPath?: number[] | null },
  ) => void;
  user: SearchWorkspaceUser;
}): {
  editStructuredDraftMetadata: (entry: SearchStructuredDraftEntry) => Promise<void>;
  getStructuredDraftEntryActions: (
    entry: SearchStructuredDraftEntry | null | undefined,
  ) => DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[];
  runStructuredDraftEntryAction: (
    entry: SearchStructuredDraftEntry | null | undefined,
    actionId: StructuredDraftEntryActionId,
  ) => Promise<void>;
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

  const openLiveExplorerFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      fieldOption: Pf2eTerminalQueryFieldOption,
      currentNode: MetadataFilterNode | null,
    ) => {
      const applyResult = (result: Pf2eTerminalFilterExplorerInsertionResult) => {
        if (result.kind !== "replace") {
          return;
        }

        updateStructuredDraftMetadataNode(path, () => result.node, {
          metadataFocusPath: result.node ? path : path.length > 0 ? path.slice(0, -1) : null,
        });
      };
      setStructuredDraftMetadataFocusPath(path);
      await openOntologyFieldEditor(
        query,
        fieldOption,
        currentNode,
        (result) => {
          applyResult(result);
        },
        undefined,
        (result) => {
          applyResult(result);
        },
        {
          onBack: () => {},
          onExitRoot: () => {},
        },
      );
    },
    [openOntologyFieldEditor, setStructuredDraftMetadataFocusPath, updateStructuredDraftMetadataNode],
  );

  const openLiveExplorerGroupedField = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      entry: SearchStructuredDraftEntry,
    ) => {
      const groupPath = entry.groupPath ?? [];
      const fieldMemberPaths = entry.fieldMemberPaths ?? entry.memberPaths ?? [];
      const field = entry.field;
      if (!field || fieldMemberPaths.length === 0) {
        return;
      }

      const fieldOption =
        field === "rarity"
          ? buildExplorerOnlyFieldOption(
              "rarity",
              "Rarity",
              "Browse live rarities for the current group and stage canonical rarity clauses.",
              "enumString",
            )
          : field === "actionCost"
            ? buildExplorerOnlyFieldOption(
                "actionCost",
                "Action Cost",
                "Browse live action costs for the current group and stage canonical action-cost clauses.",
                "number",
              )
            : getScopedFieldOptions(query).find((candidate) => candidate.value === field) ?? null;
      if (!fieldOption || fieldOption.editor !== "sharedExplorer") {
        await terminal.pauseForAnyKey("That grouped row cannot be edited through the shared explorer.");
        return;
      }

      const { seedGroupPath, seedQuery } = buildGroupedFieldSeedState(query, groupPath);
      await openOntologyFieldEditor(
        seedQuery,
        fieldOption,
        null,
        (_result, nextGroupQuery) => {
          const { nextQuery, nextFocusPath } = applyGroupedFieldSeedQueryToQuery(
            query,
            groupPath,
            seedGroupPath,
            field,
            nextGroupQuery,
          );
          replaceStructuredDraftProjection(() => nextQuery, { metadataFocusPath: nextFocusPath });
        },
        undefined,
        (_result, nextGroupQuery) => {
          const { nextQuery, nextFocusPath } = applyGroupedFieldSeedQueryToQuery(
            query,
            groupPath,
            seedGroupPath,
            field,
            nextGroupQuery,
          );
          replaceStructuredDraftProjection(() => nextQuery, { metadataFocusPath: nextFocusPath });
        },
        {
          onBack: () => {},
          onExitRoot: () => {},
        },
      );
    },
    [
      getScopedFieldOptions,
      openOntologyFieldEditor,
      replaceStructuredDraftProjection,
      terminal,
    ],
  );

  const openLiveExplorerQueryField = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      fieldOption: Pf2eTerminalQueryFieldOption,
    ) => {
      await openOntologyFieldEditor(
        query,
        fieldOption,
        null,
        (_result, nextQuery) => {
          replaceStructuredDraftProjection(() => nextQuery);
        },
        undefined,
        (_result, nextQuery) => {
          replaceStructuredDraftProjection(() => nextQuery);
        },
        {
          onBack: () => {},
          onExitRoot: () => {},
        },
      );
    },
    [openOntologyFieldEditor, replaceStructuredDraftProjection],
  );

  const promptForPickerDiscoveryMode = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      title: string,
      discoveryMode: SearchFilterDiscoveryMode,
    ): Promise<SearchFilterDiscoveryMode | undefined> => {
      const selected = await promptSession.promptSelectOption({
        title: `${title} Count Source`,
        prompt: "Choose how this picker should source counts",
        entries: [
          {
            value: "matching",
            label: "Use Matching Counts",
            description: "Show values and counts from the active query context.",
          },
          {
            value: "catalog",
            label: "Use Catalog Counts",
            description: "Show values and counts from the broader applicability slice.",
          },
        ],
        selectedValue: discoveryMode,
      });
      if (selected.kind === "back") {
        return discoveryMode;
      }
      if (selected.kind !== "selected") {
        return undefined;
      }
      return selected.value as SearchFilterDiscoveryMode;
    },
    [prompts],
  );

  const promptForFieldClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: "field" | "metric",
      currentNode: MetadataFilterNode | null = null,
    ): Promise<SearchFilterNodeEditorResult> => {
      const fieldOptions = getScopedFieldOptions(query).filter((fieldOption) =>
        family === "metric" ? isMetricFieldOptionValue(fieldOption.value) : !isMetricFieldOptionValue(fieldOption.value),
      );
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey(
          family === "metric"
            ? "No scoped metric filters are available for the current query."
            : "No scoped field filters are available for the current query.",
        );
        return undefined;
      }

      let preferredFieldValue =
        currentNode &&
        !("and" in currentNode) &&
        !("or" in currentNode) &&
        !("not" in currentNode) &&
        "field" in currentNode
          ? currentNode.field
          : fieldOptions[0]!.value;
      let pendingExplorerResult: SearchFilterNode | SearchFilterNode[] | null | undefined;

      while (true) {
        const selection = await promptSession.promptSelectOption({
          title: family === "metric" ? "Metric" : "Metadata",
          prompt: family === "metric" ? "Choose the metric family for the next clause" : "Choose the metadata field for the next clause",
          entries: fieldOptions.map((fieldOption) => ({
            value: fieldOption.value,
            label: fieldOption.label,
            description: fieldOption.description,
          })),
          selectedValue:
            preferredFieldValue && fieldOptions.some((fieldOption) => fieldOption.value === preferredFieldValue)
              ? preferredFieldValue
              : fieldOptions[0]!.value,
        });
        if (selection.kind === "back") {
          if (pendingExplorerResult) {
            return {
              kind: CLAUSE_CONTINUE,
              node: pendingExplorerResult,
            };
          }
          return CLAUSE_BACK;
        }
        if (selection.kind !== "selected") {
          return undefined;
        }

        preferredFieldValue = selection.value;
        const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
        if (!fieldOption) {
          return undefined;
        }

        if (fieldOption.editor === "sharedExplorer") {
          const explorerResult = await new Promise<SearchFilterNodeEditorResult | typeof CLAUSE_BACK | undefined>((resolve) => {
            void openOntologyFieldEditor(
              query,
              fieldOption,
              currentNode,
              (result) => {
                resolve(toSearchFilterNodeEditorResult(result));
              },
              () => resolve(CLAUSE_BACK),
              (result) => {
                pendingExplorerResult = toSearchFilterNodeEditorResult(result);
              },
              {
                onCancel: () => resolve(undefined),
              },
            );
          });

          if (explorerResult === CLAUSE_BACK) {
            continue;
          }
          return explorerResult;
        }

        const nextNode = await editFieldClause(query, fieldOption, currentNode);
        return nextNode === undefined ? undefined : nextNode ? metadataFilterNodeToCanonicalFilter(nextNode) ?? null : null;
      }
    },
    [editFieldClause, getScopedFieldOptions, openOntologyFieldEditor, prompts, terminal],
  );

  const getAvailableMetricFamilies = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, options: { numericOnly?: boolean } = {}): Promise<MetricFieldFamily[]> => {
      const [actorMetricOptions, itemMetricOptions] = await Promise.all([
        user.search.loadMetricKeyOptions(query, "actorMetric", "matching", options),
        user.search.loadMetricKeyOptions(query, "itemMetric", "matching", options),
      ]);
      const families: MetricFieldFamily[] = [];
      if (actorMetricOptions.length > 0) {
        families.push("actorMetric");
      }
      if (itemMetricOptions.length > 0) {
        families.push("itemMetric");
      }
      return families;
    },
    [user.search],
  );

  const promptForMetricFamily = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      availableFamilies: MetricFieldFamily[],
      currentFamily: MetricFieldFamily | null,
      title: string,
      prompt: string,
    ): Promise<PromptStepResult<MetricFieldFamily>> => {
      const metricFieldOptions = getScopedFieldOptions(query).filter((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const entries = availableFamilies
        .map((family) => metricFieldOptions.find((fieldOption) => fieldOption.value === family))
        .filter((fieldOption): fieldOption is Pf2eTerminalQueryFieldOption => Boolean(fieldOption))
        .map((fieldOption) => ({
          value: fieldOption.value as MetricFieldFamily,
          label: fieldOption.label,
          description: fieldOption.description,
        }));
      if (entries.length === 0) {
        return undefined;
      }

      const selection = await promptSession.promptSelectOption({
        title,
        prompt,
        entries,
        selectedValue:
          currentFamily && entries.some((entry) => entry.value === currentFamily) ? currentFamily : entries[0]!.value,
      });
      if (selection.kind === "back") {
        return CLAUSE_BACK;
      }
      return selection.kind === "selected" ? selection.value : undefined;
    },
    [getScopedFieldOptions, prompts],
  );

  const promptForMetricKey = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: MetricFieldFamily,
      title: string,
      prompt: string,
      currentMetric?: string,
      options: { numericOnly?: boolean } = {},
      initialDiscoveryMode: SearchFilterDiscoveryMode = "matching",
    ): Promise<PromptStepResult<MetricKeySelection>> => {
      let discoveryMode: SearchFilterDiscoveryMode = initialDiscoveryMode;

      while (true) {
        const metricOptions = await user.search.loadMetricKeyOptions(query, family, discoveryMode, options);
        if (metricOptions.length === 0) {
          return undefined;
        }

        const selection = await promptSession.promptSelectOption({
          title,
          subtitle: buildDiscoveryModeSubtitle(discoveryMode),
          prompt,
          entries: metricOptions.map((metricOption) => ({
            value: metricOption.value,
            label: metricOption.label,
            description: metricOption.description,
          })),
          selectedValue:
            currentMetric && metricOptions.some((metricOption) => metricOption.value === currentMetric)
              ? currentMetric
              : metricOptions[0]!.value,
          supportsCommands: true,
        });
        if (selection.kind === "selected") {
          return {
            value: selection.value,
            discoveryMode,
          };
        }
        if (selection.kind === "back") {
          return CLAUSE_BACK;
        }
        if (selection.kind !== "commands") {
          return undefined;
        }

        const nextMode = await promptForPickerDiscoveryMode(promptSession, title, discoveryMode);
        if (!nextMode) {
          return undefined;
        }
        discoveryMode = nextMode;
      }
    },
    [promptForPickerDiscoveryMode, prompts, user.search],
  );

  const promptForMetricCompareClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "metricCompare" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "metricCompare" }> | ClausePromptBackResult | undefined> => {
      const availableFamilies = await getAvailableMetricFamilies(query, { numericOnly: true });
      if (availableFamilies.length === 0) {
        await terminal.pauseForAnyKey("No numeric metric comparisons are available for the current query.");
        return undefined;
      }

      const currentFamily: MetricFieldFamily | null = currentNode
        ? inferMetricFieldFamily(currentNode.leftMetric, getSearchQueryCategory(query))
        : null;
      let selectedFamily = currentFamily;
      let metricDiscoveryMode: SearchFilterDiscoveryMode = "matching";

      while (true) {
        const family = await promptForMetricFamily(
          promptSession,
          query,
          availableFamilies,
          selectedFamily,
          "Metric comparison",
          "Choose the metric family for this comparison clause",
        );
        if (family === CLAUSE_BACK) {
          return CLAUSE_BACK;
        }
        if (!family) {
          return undefined;
        }
        selectedFamily = family;

        while (true) {
          const leftMetric = await promptForMetricKey(
            promptSession,
            query,
            selectedFamily,
            "Left Metric",
            "Choose the left-hand metric for this comparison clause",
            currentNode && currentFamily === selectedFamily ? currentNode.leftMetric : undefined,
            { numericOnly: true },
            metricDiscoveryMode,
          );
          if (leftMetric === CLAUSE_BACK) {
            break;
          }
          if (!leftMetric) {
            return undefined;
          }
          metricDiscoveryMode = leftMetric.discoveryMode;

          while (true) {
            const operatorSelection = await promptSession.promptSelectOption({
              title: "Comparison Operator",
              prompt: "Choose how the left metric should compare to the right metric",
              entries: [
                { value: "eq", label: "Equals", description: "Require both metrics to be equal." },
                { value: "notEq", label: "Does Not Equal", description: "Require both metrics to differ." },
                { value: "gt", label: "Greater Than", description: "Require the left metric to be greater." },
                { value: "gte", label: "Greater Or Equal", description: "Require the left metric to be greater or equal." },
                { value: "lt", label: "Less Than", description: "Require the left metric to be less." },
                { value: "lte", label: "Less Or Equal", description: "Require the left metric to be less or equal." },
              ],
              selectedValue: currentNode?.op ?? "gte",
            });
            if (operatorSelection.kind === "back") {
              break;
            }
            if (operatorSelection.kind !== "selected") {
              return undefined;
            }

            const operator = operatorSelection.value as MetricCompareOperator;
            const rightMetric = await promptForMetricKey(
              promptSession,
              query,
              selectedFamily,
              "Right Metric",
              "Choose the right-hand metric for this comparison clause",
              currentNode && currentFamily === selectedFamily ? currentNode.rightMetric : leftMetric.value,
              { numericOnly: true },
              metricDiscoveryMode,
            );
            if (rightMetric === CLAUSE_BACK) {
              continue;
            }
            if (!rightMetric) {
              return undefined;
            }
            metricDiscoveryMode = rightMetric.discoveryMode;

            return {
              kind: "metricCompare",
              leftMetric: leftMetric.value,
              op: operator,
              rightMetric: rightMetric.value,
            };
          }
        }
      }
    },
    [getAvailableMetricFamilies, promptForMetricFamily, promptForMetricKey, prompts, terminal],
  );

  const promptForPackClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "pack" }>,
    ): Promise<
      Extract<SearchFilterNode, { kind: "pack" }> | Extract<SearchFilterNode, { kind: "pack" }>[] | ClausePromptBackResult | undefined
    > => {
      let discoveryMode: SearchFilterDiscoveryMode = "matching";

      while (true) {
        const packOptions = await user.search.loadPackOptions(query, discoveryMode);
        if (packOptions.length === 0) {
          await terminal.pauseForAnyKey("No packs are available for the current query.");
          return undefined;
        }

        if (currentNode) {
          const selection = await promptSession.promptSelectOption({
            title: "Pack",
            subtitle: buildDiscoveryModeSubtitle(discoveryMode),
            prompt: "Choose the pack for this clause",
            entries: packOptions.map((packOption) => ({
              value: packOption.value,
              label: packOption.label,
              description: packOption.description,
            })),
            selectedValue:
              currentNode.value && packOptions.some((packOption) => packOption.value === currentNode.value)
                ? currentNode.value
                : packOptions[0]!.value,
            supportsCommands: true,
          });
          if (selection.kind === "selected") {
            return {
              kind: "pack",
              value: selection.value,
            };
          }
          if (selection.kind === "back") {
            return CLAUSE_BACK;
          }
          if (selection.kind !== "commands") {
            return undefined;
          }
        } else {
          const selection = await promptSession.promptMultiSelectOption({
            title: "Pack",
            subtitle: buildDiscoveryModeSubtitle(discoveryMode),
            prompt: "Choose one or more packs for this clause",
            entries: packOptions.map((packOption) => ({
              value: packOption.value,
              label: packOption.label,
              description: packOption.description,
            })),
            supportsCommands: true,
          });
          if (selection.kind === "selected") {
            return selection.values.map((value) => ({
              kind: "pack" as const,
              value,
            }));
          }
          if (selection.kind === "back") {
            return CLAUSE_BACK;
          }
          if (selection.kind !== "commands") {
            return undefined;
          }
        }

        const nextMode = await promptForPickerDiscoveryMode(promptSession, "Pack", discoveryMode);
        if (!nextMode) {
          return undefined;
        }
        discoveryMode = nextMode;
      }
    },
    [promptForPickerDiscoveryMode, prompts, terminal, user.search],
  );

  const promptForScopeClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "scope" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "scope" }> | ClausePromptBackResult | undefined> => {
      const [, ...categoryOptions] = user.search.getCategoryOptions();
      if (categoryOptions.length === 0) {
        await terminal.pauseForAnyKey("No categories are available for the current query.");
        return undefined;
      }

      while (true) {
        const categorySelection = await promptSession.promptSelectOption({
          title: "Scope",
          prompt: "Choose the category for this scope clause",
          entries: categoryOptions.map((option) => ({
            value: option.value,
            label: option.label,
            description: option.description,
          })),
          selectedValue: currentNode?.category ?? categoryOptions[0]!.value,
        });
        if (categorySelection.kind === "back") {
          return CLAUSE_BACK;
        }
        if (categorySelection.kind !== "selected") {
          return undefined;
        }

        const category = normalizeSearchCategory(categorySelection.value) ?? null;
        if (!category) {
          return undefined;
        }

        const subcategoryOptions = user.search.getSubcategoryOptions(category).filter((option) => option.value !== null);
        const currentMode =
          currentNode?.category === category && currentNode?.subcategory.kind === "eq"
            ? "specific"
            : currentNode?.category === category && currentNode?.subcategory.kind === "isNull"
              ? "none"
              : "any";

        while (true) {
          const modeSelection = await promptSession.promptSelectOption({
            title: "Subcategory Mode",
            prompt: "Choose how this scope clause should treat subcategories",
            entries: [
              {
                value: "any",
                label: "Any subcategory",
                description: "Match any subcategory inside the selected category.",
              },
              {
                value: "specific",
                label: "Specific subcategory",
                description: "Choose one exact subcategory inside the selected category.",
              },
              { value: "none", label: "No subcategory", description: "Match only records without a subcategory." },
            ],
            selectedValue: currentMode,
          });
          if (modeSelection.kind === "back") {
            break;
          }
          if (modeSelection.kind !== "selected") {
            return undefined;
          }

          let subcategory: SearchScopeSubcategoryMatch = { kind: "any" };
          if (modeSelection.value === "none") {
            subcategory = { kind: "isNull" };
          } else if (modeSelection.value === "specific") {
            if (subcategoryOptions.length === 0) {
              await terminal.pauseForAnyKey("No subcategories are available for the selected category.");
              return undefined;
            }
            const currentSubcategoryValue =
              currentNode?.category === category && currentNode?.subcategory.kind === "eq"
                ? currentNode.subcategory.value
                : null;
            const subcategorySelection = await promptSession.promptSelectOption({
              title: "Specific Subcategory",
              prompt: "Choose the exact subcategory for this scope clause",
              entries: subcategoryOptions.map((option) => ({
                value: option.value,
                label: option.label,
                description: option.description,
              })),
              selectedValue:
                currentSubcategoryValue && subcategoryOptions.some((option) => option.value === currentSubcategoryValue)
                  ? currentSubcategoryValue
                  : subcategoryOptions[0]!.value,
            });
            if (subcategorySelection.kind === "back") {
              continue;
            }
            if (subcategorySelection.kind !== "selected") {
              return undefined;
            }
            const normalizedSubcategory = normalizeSearchSubcategory(subcategorySelection.value) ?? null;
            if (!normalizedSubcategory) {
              return undefined;
            }
            subcategory = { kind: "eq", value: normalizedSubcategory };
          }

          return {
            kind: "scope",
            category,
            subcategory,
          };
        }
      }
    },
    [prompts, terminal, user.search],
  );

  const promptForNumericMatchClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      nodeKind: "level" | "price" | "actionCost",
      node?:
        | Extract<SearchFilterNode, { kind: "level" }>
        | Extract<SearchFilterNode, { kind: "price" }>
        | Extract<SearchFilterNode, { kind: "actionCost" }>,
    ):
      Promise<
        | Extract<SearchFilterNode, { kind: "level" }>
        | Extract<SearchFilterNode, { kind: "price" }>
        | Extract<SearchFilterNode, { kind: "actionCost" }>
        | undefined
      > => {
      if (node?.kind === "actionCost" && (node.match.kind === "isNull" || node.match.kind === "isNotNull")) {
        await terminal.pauseForAnyKey("Null action-cost clauses cannot be edited through the numeric matcher.");
        return undefined;
      }
      let currentNumericMatch: SearchNumericMatch | null = null;
      if (node?.kind === "actionCost") {
        switch (node.match.kind) {
          case "eq":
          case "gte":
          case "lte":
          case "between":
            currentNumericMatch = node.match;
            break;
          case "isNull":
          case "isNotNull":
            currentNumericMatch = null;
            break;
        }
      } else if (node) {
        currentNumericMatch = node.match;
      }
      if (nodeKind === "level") {
        const parsed = await promptLevelRangeDraft(promptSession, terminal, {
          defaultValue: currentNumericMatch ? formatNumericMatch(currentNumericMatch) : "",
        });
        if (parsed === undefined) {
          return undefined;
        }
        if (parsed.levelMin === null && parsed.levelMax === null) {
          return undefined;
        }
        if (parsed.levelMin !== null && parsed.levelMax !== null) {
          return {
            kind: nodeKind,
            match:
              parsed.levelMin === parsed.levelMax
                ? { kind: "eq", value: parsed.levelMin }
                : {
                    kind: "between",
                    min: Math.min(parsed.levelMin, parsed.levelMax),
                    max: Math.max(parsed.levelMin, parsed.levelMax),
                  },
          };
        }
        return {
          kind: nodeKind,
          match: parsed.levelMin !== null ? { kind: "gte", value: parsed.levelMin } : { kind: "lte", value: parsed.levelMax! },
        };
      }

      const parsed = await promptNumericScalarClause(promptSession, terminal, {
        title: nodeKind === "price" ? "Price Matcher" : "Action Cost Matcher",
        currentClause:
          currentNumericMatch?.kind === "between"
            ? { op: "between", min: currentNumericMatch.min, max: currentNumericMatch.max }
            : currentNumericMatch?.kind === "eq" || currentNumericMatch?.kind === "gte" || currentNumericMatch?.kind === "lte"
              ? { op: currentNumericMatch.kind, value: currentNumericMatch.value }
              : null,
      });
      if (!parsed) {
        return undefined;
      }
      if (parsed.op === "neq") {
        await terminal.pauseForAnyKey("`!=` is not supported for this matcher. Use an exact, minimum, maximum, or range value.");
        return undefined;
      }

      return {
        kind: nodeKind,
        match:
          parsed.op === "between"
            ? {
                kind: "between",
                min: Math.min(parsed.min, parsed.max),
                max: Math.max(parsed.min, parsed.max),
              }
            : {
                kind: parsed.op,
                value: parsed.value,
              },
      };
    },
    [terminal],
  );

  const promptForRarityClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      node?: Extract<SearchFilterNode, { kind: "rarity" }>,
    ): Promise<SearchFilterNodeEditorResult> => {
      return new Promise<SearchFilterNodeEditorResult>((resolve) => {
        void openOntologyFieldEditor(
          query,
          buildExplorerOnlyFieldOption(
            "rarity",
            "Rarity",
            "Browse live rarities for the current scope and stage canonical rarity clauses.",
            "enumString",
          ),
          node ? canonicalFilterToMetadataNode(node) : null,
          (result) => {
            if (result.kind === "insert") {
              resolve(
                result.nodes
                  .map((metadataNode) => metadataFilterNodeToCanonicalFilter(metadataNode))
                  .filter((candidate): candidate is SearchFilterNode => Boolean(candidate)),
              );
              return;
            }
            resolve(metadataFilterNodeToCanonicalFilter(result.node) ?? null);
          },
          () => resolve(CLAUSE_BACK),
          undefined,
          {
            onCancel: () => resolve(undefined),
          },
        );
      });
    },
    [openOntologyFieldEditor],
  );

  const promptForClauseNode = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      clauseKind: ClauseKind,
      currentNode?: SearchFilterNode,
    ): Promise<SearchFilterNodeEditorResult> => {
      switch (clauseKind) {
        case "field":
          return promptForFieldClause(promptSession, query, "field", currentNode ? canonicalFilterToMetadataNode(currentNode) : null);
        case "metric":
          return promptForFieldClause(promptSession, query, "metric", currentNode ? canonicalFilterToMetadataNode(currentNode) : null);
        case "metricCompare":
          return promptForMetricCompareClause(promptSession, query, currentNode?.kind === "metricCompare" ? currentNode : undefined);
        case "pack":
          return promptForPackClause(promptSession, query, currentNode?.kind === "pack" ? currentNode : undefined);
        case "scope":
          return promptForScopeClause(promptSession, query, currentNode?.kind === "scope" ? currentNode : undefined);
        case "level":
          return promptForNumericMatchClause(promptSession, "level", currentNode?.kind === "level" ? currentNode : undefined);
        case "price":
          return promptForNumericMatchClause(promptSession, "price", currentNode?.kind === "price" ? currentNode : undefined);
        case "rarity":
          return promptForRarityClause(query, currentNode?.kind === "rarity" ? currentNode : undefined);
        case "actionCost":
          return promptForNumericMatchClause(promptSession, "actionCost", currentNode?.kind === "actionCost" ? currentNode : undefined);
      }
    },
    [
      promptForFieldClause,
      promptForMetricCompareClause,
      promptForNumericMatchClause,
      promptForPackClause,
      promptForRarityClause,
      promptForScopeClause,
    ],
  );

  const promptForClauseKind = React.useCallback(
    async (promptSession: SearchWorkspacePromptAdapters, query: Pf2eTerminalSearchQuery): Promise<ClausePromptResult> => {
      const fieldOptions = getScopedFieldOptions(query);
      const hasFieldClauses = fieldOptions.some((fieldOption) => !isMetricFieldOptionValue(fieldOption.value));
      const hasMetricClauses = fieldOptions.some((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const hasMetricCompareClauses = hasMetricClauses;
      const hasPackClauses = true;
      const hasPrice = fieldOptions.some((fieldOption) => fieldOption.value === "priceCp");
      const hasActionCost = user.search.getActionCostOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)).length > 0;
      const entryByValue = new Map<ClauseKind, { value: ClauseKind; label: string; description: string }>();
      entryByValue.set("scope", {
        value: "scope",
        label: "Scope",
        description: "Add a category and subcategory scope clause.",
      });
      if (hasFieldClauses) {
        entryByValue.set("field", {
          value: "field",
          label: "Metadata",
          description: "Filter on a metadata field such as traits or other categorical fields.",
        });
      }
      if (hasMetricClauses) {
        entryByValue.set("metric", {
          value: "metric",
          label: "Metric",
          description: "Filter on one discovered metric key.",
        });
      }
      if (hasMetricCompareClauses) {
        entryByValue.set("metricCompare", {
          value: "metricCompare",
          label: "Metric comparison",
          description: "Compare two numeric metrics from the current scoped discovery families.",
        });
      }
      if (hasPackClauses) {
        entryByValue.set("pack", {
          value: "pack",
          label: "Pack",
          description: "Restrict results to one or more selected packs without waiting on preflight discovery checks.",
        });
      }
      entryByValue.set("level", {
        value: "level",
        label: "Level",
        description: "Add a level matcher such as 1, >=5, <=10, or 1-5.",
      });
      if (hasPrice) {
        entryByValue.set("price", {
          value: "price",
          label: "Price",
          description: "Add a price matcher such as 100, >=500, <=1000, or 100-500.",
        });
      }
      entryByValue.set("rarity", {
        value: "rarity",
        label: "Rarity",
        description: "Add one rarity clause using the shared categorical picker family.",
      });
      if (hasActionCost) {
        entryByValue.set("actionCost", {
          value: "actionCost",
          label: "Action Cost",
          description: "Add an action-cost matcher such as 1, >=2, <=3, or 1-2.",
        });
      }
      const entries = (
        ["scope", "field", "metric", "metricCompare", "pack", "level", "price", "rarity", "actionCost"] as const
      )
        .map((value) => entryByValue.get(value))
        .filter((entry): entry is { value: ClauseKind; label: string; description: string } => Boolean(entry));
      const result = await promptSession.promptSelectOption({
        title: "Add Clause",
        prompt: "Choose the clause kind to insert into the current group",
        entries,
        selectedValue: entries[0]?.value ?? "scope",
      });
      if (result.kind === "back") {
        return CLAUSE_BACK;
      }
      return result.kind === "selected" ? result.value : null;
    },
    [getScopedFieldOptions, user.search],
  );

  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = [], wrapper?: "allOf" | "anyOf" | "not"): Promise<ClauseApplyResult> => {
      return terminal.runPromptSession(async (session) => {
        let workingQuery = query;
        while (true) {
          const clauseKind = await promptForClauseKind(session, workingQuery);
          if (clauseKind === CLAUSE_BACK) {
            return "back";
          }
          if (!clauseKind) {
            return "cancelled";
          }
          const nextNode = await promptForClauseNode(session, workingQuery, clauseKind);
          if (nextNode === CLAUSE_BACK) {
            continue;
          }
          if (!nextNode) {
            return "cancelled";
          }
          const shouldContinue = isClausePromptContinueResult(nextNode);
          const appliedNode = shouldContinue ? nextNode.node : nextNode;
          const wrappedNode =
            wrapper === "allOf" || wrapper === "anyOf"
              ? ({
                  kind: wrapper,
                  children: Array.isArray(appliedNode) ? appliedNode : [appliedNode],
                } as SearchFilterNode)
              : wrapper === "not"
                ? ({
                    kind: "not",
                    child: Array.isArray(appliedNode)
                      ? ({ kind: "allOf", children: appliedNode } as SearchFilterNode)
                      : appliedNode,
                  } as SearchFilterNode)
                : appliedNode;
          const nextFilter = Array.isArray(wrappedNode)
            ? appendSearchFilterNodesAtPath(workingQuery.filter, path, wrappedNode, getSearchQueryRootOperator(workingQuery))
            : appendSearchFilterNodeAtPath(workingQuery.filter, path, wrappedNode, getSearchQueryRootOperator(workingQuery));
          workingQuery = {
            ...workingQuery,
            filter: nextFilter,
          };
          applyNextTree(
            nextFilter,
        );
        if (shouldContinue) {
          continue;
        }
          return "applied";
        }
      });
    },
    [applyNextTree, promptForClauseKind, promptForClauseNode, terminal],
  );

  const runInsertionAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (actionId === "moveHere") {
        if (!moveSourcePath) {
          return;
        }
        applyNextTree(
          moveSearchFilterNodeToGroupPath(
            query.filter,
            moveSourcePath,
            path,
            getSearchQueryRootOperator(query),
          ),
        );
        clearStructuredDraftMoveSource();
        return;
      }

      if (actionId === "addClause") {
        return addQueryClauseAtPath(query, path);
      }

      if (actionId === "addAndGroup" || actionId === "addOrGroup" || actionId === "addNotGroup") {
        return addQueryClauseAtPath(
          query,
          path,
          actionId === "addAndGroup" ? "allOf" : actionId === "addOrGroup" ? "anyOf" : "not",
        );
      }
      return "cancelled";
    },
    [addQueryClauseAtPath, applyNextTree, clearStructuredDraftMoveSource, moveSourcePath],
  );

  const promptForInsertionAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[]) => {
      while (true) {
        const entries = buildInsertionActionEntries(Boolean(moveSourcePath));
        const result = await prompts.promptSelectOption({
          title: "Insertion Slot",
          prompt: moveSourcePath ? "Choose where to move the selected node" : "Choose what to add at this insertion slot",
          entries: entries.map((entry) => ({
            value: entry.id,
            label: entry.label,
            description: entry.description,
          })),
          selectedValue: entries[0]?.id ?? "addClause",
        });
        if (result.kind !== "selected") {
          return;
        }
        const insertionResult = await runInsertionAction(query, path, result.value as StructuredDraftEntryActionId);
        if (insertionResult !== "back") {
          return;
        }
      }
    },
    [moveSourcePath, runInsertionAction],
  );

  const runRootAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (actionId === "toggleRoot") {
        applyNextTree(toggleSearchFilterRootGroupOperator(query.filter));
        return;
      }

      await runInsertionAction(query, [], actionId);
    },
    [applyNextTree, runInsertionAction],
  );

  const promptForRootAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery) => {
      const entries = buildRootActionEntries(query);
      const result = await prompts.promptSelectOption({
        title: "Root Group",
        prompt: "Choose how to update the visible root group",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runRootAction(query, result.value as StructuredDraftEntryActionId);
    },
    [runRootAction],
  );

  const getLeafActionEntries = React.useCallback(
    (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const fieldOptions = getScopedFieldOptions(query);
      const fieldOptionValue = getQueryFieldValueForNode(node);
      const fieldOption = fieldOptionValue
        ? fieldOptions.find((candidate) => candidate.value === fieldOptionValue) ?? null
        : null;
      const editableMetadataNode = canonicalFilterToMetadataNode(node);
      const editableClauseKind: ClauseKind | null =
        node.kind === "scope"
          ? "scope"
          : node.kind === "level"
            ? "level"
            : node.kind === "price"
              ? "price"
              : node.kind === "pack"
                ? "pack"
                : node.kind === "metricCompare"
                  ? "metricCompare"
                  : node.kind === "rarity"
                    ? "rarity"
                    : node.kind === "actionCost"
                      ? "actionCost"
                      : fieldOption && editableMetadataNode
                        ? isMetricFieldOptionValue(fieldOption.value)
                          ? "metric"
                          : "field"
                        : null;
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);

      return [
        ...(editableClauseKind
          ? [{ id: "edit" as const, label: "Edit Clause", description: "Change this canonical clause without leaving the tree editor." }]
          : []),
        {
          id: "wrapNot",
          label: `Wrap In ${formatFriendlyGroupLabel("not")}`,
          description: "Exclude this clause without changing its content.",
        },
        {
          id: "wrapAnd",
          label: `Wrap In ${formatFriendlyGroupLabel("allOf")}`,
          description: "Place this clause inside a new group where every child must match.",
        },
        {
          id: "wrapOr",
          label: `Wrap In ${formatFriendlyGroupLabel("anyOf")}`,
          description: "Place this clause inside a new group where any child may match.",
        },
        { id: "move", label: "Move Node", description: "Move this clause to another visible insertion slot." },
        ...(canLift ? [{ id: "lift" as const, label: "Lift Node", description: "Lift this clause out of its current boolean group." }] : []),
        { id: "remove", label: "Remove Clause", description: "Delete this clause from the live query tree." },
      ];
    },
    [getScopedFieldOptions],
  );

  const runLeafAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: SearchFilterNode,
      actionId: StructuredDraftEntryActionId,
    ) => {
      const fieldOptions = getScopedFieldOptions(query);
      const fieldOptionValue = getQueryFieldValueForNode(node);
      const fieldOption = fieldOptionValue
        ? fieldOptions.find((candidate) => candidate.value === fieldOptionValue) ?? null
        : null;
      const editableMetadataNode = canonicalFilterToMetadataNode(node);
      const editableClauseKind: ClauseKind | null =
        node.kind === "scope"
          ? "scope"
          : node.kind === "level"
            ? "level"
            : node.kind === "price"
              ? "price"
              : node.kind === "pack"
                ? "pack"
                : node.kind === "metricCompare"
                  ? "metricCompare"
                  : node.kind === "rarity"
                    ? "rarity"
                    : node.kind === "actionCost"
                      ? "actionCost"
                      : fieldOption && editableMetadataNode
                        ? isMetricFieldOptionValue(fieldOption.value)
                          ? "metric"
                          : "field"
                        : null;

      if (actionId === "edit") {
        if (!editableClauseKind) {
          await terminal.pauseForAnyKey("That clause cannot be edited through the current canonical editor set.");
          return;
        }
        if (editableClauseKind === "rarity") {
          await openLiveExplorerQueryField(
            query,
            buildExplorerOnlyFieldOption(
              "rarity",
              "Rarity",
              "Browse live rarities for the current scope and stage canonical rarity clauses.",
              "enumString",
            ),
          );
          return;
        }
        if (editableClauseKind === "actionCost") {
          await openLiveExplorerQueryField(
            query,
            buildExplorerOnlyFieldOption(
              "actionCost",
              "Action Cost",
              "Browse live action costs for the current scope and stage canonical action-cost clauses.",
              "number",
            ),
          );
          return;
        }
        if ((editableClauseKind === "field" || editableClauseKind === "metric") && fieldOption && editableMetadataNode) {
          if (fieldOption.editor === "sharedExplorer") {
            await openLiveExplorerFieldClause(query, path, fieldOption, editableMetadataNode);
            return;
          }
          const nextNode = await editFieldClause(query, fieldOption, editableMetadataNode);
          if (nextNode !== undefined) {
            updateStructuredDraftMetadataNode(path, () => nextNode);
          }
          return;
        }

        const nextNode = await terminal.runPromptSession((session) =>
          promptForClauseNode(session, query, editableClauseKind, node),
        );
        const shouldContinue = isClausePromptContinueResult(nextNode);
        if (nextNode === CLAUSE_BACK || nextNode === undefined || Array.isArray(nextNode) || shouldContinue) {
          return;
        }
        if (nextNode === null) {
          applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
          return;
        }
        applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => nextNode));
        return;
      }

      if (actionId === "wrapNot" || actionId === "wrapAnd" || actionId === "wrapOr") {
        applyNextTree(
          wrapSearchFilterNodeAtPath(
            query.filter,
            path,
            actionId === "wrapNot" ? "not" : actionId === "wrapAnd" ? "allOf" : "anyOf",
          ),
        );
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
      }
    },
    [
      applyNextTree,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      openLiveExplorerQueryField,
      openLiveExplorerFieldClause,
      promptForClauseNode,
      terminal,
      updateStructuredDraftMetadataNode,
    ],
  );

  const promptForLeafAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode) => {
      const entries = getLeafActionEntries(query, path, node);
      const result = await prompts.promptSelectOption({
        title: "Query Clause",
        prompt: "Choose how to update this live query clause",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "wrapNot",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runLeafAction(query, path, node, result.value as StructuredDraftEntryActionId);
    },
    [getLeafActionEntries, runLeafAction],
  );

  const getNotActionEntries = React.useCallback(
    (query: Pf2eTerminalSearchQuery, path: number[]): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      return [
        {
          id: "unwrap",
          label: `Remove ${formatFriendlyGroupLabel("not")}`,
          description: "Keep the child clause and remove the exclusion.",
        },
        {
          id: "move",
          label: "Move Node",
          description: "Move this excluded group to another visible insertion slot.",
        },
        ...(canLift
          ? [{ id: "lift" as const, label: "Lift Node", description: "Lift this excluded group out of its current parent group." }]
          : []),
        { id: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
      ];
    },
    [],
  );

  const runNotAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "not" }>,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (actionId === "unwrap") {
        applyNextTree(path.length === 0 ? node.child : updateSearchFilterNodeAtPath(query.filter, path, () => node.child));
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
      }
    },
    [applyNextTree, enterStructuredDraftMoveMode],
  );

  const promptForNotAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: Extract<SearchFilterNode, { kind: "not" }>) => {
      const entries = getNotActionEntries(query, path);
      const result = await prompts.promptSelectOption({
        title: `${formatFriendlyGroupLabel("not")} Group`,
        prompt: "Choose how to update this negated live clause",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "unwrap",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runNotAction(query, path, node, result.value as StructuredDraftEntryActionId);
    },
    [getNotActionEntries, runNotAction],
  );

  const getGroupActionEntries = React.useCallback(
    (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const canUnwrap = canUnwrapSearchFilterNodeAtPath(query.filter, path);
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      return [
        { id: "addClause", label: "Add Clause", description: "Append a new canonical clause at this group bottom." },
        {
          id: "addAndGroup",
          label: `Add ${formatFriendlyGroupLabel("allOf")} Group`,
          description: "Append a nested group where every child must match.",
        },
        {
          id: "addOrGroup",
          label: `Add ${formatFriendlyGroupLabel("anyOf")} Group`,
          description: "Append a nested group where any child may match.",
        },
        {
          id: "addNotGroup",
          label: `Add ${formatFriendlyGroupLabel("not")} Group`,
          description: "Append a nested excluded group.",
        },
        {
          id: "toggleGroup",
          label:
            node.kind === "allOf"
              ? `Change To ${formatFriendlyGroupLabel("anyOf")}`
              : `Change To ${formatFriendlyGroupLabel("allOf")}`,
          description: "Reshape this group without changing its current children.",
        },
        {
          id: "wrapNot",
          label: `Wrap In ${formatFriendlyGroupLabel("not")}`,
          description: "Wrap this group in an excluded group.",
        },
        { id: "move", label: "Move Node", description: "Move this group to another visible insertion slot." },
        ...(canUnwrap ? [{ id: "unwrap" as const, label: "Unwrap Group", description: "Replace this group with its current children." }] : []),
        ...(canLift ? [{ id: "lift" as const, label: "Lift Node", description: "Lift this group out of its current parent group." }] : []),
        { id: "remove", label: "Remove Group", description: "Delete this group and all of its children." },
      ];
    },
    [],
  );

  const runGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
      actionId: StructuredDraftEntryActionId,
    ) => {
      if (actionId === "addClause" || actionId === "addAndGroup" || actionId === "addOrGroup" || actionId === "addNotGroup") {
        await runInsertionAction(query, path, actionId);
        return;
      }
      if (actionId === "toggleGroup") {
        applyNextTree(
          reshapeSearchFilterBooleanGroupAtPath(query.filter, path, node.kind === "allOf" ? "anyOf" : "allOf"),
        );
        return;
      }
      if (actionId === "wrapNot") {
        applyNextTree(wrapSearchFilterNodeAtPath(query.filter, path, "not"));
        return;
      }
      if (actionId === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (actionId === "unwrap") {
        applyNextTree(unwrapSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (actionId === "remove") {
        applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
      }
    },
    [applyNextTree, enterStructuredDraftMoveMode, runInsertionAction],
  );

  const promptForGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
    ) => {
      const entries = getGroupActionEntries(query, path, node);
      const result = await prompts.promptSelectOption({
        title: "Boolean Group",
        prompt: "Choose how to update this live boolean group",
        entries: entries.map((entry) => ({
          value: entry.id,
          label: entry.label,
          description: entry.description,
        })),
        selectedValue: entries[0]?.id ?? "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      await runGroupAction(query, path, node, result.value as StructuredDraftEntryActionId);
    },
    [getGroupActionEntries, runGroupAction],
  );

  const editStructuredDraftMetadata = React.useCallback(
    async (entry: SearchStructuredDraftEntry) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await promptForGroupAction(draftQuery, rootPath, rootNode);
            return;
          }
        }
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
        await runInsertionAction(draftQuery, entry.insertionPath ?? [], "addClause");
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        await openLiveExplorerGroupedField(draftQuery, entry);
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
      moveSourcePath,
      promptForGroupAction,
      promptForInsertionAction,
      promptForLeafAction,
      promptForNotAction,
      promptForRootAction,
      structuredDraftQuery,
    ],
  );

  const getStructuredDraftEntryActions = React.useCallback(
    (
      entry: SearchStructuredDraftEntry | null | undefined,
    ): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return [];
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            return getGroupActionEntries(draftQuery, rootPath, rootNode);
          }
        }
        return buildRootActionEntries(draftQuery);
      }
      if (entry.kind === "queryInsertionSlot") {
        return buildInsertionActionEntries(Boolean(moveSourcePath));
      }
      if (entry.kind === "queryFieldBucket") {
        return [{ id: "edit", label: "Edit Clause", description: "Edit this current-group field bucket through the shared explorer." }];
      }
      if (entry.kind !== "queryNode") {
        return [];
      }

      const node = getSearchFilterNodeAtPath(draftQuery.filter, entry.treePath ?? []);
      if (!node) {
        return [];
      }
      if (node.kind === "not") {
        return getNotActionEntries(draftQuery, entry.treePath ?? []);
      }
      if (isSearchFilterBooleanGroup(node)) {
        return getGroupActionEntries(draftQuery, entry.treePath ?? [], node);
      }
      return getLeafActionEntries(draftQuery, entry.treePath ?? [], node);
    },
    [
      getGroupActionEntries,
      getLeafActionEntries,
      getNotActionEntries,
      moveSourcePath,
      structuredDraftQuery,
    ],
  );

  const runStructuredDraftEntryAction = React.useCallback(
    async (
      entry: SearchStructuredDraftEntry | null | undefined,
      actionId: StructuredDraftEntryActionId,
    ) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery || !entry) {
        return;
      }
      if (entry.kind === "queryTreeRoot") {
        const rootPath = entry.treePath ?? [];
        if (rootPath.length > 0) {
          const rootNode = getSearchFilterNodeAtPath(draftQuery.filter, rootPath);
          if (rootNode && isSearchFilterBooleanGroup(rootNode)) {
            await runGroupAction(draftQuery, rootPath, rootNode, actionId);
            return;
          }
        }
        await runRootAction(draftQuery, actionId);
        return;
      }
      if (entry.kind === "queryInsertionSlot") {
        await runInsertionAction(draftQuery, entry.insertionPath ?? [], actionId);
        return;
      }
      if (entry.kind === "queryFieldBucket") {
        if (actionId === "edit") {
          await openLiveExplorerGroupedField(draftQuery, entry);
        }
        return;
      }
      if (entry.kind !== "queryNode") {
        return;
      }

      const path = entry.treePath ?? [];
      const node = getSearchFilterNodeAtPath(draftQuery.filter, path);
      if (!node) {
        return;
      }
      if (node.kind === "not") {
        await runNotAction(draftQuery, path, node, actionId);
        return;
      }
      if (isSearchFilterBooleanGroup(node)) {
        await runGroupAction(draftQuery, path, node, actionId);
        return;
      }
      await runLeafAction(draftQuery, path, node, actionId);
    },
    [
      runGroupAction,
      runInsertionAction,
      runLeafAction,
      runNotAction,
      openLiveExplorerGroupedField,
      runRootAction,
      structuredDraftQuery,
    ],
  );

  return {
    editStructuredDraftMetadata,
    getStructuredDraftEntryActions,
    runStructuredDraftEntryAction,
  };
}
