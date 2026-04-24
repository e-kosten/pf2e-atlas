import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { SearchDiscoveryField, SearchDiscoveryMetricGroup } from "../../app/search-discovery-service.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
} from "../../domain/presentation-vocabulary.js";
import { normalizeMetadataNode } from "./query-core.js";
import type { MetadataFilterNode, MetadataPredicate } from "./metadata-filter-draft.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import {
  createEmptyStringPolicy,
  createScopedSelectionMap,
  mergeSelectionMaps,
  mergeStringPolicies,
  normalizeQueryFieldPolicy,
} from "./policies.js";
import {
  buildMetadataNodeForQueryFieldSelection,
  getSearchQueryMetadataTree,
  setSearchQueryMetadataTree,
} from "./query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldEditor,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
} from "./service-types.js";

function extractPolicyFromMetadataPredicate(
  node: MetadataPredicate,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): { field: Pf2eTerminalQueryField; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  const fieldSemantics = fieldSemanticsByName.get(node.field as Pf2eTerminalFacetField);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    if ("values" in node && node.op === "includesAny") {
      return { field: node.field, policy: { any: node.values.map(String), all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "includesAll") {
      return { field: node.field, policy: { any: [], all: node.values.map(String), exclude: [] } };
    }
    if ("values" in node && node.op === "excludesAny") {
      return { field: node.field, policy: { any: [], all: [], exclude: node.values.map(String) } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "enumString") {
    if ("value" in node && node.op === "eq") {
      return { field: node.field, policy: { any: [String(node.value)], all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "in") {
      return { field: node.field, policy: { any: node.values.map(String), all: [], exclude: [] } };
    }
    if ("values" in node && node.op === "notIn") {
      return { field: node.field, policy: { any: [], all: [], exclude: node.values.map(String) } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "boolean" && "value" in node && node.op === "eq") {
    return { field: node.field, policy: { any: [String(node.value)], all: [], exclude: [] } };
  }

  return null;
}

function tryExtractScopedPolicyNode(
  node: MetadataFilterNode,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): { field: Pf2eTerminalQueryField; policy: Pf2eTerminalFilterValuePolicy<string> } | null {
  if ("and" in node) {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedPolicy = createEmptyStringPolicy();

    for (const child of node.and) {
      if ("and" in child || "or" in child || "not" in child) {
        return null;
      }
      const extracted = extractPolicyFromMetadataPredicate(child, fieldSemanticsByName);
      if (!extracted || !scopedFieldSet.has(extracted.field)) {
        return null;
      }
      if (extractedField && extracted.field !== extractedField) {
        return null;
      }
      extractedField = extracted.field;
      mergedPolicy = mergeStringPolicies(mergedPolicy, extracted.policy);
    }

    return extractedField ? { field: extractedField, policy: mergedPolicy } : null;
  }

  if ("or" in node || "not" in node) {
    return null;
  }

  const extracted = extractPolicyFromMetadataPredicate(node, fieldSemanticsByName);
  return extracted && scopedFieldSet.has(extracted.field) ? extracted : null;
}

function extractScopedQueryFieldSelections(
  node: MetadataFilterNode | null,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
} {
  if (!node) {
    return {
      metadata: null,
      selections: {},
    };
  }

  const directExtraction = tryExtractScopedPolicyNode(node, scopedFieldSet, fieldSemanticsByName);
  if (directExtraction) {
    return {
      metadata: null,
      selections: {
        [directExtraction.field]: directExtraction.policy,
      },
    };
  }

  if ("and" in node) {
    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    const children: MetadataFilterNode[] = [];

    for (const child of node.and) {
      const extracted = extractScopedQueryFieldSelections(child, scopedFieldSet, fieldSemanticsByName);
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

  return {
    metadata: node,
    selections: {},
  };
}

export function partitionDiscoverableQueryFieldSelections(
  node: MetadataFilterNode | null,
  scopedFields: readonly string[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): {
  metadata: MetadataFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
} {
  return extractScopedQueryFieldSelections(node, new Set(scopedFields), fieldSemanticsByName);
}

export function getQueryFieldEditor(field: Pick<MetadataFieldSemantics, "fieldType">): Pf2eTerminalQueryFieldEditor {
  if (["set", "enumString", "boolean"].includes(field.fieldType)) {
    return "sharedExplorer";
  }
  return "structuredForm";
}

function getMetricQueryFieldOptions(
  metricGroups: readonly SearchDiscoveryMetricGroup[],
  category: SearchCategory | null,
): Pf2eTerminalQueryFieldOption[] {
  if (!category) {
    return [];
  }

  const options: Pf2eTerminalQueryFieldOption[] = [];
  if (metricGroups.some((group) => group.metadataField === "actorMetric")) {
    options.push({
      value: "actorMetric",
      label: getMetricDiscoveryGroupLabel(category, "actorMetrics"),
      description: "Browse live statistic keys and author exact or numeric literal filters for the current scope.",
      fieldType: "enumString",
      editor: "sharedExplorer",
    });
  }

  if (metricGroups.some((group) => group.metadataField === "itemMetric")) {
    options.push({
      value: "itemMetric",
      label: getMetricDiscoveryGroupLabel(category, "itemMetrics"),
      description: "Browse live item property keys and author exact or numeric literal filters for the current scope.",
      fieldType: "enumString",
      editor: "sharedExplorer",
    });
  }

  return options;
}

export function getQueryFieldOptions(
  scopedFields: readonly SearchDiscoveryField[],
  metricGroups: readonly SearchDiscoveryMetricGroup[],
  category: SearchCategory | null,
): Pf2eTerminalQueryFieldOption[] {
  return [
    ...scopedFields
      .filter((field) => field.discoverable && !["rarity", "actionCost"].includes(field.field))
      .map((field) => ({
        value: field.field,
        label: formatMetadataFieldLabel(field.field),
        description:
          field.notes ??
          (field.field === "derivedTags"
            ? "Derived-tag field with hierarchy-capable ontology browsing."
            : `${describeMetadataFieldType(field.fieldType)} query field for the current browse scope.`),
        fieldType: field.fieldType,
        editor: getQueryFieldEditor(field),
      })),
    ...getMetricQueryFieldOptions(metricGroups, category),
  ];
}

export function buildDiscoverableQueryFieldSelections(
  query: Pf2eTerminalSearchQuery,
  scopedFields: string[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalQueryFieldSelectionMap {
  const selectionMap = createScopedSelectionMap(scopedFields);
  const extracted = partitionDiscoverableQueryFieldSelections(
    getSearchQueryMetadataTree(query),
    scopedFields,
    fieldSemanticsByName,
  );
  for (const [field, policy] of Object.entries(extracted.selections)) {
    const normalizedPolicy = normalizeQueryFieldPolicy(field as Pf2eTerminalFacetField, policy, fieldSemanticsByName);
    if (!normalizedPolicy) {
      continue;
    }
    selectionMap[field] = mergeStringPolicies(selectionMap[field] ?? createEmptyStringPolicy(), normalizedPolicy);
  }

  for (const field of scopedFields) {
    const normalizedPolicy = normalizeQueryFieldPolicy(
      field as Pf2eTerminalFacetField,
      selectionMap[field],
      fieldSemanticsByName,
    );
    selectionMap[field] = normalizedPolicy ?? createEmptyStringPolicy();
  }

  return selectionMap;
}

export function applyDiscoverableQueryFieldSelections(
  query: Pf2eTerminalSearchQuery,
  selections: Pf2eTerminalQueryFieldSelectionMap,
  scopedFields: string[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  const extracted = partitionDiscoverableQueryFieldSelections(
    getSearchQueryMetadataTree(query),
    scopedFields,
    fieldSemanticsByName,
  );
  const metadataClauses: MetadataFilterNode[] = extracted.metadata ? [extracted.metadata] : [];

  for (const field of scopedFields) {
    const nextPolicy = normalizeQueryFieldPolicy(
      field as Pf2eTerminalFacetField,
      selections[field] ?? createEmptyStringPolicy(),
      fieldSemanticsByName,
    );
    if (!nextPolicy) {
      continue;
    }

    const metadataNode = buildMetadataNodeForQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      nextPolicy,
      fieldSemanticsByName,
    );
    if (metadataNode) {
      metadataClauses.push(metadataNode);
    }
  }
  return setSearchQueryMetadataTree(
    query,
    metadataClauses.length === 0
      ? null
      : metadataClauses.length === 1
        ? metadataClauses[0]!
        : normalizeMetadataNode({ and: metadataClauses }),
  );
}
