import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import type { MetadataFieldSemantics } from "../../domain/metadata-semantics.js";
import type { MetadataFilterNode, MetadataPredicate } from "../../domain/metadata-types.js";
import { getOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import { buildHostedOntologyPickerInitialSnapshot } from "../ontology-explorer/picker-hosting.js";
import { isMetadataPredicate, normalizeMetadataNode } from "./query-core.js";
import { partitionDiscoverableQueryFieldSelections } from "./discoverable-fields.js";
import { createEmptyStringPolicy, mergeStringPolicies, normalizeQueryFieldPolicy } from "./policies.js";
import { buildMetadataNodeForQueryFieldSelection, getSearchQueryMetadataTree, setSearchQueryMetadataTree } from "./query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterExplorerDraft,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./service-types.js";
import type { FilterExplorerComposeTarget } from "../filter-explorer/index.js";

type SearchFilterExplorerMetricField = "actorMetric" | "itemMetric";

type HostedPickerInitialSnapshotCandidate = {
  drillToFirstChild?: boolean;
  selectedNodeIds: string[];
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

function findNodeById(nodes: readonly OntologyNode[], id: string): OntologyNode | undefined {
  for (const node of nodes) {
    if (node.id === id) {
      return node;
    }
    const match = findNodeById(getOntologyNodeChildren(node), id);
    if (match) {
      return match;
    }
  }
  return undefined;
}

function buildHostedPickerInitialSnapshotCandidates(options: {
  category: string;
  subcategory: string | null;
  fieldOptions: Pf2eTerminalQueryFieldOption[];
  singleFieldBehavior: "list" | "directValues";
}): HostedPickerInitialSnapshotCandidate[] {
  const { category, subcategory, fieldOptions, singleFieldBehavior } = options;
  const categoryId = `searchSemantics:${category}`;
  const candidates: HostedPickerInitialSnapshotCandidate[] = [];
  const singleField = fieldOptions.length === 1 ? fieldOptions[0] : null;

  if (singleFieldBehavior === "directValues" && singleField) {
    if (singleField.value === "actorMetric" || singleField.value === "itemMetric") {
      const metricField = singleField.value === "actorMetric" ? "actorMetrics" : "itemMetrics";
      if (subcategory) {
        candidates.push({
          selectedNodeIds: [
            categoryId,
            `${category}:subcategories`,
            `${category}:subcategory:${subcategory}`,
            `${category}:${subcategory}:${metricField}:discovery`,
          ],
          drillToFirstChild: true,
        });
      }
      candidates.push({
        selectedNodeIds: [categoryId, `${category}:${metricField}:discovery`],
        drillToFirstChild: true,
      });
    } else {
      if (subcategory) {
        candidates.push({
          selectedNodeIds: [
            categoryId,
            `${category}:subcategories`,
            `${category}:subcategory:${subcategory}`,
            `${category}:${subcategory}:metadataFields`,
            `${category}:${subcategory}:field:${singleField.value}`,
          ],
          drillToFirstChild: true,
        });
      }
      candidates.push({
        selectedNodeIds: [categoryId, `${category}:metadataFields`, `${category}:field:${singleField.value}`],
        drillToFirstChild: true,
      });
      if (singleField.value === "derivedTags") {
        candidates.push({
          selectedNodeIds: [categoryId, `${category}:commonDerivedTags`],
          drillToFirstChild: true,
        });
      }
      if (singleField.value === "traits") {
        candidates.push({
          selectedNodeIds: [categoryId, `${category}:commonTraits`],
          drillToFirstChild: true,
        });
      }
    }
  }

  if (subcategory) {
    candidates.push({
      selectedNodeIds: [categoryId, `${category}:subcategories`, `${category}:subcategory:${subcategory}`],
    });
  }
  candidates.push({
    selectedNodeIds: [categoryId],
    drillToFirstChild: true,
  });

  return candidates;
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

function extractMetricSelections(
  node: MetadataFilterNode | null,
  allowedMetricFields: ReadonlySet<SearchFilterExplorerMetricField>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
} {
  if (!node || allowedMetricFields.size === 0) {
    return {
      metadata: node,
      selections: {},
    };
  }

  if (isMetadataPredicate(node)) {
    const extracted = extractMetricPolicyFromPredicate(node, allowedMetricFields);
    return extracted
      ? {
          metadata: null,
          selections: { [extracted.key]: extracted.policy },
        }
      : {
          metadata: node,
          selections: {},
        };
  }

  if ("and" in node) {
    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    const children: MetadataFilterNode[] = [];

    for (const child of node.and) {
      const extracted = extractMetricSelections(child, allowedMetricFields);
      selections = mergeSelectionMaps(selections, extracted.selections);
      if (extracted.metadata) {
        children.push(extracted.metadata);
      }
    }

    return {
      metadata: normalizeMetadataNode(children.length === 0 ? null : { and: children }),
      selections,
    };
  }

  if ("or" in node) {
    const childSelections = node.or.map((child) => extractMetricSelections(child, allowedMetricFields));
    const allExtracted = childSelections.every((entry) => entry.metadata === null);
    if (!allExtracted) {
      return {
        metadata: node,
        selections: {},
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
        }
      : {
          metadata: node,
          selections: {},
        };
  }

  return {
    metadata: node,
    selections: {},
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
    fieldOptions: Pf2eTerminalQueryFieldOption[];
  },
): OntologyDomainModel {
  const categoryNode = findNodeById(searchSemanticsDomain.rootNodes, `searchSemantics:${options.category}`);
  return {
    ...searchSemanticsDomain,
    label: options.fieldOptions.length === 1 ? `${options.fieldOptions[0]!.label} Explorer` : "Filter Explorer",
    description: searchSemanticsDomain.description,
    rootNodes: categoryNode ? [categoryNode] : [],
  };
}

export function buildSearchFilterExplorerInitialSnapshot(
  model: OntologyDomainModel,
  options: {
    category: string;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
) {
  return (
    buildHostedPickerInitialSnapshotCandidates(options)
      .map((candidate) => buildHostedOntologyPickerInitialSnapshot(model, candidate))
      .find((snapshot) => Boolean(snapshot)) ?? undefined
  );
}

function buildFieldSelectionTarget(
  fieldOption: Pf2eTerminalQueryFieldOption,
  node: OntologyNode,
  value: string,
): FilterExplorerComposeTarget {
  return {
    field: fieldOption.value,
    fieldLabel: fieldOption.label,
    value,
    valueLabel: node.label,
    allowedStates: fieldOption.fieldType === "set" ? ["any", "all", "exclude"] : ["any", "exclude"],
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
        .map((fieldOption) => buildFallbackFieldSelectionTarget(node, fieldOption))
        .find((target): target is FilterExplorerComposeTarget => Boolean(target));
    }

    if (predicate.field === "actorMetric" || predicate.field === "itemMetric") {
      const fieldOption = allowedFields.get(predicate.field);
      if (!fieldOption || !("metric" in predicate)) {
        return undefined;
      }
      const value = getSelectionValueFromPredicate(predicate);
      if (value === null) {
        return undefined;
      }
      return {
        field: getMetricSelectionKey(predicate.field, predicate.metric),
        fieldLabel: `${fieldOption.label} / ${predicate.metric}`,
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
  const metricPartition = extractMetricSelections(
    discoverablePartition.metadata,
    new Set(scopedFields.filter(isMetricQueryField)),
  );

  return {
    fieldSelections: normalizeFilterExplorerSelectionMap(
      mergeSelectionMaps(discoverablePartition.selections, metricPartition.selections),
      fieldSemanticsByName,
    ),
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
