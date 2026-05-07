import type { SearchFilterNode } from "../../domain/search-request-types.js";
import { inferActorMetricValueType } from "../../domain/actor-metrics.js";
import { inferItemMetricValueType } from "../../domain/item-metrics.js";
import { getMetricQueryFieldLabel } from "../../domain/metric-discovery-group-label.js";
import { formatMetadataFieldLabel, humanizeOntologySearchIdentifier } from "../../domain/presentation-vocabulary.js";
import type { SearchCategory } from "../../domain/search-types.js";

export type SearchFilterPresentationAliasStyle = "compact" | "tree";
export type SearchFilterRenderOptions = {
  category?: SearchCategory | null;
  packLabelResolver?: (packValue: string) => string | null | undefined;
};

type MetricQueryFieldPresentation =
  | "actorMetric"
  | "actorMetricCompare"
  | "itemMetric"
  | "itemMetricCompare";

function formatPackPresentationValue(
  packValue: string,
  resolver?: (packValue: string) => string | null | undefined,
): string {
  return resolver?.(packValue) ?? packValue;
}

function resolveMetricQueryFieldPresentation(
  metric: string,
  comparison: boolean,
  category: SearchCategory | null | undefined,
): MetricQueryFieldPresentation {
  const actorValueType = inferActorMetricValueType(metric);
  const itemValueType = inferItemMetricValueType(metric);

  if (actorValueType && !itemValueType) {
    return comparison ? "actorMetricCompare" : "actorMetric";
  }
  if (itemValueType && !actorValueType) {
    return comparison ? "itemMetricCompare" : "itemMetric";
  }

  if (category === "equipment") {
    return comparison ? "itemMetricCompare" : "itemMetric";
  }

  return comparison ? "actorMetricCompare" : "actorMetric";
}

export function isSearchFilterBooleanGroup(
  node: SearchFilterNode,
): node is Extract<SearchFilterNode, { kind: "anyOf"; children: SearchFilterNode[] } | { kind: "allOf"; children: SearchFilterNode[] }> {
  return node.kind === "anyOf" || node.kind === "allOf";
}

function createSearchFilterBooleanGroup(
  operator: "allOf" | "anyOf",
  children: SearchFilterNode[],
): Extract<SearchFilterNode, { kind: "allOf"; children: SearchFilterNode[] } | { kind: "anyOf"; children: SearchFilterNode[] }> {
  return operator === "allOf" ? { kind: "allOf", children } : { kind: "anyOf", children };
}

export function getSearchFilterNodeChildren(node: SearchFilterNode): SearchFilterNode[] {
  if (node.kind === "anyOf" || node.kind === "allOf") {
    return node.children;
  }
  if (node.kind === "not") {
    return [node.child];
  }
  return [];
}

export function getSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): SearchFilterNode | null {
  let current = node ?? null;
  for (const segment of path) {
    if (!current) {
      return null;
    }
    current = getSearchFilterNodeChildren(current)[segment] ?? null;
  }
  return current;
}

export function normalizeSearchFilterNode(node: SearchFilterNode | undefined | null): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if (node.kind === "anyOf" || node.kind === "allOf") {
    const children = node.children
      .map((child) => normalizeSearchFilterNode(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    if (children.length === 0) {
      return undefined;
    }
    return children.length === 1 ? children[0]! : { kind: node.kind, children };
  }

  if (node.kind === "not") {
    const child = normalizeSearchFilterNode(node.child);
    if (!child) {
      return undefined;
    }
    return child.kind === "not" ? normalizeSearchFilterNode(child.child) : { kind: "not", child };
  }

  return node;
}

function normalizeEditableSearchFilterNode(node: SearchFilterNode | undefined | null): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if (node.kind === "anyOf" || node.kind === "allOf") {
    const children = node.children
      .map((child) => normalizeEditableSearchFilterNode(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    return children.length > 0 ? { kind: node.kind, children } : undefined;
  }

  if (node.kind === "not") {
    const child = normalizeEditableSearchFilterNode(node.child);
    return child ? { kind: "not", child } : undefined;
  }

  return node;
}

export function updateSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
  update: (current: SearchFilterNode) => SearchFilterNode | undefined,
): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if (path.length === 0) {
    return normalizeEditableSearchFilterNode(update(node));
  }

  const [segment, ...rest] = path;
  if (segment === undefined) {
    return normalizeEditableSearchFilterNode(node);
  }

  if (node.kind === "anyOf" || node.kind === "allOf") {
    const children = [...node.children];
    const updatedChild = updateSearchFilterNodeAtPath(children[segment], rest, update);
    if (updatedChild) {
      children[segment] = updatedChild;
    } else {
      children.splice(segment, 1);
    }
    return normalizeEditableSearchFilterNode({ kind: node.kind, children });
  }

  if (node.kind === "not") {
    if (segment !== 0) {
      return node;
    }
    const updatedChild = updateSearchFilterNodeAtPath(node.child, rest, update);
    return normalizeEditableSearchFilterNode(updatedChild ? { kind: "not", child: updatedChild } : undefined);
  }

  return node;
}

export function appendSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
  nextNode: SearchFilterNode,
  rootOperator: "allOf" | "anyOf" = "allOf",
): SearchFilterNode | undefined {
  if (!node) {
    return normalizeEditableSearchFilterNode(nextNode);
  }

  if (path.length === 0) {
    if (node.kind === "allOf" || node.kind === "anyOf") {
      return normalizeEditableSearchFilterNode({ kind: node.kind, children: [...node.children, nextNode] });
    }
    return normalizeEditableSearchFilterNode({
      kind: rootOperator,
      children: [node, nextNode],
    });
  }

  return updateSearchFilterNodeAtPath(node, path, (current) => {
    if (current.kind === "allOf" || current.kind === "anyOf") {
      return { kind: current.kind, children: [...current.children, nextNode] };
    }
    return { kind: "allOf", children: [current, nextNode] };
  });
}

export function appendSearchFilterNodesAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
  nextNodes: readonly SearchFilterNode[],
  rootOperator: "allOf" | "anyOf" = "allOf",
): SearchFilterNode | undefined {
  return nextNodes.reduce<SearchFilterNode | undefined>(
    (current, nextNode) => appendSearchFilterNodeAtPath(current, path, nextNode, rootOperator),
    node,
  );
}

export function wrapSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
  wrapper: "allOf" | "anyOf" | "not",
): SearchFilterNode | undefined {
  return updateSearchFilterNodeAtPath(node, path, (current) =>
    wrapper === "not" ? { kind: "not", child: current } : createSearchFilterBooleanGroup(wrapper, [current]),
  );
}

export function reshapeSearchFilterBooleanGroupAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
  operator: "allOf" | "anyOf",
): SearchFilterNode | undefined {
  return updateSearchFilterNodeAtPath(node, path, (current) => {
    if (!isSearchFilterBooleanGroup(current)) {
      return current;
    }
    return createSearchFilterBooleanGroup(operator, [...current.children]);
  });
}

export function removeSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): SearchFilterNode | undefined {
  return updateSearchFilterNodeAtPath(node, path, () => undefined);
}

function extractSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): { nextRoot: SearchFilterNode | undefined; extracted: SearchFilterNode | null } {
  const extracted = getSearchFilterNodeAtPath(node, path);
  return {
    nextRoot: removeSearchFilterNodeAtPath(node, path),
    extracted,
  };
}

function adjustSearchFilterPathForSingleRemoval(targetPath: readonly number[], removedPath: readonly number[]): number[] {
  const nextPath = [...targetPath];
  const removedParentPath = removedPath.slice(0, -1);
  const removedIndex = removedPath[removedPath.length - 1];
  if (removedIndex === undefined || targetPath.length <= removedParentPath.length) {
    return nextPath;
  }

  if (!removedParentPath.every((segment, index) => targetPath[index] === segment)) {
    return nextPath;
  }

  const targetSiblingIndex = targetPath[removedParentPath.length];
  if (targetSiblingIndex !== undefined && targetSiblingIndex > removedIndex) {
    nextPath[removedParentPath.length] = targetSiblingIndex - 1;
  }
  return nextPath;
}

function adjustSearchFilterPathAfterRemoval(
  node: SearchFilterNode | undefined,
  targetPath: readonly number[],
  removedPath: readonly number[],
): number[] {
  let nextPath = [...targetPath];
  let currentRemovedPath = [...removedPath];

  while (currentRemovedPath.length > 0) {
    nextPath = adjustSearchFilterPathForSingleRemoval(nextPath, currentRemovedPath);

    const parentPath = currentRemovedPath.slice(0, -1);
    const parentNode = getSearchFilterNodeAtPath(node, parentPath);
    if (!parentNode) {
      break;
    }

    if (parentNode.kind === "not") {
      currentRemovedPath = parentPath;
      continue;
    }

    if (isSearchFilterBooleanGroup(parentNode) && parentNode.children.length === 1) {
      currentRemovedPath = parentPath;
      continue;
    }

    break;
  }

  return nextPath;
}

export function isValidSearchFilterMoveTargetGroupPath(
  node: SearchFilterNode | undefined,
  sourcePath: readonly number[],
  targetGroupPath: readonly number[],
): boolean {
  if (!node) {
    return targetGroupPath.length === 0;
  }

  if (sourcePath.length === 0) {
    return false;
  }

  if (targetGroupPath.length === 0) {
    return true;
  }

  const targetNode = getSearchFilterNodeAtPath(node, targetGroupPath);
  if (!targetNode || !isSearchFilterBooleanGroup(targetNode)) {
    return false;
  }

  if (targetGroupPath.length >= sourcePath.length) {
    return !sourcePath.every((segment, index) => targetGroupPath[index] === segment);
  }

  return true;
}

export function moveSearchFilterNodeToGroupPath(
  node: SearchFilterNode | undefined,
  sourcePath: readonly number[],
  targetGroupPath: readonly number[],
  rootOperator: "allOf" | "anyOf" = "allOf",
): SearchFilterNode | undefined {
  if (!isValidSearchFilterMoveTargetGroupPath(node, sourcePath, targetGroupPath)) {
    return node;
  }

  const { nextRoot, extracted } = extractSearchFilterNodeAtPath(node, sourcePath);
  if (!extracted) {
    return node;
  }

  const resolvedTargetPath = adjustSearchFilterPathAfterRemoval(node, targetGroupPath, sourcePath);
  return appendSearchFilterNodeAtPath(nextRoot, resolvedTargetPath, extracted, rootOperator);
}

export function canUnwrapSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): boolean {
  const targetNode = getSearchFilterNodeAtPath(node, path);
  const parentNode = getSearchFilterNodeAtPath(node, path.slice(0, -1));
  return Boolean(targetNode && parentNode && isSearchFilterBooleanGroup(targetNode) && isSearchFilterBooleanGroup(parentNode));
}

export function unwrapSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): SearchFilterNode | undefined {
  if (!canUnwrapSearchFilterNodeAtPath(node, path)) {
    return node;
  }

  const targetNode = getSearchFilterNodeAtPath(node, path);
  const parentPath = path.slice(0, -1);
  const targetIndex = path[path.length - 1];
  if (!targetNode || !isSearchFilterBooleanGroup(targetNode) || targetIndex === undefined) {
    return node;
  }

  return updateSearchFilterNodeAtPath(node, parentPath, (current) => {
    if (!isSearchFilterBooleanGroup(current)) {
      return current;
    }
    const children = [...current.children];
    children.splice(targetIndex, 1, ...targetNode.children);
    return { kind: current.kind, children };
  });
}

export function canLiftSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): boolean {
  if (!node || path.length < 2) {
    return false;
  }

  const targetNode = getSearchFilterNodeAtPath(node, path);
  const parentNode = getSearchFilterNodeAtPath(node, path.slice(0, -1));
  const grandparentNode = getSearchFilterNodeAtPath(node, path.slice(0, -2));
  return Boolean(
    targetNode &&
      parentNode &&
      grandparentNode &&
      isSearchFilterBooleanGroup(parentNode) &&
      isSearchFilterBooleanGroup(grandparentNode),
  );
}

export function liftSearchFilterNodeAtPath(
  node: SearchFilterNode | undefined,
  path: readonly number[],
): SearchFilterNode | undefined {
  if (!canLiftSearchFilterNodeAtPath(node, path)) {
    return node;
  }

  const targetNode = getSearchFilterNodeAtPath(node, path);
  const parentPath = path.slice(0, -1);
  const grandparentPath = path.slice(0, -2);
  const parentIndex = path[path.length - 2];
  const targetIndex = path[path.length - 1];
  const parentNode = getSearchFilterNodeAtPath(node, parentPath);
  const grandparentNode = getSearchFilterNodeAtPath(node, grandparentPath);
  if (
    !targetNode ||
    parentIndex === undefined ||
    targetIndex === undefined ||
    !parentNode ||
    !grandparentNode ||
    !isSearchFilterBooleanGroup(parentNode) ||
    !isSearchFilterBooleanGroup(grandparentNode)
  ) {
    return node;
  }

  const parentChildCount = parentNode.children.length;
  return updateSearchFilterNodeAtPath(node, grandparentPath, (current) => {
    if (!isSearchFilterBooleanGroup(current)) {
      return current;
    }

    const currentParent = current.children[parentIndex];
    if (!currentParent || !isSearchFilterBooleanGroup(currentParent)) {
      return current;
    }

    const nextParentChildren = [...currentParent.children];
    nextParentChildren.splice(targetIndex, 1);

    const nextChildren = [...current.children];
    const parentReplacement =
      nextParentChildren.length > 0
        ? ({
            kind: currentParent.kind,
            children: nextParentChildren,
          } satisfies SearchFilterNode)
        : null;

    nextChildren.splice(parentIndex, 1, ...(parentReplacement ? [parentReplacement] : []));
    const insertionIndex = parentReplacement && parentChildCount > 1 ? parentIndex + 1 : parentIndex;
    nextChildren.splice(insertionIndex, 0, targetNode);
    return {
      kind: current.kind,
      children: nextChildren,
    };
  });
}

export function toggleSearchFilterRootGroupOperator(
  node: SearchFilterNode | undefined,
): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if (node.kind === "allOf") {
    return { kind: "anyOf", children: [...node.children] };
  }

  if (node.kind === "anyOf") {
    return { kind: "allOf", children: [...node.children] };
  }

  return { kind: "anyOf", children: [node] };
}

export function isSearchFilterPredicateNode(
  node: SearchFilterNode,
): node is Extract<SearchFilterNode, { kind: "metadataPredicate" } | { kind: "metric" } | { kind: "metricCompare" }> {
  return node.kind === "metadataPredicate" || node.kind === "metric" || node.kind === "metricCompare";
}

export function countSearchFilterPredicateNodes(node: SearchFilterNode | undefined): number {
  if (!node) {
    return 0;
  }
  if (isSearchFilterPredicateNode(node)) {
    return 1;
  }
  return getSearchFilterNodeChildren(node).reduce((total, child) => total + countSearchFilterPredicateNodes(child), 0);
}

function formatSearchNumericMatch(match: Extract<SearchFilterNode, { kind: "level" | "price" }>["match"]): string {
  switch (match.kind) {
    case "eq":
      return `${match.value}`;
    case "gt":
      return `> ${match.value}`;
    case "gte":
      return `>= ${match.value}`;
    case "lt":
      return `< ${match.value}`;
    case "lte":
      return `<= ${match.value}`;
    case "between":
      return `${match.min}-${match.max}`;
  }
}

function formatCanonicalMetricOperator(
  op: Extract<SearchFilterNode, { kind: "metric" }>["op"] | Extract<SearchFilterNode, { kind: "metricCompare" }>["op"],
): string {
  switch (op) {
    case "eq":
      return "=";
    case "notEq":
      return "!=";
    case "gt":
      return ">";
    case "gte":
      return ">=";
    case "lt":
      return "<";
    case "lte":
      return "<=";
  }
}

function formatBooleanGroupAlias(
  operator: "allOf" | "anyOf",
  count: number,
  style: SearchFilterPresentationAliasStyle,
): string {
  const label = operator === "allOf" ? "All of" : "Any of";
  if (style === "compact") {
    return `${label} (${count} filter${count === 1 ? "" : "s"})`;
  }
  return label;
}

function formatNegationAlias(childAlias: string | null): string {
  return childAlias ? `! ${childAlias}` : "!";
}

function formatSearchNullableMatch(match: Extract<SearchFilterNode, { kind: "rarity" }>["match"] | Extract<SearchFilterNode, { kind: "actionCost" }>["match"]): string {
  switch (match.kind) {
    case "eq":
      return humanizeOntologySearchIdentifier(String(match.value));
    case "in":
      return match.values.map((value) => humanizeOntologySearchIdentifier(value)).join(", ");
    case "notIn":
      return `not ${match.values.map((value) => humanizeOntologySearchIdentifier(value)).join(", ")}`;
    case "gt":
      return `> ${match.value}`;
    case "gte":
      return `>= ${match.value}`;
    case "lt":
      return `< ${match.value}`;
    case "lte":
      return `<= ${match.value}`;
    case "between":
      return `${match.min}-${match.max}`;
    case "isNull":
      return "is empty";
    case "isNotNull":
      return "has value";
  }
}

function formatCanonicalMetadataPredicateValue(
  predicate: Extract<SearchFilterNode, { kind: "metadataPredicate" }>["predicate"],
): string {
  if (predicate.op === "includes" && "value" in predicate && predicate.value !== undefined) {
    return `includes ${formatMetadataScalar(predicate.value)}`;
  }
  if ("min" in predicate && "max" in predicate) {
    return `between ${predicate.min} and ${predicate.max}`;
  }
  if ("value" in predicate && predicate.value !== undefined) {
    switch (predicate.op) {
      case "contains":
        return `contains ${formatMetadataScalar(predicate.value)}`;
      case "notContains":
        return `does not contain ${formatMetadataScalar(predicate.value)}`;
      case "eq":
        return `is ${formatMetadataScalar(predicate.value)}`;
      case "notEq":
        return `is not ${formatMetadataScalar(predicate.value)}`;
      case "gt":
        return `> ${predicate.value}`;
      case "gte":
        return `>= ${predicate.value}`;
      case "lt":
        return `< ${predicate.value}`;
      case "lte":
        return `<= ${predicate.value}`;
      case "isNull":
        return "is empty";
      case "isNotNull":
        return "has value";
      case "includes":
      case "between":
        return predicate.op;
    }
  }
  return predicate.op;
}

export function formatSearchFilterNodePresentationAlias(
  node: SearchFilterNode,
  options: SearchFilterRenderOptions & {
    style?: SearchFilterPresentationAliasStyle;
  } = {},
): string {
  const style = options.style ?? "tree";

  switch (node.kind) {
    case "allOf":
      return formatBooleanGroupAlias("allOf", node.children.length, style);
    case "anyOf":
      return formatBooleanGroupAlias("anyOf", node.children.length, style);
    case "not":
      return formatNegationAlias(formatSearchFilterNodePresentationAlias(node.child, { ...options, style }));
    case "scope":
      return `Scope: ${humanizeOntologySearchIdentifier(node.category)}${
        node.subcategory.kind === "eq" ? ` / ${humanizeOntologySearchIdentifier(node.subcategory.value)}` : ""
      }`;
    case "level":
      return `Level: ${formatSearchNumericMatch(node.match)}`;
    case "price":
      return `Price: ${formatSearchNumericMatch(node.match)}`;
    case "rarity":
      return `Rarity: ${formatSearchNullableMatch(node.match)}`;
    case "actionCost":
      return `Action Cost: ${formatSearchNullableMatch(node.match)}`;
    case "pack":
      return `Pack: ${formatPackPresentationValue(node.value, options.packLabelResolver)}`;
    case "linksTo":
      return `Links To: ${node.target}`;
    case "linkedFrom":
      return `Linked From: ${node.source}`;
    case "metadataPredicate": {
      const predicateValue = formatCanonicalMetadataPredicateValue(node.predicate);
      return `${formatMetadataFieldLabel(node.predicate.field)}: ${predicateValue}`;
    }
    case "metric":
      return `${getMetricQueryFieldLabel(
        resolveMetricQueryFieldPresentation(node.metric, false, options.category ?? null),
        options.category ?? null,
      )}: ${node.metric} ${formatCanonicalMetricOperator(node.op)} ${formatMetadataScalar(node.value)}`;
    case "metricCompare":
      return `${getMetricQueryFieldLabel(
        resolveMetricQueryFieldPresentation(node.leftMetric, true, options.category ?? null),
        options.category ?? null,
      )}: ${node.leftMetric} ${formatCanonicalMetricOperator(node.op)} ${node.rightMetric}`;
  }
}

export function formatMetadataScalar(value: boolean | number | string): string {
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return typeof value === "number" ? String(value) : humanizeOntologySearchIdentifier(value);
}
