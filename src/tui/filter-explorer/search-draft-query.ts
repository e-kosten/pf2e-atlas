import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataSetField,
} from "../../domain/metadata-field-types.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { MetadataFilterNode, MetadataPredicate } from "../search/metadata-filter-draft.js";
import { isMetadataPredicate, normalizeMetadataNode } from "../search/query-core.js";
import { partitionDiscoverableQueryFieldSelections } from "../search/discoverable-fields.js";
import { createEmptyStringSelection, normalizeQueryFieldSelection } from "../search/selections.js";
import {
  buildMetadataNodeForQueryFieldSelection,
  getSearchQueryActionCostSelection,
  getSearchQueryMetadataTree,
  getSearchQueryRaritySelection,
  setSearchQueryActionCostSelection,
  setSearchQueryMetadataTree,
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
type SearchFilterExplorerTopLevelField = "rarity" | "actionCost";
type MetricValueType = "number" | "text" | "boolean";

function isMetricQueryField(field: Pf2eTerminalQueryField): field is SearchFilterExplorerMetricField {
  return field === "actorMetric" || field === "itemMetric";
}

function isTopLevelSearchFilterExplorerField(field: string): field is SearchFilterExplorerTopLevelField {
  return field === "rarity" || field === "actionCost";
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
    if (metricField || fieldKey === "actionCost" || fieldKey === "rarity") {
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

function extractMetricSelectionFromPredicate(
  node: MetadataPredicate,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; selection: Pf2eTerminalValueSelection<string> } | null {
  if (!("metric" in node) || !("value" in node)) {
    return null;
  }
  if (!allowedMetricFields.has(node.field) || typeof node.value === "number") {
    return null;
  }

  const selectionValue = String(node.value);
  if (node.op === "==") {
    return {
      key: getMetricSelectionKey(node.field, node.metric),
      selection: { include: [selectionValue], exclude: [] },
    };
  }

  return {
    key: getMetricSelectionKey(node.field, node.metric),
    selection: { include: [], exclude: [selectionValue] },
  };
}

function extractMetricScalarClauseFromPredicate(
  node: MetadataPredicate,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: FilterExplorerScalarClause } | null {
  if (!("metric" in node) || !("value" in node)) {
    return null;
  }
  if (!allowedMetricFields.has(node.field)) {
    return null;
  }

  const valueType = inferMetricValueType(node.field, node.metric);
  if (valueType !== "number" || typeof node.value !== "number") {
    return null;
  }

  const operator =
    node.op === "=="
      ? "eq"
      : node.op === "!="
        ? "neq"
        : node.op === ">="
          ? "gte"
          : node.op === "<="
            ? "lte"
            : null;
  if (!operator) {
    return null;
  }

  return {
    key: buildMetricClauseKey(node.field, node.metric),
    clause: {
      operator,
      value: node.value,
    },
  };
}

function tryExtractMetricBetweenClause(
  node: MetadataFilterNode,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: FilterExplorerScalarClause } | null {
  if (!("and" in node) || node.and.length !== 2) {
    return null;
  }

  const [left, right] = node.and;
  if (!left || !right || !isMetadataPredicate(left) || !isMetadataPredicate(right)) {
    return null;
  }
  if (
    !("metric" in left) ||
    !("metric" in right) ||
    !("value" in left) ||
    !("value" in right) ||
    left.field !== right.field ||
    left.metric !== right.metric ||
    !allowedMetricFields.has(left.field)
  ) {
    return null;
  }
  if (inferMetricValueType(left.field, left.metric) !== "number") {
    return null;
  }
  if (typeof left.value !== "number" || typeof right.value !== "number") {
    return null;
  }

  const lower = left.op === ">=" ? left : right.op === ">=" ? right : null;
  const upper = left.op === "<=" ? left : right.op === "<=" ? right : null;
  if (!lower || !upper) {
    return null;
  }

  return {
    key: buildMetricClauseKey(left.field, left.metric),
    clause: {
      operator: "between",
      min: lower.value,
      max: upper.value,
    },
  };
}

function extractMetricDraftEntries(
  node: MetadataFilterNode | null,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
  scalarClauses: FilterExplorerScalarClauseMap;
} {
  if (!node || allowedMetricFields.size === 0) {
    return { metadata: node, selections: {}, scalarClauses: {} };
  }

  if (isMetadataPredicate(node)) {
    const scalarClause = extractMetricScalarClauseFromPredicate(node, allowedMetricFields);
    if (scalarClause) {
      return { metadata: null, selections: {}, scalarClauses: { [scalarClause.key]: scalarClause.clause } };
    }

    const extracted = extractMetricSelectionFromPredicate(node, allowedMetricFields);
    return extracted
      ? { metadata: null, selections: { [extracted.key]: extracted.selection }, scalarClauses: {} }
      : { metadata: node, selections: {}, scalarClauses: {} };
  }

  if ("and" in node) {
    const betweenClause = tryExtractMetricBetweenClause(node, allowedMetricFields);
    if (betweenClause) {
      return { metadata: null, selections: {}, scalarClauses: { [betweenClause.key]: betweenClause.clause } };
    }

    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    let scalarClauses: FilterExplorerScalarClauseMap = {};
    const children: MetadataFilterNode[] = [];

    for (const child of node.and) {
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
      metadata: normalizeMetadataNode(children.length === 0 ? null : { and: children }),
      selections,
      scalarClauses,
    };
  }

  if ("or" in node) {
    const childSelections = node.or.map((child) => extractMetricDraftEntries(child, allowedMetricFields));
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
): MetadataFilterNode {
  return {
    field,
    metric,
    op: operator === "include" ? "==" : "!=",
    value: normalizeMetricSelectionValue(value),
  };
}

function buildMetricScalarClauseMetadataNode(key: string, clause: FilterExplorerScalarClause): MetadataFilterNode | null {
  const metricKey = parseMetricSelectionKey(key);
  if (!metricKey) {
    return null;
  }

  const valueType = inferMetricValueType(metricKey.field, metricKey.metric);
  if (!valueType) {
    return null;
  }

  if (clause.operator === "between") {
    return normalizeMetadataNode({
      and: [
        { field: metricKey.field, metric: metricKey.metric, op: ">=", value: clause.min },
        { field: metricKey.field, metric: metricKey.metric, op: "<=", value: clause.max },
      ],
    });
  }

  let op: "==" | "!=" | ">=" | "<=";
  switch (clause.operator) {
    case "eq":
      op = "==";
      break;
    case "neq":
      op = "!=";
      break;
    case "gte":
      op = ">=";
      break;
    case "lte":
      op = "<=";
      break;
  }

  if (valueType !== "number") {
    return {
      field: metricKey.field,
      metric: metricKey.metric,
      op: op === ">=" || op === "<=" ? "==" : op,
      value: clause.value as string | boolean,
    };
  }

  return {
    field: metricKey.field,
    metric: metricKey.metric,
    op,
    value: clause.value as number,
  };
}

function buildMetadataNodeForDiscreteClause(
  clause: FilterExplorerDiscreteClause,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
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
          field: clause.field as MetadataSetField,
          op: "includes",
          value: clause.value,
        } satisfies MetadataFilterNode)
      : ({
          not: {
            field: clause.field as MetadataSetField,
            op: "includes",
            value: clause.value,
          },
        } satisfies MetadataFilterNode);
  }

  if (fieldSemantics.fieldType === "enumString") {
    return clause.operator === "include"
      ? { field: clause.field as MetadataEnumStringField, op: "eq", value: clause.value }
      : { not: { field: clause.field as MetadataEnumStringField, op: "eq", value: clause.value } };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const value = clause.value === "true";
    return clause.operator === "include"
      ? { field: clause.field as MetadataBooleanField, op: "eq", value }
      : { not: { field: clause.field as MetadataBooleanField, op: "eq", value } };
  }

  return buildMetadataNodeForQueryFieldSelection(
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
): MetadataFilterNode[] {
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
    const includeValues = sortUnique(selection.include ?? []);
    for (const value of includeValues) {
      clauses.push({ field, value, operator: "include" });
    }
    for (const value of sortUnique(selection.exclude ?? [])) {
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
    if (clause.field === "rarity" || clause.field === "actionCost") {
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

export function prepareFilterExplorerDraftFromMetadataNode(
  node: MetadataFilterNode | null,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalPreparedFilterExplorerDraft {
  const regularFields = scopedFields.filter((field): field is Pf2eTerminalFacetField => !isMetricQueryField(field));
  const discoverablePartition = partitionDiscoverableQueryFieldSelections(node, regularFields, fieldSemanticsByName);
  const metricPartition = extractMetricDraftEntries(
    discoverablePartition.metadata,
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
    preservedMetadata: normalizeMetadataNode(metricPartition.metadata),
  };
}

export function prepareFilterExplorerDraftFromQuery(
  query: Pf2eTerminalSearchQuery,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalPreparedFilterExplorerDraft {
  const preparedDraft = prepareFilterExplorerDraftFromMetadataNode(
    getSearchQueryMetadataTree(query),
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

  return {
    scopedFields: [...scopedFields],
    draft: normalizeFilterExplorerComposeDraft({
      discreteClauses,
      scalarClauses: cloneScalarClauseMap(preparedDraft.draft.scalarClauses),
    }),
    preservedMetadata: preparedDraft.preservedMetadata,
  };
}

export function buildFilterExplorerMetadataNode(
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: { preservedMetadata?: MetadataFilterNode | null } = {},
): MetadataFilterNode | null {
  const metadataClauses: MetadataFilterNode[] = [];

  if (options.preservedMetadata) {
    metadataClauses.push(options.preservedMetadata);
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
  return normalizeMetadataNode({ and: metadataClauses });
}

export function buildFilterExplorerInsertionResult(
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  options: { preservedMetadata?: MetadataFilterNode | null; preferReplace?: boolean } = {},
): Pf2eTerminalFilterExplorerInsertionResult {
  if (options.preservedMetadata || options.preferReplace) {
    return {
      kind: "replace",
      node: buildFilterExplorerMetadataNode(draft, fieldSemanticsByName, {
        preservedMetadata: options.preservedMetadata,
      }),
    };
  }

  const insertionNodes: MetadataFilterNode[] = [];

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
    preservedMetadata?: MetadataFilterNode | null;
    scopedFields?: readonly Pf2eTerminalQueryField[];
  } = {},
): Pf2eTerminalSearchQuery {
  const scopedFields = options.scopedFields ? [...options.scopedFields] : inferScopedFieldsFromDraft(draft);
  const scopedFieldSet = new Set(scopedFields);
  const preservedMetadata =
    options.preservedMetadata ??
    (scopedFields.length > 0
      ? prepareFilterExplorerDraftFromMetadataNode(getSearchQueryMetadataTree(query), scopedFields, fieldSemanticsByName)
          .preservedMetadata
      : getSearchQueryMetadataTree(query));

  let nextQuery = setSearchQueryMetadataTree(
    query,
    buildFilterExplorerMetadataNode(draft, fieldSemanticsByName, { preservedMetadata }),
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

  return nextQuery;
}
