import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import type { MetadataFilterNode, MetadataPredicate } from "../../domain/metadata-types.js";
import { getOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import { isMetadataPredicate, normalizeMetadataNode } from "./query-core.js";
import { partitionDiscoverableQueryFieldSelections } from "./discoverable-fields.js";
import { createEmptyStringPolicy, normalizeQueryFieldPolicy } from "./policies.js";
import { buildMetadataNodeForQueryFieldSelection, getSearchQueryMetadataTree, setSearchQueryMetadataTree } from "./query-state.js";
import { humanizeIdentifier } from "./service-options.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterExplorerMetricClause,
  Pf2eTerminalFilterExplorerScalarClauseMap,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./service-types.js";
import type { FilterExplorerComposeDraft, FilterExplorerComposeTarget } from "../filter-explorer/index.js";

type SearchFilterExplorerMetricField = "actorMetric" | "itemMetric";

type MetricValueType = "number" | "text" | "boolean";

type MetricFieldSelectionKey = {
  field: SearchFilterExplorerMetricField;
  metric: string;
};

function isMetricQueryField(field: Pf2eTerminalQueryField): field is SearchFilterExplorerMetricField {
  return field === "actorMetric" || field === "itemMetric";
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

function normalizeMetricPolicy(policy: Pf2eTerminalFilterValuePolicy<string> | undefined): Pf2eTerminalFilterValuePolicy<string> {
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

    const normalizedPolicy = normalizeQueryFieldPolicy(fieldKey as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
    next[fieldKey] = normalizedPolicy ?? createEmptyStringPolicy();
  }

  return next;
}

function findDirectNodeById(nodes: readonly OntologyNode[], id: string): OntologyNode | undefined {
  return nodes.find((node) => node.id === id);
}

function findScopedSearchFilterExplorerSubcategoryNode(
  categoryNode: OntologyNode,
  category: string,
  subcategory: string | null,
): OntologyNode | undefined {
  if (!subcategory) {
    return undefined;
  }

  const categoryChildren = getOntologyNodeChildren(categoryNode);
  const subcategoriesGroup = findDirectNodeById(categoryChildren, `${category}:subcategories`);
  return subcategoriesGroup
    ? findDirectNodeById(getOntologyNodeChildren(subcategoriesGroup), `${category}:subcategory:${subcategory}`)
    : undefined;
}

function findScopedSearchFilterExplorerMetadataFieldNode(
  scopeNode: OntologyNode | undefined,
  metadataGroupId: string,
  fieldNodeId: string,
): OntologyNode | undefined {
  if (!scopeNode) {
    return undefined;
  }

  const metadataGroup = findDirectNodeById(getOntologyNodeChildren(scopeNode), metadataGroupId);
  return metadataGroup ? findDirectNodeById(getOntologyNodeChildren(metadataGroup), fieldNodeId) : undefined;
}

function getSelectionValueFromPredicate(node: MetadataPredicate): string | null {
  if ("value" in node) {
    return typeof node.value === "boolean" ? String(node.value) : String(node.value);
  }
  if ("values" in node && node.values.length === 1) {
    return String(node.values[0]);
  }
  return null;
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

function formatMetricLabel(metric: string, fallbackLabel?: string): string {
  const label = fallbackLabel?.trim();
  if (label) {
    return label;
  }
  return humanizeIdentifier(metric.replaceAll(".", " "));
}

function cloneMetricClause(
  clause: Pf2eTerminalFilterExplorerMetricClause,
): Pf2eTerminalFilterExplorerMetricClause {
  return {
    ...clause,
    clause: clause.clause.operator === "between" ? { ...clause.clause } : { ...clause.clause },
  };
}

function cloneMetricClauseMap(
  scalarClauses: Pf2eTerminalFilterExplorerScalarClauseMap,
): Pf2eTerminalFilterExplorerScalarClauseMap {
  return Object.fromEntries(
    Object.entries(scalarClauses).map(([key, clause]) => [key, cloneMetricClause(clause)]),
  );
}

function mergeScalarClauseMaps(
  left: Pf2eTerminalFilterExplorerScalarClauseMap,
  right: Pf2eTerminalFilterExplorerScalarClauseMap,
): Pf2eTerminalFilterExplorerScalarClauseMap | null {
  const next = cloneMetricClauseMap(left);

  for (const [key, clause] of Object.entries(right)) {
    if (key in next) {
      return null;
    }
    next[key] = cloneMetricClause(clause);
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

function buildMetricClauseRecord(
  field: SearchFilterExplorerMetricField,
  metric: string,
  clause: Pf2eTerminalFilterExplorerMetricClause["clause"],
): Pf2eTerminalFilterExplorerMetricClause | null {
  const valueType = inferMetricValueType(field, metric);
  if (!valueType) {
    return null;
  }

  return {
    field,
    metric,
    valueType,
    clause: clause.operator === "between" ? { ...clause } : { ...clause },
  };
}

function extractMetricPolicyFromPredicate(
  node: MetadataPredicate,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  if (!("metric" in node) || !("value" in node) || (node.field !== "actorMetric" && node.field !== "itemMetric")) {
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
  if (node.op === "!=") {
    return {
      key: getMetricSelectionKey(node.field, node.metric),
      policy: { any: [], all: [], exclude: [selectionValue] },
    };
  }
  return null;
}

function extractMetricScalarClauseFromPredicate(
  node: MetadataPredicate,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: Pf2eTerminalFilterExplorerMetricClause } | null {
  if (!("metric" in node) || !("value" in node) || (node.field !== "actorMetric" && node.field !== "itemMetric")) {
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
      field: node.field,
      metric: node.metric,
      valueType,
      clause: {
        operator,
        value: node.value,
      },
    },
  };
}

function tryExtractMetricBetweenClause(
  node: MetadataFilterNode,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): { key: string; clause: Pf2eTerminalFilterExplorerMetricClause } | null {
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
    (left.field !== "actorMetric" && left.field !== "itemMetric") ||
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
      field: left.field,
      metric: left.metric,
      valueType: "number",
      clause: {
        operator: "between",
        min: lower.value,
        max: upper.value,
      },
    },
  };
}

function extractMetricDraftEntries(
  node: MetadataFilterNode | null,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
  scalarClauses: Pf2eTerminalFilterExplorerScalarClauseMap;
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
    let scalarClauses: Pf2eTerminalFilterExplorerScalarClauseMap = {};
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

function buildMetricScalarClauseMetadataNode(
  clause: Pf2eTerminalFilterExplorerMetricClause,
): MetadataFilterNode | null {
  if (clause.clause.operator === "between") {
    return normalizeMetadataNode({
      and: [
        {
          field: clause.field,
          metric: clause.metric,
          op: ">=",
          value: clause.clause.min,
        },
        {
          field: clause.field,
          metric: clause.metric,
          op: "<=",
          value: clause.clause.max,
        },
      ],
    });
  }

  const op =
    clause.clause.operator === "eq"
      ? "=="
      : clause.clause.operator === "neq"
        ? "!="
        : clause.clause.operator === "gte"
          ? ">="
          : clause.clause.operator === "lte"
            ? "<="
            : null;
  if (!op) {
    return null;
  }

  if (clause.valueType !== "number") {
    if (clause.field === "actorMetric") {
      return {
        field: "actorMetric",
        metric: clause.metric,
        op: op === ">=" || op === "<=" ? "==" : op,
        value: clause.clause.value as string | boolean,
      };
    }

    return {
      field: "itemMetric",
      metric: clause.metric,
      op: op === ">=" || op === "<=" ? "==" : op,
      value: clause.clause.value as string | boolean,
    };
  }

  if (clause.field === "actorMetric") {
    return {
      field: "actorMetric",
      metric: clause.metric,
      op,
      value: clause.clause.value as number,
    };
  }

  return {
    field: "itemMetric",
    metric: clause.metric,
    op,
    value: clause.clause.value as number,
  };
}

function buildMetadataNodeForSelectionKey(
  key: string,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const metricKey = parseMetricSelectionKey(key);
  if (metricKey) {
    return buildMetricSelectionMetadataNode(metricKey.field, metricKey.metric, policy);
  }
  return buildMetadataNodeForQueryFieldSelection(key as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
}

export function buildSearchFilterExplorerModel(
  searchSemanticsDomain: OntologyDomainModel,
  options: {
    category: string;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): OntologyDomainModel {
  const categoryNode = findDirectNodeById(searchSemanticsDomain.rootNodes, `searchSemantics:${options.category}`);
  const rootNodes = categoryNode
    ? buildSearchFilterExplorerRootNodes(categoryNode, {
        category: options.category,
        subcategory: options.subcategory,
        fieldOptions: options.fieldOptions,
        singleFieldBehavior: options.singleFieldBehavior,
      })
    : [];
  return {
    ...searchSemanticsDomain,
    label: options.fieldOptions.length === 1 ? `${options.fieldOptions[0]!.label} Explorer` : "Filter Explorer",
    description: searchSemanticsDomain.description,
    rootNodes,
  };
}

function buildSearchFilterExplorerRootNodes(
  categoryNode: OntologyNode,
  options: {
    category: string;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): OntologyNode[] {
  const scopedFieldNodes = options.fieldOptions
    .map((fieldOption) => findSearchFilterExplorerFieldNode(categoryNode, options.category, options.subcategory, fieldOption))
    .filter((node): node is OntologyNode => Boolean(node));
  const uniqueScopedFieldNodes = [...new Map(scopedFieldNodes.map((node) => [node.id, node])).values()];

  if (
    options.singleFieldBehavior === "directValues" &&
    options.fieldOptions.length === 1 &&
    uniqueScopedFieldNodes.length === 1
  ) {
    const children = getOntologyNodeChildren(uniqueScopedFieldNodes[0]!);
    return children.length > 0 ? [...children] : uniqueScopedFieldNodes;
  }

  if (uniqueScopedFieldNodes.length > 0) {
    return uniqueScopedFieldNodes;
  }

  return [categoryNode];
}

function findSearchFilterExplorerFieldNode(
  categoryNode: OntologyNode,
  category: string,
  subcategory: string | null,
  fieldOption: Pf2eTerminalQueryFieldOption,
): OntologyNode | undefined {
  const categoryChildren = getOntologyNodeChildren(categoryNode);
  const subcategoryNode = findScopedSearchFilterExplorerSubcategoryNode(categoryNode, category, subcategory);
  const subcategoryChildren = subcategoryNode ? getOntologyNodeChildren(subcategoryNode) : [];

  if (fieldOption.value === "actorMetric") {
    return (
      (subcategory ? findDirectNodeById(subcategoryChildren, `${category}:${subcategory}:actorMetrics:discovery`) : undefined) ??
      findDirectNodeById(categoryChildren, `${category}:actorMetrics:discovery`)
    );
  }

  if (fieldOption.value === "itemMetric") {
    return (
      (subcategory ? findDirectNodeById(subcategoryChildren, `${category}:${subcategory}:itemMetrics:discovery`) : undefined) ??
      findDirectNodeById(categoryChildren, `${category}:itemMetrics:discovery`)
    );
  }

  const subcategoryFieldNode = subcategory
    ? findScopedSearchFilterExplorerMetadataFieldNode(
        subcategoryNode,
        `${category}:${subcategory}:metadataFields`,
        `${category}:${subcategory}:field:${fieldOption.value}`,
      )
    : undefined;
  if (subcategoryFieldNode) {
    return subcategoryFieldNode;
  }

  const categoryFieldNode = findScopedSearchFilterExplorerMetadataFieldNode(
    categoryNode,
    `${category}:metadataFields`,
    `${category}:field:${fieldOption.value}`,
  );
  if (categoryFieldNode) {
    return categoryFieldNode;
  }

  if (fieldOption.value === "derivedTags") {
    return findDirectNodeById(categoryChildren, `${category}:commonDerivedTags`);
  }

  if (fieldOption.value === "traits") {
    return findDirectNodeById(categoryChildren, `${category}:commonTraits`);
  }

  return undefined;
}

function buildFieldSelectionTarget(
  fieldOption: Pf2eTerminalQueryFieldOption,
  node: OntologyNode,
  value: string,
): FilterExplorerComposeTarget {
  return {
    kind: "discrete",
    field: fieldOption.value,
    fieldLabel: fieldOption.label,
    value,
    valueLabel: node.label,
    allowedStates: fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
  };
}

function parseMetricNodeSelectionKey(node: OntologyNode): MetricFieldSelectionKey | null {
  const actorMetricMatch = node.id.match(/:(actorMetrics):([^:]+)$/);
  if (actorMetricMatch) {
    return {
      field: "actorMetric",
      metric: actorMetricMatch[2]!,
    };
  }

  const itemMetricMatch = node.id.match(/:(itemMetrics):([^:]+)$/);
  if (itemMetricMatch) {
    return {
      field: "itemMetric",
      metric: itemMetricMatch[2]!,
    };
  }

  return null;
}

function buildMetricScalarTarget(
  node: OntologyNode,
  fieldOption: Pf2eTerminalQueryFieldOption | undefined,
): FilterExplorerComposeTarget | undefined {
  if (!fieldOption || node.kind !== "metric") {
    return undefined;
  }

  const metricTarget = parseMetricNodeSelectionKey(node);
  if (!metricTarget || metricTarget.field !== fieldOption.value) {
    return undefined;
  }

  const valueType = inferMetricValueType(metricTarget.field, metricTarget.metric);
  if (valueType !== "number") {
    return undefined;
  }

  const metricLabel = formatMetricLabel(metricTarget.metric, node.label);

  return {
    kind: "scalar",
    key: buildMetricClauseKey(metricTarget.field, metricTarget.metric),
    fieldLabel: fieldOption.label,
    subjectLabel: metricLabel,
    valueType,
    editorLabel: `${fieldOption.label} / ${metricLabel}`,
  };
}

function buildFallbackFieldSelectionTarget(
  node: OntologyNode,
  fieldOption: Pf2eTerminalQueryFieldOption | undefined,
): FilterExplorerComposeTarget | undefined {
  if (!fieldOption) {
    return undefined;
  }

  if (fieldOption.value === "derivedTags" && node.kind === "tag") {
    return buildFieldSelectionTarget(fieldOption, node, node.label);
  }
  if (fieldOption.value === "traits" && node.kind === "trait") {
    return buildFieldSelectionTarget(fieldOption, node, node.label);
  }
  if (node.kind !== "value") {
    return undefined;
  }

  const idSegments = node.id.split(":");
  const fieldIndex = idSegments.findIndex((segment) => segment === fieldOption.value);
  if (fieldIndex === -1 || fieldIndex === idSegments.length - 1) {
    return undefined;
  }

  return buildFieldSelectionTarget(fieldOption, node, idSegments.slice(fieldIndex + 1).join(":"));
}

export function buildSearchFilterExplorerTargetResolver(
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[],
): (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined {
  const allowedFields = new Map(fieldOptions.map((field) => [field.value, field]));

  return (node) => {
    if (!node) {
      return undefined;
    }

    const predicate = node?.query?.filters.metadata;
    if (!predicate || !isMetadataPredicate(predicate)) {
      return fieldOptions
        .map((fieldOption) => buildMetricScalarTarget(node, fieldOption) ?? buildFallbackFieldSelectionTarget(node, fieldOption))
        .find((target): target is FilterExplorerComposeTarget => Boolean(target));
    }

    if (predicate.field === "actorMetric" || predicate.field === "itemMetric") {
      const fieldOption = allowedFields.get(predicate.field);
      if (!fieldOption || !("metric" in predicate)) {
        return undefined;
      }
      const metricLabel = formatMetricLabel(predicate.metric, node.label);
      const valueType = inferMetricValueType(predicate.field, predicate.metric);
      if (valueType === "number" && "value" in predicate && typeof predicate.value === "number") {
        return {
          kind: "scalar",
          key: buildMetricClauseKey(predicate.field, predicate.metric),
          fieldLabel: fieldOption.label,
          subjectLabel: metricLabel,
          valueType,
          editorLabel: `${fieldOption.label} / ${metricLabel}`,
        };
      }
      const value = getSelectionValueFromPredicate(predicate);
      if (value === null) {
        return undefined;
      }
      return {
        kind: "discrete",
        field: getMetricSelectionKey(predicate.field, predicate.metric),
        fieldLabel: `${fieldOption.label} / ${metricLabel}`,
        value,
        valueLabel: node.label,
        allowedStates: ["any", "exclude"],
      };
    }

    const fieldOption = allowedFields.get(predicate.field as Pf2eTerminalQueryField);
    if (!fieldOption) {
      return undefined;
    }
    const value = getSelectionValueFromPredicate(predicate);
    if (value === null) {
      return undefined;
    }
    return buildFieldSelectionTarget(fieldOption, node, value);
  };
}

export function createFilterExplorerDraftFromMetadataNode(
  node: MetadataFilterNode | null,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalFilterExplorerDraft {
  const regularFields = scopedFields.filter((field): field is Pf2eTerminalFacetField => !isMetricQueryField(field));
  const discoverablePartition = partitionDiscoverableQueryFieldSelections(node, regularFields, fieldSemanticsByName);
  const metricPartition = extractMetricDraftEntries(
    discoverablePartition.metadata,
    new Set(scopedFields.filter(isMetricQueryField)),
  );

  return {
    fieldSelections: normalizeFilterExplorerSelectionMap(
      mergeSelectionMaps(discoverablePartition.selections, metricPartition.selections),
      fieldSemanticsByName,
    ),
    scalarClauses: cloneMetricClauseMap(metricPartition.scalarClauses),
    structuredMetadata: normalizeMetadataNode(metricPartition.metadata),
  };
}

export function createFilterExplorerDraftFromQuery(
  query: Pf2eTerminalSearchQuery,
  scopedFields: readonly Pf2eTerminalQueryField[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalFilterExplorerDraft {
  return createFilterExplorerDraftFromMetadataNode(
    getSearchQueryMetadataTree(query),
    scopedFields,
    fieldSemanticsByName,
  );
}

export function createFilterExplorerComposeDraft(
  draft: Pf2eTerminalFilterExplorerDraft,
): FilterExplorerComposeDraft {
  return {
    selection: Object.fromEntries(
      Object.entries(draft.fieldSelections).map(([field, policy]) => [field, clonePolicy(policy)]),
    ),
    scalarClauses: Object.fromEntries(
      Object.entries(draft.scalarClauses).map(([key, clause]) => [
        key,
        clause.clause.operator === "between" ? { ...clause.clause } : { ...clause.clause },
      ]),
    ),
  };
}

export function applyFilterExplorerComposeDraft(
  currentDraft: Pf2eTerminalFilterExplorerDraft,
  composeDraft: FilterExplorerComposeDraft,
): Pf2eTerminalFilterExplorerDraft {
  const scalarClauses: Pf2eTerminalFilterExplorerScalarClauseMap = {};

  for (const [key, clause] of Object.entries(composeDraft.scalarClauses)) {
    const existingClause = currentDraft.scalarClauses[key];
    if (existingClause) {
      scalarClauses[key] = {
        ...existingClause,
        clause: clause.operator === "between" ? { ...clause } : { ...clause },
      };
      continue;
    }

    const metricKey = parseMetricSelectionKey(key);
    if (!metricKey) {
      continue;
    }

    const nextClause = buildMetricClauseRecord(metricKey.field, metricKey.metric, clause);
    if (nextClause) {
      scalarClauses[key] = nextClause;
    }
  }

  return {
    fieldSelections: Object.fromEntries(
      Object.entries(composeDraft.selection).map(([field, policy]) => [field, clonePolicy(policy)]),
    ),
    scalarClauses,
    structuredMetadata: currentDraft.structuredMetadata,
  };
}

export function buildFilterExplorerMetadataNode(
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const metadataClauses: MetadataFilterNode[] = [];

  if (draft.structuredMetadata) {
    metadataClauses.push(draft.structuredMetadata);
  }

  const normalizedSelections = normalizeFilterExplorerSelectionMap(draft.fieldSelections, fieldSemanticsByName);
  for (const [field, policy] of Object.entries(normalizedSelections)) {
    const metadataNode = buildMetadataNodeForSelectionKey(field, policy, fieldSemanticsByName);
    if (metadataNode) {
      metadataClauses.push(metadataNode);
    }
  }

  for (const clause of Object.values(draft.scalarClauses ?? {})) {
    const metadataNode = buildMetricScalarClauseMetadataNode(clause);
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

export function applyFilterExplorerDraft(
  query: Pf2eTerminalSearchQuery,
  draft: Pf2eTerminalFilterExplorerDraft,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  return setSearchQueryMetadataTree(query, buildFilterExplorerMetadataNode(draft, fieldSemanticsByName));
}
