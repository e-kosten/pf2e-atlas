import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../../domain/categories.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type {
  SearchNumericMatch,
  SearchFilterNode,
  SearchScopeSubcategoryMatch,
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
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalPreparedFilterExplorerContext,
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
const CLAUSE_BACK = Symbol("search-structured-draft-clause-back");
type ClausePromptBackResult = typeof CLAUSE_BACK;
type SearchFilterNodeEditorResult = SearchFilterNode | SearchFilterNode[] | ClausePromptBackResult | null | undefined;
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
    { id: "addAndGroup", label: "Add allOf Group", description: "Insert a nested allOf group with its first child." },
    { id: "addOrGroup", label: "Add anyOf Group", description: "Insert a nested anyOf group with its first child." },
    { id: "addNotGroup", label: "Add NOT Group", description: "Insert a nested NOT group with its first child." },
  ];
}

function buildRootActionEntries(
  query: Pf2eTerminalSearchQuery,
): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] {
  return [
    { id: "addClause", label: "Add Clause", description: "Append a new top-level clause." },
    { id: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
    { id: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
    { id: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
    ...(query.filter
      ? [
          {
            id: "toggleRoot" as const,
            label: getSearchQueryRootOperator(query) === "anyOf" ? "Change Root To allOf" : "Change Root To anyOf",
            description: "Reshape the visible root group without changing its current children.",
          },
        ]
      : []),
  ];
}

function isMetricFieldOptionValue(value: Pf2eTerminalQueryFieldOption["value"]): boolean {
  return value === "actorMetric" || value === "itemMetric";
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
      draft: Pf2eTerminalFilterExplorerDraft,
      context: Pf2eTerminalPreparedFilterExplorerContext,
    ) => void,
    onReturn?: () => void,
  ) => Promise<boolean>;
  moveSourcePath: number[] | null;
  prompts: SearchWorkspacePromptAdapters;
  replaceStructuredDraftProjection: (update: (draftQuery: Pf2eTerminalSearchQuery) => Pf2eTerminalSearchQuery) => void;
  structuredDraftQuery: Pf2eTerminalSearchQuery | null;
  terminal: SearchWorkspaceTerminal;
  updateStructuredDraftMetadataNode: (
    path: number[],
    update: (current: MetadataFilterNode) => MetadataFilterNode | null,
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

  const promptForPickerDiscoveryMode = React.useCallback(
    async (
      title: string,
      discoveryMode: SearchFilterDiscoveryMode,
    ): Promise<SearchFilterDiscoveryMode | undefined> => {
      const selected = await prompts.promptSelectOption({
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

      const selectedValue =
        currentNode &&
        !("and" in currentNode) &&
        !("or" in currentNode) &&
        !("not" in currentNode) &&
        "field" in currentNode
          ? currentNode.field
          : fieldOptions[0]!.value;
      const selection = await prompts.promptSelectOption({
        title: family === "metric" ? "Metric" : "Metadata",
        prompt: family === "metric" ? "Choose the metric family for the next clause" : "Choose the metadata field for the next clause",
        entries: fieldOptions.map((fieldOption) => ({
          value: fieldOption.value,
          label: fieldOption.label,
          description: fieldOption.description,
        })),
        selectedValue:
          selectedValue && fieldOptions.some((fieldOption) => fieldOption.value === selectedValue)
            ? selectedValue
            : fieldOptions[0]!.value,
      });
      if (selection.kind === "back") {
        return CLAUSE_BACK;
      }
      if (selection.kind !== "selected") {
        return undefined;
      }

      const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
      if (!fieldOption) {
        return undefined;
      }

      if (fieldOption.editor === "sharedExplorer") {
        return new Promise<SearchFilterNodeEditorResult>((resolve) => {
          void openOntologyFieldEditor(query, fieldOption, currentNode, (result) => {
            if (result.kind === "insert") {
              resolve(
                result.nodes
                  .map((node) => metadataFilterNodeToCanonicalFilter(node))
                  .filter((node): node is SearchFilterNode => Boolean(node)),
              );
              return;
            }

            resolve(metadataFilterNodeToCanonicalFilter(result.node) ?? null);
          });
        });
      }

      const nextNode = await editFieldClause(query, fieldOption, currentNode);
      return nextNode === undefined ? undefined : nextNode ? metadataFilterNodeToCanonicalFilter(nextNode) ?? null : null;
    },
    [editFieldClause, getScopedFieldOptions, openOntologyFieldEditor, prompts, terminal],
  );

  const getAvailableMetricFamilies = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, options: { numericOnly?: boolean } = {}): Promise<MetricFieldFamily[]> => {
      const families: MetricFieldFamily[] = [];
      if ((await user.search.loadMetricKeyOptions(query, "actorMetric", "matching", options)).length > 0) {
        families.push("actorMetric");
      }
      if ((await user.search.loadMetricKeyOptions(query, "itemMetric", "matching", options)).length > 0) {
        families.push("itemMetric");
      }
      return families;
    },
    [user.search],
  );

  const promptForMetricFamily = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      availableFamilies: MetricFieldFamily[],
      currentFamily: MetricFieldFamily | null,
      title: string,
      prompt: string,
    ): Promise<MetricFieldFamily | undefined> => {
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

      const selection = await prompts.promptSelectOption({
        title,
        prompt,
        entries,
        selectedValue:
          currentFamily && entries.some((entry) => entry.value === currentFamily) ? currentFamily : entries[0]!.value,
      });
      return selection.kind === "selected" ? selection.value : undefined;
    },
    [getScopedFieldOptions, prompts],
  );

  const promptForMetricKey = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      family: MetricFieldFamily,
      title: string,
      prompt: string,
      currentMetric?: string,
      options: { numericOnly?: boolean } = {},
    ): Promise<string | undefined> => {
      let discoveryMode: SearchFilterDiscoveryMode = "matching";

      while (true) {
        const metricOptions = await user.search.loadMetricKeyOptions(query, family, discoveryMode, options);
        if (metricOptions.length === 0) {
          return undefined;
        }

        const selection = await prompts.promptSelectOption({
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
          return selection.value;
        }
        if (selection.kind !== "commands") {
          return undefined;
        }

        const nextMode = await promptForPickerDiscoveryMode(title, discoveryMode);
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
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "metricCompare" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "metricCompare" }> | undefined> => {
      const availableFamilies = await getAvailableMetricFamilies(query, { numericOnly: true });
      if (availableFamilies.length === 0) {
        await terminal.pauseForAnyKey("No numeric metric comparisons are available for the current query.");
        return undefined;
      }

      const currentFamily: MetricFieldFamily | null = currentNode
        ? inferMetricFieldFamily(currentNode.leftMetric, getSearchQueryCategory(query))
        : null;
      const family = await promptForMetricFamily(
        query,
        availableFamilies,
        currentFamily,
        "Metric Comparison",
        "Choose the metric family for this comparison clause",
      );
      if (!family) {
        return undefined;
      }

      const leftMetric = await promptForMetricKey(
        query,
        family,
        "Left Metric",
        "Choose the left-hand metric for this comparison clause",
        currentNode && currentFamily === family ? currentNode.leftMetric : undefined,
        { numericOnly: true },
      );
      if (!leftMetric) {
        return undefined;
      }

      const operatorSelection = await prompts.promptSelectOption({
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
      if (operatorSelection.kind !== "selected") {
        return undefined;
      }

      const rightMetric = await promptForMetricKey(
        query,
        family,
        "Right Metric",
        "Choose the right-hand metric for this comparison clause",
        currentNode && currentFamily === family ? currentNode.rightMetric : leftMetric,
        { numericOnly: true },
      );
      if (!rightMetric) {
        return undefined;
      }

      return {
        kind: "metricCompare",
        leftMetric,
        op: operatorSelection.value as MetricCompareOperator,
        rightMetric,
      };
    },
    [getAvailableMetricFamilies, promptForMetricFamily, promptForMetricKey, prompts, terminal],
  );

  const promptForPackClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "pack" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "pack" }> | Extract<SearchFilterNode, { kind: "pack" }>[] | undefined> => {
      let discoveryMode: SearchFilterDiscoveryMode = "matching";

      while (true) {
        const packOptions = await user.search.loadPackOptions(query, discoveryMode);
        if (packOptions.length === 0) {
          await terminal.pauseForAnyKey("No packs are available for the current query.");
          return undefined;
        }

        if (currentNode) {
          const selection = await prompts.promptSelectOption({
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
          if (selection.kind !== "commands") {
            return undefined;
          }
        } else {
          const selection = await prompts.promptMultiSelectOption({
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
          if (selection.kind !== "commands") {
            return undefined;
          }
        }

        const nextMode = await promptForPickerDiscoveryMode("Pack", discoveryMode);
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
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "scope" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "scope" }> | ClausePromptBackResult | undefined> => {
      const [, ...categoryOptions] = user.search.getCategoryOptions();
      if (categoryOptions.length === 0) {
        await terminal.pauseForAnyKey("No categories are available for the current query.");
        return undefined;
      }

      while (true) {
        const categorySelection = await prompts.promptSelectOption({
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
          const modeSelection = await prompts.promptSelectOption({
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
            const subcategorySelection = await prompts.promptSelectOption({
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
        const parsed = await promptLevelRangeDraft(prompts, terminal, {
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

      const parsed = await promptNumericScalarClause(prompts, terminal, {
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
    [prompts, terminal],
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
        );
      });
    },
    [openOntologyFieldEditor],
  );

  const promptForClauseNode = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      clauseKind: ClauseKind,
      currentNode?: SearchFilterNode,
    ): Promise<SearchFilterNodeEditorResult> => {
      switch (clauseKind) {
        case "field":
          return promptForFieldClause(query, "field", currentNode ? canonicalFilterToMetadataNode(currentNode) : null);
        case "metric":
          return promptForFieldClause(query, "metric", currentNode ? canonicalFilterToMetadataNode(currentNode) : null);
        case "metricCompare":
          return promptForMetricCompareClause(query, currentNode?.kind === "metricCompare" ? currentNode : undefined);
        case "pack":
          return promptForPackClause(query, currentNode?.kind === "pack" ? currentNode : undefined);
        case "scope":
          return promptForScopeClause(query, currentNode?.kind === "scope" ? currentNode : undefined);
        case "level":
          return promptForNumericMatchClause("level", currentNode?.kind === "level" ? currentNode : undefined);
        case "price":
          return promptForNumericMatchClause("price", currentNode?.kind === "price" ? currentNode : undefined);
        case "rarity":
          return promptForRarityClause(query, currentNode?.kind === "rarity" ? currentNode : undefined);
        case "actionCost":
          return promptForNumericMatchClause("actionCost", currentNode?.kind === "actionCost" ? currentNode : undefined);
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
    async (query: Pf2eTerminalSearchQuery): Promise<ClausePromptResult> => {
      const fieldOptions = getScopedFieldOptions(query);
      const hasFieldClauses = fieldOptions.some((fieldOption) => !isMetricFieldOptionValue(fieldOption.value));
      const hasMetricClauses = fieldOptions.some((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const hasMetricCompareClauses = (await getAvailableMetricFamilies(query, { numericOnly: true })).length > 0;
      const hasPackClauses = (await user.search.loadPackOptions(query, "matching")).length > 0;
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
          label: "Metric Comparison",
          description: "Compare two numeric metrics from the same discovery family.",
        });
      }
      if (hasPackClauses) {
        entryByValue.set("pack", {
          value: "pack",
          label: "Pack",
          description: "Restrict results to one or more selected packs.",
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
      const result = await prompts.promptSelectOption({
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
    [getAvailableMetricFamilies, getScopedFieldOptions, prompts, user.search],
  );

  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = [], wrapper?: "allOf" | "anyOf" | "not"): Promise<ClauseApplyResult> => {
      while (true) {
        const clauseKind = await promptForClauseKind(query);
        if (clauseKind === CLAUSE_BACK) {
          return "back";
        }
        if (!clauseKind) {
          return "cancelled";
        }
        const nextNode = await promptForClauseNode(query, clauseKind);
        if (nextNode === CLAUSE_BACK) {
          continue;
        }
        if (!nextNode) {
          return "cancelled";
        }
        const wrappedNode =
          wrapper === "allOf" || wrapper === "anyOf"
            ? ({
                kind: wrapper,
                children: Array.isArray(nextNode) ? nextNode : [nextNode],
              } as SearchFilterNode)
            : wrapper === "not"
              ? ({
                  kind: "not",
                  child: Array.isArray(nextNode)
                    ? ({ kind: "allOf", children: nextNode } as SearchFilterNode)
                    : nextNode,
                } as SearchFilterNode)
              : nextNode;
        applyNextTree(
          Array.isArray(wrappedNode)
            ? appendSearchFilterNodesAtPath(query.filter, path, wrappedNode, getSearchQueryRootOperator(query))
            : appendSearchFilterNodeAtPath(query.filter, path, wrappedNode, getSearchQueryRootOperator(query)),
        );
        return "applied";
      }
    },
    [applyNextTree, promptForClauseKind, promptForClauseNode],
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
    [moveSourcePath, prompts, runInsertionAction],
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
    [prompts, runRootAction],
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
        { id: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
        { id: "wrapAnd", label: "Wrap In allOf", description: "Place this clause inside a new allOf group." },
        { id: "wrapOr", label: "Wrap In anyOf", description: "Place this clause inside a new anyOf group." },
        { id: "move", label: "Move Node", description: "Move this clause to another visible insertion slot." },
        ...(canLift ? [{ id: "lift" as const, label: "Lift Node", description: "Lift this clause out of its current boolean group." }] : []),
        { id: "remove", label: "Remove Clause", description: "Delete this clause from the staged query." },
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
        if ((editableClauseKind === "field" || editableClauseKind === "metric") && fieldOption && editableMetadataNode) {
          if (fieldOption.editor === "sharedExplorer") {
            await openOntologyFieldEditor(query, fieldOption, editableMetadataNode, (result) => {
              if (result.kind !== "replace") {
                return;
              }
              updateStructuredDraftMetadataNode(path, () => result.node);
            });
            return;
          }
          const nextNode = await editFieldClause(query, fieldOption, editableMetadataNode);
          if (nextNode !== undefined) {
            updateStructuredDraftMetadataNode(path, () => nextNode);
          }
          return;
        }

        const nextNode = await promptForClauseNode(query, editableClauseKind, node);
        if (nextNode === CLAUSE_BACK || nextNode === undefined || Array.isArray(nextNode)) {
          return;
        }
        applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => nextNode ?? undefined));
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
      openOntologyFieldEditor,
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
        prompt: "Choose how to update this staged query clause",
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
    [getLeafActionEntries, prompts, runLeafAction],
  );

  const getNotActionEntries = React.useCallback(
    (query: Pf2eTerminalSearchQuery, path: number[]): DerivedTagTerminalActionTargetOption<StructuredDraftEntryActionId>[] => {
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      return [
        { id: "unwrap", label: "Remove NOT", description: "Keep the child clause and remove the negation." },
        { id: "move", label: "Move Node", description: "Move this NOT group to another visible insertion slot." },
        ...(canLift ? [{ id: "lift" as const, label: "Lift Node", description: "Lift this NOT group out of its current parent group." }] : []),
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
        title: "NOT Group",
        prompt: "Choose how to update this negated staged clause",
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
    [getNotActionEntries, prompts, runNotAction],
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
        { id: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
        { id: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
        { id: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
        {
          id: "toggleGroup",
          label: node.kind === "allOf" ? "Change To anyOf" : "Change To allOf",
          description: "Reshape this group without changing its current children.",
        },
        { id: "wrapNot", label: "Wrap In NOT", description: "Wrap this group in a NOT node." },
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
        prompt: "Choose how to update this staged boolean group",
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
    [getGroupActionEntries, prompts, runGroupAction],
  );

  const editStructuredDraftMetadata = React.useCallback(
    async (entry: SearchStructuredDraftEntry) => {
      const draftQuery = structuredDraftQuery;
      if (!draftQuery) {
        return;
      }
      if (entry.kind === "queryTreeRoot") {
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
        return buildRootActionEntries(draftQuery);
      }
      if (entry.kind === "queryInsertionSlot") {
        return buildInsertionActionEntries(Boolean(moveSourcePath));
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
        await runRootAction(draftQuery, actionId);
        return;
      }
      if (entry.kind === "queryInsertionSlot") {
        await runInsertionAction(draftQuery, entry.insertionPath ?? [], actionId);
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
