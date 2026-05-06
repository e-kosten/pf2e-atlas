import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type { SearchDiscoveryField, SearchDiscoveryMetricGroup } from "../../app/search-discovery-service.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
} from "../../domain/presentation-vocabulary.js";
import { normalizeMetadataNode } from "./query-core.js";
import type { MetadataFilterNode, MetadataPredicate } from "./metadata-filter-draft.js";
import type { SearchCategory } from "../../domain/search-types.js";
import {
  createEmptyStringSelection,
  createScopedSelectionMap,
  mergeSelectionMaps,
  mergeStringSelections,
  normalizeQueryFieldSelection,
} from "./selections.js";
import {
  buildMetadataNodeForQueryFieldSelection,
  getSearchQueryMetadataTree,
  setSearchQueryMetadataTree,
} from "./query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalQueryField,
  Pf2eTerminalQueryFieldEditor,
  Pf2eTerminalQueryFieldOption,
  Pf2eTerminalQueryFieldSelectionMap,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalValueSelection,
} from "./service-types.js";

function extractSelectionFromMetadataPredicate(
  node: MetadataPredicate,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): { field: Pf2eTerminalQueryField; selection: Pf2eTerminalValueSelection<string> } | null {
  const fieldSemantics = fieldSemanticsByName.get(node.field as Pf2eTerminalFacetField);
  if (!fieldSemantics || !fieldSemantics.discoverable) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    if ("value" in node && node.op === "includes") {
      return { field: node.field, selection: { include: [String(node.value)], exclude: [] } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "enumString") {
    if ("value" in node && node.op === "eq") {
      return { field: node.field, selection: { include: [String(node.value)], exclude: [] } };
    }
    if ("values" in node && node.op === "in") {
      return { field: node.field, selection: { include: node.values.map(String), exclude: [] } };
    }
    if ("values" in node) {
      return { field: node.field, selection: { include: [], exclude: node.values.map(String) } };
    }
    return null;
  }

  if (fieldSemantics.fieldType === "boolean" && "value" in node && node.op === "eq") {
    return { field: node.field, selection: { include: [String(node.value)], exclude: [] } };
  }

  return null;
}

type ScopedSelectionExtraction = {
  field: Pf2eTerminalQueryField;
  selection: Pf2eTerminalValueSelection<string>;
  positiveGroups: 0 | 1;
};

function tryExtractScopedSelectionNode(
  node: MetadataFilterNode,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): ScopedSelectionExtraction | null {
  if ("not" in node) {
    const extracted = tryExtractScopedSelectionNode(node.not, scopedFieldSet, fieldSemanticsByName);
    if (!extracted || extracted.selection.exclude.length > 0 || extracted.positiveGroups !== 1) {
      return null;
    }
    return {
      field: extracted.field,
      selection: {
        include: [],
        exclude: [...extracted.selection.include],
      },
      positiveGroups: 0,
    };
  }

  if ("or" in node) {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedSelection = createEmptyStringSelection();

    for (const child of node.or) {
      const extracted = tryExtractScopedSelectionNode(child, scopedFieldSet, fieldSemanticsByName);
      if (!extracted || extracted.selection.exclude.length > 0 || extracted.positiveGroups !== 1) {
        return null;
      }
      if (extractedField && extracted.field !== extractedField) {
        return null;
      }
      extractedField = extracted.field;
      mergedSelection = mergeStringSelections(mergedSelection, extracted.selection);
    }

    return extractedField
      ? { field: extractedField, selection: mergedSelection, positiveGroups: 1 }
      : null;
  }

  if ("and" in node) {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedSelection = createEmptyStringSelection();
    let positiveGroups = 0;

    for (const child of node.and) {
      const extracted = tryExtractScopedSelectionNode(child, scopedFieldSet, fieldSemanticsByName);
      if (!extracted || !scopedFieldSet.has(extracted.field)) {
        return null;
      }
      if (extractedField && extracted.field !== extractedField) {
        return null;
      }
      extractedField = extracted.field;
      mergedSelection = mergeStringSelections(mergedSelection, extracted.selection);
      positiveGroups += extracted.positiveGroups;
      if (positiveGroups > 1) {
        return null;
      }
    }

    return extractedField
      ? {
          field: extractedField,
          selection: mergedSelection,
          positiveGroups: positiveGroups === 0 ? 0 : 1,
        }
      : null;
  }

  const extracted = extractSelectionFromMetadataPredicate(node, fieldSemanticsByName);
  return extracted && scopedFieldSet.has(extracted.field)
    ? {
        ...extracted,
        positiveGroups: extracted.selection.include.length > 0 ? 1 : 0,
      }
    : null;
}

function shouldPreserveUnsupportedScopedSelectionGroup(
  node: Extract<MetadataFilterNode, { and: MetadataFilterNode[] } | { or: MetadataFilterNode[] }>,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): boolean {
  const children = "and" in node ? node.and : node.or;
  const childExtractions = children.map((child) =>
    tryExtractScopedSelectionNode(child, scopedFieldSet, fieldSemanticsByName),
  );
  const [firstExtraction] = childExtractions;
  if (!firstExtraction) {
    return false;
  }

  return childExtractions.every((extracted) => extracted && extracted.field === firstExtraction.field);
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

  const directExtraction = tryExtractScopedSelectionNode(node, scopedFieldSet, fieldSemanticsByName);
  if (directExtraction) {
    return {
      metadata: null,
      selections: {
        [directExtraction.field]: directExtraction.selection,
      },
    };
  }

  if ("and" in node) {
    if (shouldPreserveUnsupportedScopedSelectionGroup(node, scopedFieldSet, fieldSemanticsByName)) {
      return {
        metadata: node,
        selections: {},
      };
    }

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

  if ("or" in node && shouldPreserveUnsupportedScopedSelectionGroup(node, scopedFieldSet, fieldSemanticsByName)) {
    return {
      metadata: node,
      selections: {},
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
        valueOrdering: field.valueOrdering,
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
  for (const [field, selection] of Object.entries(extracted.selections)) {
    const normalizedSelection = normalizeQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      selection,
      fieldSemanticsByName,
    );
    if (!normalizedSelection) {
      continue;
    }
    selectionMap[field] = mergeStringSelections(
      selectionMap[field] ?? createEmptyStringSelection(),
      normalizedSelection,
    );
  }

  for (const field of scopedFields) {
    const normalizedSelection = normalizeQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      selectionMap[field],
      fieldSemanticsByName,
    );
    selectionMap[field] = normalizedSelection ?? createEmptyStringSelection();
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
    const nextSelection = normalizeQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      selections[field] ?? createEmptyStringSelection(),
      fieldSemanticsByName,
    );
    if (!nextSelection) {
      continue;
    }

    const metadataNode = buildMetadataNodeForQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      nextSelection,
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
