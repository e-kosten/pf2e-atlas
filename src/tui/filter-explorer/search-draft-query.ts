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
import { createEmptyStringPolicy, normalizeQueryFieldPolicy } from "../search/policies.js";
import {
  buildMetadataNodeForQueryFieldSelection,
  getSearchQueryActionCostPolicy,
  getSearchQueryMetadataTree,
  getSearchQueryRarityPolicy,
  setSearchQueryActionCostPolicy,
  setSearchQueryMetadataTree,
  setSearchQueryRarityPolicy,
} from "../search/query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterExplorerInsertionResult,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalPreparedFilterExplorerDraft,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "../search/service-types.js";
import type { FilterExplorerScalarClause, FilterExplorerScalarClauseMap } from "./types.js";

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

function clonePolicy(policy: Pf2eTerminalFilterValuePolicy<string> | undefined): Pf2eTerminalFilterValuePolicy<string> {
  return {
    any: [...(policy?.any ?? [])],
    all: [...(policy?.all ?? [])],
    exclude: [...(policy?.exclude ?? [])],
  };
}

function normalizeMetricPolicy(
  policy: Pf2eTerminalFilterValuePolicy<string> | undefined,
): Pf2eTerminalFilterValuePolicy<string> {
  const exclude = sortUnique(policy?.exclude ?? []);
  const all = sortUnique(policy?.all ?? []).filter((value) => !exclude.includes(value));
  const any = sortUnique(policy?.any ?? []).filter((value) => !exclude.includes(value) && !all.includes(value));
  return { any, all, exclude };
}

function normalizeFilterExplorerSelectionMap(
  fieldSelections: Pf2eTerminalQueryFieldSelectionMap,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalQueryFieldSelectionMap {
  const next: Pf2eTerminalQueryFieldSelectionMap = {};

  for (const [fieldKey, policy] of Object.entries(fieldSelections)) {
    const metricField = parseMetricSelectionKey(fieldKey);
    if (metricField) {
      next[fieldKey] = normalizeMetricPolicy(policy);
      continue;
    }

    if (fieldKey === "actionCost") {
      next[fieldKey] = normalizeMetricPolicy(policy);
      continue;
    }

    const normalizedPolicy = normalizeQueryFieldPolicy(fieldKey as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
    next[fieldKey] = normalizedPolicy ?? createEmptyStringPolicy();
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

function cloneScalarClauseMap(
  scalarClauses: FilterExplorerScalarClauseMap,
): FilterExplorerScalarClauseMap {
  return Object.fromEntries(
    Object.entries(scalarClauses).map(([key, clause]) => [key, cloneScalarClause(clause)]),
  );
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
    ...Object.fromEntries(Object.entries(left).map(([field, policy]) => [field, clonePolicy(policy)])),
  };

  for (const [field, policy] of Object.entries(right)) {
    if (!(field in next)) {
      next[field] = clonePolicy(policy);
      continue;
    }
    next[field] = {
      any: [...next[field]!.any, ...policy.any],
      all: [...next[field]!.all, ...policy.all],
      exclude: [...next[field]!.exclude, ...policy.exclude],
    };
  }

  return next;
}

function buildMetricClauseKey(field: SearchFilterExplorerMetricField, metric: string): string {
  return getMetricSelectionKey(field, metric);
}

function extractMetricPolicyFromPredicate(
  node: MetadataPredicate,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  if (!("metric" in node) || !("value" in node)) {
    return null;
  }
  if (!allowedMetricFields.has(node.field)) {
    return null;
  }
  if (typeof node.value === "number") {
    return null;
  }

  const selectionValue = String(node.value);
  if (node.op === "==") {
    return {
      key: getMetricSelectionKey(node.field, node.metric),
      policy: { any: [selectionValue], all: [], exclude: [] },
    };
  }

  return {
    key: getMetricSelectionKey(node.field, node.metric),
    policy: { any: [], all: [], exclude: [selectionValue] },
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
  if (!left || !right) {
    return null;
  }
  if (!isMetadataPredicate(left) || !isMetadataPredicate(right)) {
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
    return {
      metadata: node,
      selections: {},
      scalarClauses: {},
    };
  }

  if (isMetadataPredicate(node)) {
    const scalarClause = extractMetricScalarClauseFromPredicate(node, allowedMetricFields);
    if (scalarClause) {
      return {
        metadata: null,
        selections: {},
        scalarClauses: { [scalarClause.key]: scalarClause.clause },
      };
    }

    const extracted = extractMetricPolicyFromPredicate(node, allowedMetricFields);
    return extracted
      ? {
          metadata: null,
          selections: { [extracted.key]: extracted.policy },
          scalarClauses: {},
        }
      : {
          metadata: node,
          selections: {},
          scalarClauses: {},
        };
  }

  if ("and" in node) {
    const betweenClause = tryExtractMetricBetweenClause(node, allowedMetricFields);
    if (betweenClause) {
      return {
        metadata: null,
        selections: {},
        scalarClauses: { [betweenClause.key]: betweenClause.clause },
      };
    }

    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    let scalarClauses: FilterExplorerScalarClauseMap = {};
    const children: MetadataFilterNode[] = [];

    for (const child of node.and) {
      const extracted = extractMetricDraftEntries(child, allowedMetricFields);
      const mergedScalarClauses = mergeScalarClauseMaps(scalarClauses, extracted.scalarClauses);
      if (!mergedScalarClauses) {
        return {
          metadata: node,
          selections: {},
          scalarClauses: {},
        };
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
      return {
        metadata: node,
        selections: {},
        scalarClauses: {},
      };
    }

    const merged = childSelections.reduce<Pf2eTerminalQueryFieldSelectionMap>(
      (selectionMap, entry) => mergeSelectionMaps(selectionMap, entry.selections),
      {},
    );
    const anyOnly = Object.values(merged).every((policy) => policy.all.length === 0 && policy.exclude.length === 0);
    return anyOnly
      ? {
          metadata: null,
          selections: merged,
          scalarClauses: {},
        }
      : {
          metadata: node,
          selections: {},
          scalarClauses: {},
        };
  }

  return {
    metadata: node,
    selections: {},
    scalarClauses: {},
  };
}

function buildMetricSelectionMetadataNode(
  field: SearchFilterExplorerMetricField,
  metric: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
): MetadataFilterNode | null {
  const normalized = normalizeMetricPolicy(policy);
  const clauses: MetadataFilterNode[] = [];

  if (normalized.any.length === 1) {
    clauses.push({
      field,
      metric,
      op: "==",
      value: normalizeMetricSelectionValue(normalized.any[0]!),
    });
  } else if (normalized.any.length > 1) {
    clauses.push({
      or: normalized.any.map((value) => ({
        field,
        metric,
        op: "==",
        value: normalizeMetricSelectionValue(value),
      })),
    });
  }

  if (normalized.all.length > 0) {
    clauses.push({
      and: normalized.all.map((value) => ({
        field,
        metric,
        op: "==",
        value: normalizeMetricSelectionValue(value),
      })),
    });
  }

  if (normalized.exclude.length > 0) {
    clauses.push({
      and: normalized.exclude.map((value) => ({
        field,
        metric,
        op: "!=",
        value: normalizeMetricSelectionValue(value),
      })),
    });
  }

  if (clauses.length === 0) {
    return null;
  }
  if (clauses.length === 1) {
    return clauses[0]!;
  }
  return normalizeMetadataNode({ and: clauses });
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
        {
          field: metricKey.field,
          metric: metricKey.metric,
          op: ">=",
          value: clause.min,
        },
        {
          field: metricKey.field,
          metric: metricKey.metric,
          op: "<=",
          value: clause.max,
        },
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
    if (metricKey.field === "actorMetric") {
      return {
        field: "actorMetric",
        metric: metricKey.metric,
        op: op === ">=" || op === "<=" ? "==" : op,
        value: clause.value as string | boolean,
      };
    }

    return {
      field: "itemMetric",
      metric: metricKey.metric,
      op: op === ">=" || op === "<=" ? "==" : op,
      value: clause.value as string | boolean,
    };
  }

  if (metricKey.field === "actorMetric") {
    return {
      field: "actorMetric",
      metric: metricKey.metric,
      op,
      value: clause.value as number,
    };
  }

  return {
    field: "itemMetric",
    metric: metricKey.metric,
    op,
    value: clause.value as number,
  };
}

function buildMetadataNodeForSelectionKey(
  key: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  if (isTopLevelSearchFilterExplorerField(key)) {
    return null;
  }

  const metricKey = parseMetricSelectionKey(key);
  if (metricKey) {
    return buildMetricSelectionMetadataNode(metricKey.field, metricKey.metric, policy);
  }
  return buildMetadataNodeForQueryFieldSelection(key as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
}

function buildInsertionMetadataNodesForSelectionKey(
  key: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode[] {
  if (isTopLevelSearchFilterExplorerField(key)) {
    return [];
  }

  const metricKey = parseMetricSelectionKey(key);
  if (metricKey) {
    const normalized = normalizeMetricPolicy(policy);
    return [
      ...normalized.any.map(
        (value) =>
          ({
            field: metricKey.field,
            metric: metricKey.metric,
            op: "==",
            value: normalizeMetricSelectionValue(value),
          }) satisfies MetadataFilterNode,
      ),
      ...normalized.all.map(
        (value) =>
          ({
            field: metricKey.field,
            metric: metricKey.metric,
            op: "==",
            value: normalizeMetricSelectionValue(value),
          }) satisfies MetadataFilterNode,
      ),
      ...normalized.exclude.map(
        (value) =>
          ({
            field: metricKey.field,
            metric: metricKey.metric,
            op: "!=",
            value: normalizeMetricSelectionValue(value),
          }) satisfies MetadataFilterNode,
      ),
    ];
  }

  const fieldSemantics = fieldSemanticsByName.get(key as Pf2eTerminalFacetField);
  const normalizedPolicy = normalizeQueryFieldPolicy(key as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
  if (!fieldSemantics || !normalizedPolicy) {
    return [];
  }

  if (fieldSemantics.fieldType === "set") {
    const field = key as MetadataSetField;
    return [
      ...normalizedPolicy.any.map(
        (value) =>
          ({
            field,
            op: "includesAny",
            values: [value],
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.all.map(
        (value) =>
          ({
            field,
            op: "includesAny",
            values: [value],
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.exclude.map(
        (value) =>
          ({
            field,
            op: "excludesAny",
            values: [value],
          }) satisfies MetadataFilterNode,
      ),
    ];
  }

  if (fieldSemantics.fieldType === "enumString") {
    const field = key as MetadataEnumStringField;
    return [
      ...normalizedPolicy.any.map(
        (value) =>
          ({
            field,
            op: "eq",
            value,
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.all.map(
        (value) =>
          ({
            field,
            op: "eq",
            value,
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.exclude.map(
        (value) =>
          ({
            not: {
              field,
              op: "eq",
              value,
            },
          }) satisfies MetadataFilterNode,
      ),
    ];
  }

  if (fieldSemantics.fieldType === "boolean") {
    const field = key as MetadataBooleanField;
    const toBoolean = (value: string) => value === "true";
    return [
      ...normalizedPolicy.any.map(
        (value) =>
          ({
            field,
            op: "eq",
            value: toBoolean(value),
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.all.map(
        (value) =>
          ({
            field,
            op: "eq",
            value: toBoolean(value),
          }) satisfies MetadataFilterNode,
      ),
      ...normalizedPolicy.exclude.map(
        (value) =>
          ({
            not: {
              field,
              op: "eq",
              value: toBoolean(value),
            },
          }) satisfies MetadataFilterNode,
      ),
    ];
  }

  return [];
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
    draft: {
      selection: normalizeFilterExplorerSelectionMap(
        mergeSelectionMaps(discoverablePartition.selections, metricPartition.selections),
        fieldSemanticsByName,
      ),
      scalarClauses: cloneScalarClauseMap(metricPartition.scalarClauses),
    },
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
  const draft: Pf2eTerminalFilterExplorerDraft = {
    selection: Object.fromEntries(
      Object.entries(preparedDraft.draft.selection).map(([field, policy]) => [field, clonePolicy(policy)]),
    ),
    scalarClauses: cloneScalarClauseMap(preparedDraft.draft.scalarClauses),
  };

  if (scopedFields.includes("rarity")) {
    draft.selection.rarity = clonePolicy(getSearchQueryRarityPolicy(query));
  }

  if (scopedFields.includes("actionCost")) {
    const actionCostPolicy = getSearchQueryActionCostPolicy(query);
    draft.selection.actionCost = {
      any: actionCostPolicy.any.map(String),
      all: [],
      exclude: actionCostPolicy.exclude.map(String),
    };
  }

  return {
    scopedFields: [...scopedFields],
    draft: {
      ...draft,
      selection: normalizeFilterExplorerSelectionMap(draft.selection, fieldSemanticsByName),
    },
    preservedMetadata: preparedDraft.preservedMetadata,
  };
}

function inferScopedFieldsFromDraft(draft: Pf2eTerminalFilterExplorerDraft): Pf2eTerminalQueryField[] {
  const scopedFields = new Set<Pf2eTerminalQueryField>();

  for (const field of Object.keys(draft.selection)) {
    if (field === "actorMetric" || field === "itemMetric") {
      scopedFields.add(field);
      continue;
    }

    const metricKey = parseMetricSelectionKey(field);
    if (metricKey) {
      scopedFields.add(metricKey.field);
      continue;
    }

    scopedFields.add(field as Pf2eTerminalQueryField);
  }

  for (const key of Object.keys(draft.scalarClauses)) {
    const metricKey = parseMetricSelectionKey(key);
    if (metricKey) {
      scopedFields.add(metricKey.field);
    }
  }

  return [...scopedFields];
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

  const normalizedSelections = normalizeFilterExplorerSelectionMap(draft.selection, fieldSemanticsByName);
  for (const [field, policy] of Object.entries(normalizedSelections)) {
    const metadataNode = buildMetadataNodeForSelectionKey(field, policy, fieldSemanticsByName);
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
  options: { preservedMetadata?: MetadataFilterNode | null } = {},
): Pf2eTerminalFilterExplorerInsertionResult {
  if (options.preservedMetadata) {
    return {
      kind: "replace",
      node: buildFilterExplorerMetadataNode(draft, fieldSemanticsByName, options),
    };
  }

  const normalizedSelections = normalizeFilterExplorerSelectionMap(draft.selection, fieldSemanticsByName);
  const insertionNodes: MetadataFilterNode[] = [];

  for (const [field, policy] of Object.entries(normalizedSelections)) {
    insertionNodes.push(...buildInsertionMetadataNodesForSelectionKey(field, policy, fieldSemanticsByName));
  }

  for (const [key, clause] of Object.entries(draft.scalarClauses)) {
    const metadataNode = buildMetricScalarClauseMetadataNode(key, clause);
    if (metadataNode) {
      insertionNodes.push(metadataNode);
    }
  }

  return insertionNodes.length > 0
    ? { kind: "insert", nodes: insertionNodes }
    : { kind: "replace", node: null };
}

function hasStringPolicyValues(policy: Pf2eTerminalFilterValuePolicy<string> | undefined): boolean {
  return Boolean(policy && (policy.any.length > 0 || policy.all.length > 0 || policy.exclude.length > 0));
}

function applyRarityExplorerSelection(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<string> | undefined,
): Pf2eTerminalSearchQuery {
  return setSearchQueryRarityPolicy(query, {
    any: [...(policy?.any ?? [])],
    all: [],
    exclude: [...(policy?.exclude ?? [])],
  });
}

function applyActionCostExplorerSelection(
  query: Pf2eTerminalSearchQuery,
  policy: Pf2eTerminalFilterValuePolicy<string> | undefined,
): Pf2eTerminalSearchQuery {
  const nextPolicy = {
    any: (policy?.any ?? [])
      .map((value) => Number.parseInt(value, 10))
      .filter((value, index, values) => Number.isFinite(value) && values.indexOf(value) === index),
    all: [] as number[],
    exclude: (policy?.exclude ?? [])
      .map((value) => Number.parseInt(value, 10))
      .filter((value, index, values) => Number.isFinite(value) && values.indexOf(value) === index),
  };

  return setSearchQueryActionCostPolicy(query, nextPolicy);
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
  const metadataDraft: Pf2eTerminalFilterExplorerDraft = {
    ...draft,
    selection: Object.fromEntries(
      Object.entries(draft.selection).filter(([field]) => !isTopLevelSearchFilterExplorerField(field)),
    ),
  };
  const preservedMetadata =
    options.preservedMetadata ??
    (scopedFields.length > 0
      ? prepareFilterExplorerDraftFromMetadataNode(getSearchQueryMetadataTree(query), scopedFields, fieldSemanticsByName)
          .preservedMetadata
      : getSearchQueryMetadataTree(query));

  let nextQuery = setSearchQueryMetadataTree(
    query,
    buildFilterExplorerMetadataNode(metadataDraft, fieldSemanticsByName, { preservedMetadata }),
  );

  if (scopedFieldSet.has("rarity")) {
    nextQuery = applyRarityExplorerSelection(nextQuery, draft.selection.rarity);
  }
  if (scopedFieldSet.has("actionCost")) {
    nextQuery = applyActionCostExplorerSelection(nextQuery, draft.selection.actionCost);
  }

  return nextQuery;
}
