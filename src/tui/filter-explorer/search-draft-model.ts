import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import { humanizeOntologySearchIdentifier } from "../../domain/presentation-vocabulary.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import { getOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import { isMetadataPredicate } from "../search/query-core.js";
import type { Pf2eTerminalQueryField, Pf2eTerminalQueryFieldOption } from "../search/service-types.js";
import type { FilterExplorerComposeTarget } from "./types.js";
import { canonicalFilterToMetadataNode } from "../search/query-parts.js";

type SearchFilterExplorerMetricField = "actorMetric" | "itemMetric";

type MetricValueType = "number" | "text" | "boolean";

type MetricFieldSelectionKey = {
  field: SearchFilterExplorerMetricField;
  metric: string;
};

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

function getSelectionValueFromPredicate(node: Record<string, unknown>): string | null {
  if ("value" in node) {
    return typeof node.value === "boolean" ? String(node.value) : String(node.value);
  }
  if ("values" in node && Array.isArray(node.values) && node.values.length === 1) {
    return String(node.values[0]);
  }
  return null;
}

function getMetricSelectionKey(field: SearchFilterExplorerMetricField, metric: string): string {
  return `${field}:${metric}`;
}

function inferMetricValueType(field: SearchFilterExplorerMetricField, metric: string): MetricValueType | null {
  return field === "actorMetric" ? inferActorMetricValueType(metric) : inferItemMetricValueType(metric);
}

function formatMetricLabel(metric: string, fallbackLabel?: string): string {
  const label = fallbackLabel?.trim();
  if (label) {
    return label;
  }
  return humanizeOntologySearchIdentifier(metric);
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
    uniqueScopedFieldNodes.length === 1 &&
    options.fieldOptions[0]?.value !== "derivedTags"
  ) {
    const children = getOntologyNodeChildren(uniqueScopedFieldNodes[0]);
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
    const scopedFieldNodeId = subcategory
      ? `${category}:${subcategory}:field:derivedTags`
      : `${category}:field:derivedTags`;
    const scopedMetadataFieldGroupId = subcategory
      ? `${category}:${subcategory}:metadataFields`
      : `${category}:metadataFields`;
    const derivedTagsFieldNode =
      findScopedSearchFilterExplorerMetadataFieldNode(
        subcategoryNode,
        scopedMetadataFieldGroupId,
        scopedFieldNodeId,
      ) ??
      findScopedSearchFilterExplorerMetadataFieldNode(
        categoryNode,
        `${category}:metadataFields`,
        `${category}:field:derivedTags`,
      );
    return derivedTagsFieldNode ?? findDirectNodeById(categoryChildren, `${category}:commonDerivedTags`);
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
    key: getMetricSelectionKey(metricTarget.field, metricTarget.metric),
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

    const predicate = node.query
      ? canonicalFilterToMetadataNode(node.query.request.filter)
      : null;
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
          key: getMetricSelectionKey(predicate.field, predicate.metric),
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

    if (predicate.field === "actorMetricCompare" || predicate.field === "itemMetricCompare") {
      const metricField = predicate.field === "actorMetricCompare" ? "actorMetric" : "itemMetric";
      const fieldOption = allowedFields.get(metricField);
      if (!fieldOption || !("leftMetric" in predicate) || !("rightMetric" in predicate)) {
        return undefined;
      }
      if (predicate.leftMetric !== predicate.rightMetric) {
        return undefined;
      }

      const metric = predicate.leftMetric;
      const valueType = inferMetricValueType(metricField, metric);
      if (valueType !== "number") {
        return undefined;
      }

      const metricLabel = formatMetricLabel(metric, node.label);
      return {
        kind: "scalar",
        key: getMetricSelectionKey(metricField, metric),
        fieldLabel: fieldOption.label,
        subjectLabel: metricLabel,
        valueType,
        editorLabel: `${fieldOption.label} / ${metricLabel}`,
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
