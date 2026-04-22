import type { MetadataFilterNode, MetadataPredicate } from "../../domain/metadata-filter-types.js";
import { getMetricQueryFieldLabel } from "../../domain/metric-discovery-group-label.js";
import type { SearchCategory } from "../../domain/search-types.js";
import { humanizeIdentifier } from "./service-options.js";

export type SearchMetadataNodeSummary = {
  label: string;
  value: string;
  description: string;
};

export type SearchMetadataTreeEntry = {
  depth: number;
  node: MetadataFilterNode;
  path: number[];
  summary: SearchMetadataNodeSummary;
};

export function isMetadataPredicate(node: MetadataFilterNode): node is MetadataPredicate {
  return !("and" in node) && !("or" in node) && !("not" in node);
}

export function getMetadataNodeChildren(node: MetadataFilterNode): MetadataFilterNode[] {
  if ("and" in node) {
    return node.and;
  }
  if ("or" in node) {
    return node.or;
  }
  if ("not" in node) {
    return [node.not];
  }
  return [];
}

export function getMetadataNodeAtPath(
  node: MetadataFilterNode | null,
  path: readonly number[],
): MetadataFilterNode | null {
  let current = node;
  for (const segment of path) {
    if (!current) {
      return null;
    }
    current = getMetadataNodeChildren(current)[segment] ?? null;
  }
  return current;
}

export function normalizeMetadataNode(node: MetadataFilterNode | null): MetadataFilterNode | null {
  if (!node) {
    return null;
  }

  if ("and" in node) {
    const children = node.and
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { and: children };
  }

  if ("or" in node) {
    const children = node.or
      .map((child) => normalizeMetadataNode(child))
      .filter((child): child is MetadataFilterNode => Boolean(child));
    if (children.length === 0) {
      return null;
    }
    if (children.length === 1) {
      return children[0]!;
    }
    return { or: children };
  }

  if ("not" in node) {
    const child = normalizeMetadataNode(node.not);
    if (!child) {
      return null;
    }
    if ("not" in child) {
      return normalizeMetadataNode(child.not);
    }
    return { not: child };
  }

  return node;
}

export function updateMetadataNodeAtPath(
  node: MetadataFilterNode | null,
  path: readonly number[],
  update: (current: MetadataFilterNode) => MetadataFilterNode | null,
): MetadataFilterNode | null {
  if (!node) {
    return null;
  }

  if (path.length === 0) {
    return normalizeMetadataNode(update(node));
  }

  const [segment, ...rest] = path;
  if (segment === undefined) {
    return normalizeMetadataNode(node);
  }

  if ("and" in node) {
    const children = [...node.and];
    const updatedChild = updateMetadataNodeAtPath(children[segment] ?? null, rest, update);
    if (updatedChild) {
      children[segment] = updatedChild;
    } else {
      children.splice(segment, 1);
    }
    return normalizeMetadataNode({ and: children });
  }

  if ("or" in node) {
    const children = [...node.or];
    const updatedChild = updateMetadataNodeAtPath(children[segment] ?? null, rest, update);
    if (updatedChild) {
      children[segment] = updatedChild;
    } else {
      children.splice(segment, 1);
    }
    return normalizeMetadataNode({ or: children });
  }

  if ("not" in node) {
    if (segment !== 0) {
      return node;
    }
    const updatedChild = updateMetadataNodeAtPath(node.not, rest, update);
    return normalizeMetadataNode(updatedChild ? { not: updatedChild } : null);
  }

  return node;
}

export function appendMetadataNodeAtPath(
  metadata: MetadataFilterNode | null,
  path: readonly number[],
  nextNode: MetadataFilterNode,
): MetadataFilterNode | null {
  if (!metadata) {
    return normalizeMetadataNode(nextNode);
  }

  return updateMetadataNodeAtPath(metadata, path, (current) => {
    if ("and" in current) {
      return { and: [...current.and, nextNode] };
    }
    if ("or" in current) {
      return { or: [...current.or, nextNode] };
    }
    return { and: [current, nextNode] };
  });
}

export function countMetadataPredicates(node: MetadataFilterNode | null): number {
  if (!node) {
    return 0;
  }
  if (isMetadataPredicate(node)) {
    return 1;
  }
  return getMetadataNodeChildren(node).reduce((total, child) => total + countMetadataPredicates(child), 0);
}

export function formatMetadataScalar(value: boolean | number | string): string {
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return typeof value === "number" ? String(value) : humanizeIdentifier(value);
}

export function formatMetadataPredicateValue(node: MetadataPredicate): string {
  if ("metric" in node) {
    return `${node.metric} ${node.op} ${formatMetadataScalar(node.value)}`;
  }
  if ("leftMetric" in node) {
    return `${node.leftMetric} ${node.op} ${node.rightMetric}`;
  }
  if ("values" in node) {
    const values = node.values.map((value) => formatMetadataScalar(value)).join(", ");
    switch (node.op) {
      case "includesAny":
        return `includes any ${values}`;
      case "includesAll":
        return `includes all ${values}`;
      case "excludesAny":
        return `excludes ${values}`;
      case "in":
        return `is one of ${values}`;
      case "notIn":
        return `is not ${values}`;
    }
  }
  if ("min" in node && "max" in node) {
    return `between ${node.min} and ${node.max}`;
  }
  if ("value" in node) {
    switch (node.op) {
      case "contains":
        return `contains ${formatMetadataScalar(node.value)}`;
      case "notContains":
        return `does not contain ${formatMetadataScalar(node.value)}`;
      case "eq":
        return `is ${formatMetadataScalar(node.value)}`;
      case "notEq":
        return `is not ${formatMetadataScalar(node.value)}`;
      case "gte":
        return `>= ${node.value}`;
      case "lte":
        return `<= ${node.value}`;
    }
  }
  return JSON.stringify(node);
}

export function describeMetadataNode(
  node: MetadataFilterNode,
  options: { isRoot?: boolean; rootLabel?: "node" | "query"; category?: SearchCategory | null } = {},
): SearchMetadataNodeSummary {
  const isRoot = options.isRoot ?? false;
  const rootLabel = options.rootLabel ?? "query";
  const category = options.category ?? null;

  if ("and" in node) {
    return {
      label: isRoot && rootLabel === "query" ? "Query Logic" : "AND Group",
      value: `${node.and.length} clause${node.and.length === 1 ? "" : "s"}`,
      description: "Every child clause in this group must match.",
    };
  }

  if ("or" in node) {
    return {
      label: isRoot && rootLabel === "query" ? "Query Logic" : "OR Group",
      value: `${node.or.length} clause${node.or.length === 1 ? "" : "s"}`,
      description: "Any child clause in this group may match.",
    };
  }

  if ("not" in node) {
    return {
      label: isRoot && rootLabel === "query" ? "Query Logic" : "NOT Group",
      value: "1 clause",
      description: "Negate the child clause in this group.",
    };
  }

  const label =
    node.field === "actorMetric"
      ? getMetricQueryFieldLabel("actorMetric", category)
      : node.field === "actorMetricCompare"
        ? getMetricQueryFieldLabel("actorMetricCompare", category)
        : node.field === "itemMetric"
          ? getMetricQueryFieldLabel("itemMetric", category)
          : node.field === "itemMetricCompare"
            ? getMetricQueryFieldLabel("itemMetricCompare", category)
            : humanizeIdentifier(node.field);

  return {
    label: isRoot && rootLabel === "query" ? "Query Clause" : label,
    value: formatMetadataPredicateValue(node),
    description: `Edit or remove this ${label.toLowerCase()} clause.`,
  };
}

export function flattenMetadataTree(
  node: MetadataFilterNode,
  options: {
    depth?: number;
    path?: number[];
    rootLabel?: "node" | "query";
    category?: SearchCategory | null;
  } = {},
): SearchMetadataTreeEntry[] {
  const depth = options.depth ?? 0;
  const path = options.path ?? [];
  const rootLabel = options.rootLabel ?? "query";
  const category = options.category ?? null;
  const entries: SearchMetadataTreeEntry[] = [
    {
      depth,
      node,
      path,
      summary: describeMetadataNode(node, {
        isRoot: path.length === 0,
        rootLabel,
        category,
      }),
    },
  ];

  getMetadataNodeChildren(node).forEach((child, childIndex) => {
    entries.push(
      ...flattenMetadataTree(child, {
        depth: depth + 1,
        path: [...path, childIndex],
        rootLabel: "node",
        category,
      }),
    );
  });

  return entries;
}
