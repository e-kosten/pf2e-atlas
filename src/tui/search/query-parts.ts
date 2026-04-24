import type { MetadataFilterNode, MetadataPredicate } from "../../domain/metadata-filter-types.js";
import {
  buildAllOfFilter,
  buildAnyOfFilter,
  findSearchScopeFilter,
  type SearchFilterNode,
} from "../../domain/search-request-types.js";
import { normalizeSearchCategory } from "../../domain/categories.js";
import type { SearchCategory, SearchSubcategory, SearchSubcategoryInput } from "../../domain/search-types.js";

export type Pf2eTerminalQueryPartKind =
  | "subcategory"
  | "levelRange"
  | "rarityPolicy"
  | "actionCostPolicy"
  | "metadataPredicate"
  | "metadataGroup"
  | "metadataNot";

export type Pf2eTerminalQueryPartPolicy<T extends string | number> = {
  any: T[];
  all: T[];
  exclude: T[];
};

export type Pf2eTerminalMetadataPredicateQueryPart = {
  kind: "metadataPredicate";
  predicate: MetadataPredicate;
};

export type Pf2eTerminalMetadataGroupQueryPart = {
  kind: "metadataGroup";
  operator: "and" | "or";
  children: Pf2eTerminalMetadataQueryPart[];
};

export type Pf2eTerminalMetadataNotQueryPart = {
  kind: "metadataNot";
  child: Pf2eTerminalMetadataQueryPart;
};

export type Pf2eTerminalMetadataQueryPart =
  | Pf2eTerminalMetadataPredicateQueryPart
  | Pf2eTerminalMetadataGroupQueryPart
  | Pf2eTerminalMetadataNotQueryPart;

export type Pf2eTerminalQueryPart =
  | {
      kind: "subcategory";
      subcategory: SearchSubcategoryInput;
    }
  | {
      kind: "levelRange";
      levelMin: number | null;
      levelMax: number | null;
    }
  | {
      kind: "rarityPolicy";
      policy: Pf2eTerminalQueryPartPolicy<string>;
    }
  | {
      kind: "actionCostPolicy";
      policy: Pf2eTerminalQueryPartPolicy<number>;
    }
  | Pf2eTerminalMetadataQueryPart;

export function isMetadataQueryPart(part: Pf2eTerminalQueryPart): part is Pf2eTerminalMetadataQueryPart {
  return part.kind === "metadataPredicate" || part.kind === "metadataGroup" || part.kind === "metadataNot";
}

function metadataFilterNodeToPart(node: MetadataFilterNode): Pf2eTerminalMetadataQueryPart {
  if ("and" in node) {
    return {
      kind: "metadataGroup",
      operator: "and",
      children: node.and.map(metadataFilterNodeToPart),
    };
  }

  if ("or" in node) {
    return {
      kind: "metadataGroup",
      operator: "or",
      children: node.or.map(metadataFilterNodeToPart),
    };
  }

  if ("not" in node) {
    return {
      kind: "metadataNot",
      child: metadataFilterNodeToPart(node.not),
    };
  }

  return {
    kind: "metadataPredicate",
    predicate: node,
  };
}

export function metadataFilterNodeToRootQueryParts(node: MetadataFilterNode | null): Pf2eTerminalMetadataQueryPart[] {
  if (!node) {
    return [];
  }

  if ("and" in node) {
    return node.and.map(metadataFilterNodeToPart);
  }

  return [metadataFilterNodeToPart(node)];
}

export function normalizeMetadataQueryPart(
  part: Pf2eTerminalMetadataQueryPart | null,
): Pf2eTerminalMetadataQueryPart | null {
  if (!part) {
    return null;
  }

  if (part.kind === "metadataPredicate") {
    return part;
  }

  if (part.kind === "metadataNot") {
    const child = normalizeMetadataQueryPart(part.child);
    if (!child) {
      return null;
    }
    return {
      kind: "metadataNot",
      child,
    };
  }

  const children = part.children
    .map((child) => normalizeMetadataQueryPart(child))
    .filter((child): child is Pf2eTerminalMetadataQueryPart => Boolean(child));

  if (children.length === 0) {
    return null;
  }

  if (children.length === 1) {
    return children[0]!;
  }

  return {
    kind: "metadataGroup",
    operator: part.operator,
    children,
  };
}

function metadataQueryPartToNode(part: Pf2eTerminalMetadataQueryPart): MetadataFilterNode | null {
  const normalized = normalizeMetadataQueryPart(part);
  if (!normalized) {
    return null;
  }

  if (normalized.kind === "metadataPredicate") {
    return normalized.predicate;
  }

  if (normalized.kind === "metadataNot") {
    const child = metadataQueryPartToNode(normalized.child);
    return child ? { not: child } : null;
  }

  const children = normalized.children
    .map((child) => metadataQueryPartToNode(child))
    .filter((child): child is MetadataFilterNode => Boolean(child));

  if (children.length === 0) {
    return null;
  }

  if (children.length === 1) {
    return children[0]!;
  }

  return normalized.operator === "and" ? { and: children } : { or: children };
}

export function rootMetadataQueryPartsToFilterNode(parts: readonly Pf2eTerminalQueryPart[]): MetadataFilterNode | null {
  const metadataNodes = parts
    .filter(isMetadataQueryPart)
    .map((part) => metadataQueryPartToNode(part))
    .filter((part): part is MetadataFilterNode => Boolean(part));

  if (metadataNodes.length === 0) {
    return null;
  }

  if (metadataNodes.length === 1) {
    return metadataNodes[0]!;
  }

  return { and: metadataNodes };
}

function metadataPredicateToCanonicalFilter(predicate: MetadataPredicate): SearchFilterNode {
  if (predicate.field === "actorMetric" || predicate.field === "itemMetric") {
    return {
      kind: "metric",
      metric: predicate.metric,
      op:
        predicate.op === "=="
          ? "eq"
          : predicate.op === "!="
            ? "notEq"
            : predicate.op === ">"
              ? "gt"
              : predicate.op === ">="
                ? "gte"
                : predicate.op === "<"
                  ? "lt"
                  : "lte",
      value: predicate.value,
    };
  }

  if (predicate.field === "actorMetricCompare" || predicate.field === "itemMetricCompare") {
    return {
      kind: "metricCompare",
      leftMetric: predicate.leftMetric,
      op:
        predicate.op === "=="
          ? "eq"
          : predicate.op === "!="
            ? "notEq"
            : predicate.op === ">"
              ? "gt"
              : predicate.op === ">="
                ? "gte"
                : predicate.op === "<"
                  ? "lt"
                  : "lte",
      rightMetric: predicate.rightMetric,
    };
  }

  if ("values" in predicate) {
    if (predicate.op === "includesAny" || predicate.op === "includesAll") {
      const children = predicate.values.map(
        (value) =>
          ({
            kind: "metadataPredicate",
            predicate: { field: predicate.field, op: "includes", value } as never,
          }) satisfies SearchFilterNode,
      );
      return predicate.op === "includesAll"
        ? { kind: "allOf", children }
        : children.length === 1
          ? children[0]!
          : { kind: "anyOf", children };
    }

    const child = predicate.values.length === 1
      ? ({
          kind: "metadataPredicate",
          predicate: { field: predicate.field, op: "includes", value: predicate.values[0]! } as never,
        } satisfies SearchFilterNode)
      : ({
          kind: "anyOf",
          children: predicate.values.map(
            (value) =>
              ({
                kind: "metadataPredicate",
                predicate: { field: predicate.field, op: "includes", value } as never,
              }) satisfies SearchFilterNode,
          ),
        } satisfies SearchFilterNode);
    return { kind: "not", child };
  }

  if ("min" in predicate && "max" in predicate) {
    return {
      kind: "metadataPredicate",
      predicate: { field: predicate.field, op: "between", min: predicate.min, max: predicate.max },
    };
  }

  return {
    kind: "metadataPredicate",
    predicate: ("value" in predicate ? { field: predicate.field, op: predicate.op, value: predicate.value } : predicate) as never,
  };
}

export function metadataFilterNodeToCanonicalFilter(node: MetadataFilterNode | null): SearchFilterNode | undefined {
  if (!node) {
    return undefined;
  }

  if ("and" in node) {
    return buildAllOfFilter(node.and.map((child) => metadataFilterNodeToCanonicalFilter(child)));
  }

  if ("or" in node) {
    return buildAnyOfFilter(node.or.map((child) => metadataFilterNodeToCanonicalFilter(child)));
  }

  if ("not" in node) {
    const child = metadataFilterNodeToCanonicalFilter(node.not);
    return child ? { kind: "not", child } : undefined;
  }

  return metadataPredicateToCanonicalFilter(node);
}

function buildPolicyLeaf(
  kind: "rarity" | "actionCost",
  value: string | number,
): SearchFilterNode {
  return kind === "rarity"
    ? { kind, match: { kind: "eq", value: value as string } }
    : { kind, match: { kind: "eq", value: value as number } };
}

function buildPolicyFilter(
  kind: "rarity" | "actionCost",
  policy: Pf2eTerminalQueryPartPolicy<string> | Pf2eTerminalQueryPartPolicy<number>,
): SearchFilterNode | undefined {
  const anyFilter = buildAnyOfFilter(policy.any.map((value) => buildPolicyLeaf(kind, value)));
  const allFilter = buildAllOfFilter(policy.all.map((value) => buildPolicyLeaf(kind, value)));
  const excludeChild = buildAnyOfFilter(policy.exclude.map((value) => buildPolicyLeaf(kind, value)));
  const excludeFilter = excludeChild ? ({ kind: "not", child: excludeChild } satisfies SearchFilterNode) : undefined;
  return buildAllOfFilter([anyFilter, allFilter, excludeFilter]);
}

export function legacyQueryPartsToCanonicalFilter(
  category: SearchCategory | null,
  parts: readonly Pf2eTerminalQueryPart[],
): SearchFilterNode | undefined {
  const subcategoryPart = parts.find(
    (part): part is Extract<Pf2eTerminalQueryPart, { kind: "subcategory" }> => part.kind === "subcategory",
  );
  const levelPart = parts.find(
    (part): part is Extract<Pf2eTerminalQueryPart, { kind: "levelRange" }> => part.kind === "levelRange",
  );
  const rarityPart = parts.find(
    (part): part is Extract<Pf2eTerminalQueryPart, { kind: "rarityPolicy" }> => part.kind === "rarityPolicy",
  );
  const actionCostPart = parts.find(
    (part): part is Extract<Pf2eTerminalQueryPart, { kind: "actionCostPolicy" }> => part.kind === "actionCostPolicy",
  );
  const metadataFilter = metadataFilterNodeToCanonicalFilter(rootMetadataQueryPartsToFilterNode(parts));

  return buildAllOfFilter([
    category
      ? {
          kind: "scope",
          category,
          subcategory: subcategoryPart
            ? { kind: "eq", value: subcategoryPart.subcategory }
            : { kind: "any" },
        }
      : undefined,
    levelPart
      ? levelPart.levelMin !== null && levelPart.levelMax !== null
        ? levelPart.levelMin === levelPart.levelMax
          ? { kind: "level", match: { kind: "eq", value: levelPart.levelMin } }
          : {
              kind: "level",
              match: {
                kind: "between",
                min: Math.min(levelPart.levelMin, levelPart.levelMax),
                max: Math.max(levelPart.levelMin, levelPart.levelMax),
              },
            }
        : levelPart.levelMin !== null
          ? { kind: "level", match: { kind: "gte", value: levelPart.levelMin } }
          : levelPart.levelMax !== null
            ? { kind: "level", match: { kind: "lte", value: levelPart.levelMax } }
            : undefined
      : undefined,
    rarityPart ? buildPolicyFilter("rarity", rarityPart.policy) : undefined,
    actionCostPart ? buildPolicyFilter("actionCost", actionCostPart.policy) : undefined,
    metadataFilter,
  ]);
}

function canonicalMetadataPredicateToLegacyNode(
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
      return canonicalMetadataPredicateToLegacyNode(filter.predicate);
    case "anyOf": {
      const children = filter.children.map((child) => canonicalFilterToMetadataNode(child));
      return children.every((child): child is MetadataFilterNode => child !== null) ? { or: children } : null;
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

export function extractLegacyQueryPartsFromCanonicalFilter(
  filter: SearchFilterNode | undefined,
): { category: SearchCategory | null; parts: Pf2eTerminalQueryPart[] } {
  const scope = findSearchScopeFilter(filter);
  const category = scope ? normalizeSearchCategory(scope.category) ?? null : null;
  const parts: Pf2eTerminalQueryPart[] = [];
  const topLevel = filter?.kind === "allOf" ? filter.children : filter ? [filter] : [];
  const metadataChildren: SearchFilterNode[] = [];

  if (scope?.subcategory.kind === "eq") {
    parts.push({ kind: "subcategory", subcategory: scope.subcategory.value as SearchSubcategoryInput });
  }

  for (const child of topLevel) {
    if (child.kind === "scope") {
      continue;
    }

    if (child.kind === "level") {
      parts.push({
        kind: "levelRange",
        levelMin:
          child.match.kind === "eq"
            ? child.match.value
            : child.match.kind === "gte"
              ? child.match.value
              : child.match.kind === "between"
                ? child.match.min
                : null,
        levelMax:
          child.match.kind === "eq"
            ? child.match.value
            : child.match.kind === "lte"
              ? child.match.value
              : child.match.kind === "between"
                ? child.match.max
                : null,
      });
      continue;
    }

    metadataChildren.push(child);
  }

  const metadataNode = canonicalFilterToMetadataNode(
    metadataChildren.length === 1 ? metadataChildren[0]! : buildAllOfFilter(metadataChildren),
  );
  if (metadataNode) {
    parts.push(...metadataFilterNodeToRootQueryParts(metadataNode));
  }

  return { category, parts };
}
