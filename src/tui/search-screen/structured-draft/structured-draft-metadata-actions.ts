import React from "react";

import { inferActorMetricValueType } from "../../../domain/actor-metrics.js";
import type { MetadataFilterNode } from "../../../domain/metadata-filter-types.js";
import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../../domain/categories.js";
import { inferItemMetricValueType } from "../../../domain/item-metrics.js";
import type {
  SearchNumericMatch,
  SearchFilterNode,
  SearchScopeSubcategoryMatch,
} from "../../../domain/search-request-types.js";
import {
  appendSearchFilterNodeAtPath,
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
import type { Pf2eTerminalQueryFieldOption, Pf2eTerminalSearchQuery } from "../../search/service.js";
import type { SearchStructuredDraftEntry } from "../../search/structured-draft-session.js";
import { promptLevelRangeDraft } from "../../filter-explorer/scalar-editor.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";

type ClauseKind = "field" | "metric" | "metricCompare" | "pack" | "scope" | "level" | "price" | "rarity" | "actionCost";
type MetricFieldFamily = "actorMetric" | "itemMetric";
type MetricCompareOperator = Extract<Extract<SearchFilterNode, { kind: "metricCompare" }>["op"], string>;

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
    onApply: (nextNode: MetadataFilterNode | null) => void,
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

  const promptForFieldClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      family: "field" | "metric",
      currentNode: MetadataFilterNode | null = null,
    ): Promise<SearchFilterNode | null | undefined> => {
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
        title: family === "metric" ? "Metric Filter" : "Field Filter",
        prompt: family === "metric" ? "Choose the metric family for the next clause" : "Choose the field for the next clause",
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
      if (selection.kind !== "selected") {
        return undefined;
      }

      const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
      if (!fieldOption) {
        return undefined;
      }

      if (fieldOption.editor === "sharedExplorer") {
        return new Promise<SearchFilterNode | null | undefined>((resolve) => {
          void openOntologyFieldEditor(query, fieldOption, currentNode, (appliedNode) => {
            resolve(metadataFilterNodeToCanonicalFilter(appliedNode) ?? null);
          });
        });
      }

      const nextNode = await editFieldClause(query, fieldOption, currentNode);
      return nextNode === undefined ? undefined : nextNode ? metadataFilterNodeToCanonicalFilter(nextNode) ?? null : null;
    },
    [editFieldClause, getScopedFieldOptions, openOntologyFieldEditor, prompts, terminal],
  );

  const getAvailableMetricFamilies = React.useCallback(
    (query: Pf2eTerminalSearchQuery, options: { numericOnly?: boolean } = {}): MetricFieldFamily[] => {
      const category = getSearchQueryCategory(query);
      const subcategory = getSearchQuerySubcategory(query);
      const families: MetricFieldFamily[] = [];
      if (user.search.getMetricKeyOptions(category, subcategory, "actorMetric", options).length > 0) {
        families.push("actorMetric");
      }
      if (user.search.getMetricKeyOptions(category, subcategory, "itemMetric", options).length > 0) {
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
      const category = getSearchQueryCategory(query);
      const subcategory = getSearchQuerySubcategory(query);
      const metricOptions = user.search.getMetricKeyOptions(category, subcategory, family, options);
      if (metricOptions.length === 0) {
        return undefined;
      }

      const selection = await prompts.promptSelectOption({
        title,
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
      });
      return selection.kind === "selected" ? selection.value : undefined;
    },
    [prompts, user.search],
  );

  const promptForMetricCompareClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "metricCompare" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "metricCompare" }> | undefined> => {
      const availableFamilies = getAvailableMetricFamilies(query, { numericOnly: true });
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
    ): Promise<Extract<SearchFilterNode, { kind: "pack" }> | undefined> => {
      const packOptions = user.search.getPackOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query));
      if (packOptions.length === 0) {
        await terminal.pauseForAnyKey("No packs are available for the current query.");
        return undefined;
      }

      const selection = await prompts.promptSelectOption({
        title: "Pack",
        prompt: "Choose the pack for this clause",
        entries: packOptions.map((packOption) => ({
          value: packOption.value,
          label: packOption.label,
          description: packOption.description,
        })),
        selectedValue:
          currentNode?.value && packOptions.some((packOption) => packOption.value === currentNode.value)
            ? currentNode.value
            : packOptions[0]!.value,
      });
      if (selection.kind !== "selected") {
        return undefined;
      }

      return {
        kind: "pack",
        value: selection.value,
      };
    },
    [prompts, terminal, user.search],
  );

  const promptForScopeClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "scope" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "scope" }> | undefined> => {
      const [, ...categoryOptions] = user.search.getCategoryOptions();
      if (categoryOptions.length === 0) {
        await terminal.pauseForAnyKey("No categories are available for the current query.");
        return undefined;
      }

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
      if (categorySelection.kind !== "selected") {
        return undefined;
      }

      const category = normalizeSearchCategory(categorySelection.value) ?? null;
      if (!category) {
        return undefined;
      }
      const subcategoryOptions = user.search.getSubcategoryOptions(category).filter((option) => option.value !== null);
      const currentMode =
        currentNode?.subcategory.kind === "eq"
          ? "specific"
          : currentNode?.subcategory.kind === "isNull"
            ? "none"
            : "any";
      const modeSelection = await prompts.promptSelectOption({
        title: "Subcategory Mode",
        prompt: "Choose how this scope clause should treat subcategories",
        entries: [
          { value: "any", label: "Any subcategory", description: "Match any subcategory inside the selected category." },
          {
            value: "specific",
            label: "Specific subcategory",
            description: "Choose one exact subcategory inside the selected category.",
          },
          { value: "none", label: "No subcategory", description: "Match only records without a subcategory." },
        ],
        selectedValue: currentMode,
      });
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
          currentNode?.subcategory.kind === "eq" ? currentNode.subcategory.value : null;
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
    },
    [prompts, terminal, user.search],
  );

  const promptForNumericMatchClause = React.useCallback(
    async (
      nodeKind: "level" | "price",
      node?: Extract<SearchFilterNode, { kind: "level" }> | Extract<SearchFilterNode, { kind: "price" }>,
    ): Promise<Extract<SearchFilterNode, { kind: "level" }> | Extract<SearchFilterNode, { kind: "price" }> | undefined> => {
      const parsed = await promptLevelRangeDraft(prompts, terminal, {
        defaultValue: node ? formatNumericMatch(node.match) : "",
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
    },
    [prompts, terminal],
  );

  const promptForRarityClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      node?: Extract<SearchFilterNode, { kind: "rarity" }>,
    ): Promise<SearchFilterNode | null | undefined> => {
      return new Promise<SearchFilterNode | null | undefined>((resolve) => {
        void openOntologyFieldEditor(
          query,
          buildExplorerOnlyFieldOption(
            "rarity",
            "Rarity",
            "Browse live rarities for the current scope and stage canonical rarity clauses.",
            "enumString",
          ),
          node ? canonicalFilterToMetadataNode(node) : null,
          (appliedNode) => {
            resolve(metadataFilterNodeToCanonicalFilter(appliedNode) ?? null);
          },
        );
      });
    },
    [openOntologyFieldEditor],
  );

  const promptForActionCostClause = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      node?: Extract<SearchFilterNode, { kind: "actionCost" }>,
    ): Promise<SearchFilterNode | null | undefined> => {
      return new Promise<SearchFilterNode | null | undefined>((resolve) => {
        void openOntologyFieldEditor(
          query,
          buildExplorerOnlyFieldOption(
            "actionCost",
            "Action Cost",
            "Browse live action costs for the current scope and stage canonical action-cost clauses.",
            "number",
          ),
          node ? canonicalFilterToMetadataNode(node) : null,
          (appliedNode) => {
            resolve(metadataFilterNodeToCanonicalFilter(appliedNode) ?? null);
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
    ): Promise<SearchFilterNode | null | undefined> => {
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
          return promptForActionCostClause(query, currentNode?.kind === "actionCost" ? currentNode : undefined);
      }
    },
    [
      promptForActionCostClause,
      promptForFieldClause,
      promptForMetricCompareClause,
      promptForNumericMatchClause,
      promptForPackClause,
      promptForRarityClause,
      promptForScopeClause,
    ],
  );

  const promptForClauseKind = React.useCallback(
    async (query: Pf2eTerminalSearchQuery): Promise<ClauseKind | null> => {
      const fieldOptions = getScopedFieldOptions(query);
      const hasFieldClauses = fieldOptions.some((fieldOption) => !isMetricFieldOptionValue(fieldOption.value));
      const hasMetricClauses = fieldOptions.some((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const hasMetricCompareClauses = getAvailableMetricFamilies(query, { numericOnly: true }).length > 0;
      const hasPackClauses = user.search.getPackOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)).length > 0;
      const hasPrice = fieldOptions.some((fieldOption) => fieldOption.value === "priceCp");
      const hasActionCost = user.search.getActionCostOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)).length > 0;
      const entries = [
        ...(hasFieldClauses
          ? [{ value: "field" as const, label: "Field filter", description: "Filter on a metadata field such as traits or rarity-like categorical fields." }]
          : []),
        ...(hasMetricClauses
          ? [{ value: "metric" as const, label: "Metric filter", description: "Filter on one discovered metric key." }]
          : []),
        ...(hasMetricCompareClauses
          ? [
              {
                value: "metricCompare" as const,
                label: "Metric comparison",
                description: "Compare two numeric metrics from the same discovery family.",
              },
            ]
          : []),
        ...(hasPackClauses
          ? [{ value: "pack" as const, label: "Pack", description: "Restrict results to one selected pack." }]
          : []),
        { value: "scope" as const, label: "Scope", description: "Add a category and subcategory scope clause." },
        { value: "level" as const, label: "Level", description: "Add a level matcher such as 1, >=5, <=10, or 1-5." },
        ...(hasPrice
          ? [{ value: "price" as const, label: "Price", description: "Add a price matcher such as 100, >=500, <=1000, or 100-500." }]
          : []),
        { value: "rarity" as const, label: "Rarity", description: "Add one rarity clause using the shared categorical picker family." },
        ...(hasActionCost
          ? [
              {
                value: "actionCost" as const,
                label: "Action cost",
                description: "Add one action-cost matcher using the shared categorical picker family.",
              },
            ]
          : []),
      ];
      const result = await prompts.promptSelectOption({
        title: "Add Clause",
        prompt: "Choose the clause kind to insert into the current group",
        entries,
        selectedValue: entries[0]?.value ?? "scope",
      });
      return result.kind === "selected" ? result.value : null;
    },
    [getAvailableMetricFamilies, getScopedFieldOptions, prompts, user.search],
  );

  const addQueryClauseAtPath = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[] = [], wrapper?: "allOf" | "anyOf" | "not") => {
      const clauseKind = await promptForClauseKind(query);
      if (!clauseKind) {
        return;
      }
      const nextNode = await promptForClauseNode(query, clauseKind);
      if (!nextNode) {
        return;
      }
      const wrappedNode =
        wrapper === "allOf" || wrapper === "anyOf"
          ? { kind: wrapper, children: [nextNode] as SearchFilterNode[] }
          : wrapper === "not"
            ? ({ kind: "not", child: nextNode } as SearchFilterNode)
            : nextNode;
      applyNextTree(
        appendSearchFilterNodeAtPath(query.filter, path, wrappedNode, getSearchQueryRootOperator(query)),
      );
    },
    [applyNextTree, promptForClauseKind, promptForClauseNode],
  );

  const promptForInsertionAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[]) => {
      const result = await prompts.promptSelectOption({
        title: "Insertion Slot",
        prompt: "Choose what to add at this insertion slot",
        entries: [
          { value: "addClause", label: "Add Clause", description: "Insert one canonical filter clause into this group." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Insert a nested allOf group with its first child." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Insert a nested anyOf group with its first child." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Insert a nested NOT group with its first child." },
        ],
        selectedValue: "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      if (result.value === "addClause") {
        await addQueryClauseAtPath(query, path);
        return;
      }
      await addQueryClauseAtPath(
        query,
        path,
        result.value === "addAndGroup" ? "allOf" : result.value === "addOrGroup" ? "anyOf" : "not",
      );
    },
    [addQueryClauseAtPath, prompts],
  );

  const promptForRootAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery) => {
      const result = await prompts.promptSelectOption({
        title: "Root Group",
        prompt: "Choose how to update the visible root group",
        entries: [
          { value: "addClause", label: "Add Clause", description: "Append a new top-level clause." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
          ...(query.filter
            ? [
                {
                  value: "toggle",
                  label: getSearchQueryRootOperator(query) === "anyOf" ? "Change Root To allOf" : "Change Root To anyOf",
                  description: "Reshape the visible root group without changing its current children.",
                },
              ]
            : []),
        ],
        selectedValue: "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      if (result.value === "toggle") {
        applyNextTree(toggleSearchFilterRootGroupOperator(query.filter));
        return;
      }
      await addQueryClauseAtPath(
        query,
        [],
        result.value === "addAndGroup" ? "allOf" : result.value === "addOrGroup" ? "anyOf" : result.value === "addNotGroup" ? "not" : undefined,
      );
    },
    [addQueryClauseAtPath, applyNextTree, prompts],
  );

  const promptForLeafAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: SearchFilterNode) => {
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
      const result = await prompts.promptSelectOption({
        title: "Query Clause",
        prompt: "Choose how to update this staged query clause",
        entries: [
          ...(editableClauseKind
            ? [{ value: "edit", label: "Edit Clause", description: "Change this canonical clause without leaving the tree editor." }]
            : []),
          { value: "wrapNot", label: "Wrap In NOT", description: "Negate this clause without changing its content." },
          { value: "wrapAnd", label: "Wrap In allOf", description: "Place this clause inside a new allOf group." },
          { value: "wrapOr", label: "Wrap In anyOf", description: "Place this clause inside a new anyOf group." },
          { value: "move", label: "Move Node", description: "Move this clause to another visible insertion slot." },
          ...(canLift ? [{ value: "lift", label: "Lift Node", description: "Lift this clause out of its current boolean group." }] : []),
          { value: "remove", label: "Remove Clause", description: "Delete this clause from the staged query." },
        ],
        selectedValue: editableClauseKind ? "edit" : "wrapNot",
      });
      if (result.kind !== "selected") {
        return;
      }
      if (result.value === "edit") {
        if (!editableClauseKind) {
          await terminal.pauseForAnyKey("That clause cannot be edited through the current canonical editor set.");
          return;
        }
        if ((editableClauseKind === "field" || editableClauseKind === "metric") && fieldOption && editableMetadataNode) {
          if (fieldOption.editor === "sharedExplorer") {
            await openOntologyFieldEditor(query, fieldOption, editableMetadataNode, (nextNode) => {
              updateStructuredDraftMetadataNode(path, () => nextNode);
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
        if (nextNode === undefined) {
          return;
        }
        applyNextTree(updateSearchFilterNodeAtPath(query.filter, path, () => nextNode ?? undefined));
        return;
      }
      if (result.value === "wrapNot" || result.value === "wrapAnd" || result.value === "wrapOr") {
        applyNextTree(
          wrapSearchFilterNodeAtPath(
            query.filter,
            path,
            result.value === "wrapNot" ? "not" : result.value === "wrapAnd" ? "allOf" : "anyOf",
          ),
        );
        return;
      }
      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
    },
    [
      applyNextTree,
      editFieldClause,
      enterStructuredDraftMoveMode,
      getScopedFieldOptions,
      openOntologyFieldEditor,
      promptForClauseNode,
      prompts,
      terminal,
      updateStructuredDraftMetadataNode,
    ],
  );

  const promptForNotAction = React.useCallback(
    async (query: Pf2eTerminalSearchQuery, path: number[], node: Extract<SearchFilterNode, { kind: "not" }>) => {
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      const result = await prompts.promptSelectOption({
        title: "NOT Group",
        prompt: "Choose how to update this negated staged clause",
        entries: [
          { value: "unwrap", label: "Remove NOT", description: "Keep the child clause and remove the negation." },
          { value: "move", label: "Move Node", description: "Move this NOT group to another visible insertion slot." },
          ...(canLift ? [{ value: "lift", label: "Lift Node", description: "Lift this NOT group out of its current parent group." }] : []),
          { value: "remove", label: "Remove Group", description: "Delete the negated clause entirely." },
        ],
        selectedValue: "unwrap",
      });
      if (result.kind !== "selected") {
        return;
      }
      if (result.value === "unwrap") {
        applyNextTree(path.length === 0 ? node.child : updateSearchFilterNodeAtPath(query.filter, path, () => node.child));
        return;
      }
      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
    },
    [applyNextTree, enterStructuredDraftMoveMode, prompts],
  );

  const promptForGroupAction = React.useCallback(
    async (
      query: Pf2eTerminalSearchQuery,
      path: number[],
      node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
    ) => {
      const canUnwrap = canUnwrapSearchFilterNodeAtPath(query.filter, path);
      const canLift = canLiftSearchFilterNodeAtPath(query.filter, path);
      const result = await prompts.promptSelectOption({
        title: "Boolean Group",
        prompt: "Choose how to update this staged boolean group",
        entries: [
          { value: "addClause", label: "Add Clause", description: "Append a new canonical clause at this group bottom." },
          { value: "addAndGroup", label: "Add allOf Group", description: "Append a nested allOf group." },
          { value: "addOrGroup", label: "Add anyOf Group", description: "Append a nested anyOf group." },
          { value: "addNotGroup", label: "Add NOT Group", description: "Append a nested NOT group." },
          {
            value: "toggle",
            label: node.kind === "allOf" ? "Change To anyOf" : "Change To allOf",
            description: "Reshape this group without changing its current children.",
          },
          { value: "wrapNot", label: "Wrap In NOT", description: "Wrap this group in a NOT node." },
          { value: "move", label: "Move Node", description: "Move this group to another visible insertion slot." },
          ...(canUnwrap ? [{ value: "unwrap", label: "Unwrap Group", description: "Replace this group with its current children." }] : []),
          ...(canLift ? [{ value: "lift", label: "Lift Node", description: "Lift this group out of its current parent group." }] : []),
          { value: "remove", label: "Remove Group", description: "Delete this group and all of its children." },
        ],
        selectedValue: "addClause",
      });
      if (result.kind !== "selected") {
        return;
      }
      if (result.value === "addClause") {
        await addQueryClauseAtPath(query, path);
        return;
      }
      if (result.value === "addAndGroup" || result.value === "addOrGroup" || result.value === "addNotGroup") {
        await addQueryClauseAtPath(
          query,
          path,
          result.value === "addAndGroup" ? "allOf" : result.value === "addOrGroup" ? "anyOf" : "not",
        );
        return;
      }
      if (result.value === "toggle") {
        applyNextTree(
          reshapeSearchFilterBooleanGroupAtPath(query.filter, path, node.kind === "allOf" ? "anyOf" : "allOf"),
        );
        return;
      }
      if (result.value === "wrapNot") {
        applyNextTree(wrapSearchFilterNodeAtPath(query.filter, path, "not"));
        return;
      }
      if (result.value === "move") {
        enterStructuredDraftMoveMode(path);
        return;
      }
      if (result.value === "unwrap") {
        applyNextTree(unwrapSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      if (result.value === "lift") {
        applyNextTree(liftSearchFilterNodeAtPath(query.filter, path));
        return;
      }
      applyNextTree(path.length === 0 ? undefined : updateSearchFilterNodeAtPath(query.filter, path, () => undefined));
    },
    [addQueryClauseAtPath, applyNextTree, enterStructuredDraftMoveMode, prompts],
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
        await promptForInsertionAction(draftQuery, entry.insertionPath ?? []);
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

  return {
    editStructuredDraftMetadata,
  };
}
