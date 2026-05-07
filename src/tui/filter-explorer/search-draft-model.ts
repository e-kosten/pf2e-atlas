import type { OntologyDomainModel, OntologyNode } from "../../domain/ontology-types.js";
import { humanizeOntologySearchIdentifier } from "../../domain/presentation-vocabulary.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import { resolveOntologyNodeChildren } from "../../app/ontology/node-helpers.js";
import type { Pf2eTerminalQueryField, Pf2eTerminalQueryFieldOption } from "../search/service-types.js";
import type { FilterExplorerComposeTarget } from "./types.js";

type SearchFilterExplorerMetricField = "actorMetric" | "itemMetric";

type MetricValueType = "number" | "text" | "boolean";

type MetricFieldSelectionKey = {
  field: SearchFilterExplorerMetricField;
  metric: string;
};

function findDirectNodeById(nodes: readonly OntologyNode[], id: string): OntologyNode | undefined {
  return nodes.find((node) => node.id === id);
}

async function findScopedSearchFilterExplorerSubcategoryNode(
  categoryNode: OntologyNode,
  category: string,
  subcategory: string | null,
): Promise<OntologyNode | undefined> {
  if (!subcategory) {
    return undefined;
  }

  const categoryChildren = await resolveOntologyNodeChildren(categoryNode);
  const subcategoriesGroup = findDirectNodeById(categoryChildren, `${category}:subcategories`);
  return subcategoriesGroup
    ? findDirectNodeById(await resolveOntologyNodeChildren(subcategoriesGroup), `${category}:subcategory:${subcategory}`)
    : undefined;
}

async function findScopedSearchFilterExplorerMetadataFieldNode(
  scopeNode: OntologyNode | undefined,
  metadataGroupId: string,
  fieldNodeId: string,
): Promise<OntologyNode | undefined> {
  if (!scopeNode) {
    return undefined;
  }

  const metadataGroup = findDirectNodeById(await resolveOntologyNodeChildren(scopeNode), metadataGroupId);
  return metadataGroup ? findDirectNodeById(await resolveOntologyNodeChildren(metadataGroup), fieldNodeId) : undefined;
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

export async function buildSearchFilterExplorerModel(
  searchSemanticsDomain: OntologyDomainModel,
  options: {
    category: string | null;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): Promise<OntologyDomainModel> {
  const rootNodes = options.category
    ? await buildScopedSearchFilterExplorerRootNodes(searchSemanticsDomain, {
        category: options.category,
        subcategory: options.subcategory,
        fieldOptions: options.fieldOptions,
        singleFieldBehavior: options.singleFieldBehavior,
      })
    : await buildUnscopedSearchFilterExplorerRootNodes(searchSemanticsDomain, {
        fieldOptions: options.fieldOptions,
        singleFieldBehavior: options.singleFieldBehavior,
      });
  return {
    ...searchSemanticsDomain,
    label: options.fieldOptions.length === 1 ? `${options.fieldOptions[0]!.label} Explorer` : "Filter Explorer",
    description: searchSemanticsDomain.description,
    rootNodes,
  };
}

async function buildScopedSearchFilterExplorerRootNodes(
  searchSemanticsDomain: OntologyDomainModel,
  options: {
    category: string;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): Promise<OntologyNode[]> {
  const categoryNode = findDirectNodeById(searchSemanticsDomain.rootNodes, `searchSemantics:${options.category}`);
  return categoryNode ? buildSearchFilterExplorerRootNodes(categoryNode, options) : [];
}

function getCategoryFromSearchSemanticsNode(node: OntologyNode): string | null {
  const match = node.id.match(/^searchSemantics:(.+)$/);
  return match?.[1] ?? null;
}

function isUnscopedSearchFilterExplorerField(fieldOption: Pf2eTerminalQueryFieldOption): boolean {
  return fieldOption.value === "rarity" || fieldOption.value === "pack";
}

function getUnscopedDirectValueKey(fieldOption: Pf2eTerminalQueryFieldOption, node: OntologyNode): string {
  if (fieldOption.value === "pack") {
    return node.id.match(/:pack[s]?:([^:]+)$/)?.[1] ?? node.id;
  }

  const idSegments = node.id.split(":");
  const fieldIndex = idSegments.findIndex((segment) => segment === fieldOption.value);
  const valueSegments = fieldIndex === -1 ? [] : idSegments.slice(fieldIndex + 1);
  return (valueSegments[0] === "value" ? valueSegments.slice(1) : valueSegments).join(":") || node.id;
}

function getUnscopedDirectValueCount(node: OntologyNode): number {
  const countText = node.listLabel?.match(/\|\s*([0-9,]+)\s*$/)?.[1];
  if (!countText) {
    return 0;
  }
  const count = Number.parseInt(countText.replaceAll(",", ""), 10);
  return Number.isFinite(count) ? count : 0;
}

function aggregateUnscopedDirectValueNodes(
  fieldOption: Pf2eTerminalQueryFieldOption,
  nodes: readonly OntologyNode[],
): OntologyNode[] {
  const aggregate = new Map<string, { node: OntologyNode; count: number }>();

  for (const node of nodes) {
    const key = getUnscopedDirectValueKey(fieldOption, node);
    const current = aggregate.get(key);
    aggregate.set(key, {
      node: current?.node ?? node,
      count: (current?.count ?? 0) + getUnscopedDirectValueCount(node),
    });
  }

  return [...aggregate.entries()].map(([value, entry]) => {
    const label = entry.node.label;
    const nodeId =
      fieldOption.value === "pack"
        ? `unscoped:${fieldOption.value}:${value}`
        : `unscoped:${fieldOption.value}:value:${value}`;
    return {
      ...entry.node,
      id: nodeId,
      listLabel: `${label} | ${entry.count}`,
      detailLines: [
        { text: label, tone: "section" },
        { text: `Field: ${fieldOption.label}` },
        { text: `Value: ${value}` },
        { text: `Live canonical records: ${entry.count}` },
      ],
      childSource: undefined,
      query: undefined,
    };
  });
}

async function buildUnscopedSearchFilterExplorerRootNodes(
  searchSemanticsDomain: OntologyDomainModel,
  options: {
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): Promise<OntologyNode[]> {
  if (options.fieldOptions.some((fieldOption) => !isUnscopedSearchFilterExplorerField(fieldOption))) {
    return [];
  }

  const scopedFieldNodeGroups = await Promise.all(searchSemanticsDomain.rootNodes.map(async (categoryNode) => {
    const category = getCategoryFromSearchSemanticsNode(categoryNode);
    return category
      ? (
          await Promise.all(
            options.fieldOptions.map((fieldOption) =>
              findSearchFilterExplorerFieldNode(categoryNode, category, null, fieldOption),
            ),
          )
        ).filter((node): node is OntologyNode => Boolean(node))
      : [];
  }));
  const scopedFieldNodes = scopedFieldNodeGroups.flat();
  const uniqueScopedFieldNodes = [...new Map(scopedFieldNodes.map((node) => [node.id, node])).values()];

  if (
    options.singleFieldBehavior === "directValues" &&
    options.fieldOptions.length === 1 &&
    options.fieldOptions[0]?.value !== "derivedTags"
  ) {
    const fieldOption = options.fieldOptions[0]!;
    const directValueNodeGroups = await Promise.all(
      uniqueScopedFieldNodes.map((node) => resolveOntologyNodeChildren(node)),
    );
    const directValueNodes = directValueNodeGroups.flat();
    return aggregateUnscopedDirectValueNodes(fieldOption, directValueNodes);
  }

  return uniqueScopedFieldNodes;
}

async function buildSearchFilterExplorerRootNodes(
  categoryNode: OntologyNode,
  options: {
    category: string;
    subcategory: string | null;
    fieldOptions: Pf2eTerminalQueryFieldOption[];
    singleFieldBehavior: "list" | "directValues";
  },
): Promise<OntologyNode[]> {
  const scopedFieldNodes = (
    await Promise.all(
      options.fieldOptions.map((fieldOption) =>
        findSearchFilterExplorerFieldNode(categoryNode, options.category, options.subcategory, fieldOption),
      ),
    )
  ).filter((node): node is OntologyNode => Boolean(node));
  const uniqueScopedFieldNodes = [...new Map(scopedFieldNodes.map((node) => [node.id, node])).values()];

  if (
    options.singleFieldBehavior === "directValues" &&
    options.fieldOptions.length === 1 &&
    uniqueScopedFieldNodes.length === 1 &&
    options.fieldOptions[0]?.value !== "derivedTags"
  ) {
    const children = await resolveOntologyNodeChildren(uniqueScopedFieldNodes[0]);
    return children.length > 0 ? [...children] : uniqueScopedFieldNodes;
  }

  if (uniqueScopedFieldNodes.length > 0) {
    return uniqueScopedFieldNodes;
  }

  return [categoryNode];
}

async function findSearchFilterExplorerFieldNode(
  categoryNode: OntologyNode,
  category: string,
  subcategory: string | null,
  fieldOption: Pf2eTerminalQueryFieldOption,
): Promise<OntologyNode | undefined> {
  const categoryChildren = await resolveOntologyNodeChildren(categoryNode);
  const subcategoryNode = await findScopedSearchFilterExplorerSubcategoryNode(categoryNode, category, subcategory);
  const subcategoryChildren = subcategoryNode ? await resolveOntologyNodeChildren(subcategoryNode) : [];

  if (fieldOption.value === "actorMetric") {
    return (
      (subcategory
        ? findDirectNodeById(subcategoryChildren, `${category}:${subcategory}:actorMetrics:discovery`)
        : undefined) ?? findDirectNodeById(categoryChildren, `${category}:actorMetrics:discovery`)
    );
  }

  if (fieldOption.value === "itemMetric") {
    return (
      (subcategory
        ? findDirectNodeById(subcategoryChildren, `${category}:${subcategory}:itemMetrics:discovery`)
        : undefined) ?? findDirectNodeById(categoryChildren, `${category}:itemMetrics:discovery`)
    );
  }

  if (fieldOption.value === "pack") {
    return (
      (subcategory ? findDirectNodeById(subcategoryChildren, `${category}:${subcategory}:pack`) : undefined) ??
      findDirectNodeById(categoryChildren, `${category}:pack`)
    );
  }

  const subcategoryFieldNode = subcategory
    ? await findScopedSearchFilterExplorerMetadataFieldNode(
        subcategoryNode,
        `${category}:${subcategory}:metadataFields`,
        `${category}:${subcategory}:field:${fieldOption.value}`,
      )
    : undefined;
  if (subcategoryFieldNode) {
    return subcategoryFieldNode;
  }

  const categoryFieldNode = await findScopedSearchFilterExplorerMetadataFieldNode(
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
      (await findScopedSearchFilterExplorerMetadataFieldNode(subcategoryNode, scopedMetadataFieldGroupId, scopedFieldNodeId)) ??
      (await findScopedSearchFilterExplorerMetadataFieldNode(
        categoryNode,
        `${category}:metadataFields`,
        `${category}:field:derivedTags`,
      ));
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
    allowedOperators: ["include", "exclude"],
  };
}

function buildPackSelectionTarget(
  fieldOption: Pf2eTerminalQueryFieldOption,
  node: OntologyNode,
): FilterExplorerComposeTarget | undefined {
  if (fieldOption.value !== "pack" || node.kind !== "value") {
    return undefined;
  }

  const packMatch = node.id.match(/:pack[s]?:([^:]+)$/);
  if (!packMatch?.[1]) {
    return undefined;
  }

  return {
    kind: "discrete",
    field: fieldOption.value,
    fieldLabel: fieldOption.label,
    value: packMatch[1],
    valueLabel: node.label,
    allowedOperators: ["include", "exclude"],
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

  const packTarget = buildPackSelectionTarget(fieldOption, node);
  if (packTarget) {
    return packTarget;
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

  const valueSegments = idSegments.slice(fieldIndex + 1);
  const normalizedValueSegments = valueSegments[0] === "value" ? valueSegments.slice(1) : valueSegments;
  if (normalizedValueSegments.length === 0) {
    return undefined;
  }

  return buildFieldSelectionTarget(fieldOption, node, normalizedValueSegments.join(":"));
}

export function buildSearchFilterExplorerTargetResolver(
  fieldOptions: readonly Pf2eTerminalQueryFieldOption[],
): (node: OntologyNode | undefined) => FilterExplorerComposeTarget | undefined {
  const allowedFields = new Map(fieldOptions.map((field) => [field.value, field]));

  return (node) => {
    if (!node) {
      return undefined;
    }

    const predicate = node.query?.request.filter;
    if (!predicate || (predicate.kind !== "metadataPredicate" && predicate.kind !== "metric" && predicate.kind !== "metricCompare")) {
      return fieldOptions
        .map(
          (fieldOption) =>
            buildMetricScalarTarget(node, fieldOption) ?? buildFallbackFieldSelectionTarget(node, fieldOption),
        )
        .find((target): target is FilterExplorerComposeTarget => Boolean(target));
    }

    if (predicate.kind === "metric") {
      const metricField =
        inferActorMetricValueType(predicate.metric) && allowedFields.has("actorMetric")
          ? "actorMetric"
          : inferItemMetricValueType(predicate.metric) && allowedFields.has("itemMetric")
            ? "itemMetric"
            : null;
      if (!metricField) {
        return undefined;
      }
      const fieldOption = allowedFields.get(metricField);
      if (!fieldOption) {
        return undefined;
      }
      const metricLabel = formatMetricLabel(predicate.metric, node.label);
      const valueType = inferMetricValueType(metricField, predicate.metric);
      if (valueType === "number" && typeof predicate.value === "number") {
        return {
          kind: "scalar",
          key: getMetricSelectionKey(metricField, predicate.metric),
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
        field: getMetricSelectionKey(metricField, predicate.metric),
        fieldLabel: `${fieldOption.label} / ${metricLabel}`,
        value,
        valueLabel: node.label,
        allowedOperators: ["include", "exclude"],
      };
    }

    if (predicate.kind === "metricCompare") {
      const metricField =
        inferActorMetricValueType(predicate.leftMetric) && allowedFields.has("actorMetric")
          ? "actorMetric"
          : inferItemMetricValueType(predicate.leftMetric) && allowedFields.has("itemMetric")
            ? "itemMetric"
            : null;
      if (!metricField) {
        return undefined;
      }
      const fieldOption = allowedFields.get(metricField);
      if (!fieldOption) {
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

    const fieldOption = allowedFields.get(predicate.predicate.field as Pf2eTerminalQueryField);
    if (!fieldOption) {
      return undefined;
    }
    const value = getSelectionValueFromPredicate(predicate.predicate);
    if (value === null) {
      return undefined;
    }
    return buildFieldSelectionTarget(fieldOption, node, value);
  };
}
