import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataSetField,
} from "../../domain/metadata-field-types.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type { SearchFilterNode } from "../../domain/search-request-types.js";
import type { MetricOperator } from "../../domain/search-filter-operators.js";
import { normalizeSearchFilterNode } from "../search/query-core.js";
import { partitionDiscoverableQueryFieldSelections } from "../search/discoverable-fields.js";
import { createEmptyStringSelection, normalizeQueryFieldSelection } from "../search/selections.js";
import {
  buildSearchFilterNodeForQueryFieldSelection,
  getSearchQueryActionCostSelection,
  getSearchQueryPredicateFilter,
  getSearchQueryPackSelection,
  getSearchQueryRaritySelection,
  setSearchQueryPackSelection,
  setSearchQueryActionCostSelection,
  setSearchQueryPredicateFilter,
  setSearchQueryRaritySelection,
} from "../search/query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalPreparedFilterExplorerDraft,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalValueSelection,
} from "../search/service-types.js";
import type {
  FilterExplorerDiscreteClause,
  FilterExplorerScalarClause,
  FilterExplorerScalarClauseMap,
} from "./types.js";
import { normalizeFilterExplorerComposeDraft } from "./compose-state.js";

type SearchFilterExplorerMetricField = "actorMetric" | "itemMetric";
type SearchFilterExplorerTopLevelField = "rarity" | "actionCost" | "pack";
type MetricValueType = "number" | "text" | "boolean";

function isMetricQueryField(field: Pf2eTerminalQueryField): field is SearchFilterExplorerMetricField {
  return field === "actorMetric" || field === "itemMetric";
}

function isTopLevelSearchFilterExplorerField(field: string): field is SearchFilterExplorerTopLevelField {
  return field === "rarity" || field === "actionCost" || field === "pack";
}

function isBooleanString(value: string): boolean {
  return value === "true" || value === "false";
}

function normalizeMetricSelectionValue(value: string): string | boolean {
  return isBooleanString(value) ? value === "true" : value;
}

function sortUnique(values: readonly string[]): string[] {
  return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function cloneSelection(selection: Pf2eTerminalValueSelection<string> | undefined): Pf2eTerminalValueSelection<string> {
  return {
    include: [...(selection?.include ?? [])],
    exclude: [...(selection?.exclude ?? [])],
  };
}

function normalizeMetricSelection(
  selection: Pf2eTerminalValueSelection<string> | undefined,
): Pf2eTerminalValueSelection<string> {
  const exclude = sortUnique(selection?.exclude ?? []);
  const include = sortUnique(selection?.include ?? []).filter((value) => !exclude.includes(value));
  return { include, exclude };
}

function normalizeFilterExplorerSelectionMap(
  fieldSelections: Pf2eTerminalQueryFieldSelectionMap,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = {};

  for (const [fieldKey, selection] of Object.entries(fieldSelections)) {
    const metricField = parseMetricSelectionKey(fieldKey);
    if (metricField || fieldKey === "actionCost" || fieldKey === "rarity" || fieldKey === "pack") {
      next[fieldKey] = normalizeMetricSelection(selection);
      continue;
    }

    const normalizedSelection = normalizeQueryFieldSelection(
      fieldKey as Pf2eTerminalFacetField,
      selection,
      fieldSemanticsByName,
    );
    next[fieldKey] = normalizedSelection ?? createEmptyStringSelection();
  }

  return next;
}

function getMetricSelectionKey(field: SearchFilterExplorerMetricField, metric: string): string {
  return `${field}:${metric}`;
}

function parseMetricSelectionKey(key: string): { field: SearchFilterExplorerMetricField; metric: string } | null {
  if (key.startsWith("actorMetric:")) {
    return { field: "actorMetric", metric: key.slice("actorMetric:".length) };
  }
  if (key.startsWith("itemMetric:")) {
    return { field: "itemMetric", metric: key.slice("itemMetric:".length) };
  }
  return null;
}

function inferMetricValueType(field: SearchFilterExplorerMetricField, metric: string): MetricValueType | null {
  return field === "actorMetric" ? inferActorMetricValueType(metric) : inferItemMetricValueType(metric);
}

function cloneScalarClause(clause: FilterExplorerScalarClause): FilterExplorerScalarClause {
  return clause.operator === "between" ? { ...clause } : { ...clause };
}

function cloneScalarClauseMap(scalarClauses: FilterExplorerScalarClauseMap): FilterExplorerScalarClauseMap {
  return Object.fromEntries(Object.entries(scalarClauses).map(([key, clause]) => [key, cloneScalarClause(clause)]));
}

function mergeScalarClauseMaps(
  left: FilterExplorerScalarClauseMap,
  right: FilterExplorerScalarClauseMap,
): FilterExplorerScalarClauseMap | null {
  const next = cloneScalarClauseMap(left);

  for (const [key, clause] of Object.entries(right)) {
    if (key in next) {
      return null;
    }
    next[key] = cloneScalarClause(clause);
  }

  return next;
}

function mergeSelectionMaps(
  left: Pf2eTerminalQueryFieldSelectionMap,
  right: Pf2eTerminalQueryFieldSelectionMap,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = {
    ...Object.fromEntries(Object.entries(left).map(([field, selection]) => [field, cloneSelection(selection)])),
  };

  for (const [field, selection] of Object.entries(right)) {
    if (!(field in next)) {
      next[field] = cloneSelection(selection);
      continue;
    }
    next[field] = {
      include: [...next[field]!.include, ...selection.include],
      exclude: [...next[field]!.exclude, ...selection.exclude],
    };
  }

  return next;
}

function buildMetricClauseKey(field: SearchFilterExplorerMetricField, metric: string): string {
  return getMetricSelectionKey(field, metric);
}

function inferMetricNodeField(
  metric: string,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): SearchFilterExplorerMetricField | null {
  if (allowedMetricFields.has("actorMetric") && inferActorMetricValueType(metric)) {
    return "actorMetric";
  }
  if (allowedMetricFields.has("itemMetric") && inferItemMetricValueType(metric)) {
    return "itemMetric";
  }
  return allowedMetricFields.size === 1 ? [...allowedMetricFields][0]! : null;
}

function canonicalMetricOperatorToScalarOperator(
  op: MetricOperator,
): Exclude<FilterExplorerScalarClause["operator"], "between"> {
  switch (op) {
    case "eq":
      return "eq";
    case "notEq":
      return "neq";
    case "gt":
      return "gt";
    case "gte":
      return "gte";
    case "lt":
      return "lt";
    case "lte":
      return "lte";
  }
}

function extractMetricSelectionFromPredicate(
  node: Extract<SearchFilterNode, { kind: "metric" }>,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; selection: Pf2eTerminalValueSelection<string> } | null {
  const field = inferMetricNodeField(node.metric, allowedMetricFields);
  if (!field || typeof node.value === "number") {
    return null;
  }

  const selectionValue = String(node.value);
  if (node.op === "eq") {
    return {
      key: getMetricSelectionKey(field, node.metric),
      selection: { include: [selectionValue], exclude: [] },
    };
  }

  if (node.op !== "notEq") {
    return null;
  }

  return {
    key: getMetricSelectionKey(field, node.metric),
    selection: { include: [], exclude: [selectionValue] },
  };
}

function extractMetricScalarClauseFromPredicate(
  node: Extract<SearchFilterNode, { kind: "metric" }>,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: FilterExplorerScalarClause } | null {
  const field = inferMetricNodeField(node.metric, allowedMetricFields);
  if (!field) {
    return null;
  }

  const valueType = inferMetricValueType(field, node.metric);
  if (valueType !== "number" || typeof node.value !== "number") {
    return null;
  }

  return {
    key: buildMetricClauseKey(field, node.metric),
    clause: {
      operator: canonicalMetricOperatorToScalarOperator(node.op),
      value: node.value,
    },
  };
}

function tryExtractMetricBetweenClause(
  node: SearchFilterNode,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: FilterExplorerScalarClause } | null {
  if (node.kind !== "allOf" || node.children.length !== 2) {
    return null;
  }

  const [left, right] = node.children;
  if (!left || !right || left.kind !== "metric" || right.kind !== "metric") {
    return null;
  }
  const field = inferMetricNodeField(left.metric, allowedMetricFields);
  if (!field || left.metric !== right.metric) {
    return null;
  }
  if (inferMetricValueType(field, left.metric) !== "number") {
    return null;
  }
  if (typeof left.value !== "number" || typeof right.value !== "number") {
    return null;
  }

  const lower = left.op === "gte" ? left : right.op === "gte" ? right : null;
  const upper = left.op === "lte" ? left : right.op === "lte" ? right : null;
  if (!lower || !upper) {
    return null;
  }

  return {
    key: buildMetricClauseKey(field, left.metric),
    clause: {
      operator: "between",
      min: lower.value as number,
      max: upper.value as number,
    },
  };
}

function extractMetricDraftEntries(
  node: SearchFilterNode | null,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): {
  metadata: SearchFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
  scalarClauses: FilterExplorerScalarClauseMap;
} {
  if (!node || allowedMetricFields.size === 0) {
    return { metadata: node, selections: {}, scalarClauses: {} };
  }

  if (node.kind === "metric") {
    const scalarClause = extractMetricScalarClauseFromPredicate(node, allowedMetricFields);
    if (scalarClause) {
      return { metadata: null, selections: {}, scalarClauses: { [scalarClause.key]: scalarClause.clause } };
    }

    const extracted = extractMetricSelectionFromPredicate(node, allowedMetricFields);
    return extracted
      ? { metadata: null, selections: { [extracted.key]: extracted.selection }, scalarClauses: {} }
      : { metadata: node, selections: {}, scalarClauses: {} };
  }

  if (node.kind === "allOf") {
    const betweenClause = tryExtractMetricBetweenClause(node, allowedMetricFields);
    if (betweenClause) {
      return { metadata: null, selections: {}, scalarClauses: { [betweenClause.key]: betweenClause.clause } };
    }

    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    let scalarClauses: FilterExplorerScalarClauseMap = {};
    const children: SearchFilterNode[] = [];

    for (const child of node.children) {
      const extracted = extractMetricDraftEntries(child, allowedMetricFields);
      const mergedScalarClauses = mergeScalarClauseMaps(scalarClauses, extracted.scalarClauses);
      if (!mergedScalarClauses) {
        return { metadata: node, selections: {}, scalarClauses: {} };
      }
      selections = mergeSelectionMaps(selections, extracted.selections);
      scalarClauses = mergedScalarClauses;
      if (extracted.metadata) {
        children.push(extracted.metadata);
      }
    }

    return {
      metadata: normalizeSearchFilterNode(children.length === 0 ? null : { kind: "allOf", children }) ?? null,
      selections,
      scalarClauses,
    };
  }

  if (node.kind === "anyOf") {
    const childSelections = node.children.map((child) => extractMetricDraftEntries(child, allowedMetricFields));
    const allExtracted = childSelections.every(
      (entry) => entry.metadata === null && Object.keys(entry.scalarClauses).length === 0,
    );
    if (!allExtracted) {
      return { metadata: node, selections: {}, scalarClauses: {} };
    }

    const merged = childSelections.reduce<Pf2eTerminalQueryFieldSelectionMap>(
      (selectionMap, entry) => mergeSelectionMaps(selectionMap, entry.selections),
      {},
    );
    const includeOnly = Object.values(merged).every((selection) => selection.exclude.length === 0);
    return includeOnly ? { metadata: null, selections: merged, scalarClauses: {} } : { metadata: node, selections: {}, scalarClauses: {} };
  }

  return { metadata: node, selections: {}, scalarClauses: {} };
}

function buildMetricSelectionMetadataNode(
  field: SearchFilterExplorerMetricField,
  metric: string,
  operator: FilterExplorerDiscreteClause["operator"],
  value: string,
): SearchFilterNode {
  return {
    kind: "metric",
    metric,
    op: operator === "include" ? "eq" : "notEq",
    value: normalizeMetricSelectionValue(value),
  };
}

function buildMetricScalarClauseMetadataNode(key: string, clause: FilterExplorerScalarClause): SearchFilterNode | null {
  const metricKey = parseMetricSelectionKey(key);
  if (!metricKey) {
    return null;
  }

  const valueType = inferMetricValueType(metricKey.field, metricKey.metric);
  if (!valueType) {
    return null;
  }

  if (clause.operator === "between") {
    return normalizeSearchFilterNode({
      kind: "allOf",
      children: [
        { kind: "metric", metric: metricKey.metric, op: "gte", value: clause.min },
        { kind: "metric", metric: metricKey.metric, op: "lte", value: clause.max },
      ],
    }) ?? null;
  }

  let op: MetricOperator;
  switch (clause.operator) {
    case "eq":
      op = "eq";
      break;
    case "neq":
      op = "notEq";
      break;
    case "gt":
      op = "gt";
      break;
    case "gte":
      op = "gte";
      break;
    case "lt":
      op = "lt";
      break;
    case "lte":
      op = "lte";
      break;
  }

  if (valueType !== "number") {
    return {
      kind: "metric",
      metric: metricKey.metric,
      op: op === "gt" || op === "gte" || op === "lt" || op === "lte" ? "eq" : op,
      value: clause.value as string | boolean,
    };
  }

  return {
    kind: "metric",
    metric: metricKey.metric,
    op,
    value: clause.value as number,
  };
}

function buildMetadataNodeForDiscreteClause(
  clause: FilterExplorerDiscreteClause,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): SearchFilterNode | null {
  if (isTopLevelSearchFilterExplorerField(clause.field)) {
    return null;
  }

  const metricKey = parseMetricSelectionKey(clause.field);
  if (metricKey) {
    return buildMetricSelectionMetadataNode(metricKey.field, metricKey.metric, clause.operator, clause.value);
  }

  const fieldSemantics = fieldSemanticsByName.get(clause.field as Pf2eTerminalFacetField);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    return clause.operator === "include"
      ? ({
          kind: "metadataPredicate",
          predicate: {
            field: clause.field as MetadataSetField,
            op: "includes",
            value: clause.value,
          },
        } satisfies SearchFilterNode)
      : ({
          kind: "not",
          child: {
            kind: "metadataPredicate",
            predicate: {
              field: clause.field as MetadataSetField,
              op: "includes",
              value: clause.value,
            },
          },
        } satisfies SearchFilterNode);
  }

  if (fieldSemantics.fieldType === "enumString") {
    return clause.operator === "include"
      ? {
          kind: "metadataPredicate",
          predicate: { field: clause.field as MetadataEnumStringField, op: "eq", value: clause.value },
        }
      : {
          kind: "not",
          child: {
            kind: "metadataPredicate",
            predicate: { field: clause.field as MetadataEnumStringField, op: "eq", value: clause.value },
          },
        };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const value = clause.value === "true";
    return clause.operator === "include"
      ? {
          kind: "metadataPredicate",
          predicate: { field: clause.field as MetadataBooleanField, op: "eq", value },
        }
      : {
          kind: "not",
          child: {
            kind: "metadataPredicate",
            predicate: { field: clause.field as MetadataBooleanField, op: "eq", value },
          },
        };
  }

  return buildSearchFilterNodeForQueryFieldSelection(
    clause.field as Pf2eTerminalFacetField,
    clause.operator === "include"
      ? { include: [clause.value], exclude: [] }
      : { include: [], exclude: [clause.value] },
    fieldSemanticsByName,
  );
}

function buildInsertionMetadataNodesForDiscreteClause(
  clause: FilterExplorerDiscreteClause,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): SearchFilterNode[] {
  const node = buildMetadataNodeForDiscreteClause(clause, fieldSemanticsByName);
  return node ? [node] : [];
}

function buildDiscreteClausesFromSelectionMap(
  selectionMap: Pf2eTerminalQueryFieldSelectionMap,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): FilterExplorerDiscreteClause[] {
  const normalizedSelections = normalizeFilterExplorerSelectionMap(selectionMap, fieldSemanticsByName);
  const clauses: FilterExplorerDiscreteClause[] = [];

  for (const [field, selection] of Object.entries(normalizedSelections)) {
    const includeValues = sortUnique(selection.include);
    for (const value of includeValues) {
      clauses.push({ field, value, operator: "include" });
    }
    for (const value of sortUnique(selection.exclude)) {
      clauses.push({ field, value, operator: "exclude" });
    }
  }

  return clauses;
}

function buildDiscreteClausesFromTopLevelSelection(
  field: SearchFilterExplorerTopLevelField,
  selection: Pf2eTerminalValueSelection<string>,
): FilterExplorerDiscreteClause[] {
  const normalized = normalizeMetricSelection(selection);
  return [
    ...normalized.include.map((value) => ({ field, value, operator: "include" as const })),
    ...normalized.exclude.map((value) => ({ field, value, operator: "exclude" as const })),
  ];
}

function inferScopedFieldsFromDraft(draft: Pf2eTerminalFilterExplorerDraft): Pf2eTerminalQueryField[] {
  const scopedFields = new Set<Pf2eTerminalQueryField>();

  for (const clause of draft.discreteClauses) {
    if (clause.field === "rarity" || clause.field === "actionCost" || clause.field === "pack") {
      scopedFields.add(clause.field);
      continue;
    }

    const metricKey = parseMetricSelectionKey(clause.field);
    if (metricKey) {
      scopedFields.add(metricKey.field);
      continue;
    }

    scopedFields.add(clause.field as Pf2eTerminalQueryField);
  }

  for (const key of Object.keys(draft.scalarClauses)) {
    const metricKey = parseMetricSelectionKey(key);
    if (metricKey) {
      scopedFields.add(metricKey.field);
    }
  }

  return [...scopedFields];
}

function buildRaritySelectionFromDraft(draft: Pf2eTerminalFilterExplorerDraft): Pf2eTerminalValueSelection<string> | null {
  const clauses = draft.discreteClauses.filter((clause) => clause.field === "rarity");
  if (clauses.length === 0) {
    return null;
  }

  return {
    include: clauses.filter((clause) => clause.operator === "include").map((clause) => clause.value),
    exclude: clauses.filter((clause) => clause.operator === "exclude").map((clause) => clause.value),
  };
}

function buildActionCostSelectionFromDraft(
  draft: Pf2eTerminalFilterExplorerDraft,
): Pf2eTerminalValueSelection<number> | null {
  const clauses = draft.discreteClauses.filter((clause) => clause.field === "actionCost");
  if (clauses.length === 0) {
    return null;
  }

  const parseValues = (operator: FilterExplorerDiscreteClause["operator"]) =>
    clauses
      .filter((clause) => clause.operator === operator)
      .map((clause) => Number.parseInt(clause.value, 10))
      .filter((value, index, values) => Number.isFinite(value) && values.indexOf(value) === index);

  return {
    include: parseValues("include"),
    exclude: parseValues("exclude"),
  };
}

function buildPackSelectionFromDraft(
  draft: Pf2eTerminalFilterExplorerDraft,
): Pf2eTerminalValueSelection<string> | null {
  const clauses = draft.discreteClauses.filter((clause) => clause.field === "pack");
  if (clauses.length === 0) {
    return null;
  }

  return {
    include: clauses.filter((clause) => clause.operator === "include").map((clause) => clause.value),
    exclude: clauses.filter((clause) => clause.operator === "exclude").map((clause) => clause.value),
  };
}

export function prepareFilterExplorerDraftFromFilter(
  node: SearchFilterNode | null,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalPreparedFilterExplorerDraft {
  const regularFields = scopedFields.filter((field): field is Pf2eTerminalFacetField => !isMetricQueryField(field));
  const discoverablePartition = partitionDiscoverableQueryFieldSelections(node, regularFields, fieldSemanticsByName);
  const metricPartition = extractMetricDraftEntries(
    discoverablePartition.filter,
    new Set(scopedFields.filter(isMetricQueryField)),
  );

  return {
    scopedFields: [...scopedFields],
    draft: normalizeFilterExplorerComposeDraft({
      discreteClauses: buildDiscreteClausesFromSelectionMap(
        mergeSelectionMaps(discoverablePartition.selections, metricPartition.selections),
        fieldSemanticsByName,
      ),
      scalarClauses: cloneScalarClauseMap(metricPartition.scalarClauses),
    }),
    preservedFilter: normalizeSearchFilterNode(metricPartition.metadata) ?? null,
  };
}

export function prepareFilterExplorerDraftFromQuery(
  query: Pf2eTerminalSearchQuery,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalPreparedFilterExplorerDraft {
  const preparedDraft = prepareFilterExplorerDraftFromFilter(
    getSearchQueryPredicateFilter(query),
    scopedFields,
    fieldSemanticsByName,
  );

  const discreteClauses = [...preparedDraft.draft.discreteClauses];

  if (scopedFields.includes("rarity")) {
    discreteClauses.push(
      ...buildDiscreteClausesFromTopLevelSelection("rarity", cloneSelection(getSearchQueryRaritySelection(query))),
    );
  }

  if (scopedFields.includes("actionCost")) {
    const actionCostSelection = getSearchQueryActionCostSelection(query);
    discreteClauses.push(
      ...buildDiscreteClausesFromTopLevelSelection("actionCost", {
        include: actionCostSelection.include.map(String),
        exclude: actionCostSelection.exclude.map(String),
      }),
    );
  }

  if (scopedFields.includes("pack")) {
    discreteClauses.push(
      ...buildDiscreteClausesFromTopLevelSelection("pack", cloneSelection(getSearchQueryPackSelection(query))),
    );
  }

  return {
    scopedFields: [...scopedFields],
    draft: normalizeFilterExplorerComposeDraft({
      discreteClauses,
      scalarClauses: cloneScalarClauseMap(preparedDraft.draft.scalarClauses),
    }),
    preservedFilter: preparedDraft.preservedFilter,
  };
}

export function buildFilterExplorerFilter(
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: { preservedFilter?: SearchFilterNode | null } = {},
): SearchFilterNode | null {
  const metadataClauses: SearchFilterNode[] = [];

  if (options.preservedFilter) {
    metadataClauses.push(options.preservedFilter);
  }

  for (const clause of draft.discreteClauses) {
    const metadataNode = buildMetadataNodeForDiscreteClause(clause, fieldSemanticsByName);
    if (metadataNode) {
      metadataClauses.push(metadataNode);
    }
  }

  for (const [key, clause] of Object.entries(draft.scalarClauses)) {
    const metadataNode = buildMetricScalarClauseMetadataNode(key, clause);
    if (metadataNode) {
      metadataClauses.push(metadataNode);
    }
  }

  if (metadataClauses.length === 0) {
    return null;
  }
  if (metadataClauses.length === 1) {
    return metadataClauses[0]!;
  }
  return normalizeSearchFilterNode({ kind: "allOf", children: metadataClauses }) ?? null;
}

export function buildFilterExplorerInsertionResult(
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: { preservedFilter?: SearchFilterNode | null; preferReplace?: boolean } = {},
): Pf2eTerminalFilterExplorerInsertionResult {
  if (options.preservedFilter || options.preferReplace) {
    return {
      kind: "replace",
      node: buildFilterExplorerFilter(draft, fieldSemanticsByName, {
        preservedFilter: options.preservedFilter,
      }),
    };
  }

  const insertionNodes: SearchFilterNode[] = [];

  for (const clause of draft.discreteClauses) {
    insertionNodes.push(...buildInsertionMetadataNodesForDiscreteClause(clause, fieldSemanticsByName));
  }

  for (const [key, clause] of Object.entries(draft.scalarClauses)) {
    const metadataNode = buildMetricScalarClauseMetadataNode(key, clause);
    if (metadataNode) {
      insertionNodes.push(metadataNode);
    }
  }

  return insertionNodes.length > 0 ? { kind: "insert", nodes: insertionNodes } : { kind: "replace", node: null };
}

export function applyFilterExplorerDraft(
  query: Pf2eTerminalSearchQuery,
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: {
    preservedFilter?: SearchFilterNode | null;
    scopedFields?: readonly Pf2eTerminalQueryField[];
  } = {},
): Pf2eTerminalSearchQuery {
  const scopedFields = options.scopedFields ? [...options.scopedFields] : inferScopedFieldsFromDraft(draft);
  const scopedFieldSet = new Set(scopedFields);
  const preservedFilter =
    options.preservedFilter ??
    (scopedFields.length > 0
      ? prepareFilterExplorerDraftFromFilter(getSearchQueryPredicateFilter(query), scopedFields, fieldSemanticsByName)
          .preservedFilter
      : getSearchQueryPredicateFilter(query));

  let nextQuery = setSearchQueryPredicateFilter(
    query,
    buildFilterExplorerFilter(draft, fieldSemanticsByName, { preservedFilter }),
  );

  if (scopedFieldSet.has("rarity")) {
    nextQuery = setSearchQueryRaritySelection(nextQuery, buildRaritySelectionFromDraft(draft) ?? { include: [], exclude: [] });
  }

  if (scopedFieldSet.has("actionCost")) {
    nextQuery = setSearchQueryActionCostSelection(
      nextQuery,
      buildActionCostSelectionFromDraft(draft) ?? { include: [], exclude: [] },
    );
  }

  if (scopedFieldSet.has("pack")) {
    nextQuery = setSearchQueryPackSelection(nextQuery, buildPackSelectionFromDraft(draft) ?? { include: [], exclude: [] });
  }

  return nextQuery;
}
