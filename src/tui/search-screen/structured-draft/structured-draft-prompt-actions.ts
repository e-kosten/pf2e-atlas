import React from "react";

import { normalizeSearchCategory, normalizeSearchSubcategory } from "../../../domain/categories.js";
import type { SearchFilterDiscoveryMode } from "../../../domain/search-field-domains.js";
import type {
  SearchFilterNode,
  SearchNumericMatch,
  SearchScopeSubcategoryMatch,
} from "../../../domain/search-request-types.js";
import type { SearchCategory } from "../../../domain/search-types.js";
import { promptLevelRangeDraft, promptNumericScalarClause } from "../../filter-explorer/scalar-editor.js";
import type { MetadataFilterNode } from "../../search/metadata-filter-draft.js";
import { canonicalFilterToMetadataNode, metadataFilterNodeToCanonicalFilter } from "../../search/query-parts.js";
import {
  getSearchQueryCategory,
  getSearchQuerySubcategory,
} from "../../search/query-state.js";
import type {
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalSearchQuery,
} from "../../search/service.js";
import type {
  SearchWorkspacePromptAdapters,
  SearchWorkspaceTerminal,
  SearchWorkspaceUser,
} from "../workspace/workspace-action-types.js";
import {
  structuredDraftPromptApply,
  structuredDraftPromptBack,
  structuredDraftPromptCancel,
  type StructuredDraftPromptFlowResult,
} from "./structured-draft-continuation.js";
import {
  getMetadataFilterNodeFieldValue,
  inferMetricFieldFamily,
  type MetricFieldFamily,
  type StructuredDraftExplorerMetricKeyResult,
  type StructuredDraftExplorerMetricKeySelection,
  type StructuredDraftExplorerPromptNodeResult,
} from "./structured-draft-explorer-actions.js";
import { isStructuredDraftGroupFieldOption } from "./structured-draft-edit-routes.js";

export type ClauseKind =
  | "field"
  | "metric"
  | "metricCompare"
  | "pack"
  | "scope"
  | "level"
  | "price"
  | "rarity"
  | "actionCost";

type MetricCompareOperator = Extract<Extract<SearchFilterNode, { kind: "metricCompare" }>["op"], string>;
type MetricKeySelection = StructuredDraftExplorerMetricKeySelection;
type PromptStepResult<T> = StructuredDraftPromptFlowResult<T>;
export type SearchFilterNodeEditorValue = SearchFilterNode | SearchFilterNode[] | null;
export type SearchFilterNodeEditorResult = StructuredDraftPromptFlowResult<SearchFilterNodeEditorValue>;
export type ClausePromptResult = StructuredDraftPromptFlowResult<ClauseKind>;
export type SharedExplorerFieldOptionPromptResult =
  StructuredDraftPromptFlowResult<Pf2eTerminalQueryFieldOption | null>;

export function isMetricFieldOptionValue(value: Pf2eTerminalQueryFieldOption["value"]): boolean {
  return value === "actorMetric" || value === "itemMetric";
}

function formatNumericMatch(match: SearchNumericMatch): string {
  switch (match.kind) {
    case "eq":
      return String(match.value);
    case "gt":
      return `>${match.value}`;
    case "gte":
      return `>=${match.value}`;
    case "lt":
      return `<${match.value}`;
    case "lte":
      return `<=${match.value}`;
    case "between":
      return `${match.min}-${match.max}`;
  }
}

export function useStructuredDraftPromptActions({
  editFieldClause,
  getScopedFieldOptions,
  openPromptFieldClause,
  selectPromptMetricKey,
  terminal,
  user,
}: {
  editFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode?: MetadataFilterNode | null,
  ) => Promise<MetadataFilterNode | null | undefined>;
  getScopedFieldOptions: (query: Pf2eTerminalSearchQuery) => Pf2eTerminalQueryFieldOption[];
  openPromptFieldClause: (
    query: Pf2eTerminalSearchQuery,
    fieldOption: Pf2eTerminalQueryFieldOption,
    currentNode: MetadataFilterNode | null,
  ) => Promise<StructuredDraftExplorerPromptNodeResult>;
  selectPromptMetricKey: (
    query: Pf2eTerminalSearchQuery,
    family: MetricFieldFamily,
    fieldOption: Pf2eTerminalQueryFieldOption,
    title: string,
    options?: { numericOnly?: boolean },
    initialDiscoveryMode?: SearchFilterDiscoveryMode,
  ) => Promise<StructuredDraftExplorerMetricKeyResult>;
  terminal: SearchWorkspaceTerminal;
  user: SearchWorkspaceUser;
}) {
  const promptForFieldClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: "field" | "metric",
      currentNode: MetadataFilterNode | null = null,
    ): Promise<SearchFilterNodeEditorResult> => {
      const fieldOptions = getScopedFieldOptions(query).filter((fieldOption) =>
        family === "metric"
          ? isMetricFieldOptionValue(fieldOption.value)
          : !isMetricFieldOptionValue(fieldOption.value) && !isStructuredDraftGroupFieldOption(fieldOption),
      );
      if (fieldOptions.length === 0) {
        await terminal.pauseForAnyKey(
          family === "metric"
            ? "No scoped metric filters are available for the current query."
            : "No scoped field filters are available for the current query.",
        );
        return structuredDraftPromptCancel();
      }

      let preferredFieldValue = getMetadataFilterNodeFieldValue(currentNode) ?? fieldOptions[0]!.value;
      for (;;) {
        const selection = await promptSession.promptSelectOption({
          title: family === "metric" ? "Metric" : "Metadata",
          prompt:
            family === "metric"
              ? "Choose the metric family for the next clause"
              : "Choose the metadata field for the next clause",
          entries: fieldOptions.map((fieldOption) => ({
            value: fieldOption.value,
            label: fieldOption.label,
            description: fieldOption.description,
          })),
          selectedValue: fieldOptions.some((fieldOption) => fieldOption.value === preferredFieldValue)
            ? preferredFieldValue
            : fieldOptions[0]!.value,
        });
        if (selection.kind === "back") {
          return structuredDraftPromptBack();
        }
        if (selection.kind !== "selected") {
          return structuredDraftPromptCancel();
        }

        preferredFieldValue = selection.value;
        const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
        if (!fieldOption) {
          return structuredDraftPromptCancel();
        }

        if (fieldOption.editor === "sharedExplorer") {
          return openPromptFieldClause(query, fieldOption, currentNode);
        }

        const nextNode = await editFieldClause(query, fieldOption, currentNode);
        return nextNode === undefined
          ? structuredDraftPromptCancel()
          : structuredDraftPromptApply(nextNode ? (metadataFilterNodeToCanonicalFilter(nextNode) ?? null) : null);
      }
    },
    [editFieldClause, getScopedFieldOptions, openPromptFieldClause, terminal],
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
      const metricFieldOptions = getScopedFieldOptions(query).filter((fieldOption) =>
        isMetricFieldOptionValue(fieldOption.value),
      );
      const entries = availableFamilies
        .map((family) => metricFieldOptions.find((fieldOption) => fieldOption.value === family))
        .filter((fieldOption): fieldOption is Pf2eTerminalQueryFieldOption => Boolean(fieldOption))
        .map((fieldOption) => ({
          value: fieldOption.value as MetricFieldFamily,
          label: fieldOption.label,
          description: fieldOption.description,
        }));
      if (entries.length === 0) {
        return structuredDraftPromptCancel();
      }

      const selection = await promptSession.promptSelectOption({
        title,
        prompt,
        entries,
        selectedValue:
          currentFamily && entries.some((entry) => entry.value === currentFamily) ? currentFamily : entries[0]!.value,
      });
      if (selection.kind === "back") {
        return structuredDraftPromptBack();
      }
      return selection.kind === "selected"
        ? structuredDraftPromptApply(selection.value)
        : structuredDraftPromptCancel();
    },
    [getScopedFieldOptions],
  );

  const promptForMetricKey = React.useCallback(
    async (
      _promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      family: MetricFieldFamily,
      title: string,
      _prompt: string,
      _currentMetric?: string,
      options: { numericOnly?: boolean } = {},
      initialDiscoveryMode: SearchFilterDiscoveryMode = "matching",
    ): Promise<PromptStepResult<MetricKeySelection>> => {
      const fieldOption = getScopedFieldOptions(query).find((candidate) => candidate.value === family);
      if (!fieldOption) {
        return structuredDraftPromptCancel();
      }

      return selectPromptMetricKey(query, family, fieldOption, title, options, initialDiscoveryMode);
    },
    [getScopedFieldOptions, selectPromptMetricKey],
  );

  const promptForMetricCompareClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "metricCompare" }>,
    ): Promise<StructuredDraftPromptFlowResult<Extract<SearchFilterNode, { kind: "metricCompare" }>>> => {
      const availableFamilies = await getAvailableMetricFamilies(query, { numericOnly: true });
      if (availableFamilies.length === 0) {
        await terminal.pauseForAnyKey("No numeric metric comparisons are available for the current query.");
        return structuredDraftPromptCancel();
      }

      const currentFamily: MetricFieldFamily | null = currentNode
        ? inferMetricFieldFamily(currentNode.leftMetric, getSearchQueryCategory(query))
        : null;
      let selectedFamily = currentFamily;
      let metricDiscoveryMode: SearchFilterDiscoveryMode = "matching";

      for (;;) {
        const family = await promptForMetricFamily(
          promptSession,
          query,
          availableFamilies,
          selectedFamily,
          "Metric comparison",
          "Choose the metric family for this comparison clause",
        );
        if (family.kind === "back") {
          return structuredDraftPromptBack();
        }
        if (family.kind === "cancel") {
          return structuredDraftPromptCancel();
        }
        selectedFamily = family.value;

        for (;;) {
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
          if (leftMetric.kind === "back") {
            break;
          }
          if (leftMetric.kind === "cancel") {
            return structuredDraftPromptCancel();
          }
          metricDiscoveryMode = leftMetric.value.discoveryMode;

          for (;;) {
            const operatorSelection = await promptSession.promptSelectOption({
              title: "Comparison Operator",
              prompt: "Choose how the left metric should compare to the right metric",
              entries: [
                { value: "eq", label: "Equals", description: "Require both metrics to be equal." },
                { value: "notEq", label: "Does Not Equal", description: "Require both metrics to differ." },
                { value: "gt", label: "Greater Than", description: "Require the left metric to be greater." },
                {
                  value: "gte",
                  label: "Greater Or Equal",
                  description: "Require the left metric to be greater or equal.",
                },
                { value: "lt", label: "Less Than", description: "Require the left metric to be less." },
                { value: "lte", label: "Less Or Equal", description: "Require the left metric to be less or equal." },
              ],
              selectedValue: currentNode?.op ?? "gte",
            });
            if (operatorSelection.kind === "back") {
              break;
            }
            if (operatorSelection.kind !== "selected") {
              return structuredDraftPromptCancel();
            }

            const operator = operatorSelection.value as MetricCompareOperator;
            const rightMetric = await promptForMetricKey(
              promptSession,
              query,
              selectedFamily,
              "Right Metric",
              "Choose the right-hand metric for this comparison clause",
              currentNode && currentFamily === selectedFamily ? currentNode.rightMetric : leftMetric.value.value,
              { numericOnly: true },
              metricDiscoveryMode,
            );
            if (rightMetric.kind === "back") {
              continue;
            }
            if (rightMetric.kind === "cancel") {
              return structuredDraftPromptCancel();
            }

            return structuredDraftPromptApply({
              kind: "metricCompare" as const,
              leftMetric: leftMetric.value.value,
              op: operator,
              rightMetric: rightMetric.value.value,
            });
          }
        }
      }
    },
    [getAvailableMetricFamilies, promptForMetricFamily, promptForMetricKey, terminal],
  );

  const promptForScopeClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
      currentNode?: Extract<SearchFilterNode, { kind: "scope" }>,
    ): Promise<StructuredDraftPromptFlowResult<Extract<SearchFilterNode, { kind: "scope" }>>> => {
      const categoryOptions = user.search
        .getCategoryOptions()
        .filter(
          (option): option is { value: SearchCategory; label: string; description: string } => option.value !== null,
        );
      if (categoryOptions.length === 0) {
        await terminal.pauseForAnyKey("No categories are available for the current query.");
        return structuredDraftPromptCancel();
      }

      for (;;) {
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
          return structuredDraftPromptBack();
        }
        if (categorySelection.kind !== "selected") {
          return structuredDraftPromptCancel();
        }

        const category = normalizeSearchCategory(categorySelection.value);
        if (!category) {
          return structuredDraftPromptCancel();
        }

        const subcategoryOptions = user.search
          .getSubcategoryOptions(category)
          .filter((option) => option.value !== null);
        const matchingCurrentNode = currentNode && currentNode.category === category ? currentNode : null;
        const currentMode =
          matchingCurrentNode?.subcategory.kind === "eq"
            ? "specific"
            : matchingCurrentNode?.subcategory.kind === "isNull"
              ? "none"
              : "any";

        for (;;) {
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
            return structuredDraftPromptCancel();
          }

          let subcategory: SearchScopeSubcategoryMatch = { kind: "any" };
          if (modeSelection.value === "none") {
            subcategory = { kind: "isNull" };
          } else if (modeSelection.value === "specific") {
            if (subcategoryOptions.length === 0) {
              await terminal.pauseForAnyKey("No subcategories are available for the selected category.");
              return structuredDraftPromptCancel();
            }
            const currentSubcategoryValue =
              matchingCurrentNode?.subcategory.kind === "eq" ? matchingCurrentNode.subcategory.value : null;
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
              return structuredDraftPromptCancel();
            }
            const normalizedSubcategory = normalizeSearchSubcategory(subcategorySelection.value) ?? null;
            if (!normalizedSubcategory) {
              return structuredDraftPromptCancel();
            }
            subcategory = { kind: "eq", value: normalizedSubcategory };
          }

          return structuredDraftPromptApply({
            kind: "scope",
            category,
            subcategory,
          });
        }
      }
    },
    [terminal, user.search],
  );

  const promptForNumericMatchClause = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      nodeKind: "level" | "price" | "actionCost",
      node?:
        | Extract<SearchFilterNode, { kind: "level" }>
        | Extract<SearchFilterNode, { kind: "price" }>
        | Extract<SearchFilterNode, { kind: "actionCost" }>,
    ): Promise<
      StructuredDraftPromptFlowResult<
        | Extract<SearchFilterNode, { kind: "level" }>
        | Extract<SearchFilterNode, { kind: "price" }>
        | Extract<SearchFilterNode, { kind: "actionCost" }>
        | null
      >
    > => {
      if (node?.kind === "actionCost" && (node.match.kind === "isNull" || node.match.kind === "isNotNull")) {
        await terminal.pauseForAnyKey("Null action-cost clauses cannot be edited through the numeric matcher.");
        return structuredDraftPromptCancel();
      }
      let currentNumericMatch: SearchNumericMatch | null = null;
      if (node?.kind === "actionCost") {
        switch (node.match.kind) {
          case "eq":
          case "gt":
          case "gte":
          case "lt":
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
          return structuredDraftPromptCancel();
        }
        if (parsed === null) {
          return structuredDraftPromptApply(null);
        }
        return structuredDraftPromptApply({
          kind: nodeKind,
          match:
            parsed.kind === "between"
              ? {
                  kind: "between",
                  min: Math.min(parsed.min, parsed.max),
                  max: Math.max(parsed.min, parsed.max),
                }
              : parsed,
        });
      }

      const parsed = await promptNumericScalarClause(promptSession, terminal, {
        title: nodeKind === "price" ? "Price Matcher" : "Action Cost Matcher",
        currentClause:
          currentNumericMatch?.kind === "between"
            ? { op: "between", min: currentNumericMatch.min, max: currentNumericMatch.max }
            : currentNumericMatch?.kind === "eq" ||
                currentNumericMatch?.kind === "gt" ||
                currentNumericMatch?.kind === "gte" ||
                currentNumericMatch?.kind === "lt" ||
                currentNumericMatch?.kind === "lte"
              ? { op: currentNumericMatch.kind, value: currentNumericMatch.value }
              : null,
      });
      if (parsed === undefined) {
        return structuredDraftPromptCancel();
      }
      if (parsed === null) {
        return structuredDraftPromptApply(null);
      }
      if (parsed.op === "neq") {
        await terminal.pauseForAnyKey(
          "`!=` is not supported for this matcher. Use an exact, minimum, maximum, or range value.",
        );
        return structuredDraftPromptCancel();
      }

      return structuredDraftPromptApply({
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
      });
    },
    [terminal],
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
          return promptForFieldClause(
            promptSession,
            query,
            "field",
            currentNode ? canonicalFilterToMetadataNode(currentNode) : null,
          );
        case "metric":
          return promptForFieldClause(
            promptSession,
            query,
            "metric",
            currentNode ? canonicalFilterToMetadataNode(currentNode) : null,
          );
        case "metricCompare":
          return promptForMetricCompareClause(
            promptSession,
            query,
            currentNode?.kind === "metricCompare" ? currentNode : undefined,
          );
        case "pack":
          return structuredDraftPromptCancel();
        case "scope":
          return promptForScopeClause(promptSession, query, currentNode?.kind === "scope" ? currentNode : undefined);
        case "level":
          return promptForNumericMatchClause(
            promptSession,
            "level",
            currentNode?.kind === "level" ? currentNode : undefined,
          );
        case "price":
          return promptForNumericMatchClause(
            promptSession,
            "price",
            currentNode?.kind === "price" ? currentNode : undefined,
          );
        case "rarity":
          return structuredDraftPromptCancel();
        case "actionCost":
          return promptForNumericMatchClause(
            promptSession,
            "actionCost",
            currentNode?.kind === "actionCost" ? currentNode : undefined,
          );
      }
    },
    [
      promptForFieldClause,
      promptForMetricCompareClause,
      promptForNumericMatchClause,
      promptForScopeClause,
    ],
  );

  const promptForClauseKind = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
    ): Promise<ClausePromptResult> => {
      const fieldOptions = getScopedFieldOptions(query);
      const hasFieldClauses = fieldOptions.some((fieldOption) => !isMetricFieldOptionValue(fieldOption.value));
      const hasMetricClauses = fieldOptions.some((fieldOption) => isMetricFieldOptionValue(fieldOption.value));
      const hasMetricCompareClauses = hasMetricClauses;
      const hasPrice = fieldOptions.some((fieldOption) => fieldOption.value === "priceCp");
      const hasActionCost =
        user.search.getActionCostOptions(getSearchQueryCategory(query), getSearchQuerySubcategory(query)).length > 0;
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
      entryByValue.set("pack", {
        value: "pack",
        label: "Pack",
        description: "Restrict results to one or more selected packs without waiting on preflight discovery checks.",
      });
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
        return structuredDraftPromptBack();
      }
      return result.kind === "selected"
        ? structuredDraftPromptApply(result.value)
        : structuredDraftPromptCancel();
    },
    [getScopedFieldOptions, user.search],
  );

  const promptForSharedExplorerFieldOption = React.useCallback(
    async (
      promptSession: SearchWorkspacePromptAdapters,
      query: Pf2eTerminalSearchQuery,
    ): Promise<SharedExplorerFieldOptionPromptResult> => {
      const fieldOptions = getScopedFieldOptions(query).filter(
        (fieldOption) => !isMetricFieldOptionValue(fieldOption.value) && isStructuredDraftGroupFieldOption(fieldOption),
      );
      if (fieldOptions.length === 0) {
        return structuredDraftPromptApply(null);
      }
      const selection = await promptSession.promptSelectOption({
        title: "Metadata",
        prompt: "Choose the metadata field for the next clause",
        entries: fieldOptions.map((fieldOption) => ({
          value: fieldOption.value,
          label: fieldOption.label,
          description: fieldOption.description,
        })),
        selectedValue: fieldOptions[0]!.value,
      });
      if (selection.kind === "back") {
        return structuredDraftPromptBack();
      }
      if (selection.kind !== "selected") {
        return structuredDraftPromptCancel();
      }
      const fieldOption = fieldOptions.find((candidate) => candidate.value === selection.value);
      return fieldOption ? structuredDraftPromptApply(fieldOption) : structuredDraftPromptCancel();
    },
    [getScopedFieldOptions],
  );

  return {
    promptForClauseKind,
    promptForClauseNode,
    promptForSharedExplorerFieldOption,
  };
}
