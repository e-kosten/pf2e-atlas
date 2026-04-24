import type { SearchRequestMode } from "./search-request-types.js";

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
  applicability: SearchFilterDiscoveryApplicability;
  target: SearchFilterDiscoveryTarget;
};

export type SearchFilterDiscoveryResult = {
  mode: SearchFilterDiscoveryMode;
  target: SearchFilterDiscoveryTarget;
  options: SearchFilterDiscoveryOption[];
};
