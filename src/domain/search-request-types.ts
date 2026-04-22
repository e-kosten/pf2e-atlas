import type { MetadataFilterNode, MetadataPredicate } from "../search/filters/types.js";
import type { SearchCategoryInput, SearchProfile, SearchScope, SearchSort, SearchSubcategoryInput } from "./search-types.js";

export type SearchRequestIntent = "browse" | "search" | "lookup";

export type SearchRequestPartKind =
  | "subcategory"
  | "levelRange"
  | "rarityPolicy"
  | "actionCostPolicy"
  | "metadataPredicate"
  | "metadataGroup"
  | "metadataNot";

export type SearchRequestPartPolicy<T extends string | number> = {
  any: T[];
  all: T[];
  exclude: T[];
};

export type SearchRequestMetadataPredicatePart = {
  kind: "metadataPredicate";
  predicate: MetadataPredicate;
};

export type SearchRequestMetadataGroupPart = {
  kind: "metadataGroup";
  operator: "and" | "or";
  children: SearchRequestMetadataPart[];
};

export type SearchRequestMetadataNotPart = {
  kind: "metadataNot";
  child: SearchRequestMetadataPart;
};

export type SearchRequestMetadataPart =
  | SearchRequestMetadataPredicatePart
  | SearchRequestMetadataGroupPart
  | SearchRequestMetadataNotPart;

export type SearchRequestPart =
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
      policy: SearchRequestPartPolicy<string>;
    }
  | {
      kind: "actionCostPolicy";
      policy: SearchRequestPartPolicy<number>;
    }
  | SearchRequestMetadataPart;

export interface SearchRequest {
  intent: SearchRequestIntent;
  text?: string;
  excludeQuery?: string;
  searchProfile?: SearchProfile;
  sort?: SearchSort;
  sortSeed?: number;
  explain?: boolean;
  pack?: string;
  linksTo?: string[];
  linksToMode?: "any" | "all";
  excludeLinksTo?: string[];
  category?: SearchCategoryInput;
  scopes?: SearchScope[];
  parts?: SearchRequestPart[];
  priceMin?: number;
  priceMax?: number;
  offset?: number;
  limit?: number;
}

export function isSearchRequestMetadataPart(part: SearchRequestPart): part is SearchRequestMetadataPart {
  return part.kind === "metadataPredicate" || part.kind === "metadataGroup" || part.kind === "metadataNot";
}

function metadataFilterNodeToPart(node: MetadataFilterNode): SearchRequestMetadataPart {
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

export function metadataFilterNodeToSearchRequestParts(node: MetadataFilterNode | null): SearchRequestMetadataPart[] {
  if (!node) {
    return [];
  }

  if ("and" in node) {
    return node.and.map(metadataFilterNodeToPart);
  }

  return [metadataFilterNodeToPart(node)];
}

export function normalizeSearchRequestMetadataPart(part: SearchRequestMetadataPart | null): SearchRequestMetadataPart | null {
  if (!part) {
    return null;
  }

  if (part.kind === "metadataPredicate") {
    return part;
  }

  if (part.kind === "metadataNot") {
    const child = normalizeSearchRequestMetadataPart(part.child);
    if (!child) {
      return null;
    }
    return {
      kind: "metadataNot",
      child,
    };
  }

  const children = part.children
    .map((child) => normalizeSearchRequestMetadataPart(child))
    .filter((child): child is SearchRequestMetadataPart => Boolean(child));

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

function searchRequestMetadataPartToNode(part: SearchRequestMetadataPart): MetadataFilterNode | null {
  const normalized = normalizeSearchRequestMetadataPart(part);
  if (!normalized) {
    return null;
  }

  if (normalized.kind === "metadataPredicate") {
    return normalized.predicate;
  }

  if (normalized.kind === "metadataNot") {
    const child = searchRequestMetadataPartToNode(normalized.child);
    return child ? { not: child } : null;
  }

  const children = normalized.children
    .map((child) => searchRequestMetadataPartToNode(child))
    .filter((child): child is MetadataFilterNode => Boolean(child));

  if (children.length === 0) {
    return null;
  }

  if (children.length === 1) {
    return children[0]!;
  }

  return normalized.operator === "and" ? { and: children } : { or: children };
}

export function searchRequestPartsToMetadataFilterNode(parts: readonly SearchRequestPart[]): MetadataFilterNode | null {
  const metadataNodes = parts
    .filter(isSearchRequestMetadataPart)
    .map((part) => searchRequestMetadataPartToNode(part))
    .filter((part): part is MetadataFilterNode => Boolean(part));

  if (metadataNodes.length === 0) {
    return null;
  }

  if (metadataNodes.length === 1) {
    return metadataNodes[0]!;
  }

  return { and: metadataNodes };
}
