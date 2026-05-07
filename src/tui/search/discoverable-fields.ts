import type { MetadataFieldSemantics } from "../../domain/metadata-field-catalog.js";
import type { SearchDiscoveryField, SearchDiscoveryMetricGroup } from "../../app/search-discovery-service.js";
import { getMetricDiscoveryGroupLabel } from "../../domain/metric-discovery-group-label.js";
import {
  describeMetadataFieldType,
  formatMetadataFieldLabel,
} from "../../domain/presentation-vocabulary.js";
import type { SearchFilterNode } from "../../domain/search-request-types.js";
import type { MetadataAtomicPredicate } from "../../domain/search-filter-metadata.js";
import { normalizeSearchFilterNode } from "./query-core.js";
import type { SearchCategory } from "../../domain/search-types.js";
import {
  createEmptyStringSelection,
  createScopedSelectionMap,
  mergeSelectionMaps,
  mergeStringSelections,
  normalizeQueryFieldSelection,
} from "./selections.js";
import {
  buildSearchFilterNodeForQueryFieldSelection,
  getSearchQueryPredicateFilter,
  setSearchQueryPredicateFilter,
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
  node: MetadataAtomicPredicate,
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
  node: SearchFilterNode,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): ScopedSelectionExtraction | null {
  if (node.kind === "not") {
    const extracted = tryExtractScopedSelectionNode(node.child, scopedFieldSet, fieldSemanticsByName);
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

  if (node.kind === "anyOf") {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedSelection = createEmptyStringSelection();

    for (const child of node.children) {
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

  if (node.kind === "allOf") {
    let extractedField: Pf2eTerminalQueryField | null = null;
    let mergedSelection = createEmptyStringSelection();
    let positiveGroups = 0;

    for (const child of node.children) {
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

  if (node.kind !== "metadataPredicate") {
    return null;
  }

  const extracted = extractSelectionFromMetadataPredicate(node.predicate, fieldSemanticsByName);
  return extracted && scopedFieldSet.has(extracted.field)
    ? {
        ...extracted,
        positiveGroups: extracted.selection.include.length > 0 ? 1 : 0,
      }
    : null;
}

function shouldPreserveUnsupportedScopedSelectionGroup(
  node: Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }>,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): boolean {
  const childExtractions = node.children.map((child) =>
    tryExtractScopedSelectionNode(child, scopedFieldSet, fieldSemanticsByName),
  );
  const [firstExtraction] = childExtractions;
  if (!firstExtraction) {
    return false;
  }

  return childExtractions.every((extracted) => extracted && extracted.field === firstExtraction.field);
}

function extractScopedQueryFieldSelections(
  node: SearchFilterNode | null,
  scopedFieldSet: ReadonlySet<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): {
  filter: SearchFilterNode | null;
  selections: Pf2eTerminalQueryFieldSelectionMap;
} {
  if (!node) {
    return {
      filter: null,
      selections: {},
    };
  }

  const directExtraction = tryExtractScopedSelectionNode(node, scopedFieldSet, fieldSemanticsByName);
  if (directExtraction) {
    return {
      filter: null,
      selections: {
        [directExtraction.field]: directExtraction.selection,
      },
    };
  }

  if (node.kind === "allOf") {
    if (shouldPreserveUnsupportedScopedSelectionGroup(node, scopedFieldSet, fieldSemanticsByName)) {
      return {
        filter: node,
        selections: {},
      };
    }

    let selections: Pf2eTerminalQueryFieldSelectionMap = {};
    const children: SearchFilterNode[] = [];

    for (const child of node.children) {
      const extracted = extractScopedQueryFieldSelections(child, scopedFieldSet, fieldSemanticsByName);
      selections = mergeSelectionMaps(selections, extracted.selections);
      if (extracted.filter) {
        children.push(extracted.filter);
      }
    }

    return {
      filter: normalizeSearchFilterNode(children.length === 0 ? null : { kind: "allOf", children }) ?? null,
      selections,
    };
  }

  if (node.kind === "anyOf" && shouldPreserveUnsupportedScopedSelectionGroup(node, scopedFieldSet, fieldSemanticsByName)) {
    return {
      filter: node,
      selections: {},
    };
  }

  return {
    filter: node,
    selections: {},
  };
}

export function partitionDiscoverableQueryFieldSelections(
  node: SearchFilterNode | null,
  scopedFields: readonly string[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): {
  filter: SearchFilterNode | null;
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
    getSearchQueryPredicateFilter(query),
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
    getSearchQueryPredicateFilter(query),
    scopedFields,
    fieldSemanticsByName,
  );
  const filterClauses: SearchFilterNode[] = extracted.filter ? [extracted.filter] : [];

  for (const field of scopedFields) {
    const nextSelection = normalizeQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      selections[field] ?? createEmptyStringSelection(),
      fieldSemanticsByName,
    );
    if (!nextSelection) {
      continue;
    }

    const filterNode = buildSearchFilterNodeForQueryFieldSelection(
      field as Pf2eTerminalFacetField,
      nextSelection,
      fieldSemanticsByName,
    );
    if (filterNode) {
      filterClauses.push(filterNode);
    }
  }
  return setSearchQueryPredicateFilter(
    query,
    filterClauses.length === 0
      ? null
      : filterClauses.length === 1
        ? filterClauses[0]!
        : normalizeSearchFilterNode({ kind: "allOf", children: filterClauses }) ?? null,
  );
}
