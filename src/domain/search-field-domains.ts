import type { FilterValueOrdering } from "./filter-value-ordering.js";
import {
  buildAllOfFilter,
  buildAnyOfFilter,
  buildScopeFilter,
  type SearchFilterNode,
  type SearchRequest,
  type SearchRequestMode,
} from "./search-request-types.js";
import type { SearchCategoryInput, SearchSubcategoryInput } from "./search-types.js";

export type SearchValueOrderingSpec =
  | { kind: "alpha" }
  | { kind: "countDescThenAlpha" }
  | { kind: "numericAsc" }
  | { kind: "declared" };

export type SearchValueDomainSpec =
  | {
      kind: "closedEnum";
      values: readonly string[];
      normalization: "lowercaseTrim" | "normalizedText" | "custom";
      ordering?: SearchValueOrderingSpec;
      unknownPolicy: "reject" | "drop" | "preserve";
    }
  | {
      kind: "boundedNumber";
      values: readonly number[];
      ordering?: SearchValueOrderingSpec;
      unknownPolicy: "reject" | "drop" | "preserve";
    }
  | {
      kind: "openString";
      normalization: "lowercaseTrim" | "normalizedText" | "derivedTag" | "custom";
      ordering?: SearchValueOrderingSpec;
    }
  | {
      kind: "freeText";
      normalization: "normalizedText" | "custom";
      ordering?: SearchValueOrderingSpec;
    };

export type SearchPromotedFieldDomainKey = "rarity" | "actionCost";

export type SearchPromotedFieldDomainSpec = {
  filterKind: SearchPromotedFieldDomainKey;
  valueDomain: SearchValueDomainSpec;
};

export const SEARCH_PROMOTED_FIELD_DOMAINS = {
  rarity: {
    filterKind: "rarity",
    valueDomain: {
      kind: "closedEnum",
      values: ["common", "uncommon", "rare", "unique"],
      normalization: "lowercaseTrim",
      ordering: { kind: "declared" },
      unknownPolicy: "reject",
    },
  },
  actionCost: {
    filterKind: "actionCost",
    valueDomain: {
      kind: "boundedNumber",
      values: [0, 1, 2, 3],
      ordering: { kind: "declared" },
      unknownPolicy: "reject",
    },
  },
} as const satisfies Record<SearchPromotedFieldDomainKey, SearchPromotedFieldDomainSpec>;

export function getSearchPromotedFieldDomain(field: SearchPromotedFieldDomainKey): SearchPromotedFieldDomainSpec {
  return SEARCH_PROMOTED_FIELD_DOMAINS[field];
}

export function isSearchPromotedFieldDomainKey(value: string): value is SearchPromotedFieldDomainKey {
  return value === "rarity" || value === "actionCost";
}

export function getSearchPromotedFieldValueOrdering(
  field: SearchPromotedFieldDomainKey,
): FilterValueOrdering | undefined {
  const valueDomain = getSearchPromotedFieldDomain(field).valueDomain;
  const ordering = valueDomain.ordering;
  if (!ordering) {
    return undefined;
  }

  switch (ordering.kind) {
    case "alpha":
      return { kind: "alpha" };
    case "countDescThenAlpha":
      return { kind: "countDescThenAlpha" };
    case "numericAsc":
      return { kind: "numericAsc" };
    case "declared":
      return {
        kind: "canonical",
        order:
          valueDomain.kind === "closedEnum" || valueDomain.kind === "boundedNumber"
            ? valueDomain.values.map((value) => String(value))
            : [],
      };
  }
}

type NormalizePromotedFieldOptions = {
  onInvalid?: "throw" | "null";
};

function handleInvalidPromotedFieldValue(message: string, options?: NormalizePromotedFieldOptions): null {
  if (options?.onInvalid === "null") {
    return null;
  }

  throw new Error(message);
}

export function normalizeSearchPromotedStringValue(
  field: "rarity",
  value: string,
  options?: NormalizePromotedFieldOptions,
): string | null {
  const domain = getSearchPromotedFieldDomain(field).valueDomain;
  if (domain.kind !== "closedEnum") {
    return value;
  }

  const normalized = value.trim().toLowerCase();
  if (!domain.values.includes(normalized)) {
    return handleInvalidPromotedFieldValue(`Unknown ${field} value "${value}".`, options);
  }

  return normalized;
}

export function normalizeSearchPromotedNumberValue(
  field: "actionCost",
  value: number,
  options?: NormalizePromotedFieldOptions,
): number | null {
  const domain = getSearchPromotedFieldDomain(field).valueDomain;
  if (domain.kind !== "boundedNumber") {
    return value;
  }

  if (!Number.isFinite(value)) {
    return handleInvalidPromotedFieldValue(`${field} must be a finite number.`, options);
  }

  if (!domain.values.includes(value)) {
    return handleInvalidPromotedFieldValue(`Unsupported ${field} value "${value}".`, options);
  }

  return value;
}

export type SearchFilterDiscoveryMode = "matching" | "catalog";

export type SearchFilterDiscoveryApplicability = {
  mode: SearchRequestMode;
  pack?: string;
  scopes: Array<{
    category: string;
    subcategory?: string | null;
  }>;
};

export type SearchFilterDiscoveryTarget = {
  field: string;
};

export type SearchFilterDiscoveryOption = {
  id: string;
  value: string | number;
  count: number;
};

export type SearchFilterDiscoveryRequest = {
  mode: SearchFilterDiscoveryMode;
  context: SearchFilterDiscoveryContext;
  target: SearchFilterDiscoveryTarget;
};

export type SearchFilterDiscoveryResult = {
  mode: SearchFilterDiscoveryMode;
  target: SearchFilterDiscoveryTarget;
  options: SearchFilterDiscoveryOption[];
};

export type SearchFilterDiscoveryContext = {
  request: Readonly<SearchRequest>;
  applicability: SearchFilterDiscoveryApplicability;
};

function buildApplicabilityScopeKey(scope: {
  category: string;
  subcategory?: string | null;
}): string {
  return `${scope.category}|${scope.subcategory ?? ""}`;
}

function addApplicabilityScope(
  scopes: Map<string, SearchFilterDiscoveryApplicability["scopes"][number]>,
  category: string,
  subcategory?: string | null,
): void {
  const scope = subcategory === undefined ? { category } : { category, subcategory };
  scopes.set(buildApplicabilityScopeKey(scope), scope);
}

function collectDiscoveryApplicability(
  filter: SearchFilterNode | undefined,
  scopes: Map<string, SearchFilterDiscoveryApplicability["scopes"][number]>,
  packs: Set<string>,
): void {
  if (!filter) {
    return;
  }

  switch (filter.kind) {
    case "pack":
      packs.add(filter.value);
      return;
    case "scope":
      addApplicabilityScope(
        scopes,
        filter.category,
        filter.subcategory.kind === "eq"
          ? filter.subcategory.value
          : filter.subcategory.kind === "isNull"
            ? null
            : undefined,
      );
      return;
    case "allOf":
    case "anyOf":
      for (const child of filter.children) {
        collectDiscoveryApplicability(child, scopes, packs);
      }
      return;
    case "not":
      return;
    default:
      return;
  }
}

export function extractSearchFilterDiscoveryApplicability(
  request: Readonly<SearchRequest>,
): SearchFilterDiscoveryApplicability {
  const scopes = new Map<string, SearchFilterDiscoveryApplicability["scopes"][number]>();
  const packs = new Set<string>();
  collectDiscoveryApplicability(request.filter, scopes, packs);
  const [pack] = packs;

  return {
    mode: request.mode,
    ...(pack ? { pack } : {}),
    scopes: [...scopes.values()],
  };
}

export function createSearchFilterDiscoveryContext(
  request: Readonly<SearchRequest>,
): SearchFilterDiscoveryContext {
  return {
    request,
    applicability: extractSearchFilterDiscoveryApplicability(request),
  };
}

export function buildSearchFilterDiscoveryApplicabilityFilter(
  applicability: SearchFilterDiscoveryApplicability,
): SearchFilterNode | undefined {
  return buildAllOfFilter([
    applicability.pack ? { kind: "pack", value: applicability.pack } : undefined,
    applicability.scopes.length > 0
      ? buildAnyOfFilter(
          applicability.scopes.map((scope) =>
            scope.subcategory === null
              ? ({
                  kind: "scope",
                  category: scope.category as SearchCategoryInput,
                  subcategory: { kind: "isNull" },
                } satisfies SearchFilterNode)
              : buildScopeFilter(
                  scope.category as SearchCategoryInput,
                  (scope.subcategory ?? undefined) as SearchSubcategoryInput | undefined,
                ),
          ),
        )
      : undefined,
  ]);
}

export function buildSearchFilterDiscoveryCatalogRequest(
  applicability: SearchFilterDiscoveryApplicability,
): SearchRequest {
  return {
    mode: "browse",
    filter: buildSearchFilterDiscoveryApplicabilityFilter(applicability),
  };
}
