import type { MetadataFilterNode, MetadataPredicate } from "../../domain/metadata-types.js";
import type { SearchSubcategory } from "../../domain/search-types.js";

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
      subcategory: SearchSubcategory;
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

function metadataFilterNodeToQueryPart(node: MetadataFilterNode): Pf2eTerminalMetadataQueryPart {
  if ("and" in node) {
    return {
      kind: "metadataGroup",
      operator: "and",
      children: node.and.map(metadataFilterNodeToQueryPart),
    };
  }

  if ("or" in node) {
    return {
      kind: "metadataGroup",
      operator: "or",
      children: node.or.map(metadataFilterNodeToQueryPart),
    };
  }

  if ("not" in node) {
    return {
      kind: "metadataNot",
      child: metadataFilterNodeToQueryPart(node.not),
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
    return node.and.map(metadataFilterNodeToQueryPart);
  }

  return [metadataFilterNodeToQueryPart(node)];
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

function metadataQueryPartToFilterNode(part: Pf2eTerminalMetadataQueryPart): MetadataFilterNode | null {
  const normalized = normalizeMetadataQueryPart(part);
  if (!normalized) {
    return null;
  }

  if (normalized.kind === "metadataPredicate") {
    return normalized.predicate;
  }

  if (normalized.kind === "metadataNot") {
    const child = metadataQueryPartToFilterNode(normalized.child);
    return child ? { not: child } : null;
  }

  const children = normalized.children
    .map((child) => metadataQueryPartToFilterNode(child))
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
    .map((part) => metadataQueryPartToFilterNode(part))
    .filter((part): part is MetadataFilterNode => Boolean(part));

  if (metadataNodes.length === 0) {
    return null;
  }

  if (metadataNodes.length === 1) {
    return metadataNodes[0]!;
  }

  return { and: metadataNodes };
}
