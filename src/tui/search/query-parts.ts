import type { MetadataFilterNode } from "../../domain/metadata-filter-types.js";
import type { SearchFilterNode } from "../../domain/search-request-types.js";

function metadataPredicateToCanonicalFilter(node: Exclude<MetadataFilterNode, { and: MetadataFilterNode[] } | { or: MetadataFilterNode[] } | { not: MetadataFilterNode }>): SearchFilterNode {
  if (node.field === "actorMetric" || node.field === "itemMetric") {
    return {
      kind: "metric",
      metric: node.metric,
      op:
        node.op === "=="
          ? "eq"
          : node.op === "!="
            ? "notEq"
            : node.op === ">"
              ? "gt"
              : node.op === ">="
                ? "gte"
                : node.op === "<"
                  ? "lt"
                  : "lte",
      value: node.value,
    };
  }

  if (node.field === "actorMetricCompare" || node.field === "itemMetricCompare") {
    return {
      kind: "metricCompare",
      leftMetric: node.leftMetric,
      op:
        node.op === "=="
          ? "eq"
          : node.op === "!="
            ? "notEq"
            : node.op === ">"
              ? "gt"
              : node.op === ">="
                ? "gte"
                : node.op === "<"
                  ? "lt"
                  : "lte",
      rightMetric: node.rightMetric,
    };
  }

  if ("values" in node) {
    if (node.op === "includesAny" || node.op === "includesAll") {
      const children = node.values.map(
        (value) =>
          ({
            kind: "metadataPredicate",
            predicate: { field: node.field, op: "includes", value } as never,
          }) satisfies SearchFilterNode,
      );
      return node.op === "includesAll"
        ? { kind: "allOf", children }
        : children.length === 1
          ? children[0]!
          : { kind: "anyOf", children };
    }

    const child =
      node.values.length === 1
        ? ({
            kind: "metadataPredicate",
            predicate: { field: node.field, op: "includes", value: node.values[0]! } as never,
          } satisfies SearchFilterNode)
        : ({
            kind: "anyOf",
            children: node.values.map(
              (value) =>
                ({
                  kind: "metadataPredicate",
                  predicate: { field: node.field, op: "includes", value } as never,
                }) satisfies SearchFilterNode,
            ),
          } satisfies SearchFilterNode);
    return { kind: "not", child };
  }

  if ("min" in node && "max" in node) {
    return {
      kind: "metadataPredicate",
      predicate: { field: node.field, op: "between", min: node.min, max: node.max } as never,
    };
  }

  return {
    kind: "metadataPredicate",
    predicate: ("value" in node ? { field: node.field, op: node.op, value: node.value } : node) as never,
  };
}

export function metadataFilterNodeToCanonicalFilter(node: MetadataFilterNode | null): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if ("and" in node) {
    const children = node.and
      .map((child) => metadataFilterNodeToCanonicalFilter(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    if (children.length === 0) {
      return undefined;
    }
    return children.length === 1 ? children[0]! : { kind: "allOf", children };
  }

  if ("or" in node) {
    const children = node.or
      .map((child) => metadataFilterNodeToCanonicalFilter(child))
      .filter((child): child is SearchFilterNode => Boolean(child));
    if (children.length === 0) {
      return undefined;
    }
    return children.length === 1 ? children[0]! : { kind: "anyOf", children };
  }

  if ("not" in node) {
    const child = metadataFilterNodeToCanonicalFilter(node.not);
    return child ? { kind: "not", child } : undefined;
  }

  return metadataPredicateToCanonicalFilter(node);
}

function canonicalMetadataPredicateToMetadataNode(
  predicate: Extract<SearchFilterNode, { kind: "metadataPredicate" }>["predicate"],
): MetadataFilterNode {
  if ("min" in predicate && "max" in predicate) {
    return {
      field: predicate.field,
      op: "between",
      min: predicate.min!,
      max: predicate.max!,
    };
  }

  if (predicate.op === "includes") {
    return {
      field: predicate.field,
      op: "includesAny",
      values: [predicate.value],
    } as MetadataFilterNode;
  }

  return {
    field: predicate.field,
    op: predicate.op,
    ...("value" in predicate ? { value: predicate.value } : null),
  } as MetadataFilterNode;
}

export function canonicalFilterToMetadataNode(filter: SearchFilterNode | undefined): MetadataFilterNode | null {
  if (!filter) {
    return null;
  }

  switch (filter.kind) {
    case "metric":
      return {
        field: filter.metric.startsWith("attributes.") || filter.metric.startsWith("speed.")
          ? "actorMetric"
          : "itemMetric",
        metric: filter.metric,
        op:
          filter.op === "eq"
            ? "=="
            : filter.op === "notEq"
              ? "!="
              : filter.op === "gt"
                ? ">"
                : filter.op === "gte"
                  ? ">="
                  : filter.op === "lt"
                    ? "<"
                    : "<=",
        value: filter.value,
      } as MetadataFilterNode;
    case "metricCompare":
      return {
        field: filter.leftMetric.startsWith("attributes.") || filter.leftMetric.startsWith("speed.")
          ? "actorMetricCompare"
          : "itemMetricCompare",
        leftMetric: filter.leftMetric,
        op:
          filter.op === "eq"
            ? "=="
            : filter.op === "notEq"
              ? "!="
              : filter.op === "gt"
                ? ">"
                : filter.op === "gte"
                  ? ">="
                  : filter.op === "lt"
                    ? "<"
                    : "<=",
        rightMetric: filter.rightMetric,
      } as MetadataFilterNode;
    case "metadataPredicate":
      return canonicalMetadataPredicateToMetadataNode(filter.predicate);
    case "anyOf": {
      const children = filter.children.map((child) => canonicalFilterToMetadataNode(child));
      if (!children.every((child): child is MetadataFilterNode => child !== null)) {
        return null;
      }
      return children.length === 1 ? children[0]! : { or: children };
    }
    case "allOf": {
      const children = filter.children
        .map((child) => canonicalFilterToMetadataNode(child))
        .filter((child): child is MetadataFilterNode => child !== null);
      if (children.length === 0) {
        return null;
      }
      return children.length === 1 ? children[0]! : { and: children };
    }
    case "not": {
      const child = canonicalFilterToMetadataNode(filter.child);
      return child ? { not: child } : null;
    }
    default:
      return null;
  }
}
