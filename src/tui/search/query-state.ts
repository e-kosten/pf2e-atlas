import {
  CATEGORY_SUBCATEGORY_MAP,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFilterNode,
  MetadataSetField,
} from "../../domain/metadata-filter-types.js";
import type {
  SearchCategory,
  SearchSubcategory,
} from "../../domain/search-types.js";
import {
  isMetadataQueryPart,
  metadataFilterNodeToRootQueryParts,
  rootMetadataQueryPartsToFilterNode,
  type Pf2eTerminalMetadataQueryPart,
  type Pf2eTerminalQueryPart,
} from "./query-parts.js";
import {
  cloneNumberPolicy,
  cloneStringPolicy,
  createEmptyNumberPolicy,
  createEmptyStringPolicy,
  hasNumberPolicy,
  hasStringPolicy,
  normalizeNumberPolicy,
  normalizeQueryFieldPolicy,
  normalizeStringPolicy,
} from "./policies.js";
import type {
  LegacyFacetSelection,
  Pf2eTerminalFacetField,
  Pf2eTerminalFilterValuePolicy,
  Pf2eTerminalSearchFilters,
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchStructuredPart,
  SearchServiceDependencies,
} from "./service-types.js";

export function createDefaultFilters(): Pf2eTerminalSearchFilters {
  return {
    category: null,
    parts: [],
  };
}

export function createDefaultQuery(): Pf2eTerminalSearchQuery {
  return {
    mode: "browse",
    limit: 50,
    queryText: "",
    searchProfile: "balanced",
    sourceLabel: null,
    filters: createDefaultFilters(),
  };
}

export function splitMetadataTreeIntoParts(node: MetadataFilterNode | null): Pf2eTerminalSearchStructuredPart[] {
  return metadataFilterNodeToRootQueryParts(node);
}

export function getSearchQueryCategory(query: Pf2eTerminalSearchQuery): SearchCategory | null {
  return query.filters.category;
}

export function getSearchQueryPart<TKind extends Pf2eTerminalSearchStructuredPart["kind"]>(
  query: Pf2eTerminalSearchQuery,
  kind: TKind,
): Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> | null {
  return (
    query.filters.parts.find(
      (part): part is Extract<Pf2eTerminalSearchStructuredPart, { kind: TKind }> => part.kind === kind,
    ) ?? null
  );
}

export function getSearchQuerySubcategory(query: Pf2eTerminalSearchQuery): SearchSubcategory | null {
  return normalizeSearchSubcategory(getSearchQueryPart(query, "subcategory")?.subcategory) ?? null;
}

export function getSearchQueryLevelRange(query: Pf2eTerminalSearchQuery): {
  levelMin: number | null;
  levelMax: number | null;
} {
  const part = getSearchQueryPart(query, "levelRange");
  return {
    levelMin: part?.levelMin ?? null,
    levelMax: part?.levelMax ?? null,
  };
}

export function getSearchQueryRarityPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<string> {
  return cloneStringPolicy(getSearchQueryPart(query, "rarityPolicy")?.policy ?? createEmptyStringPolicy());
}

export function getSearchQueryActionCostPolicy(query: Pf2eTerminalSearchQuery): Pf2eTerminalFilterValuePolicy<number> {
  return cloneNumberPolicy(getSearchQueryPart(query, "actionCostPolicy")?.policy ?? createEmptyNumberPolicy());
}

export function getSearchQueryMetadataTree(query: Pf2eTerminalSearchQuery): MetadataFilterNode | null {
  return rootMetadataQueryPartsToFilterNode(query.filters.parts) ?? null;
}

export function setSearchQueryCategory(
  query: Pf2eTerminalSearchQuery,
  category: SearchCategory | null,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      category,
      parts: [],
    },
  };
}

export function setSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  part: Exclude<Pf2eTerminalSearchStructuredPart, Pf2eTerminalMetadataQueryPart>,
): Pf2eTerminalSearchQuery {
  const nextParts = query.filters.parts.filter(
    (candidate) => candidate.kind !== part.kind && !isMetadataQueryPart(candidate),
  );
  nextParts.push(part);
  nextParts.push(...query.filters.parts.filter(isMetadataQueryPart));
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: nextParts,
    },
  };
}

export function removeSearchQueryPart(
  query: Pf2eTerminalSearchQuery,
  kind: Exclude<Pf2eTerminalSearchStructuredPart["kind"], Pf2eTerminalMetadataQueryPart["kind"]>,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: query.filters.parts.filter((part) => part.kind !== kind),
    },
  };
}

export function setSearchQueryMetadataTree(
  query: Pf2eTerminalSearchQuery,
  node: MetadataFilterNode | null,
): Pf2eTerminalSearchQuery {
  return {
    ...query,
    filters: {
      ...query.filters,
      parts: [...query.filters.parts.filter((part) => !isMetadataQueryPart(part)), ...splitMetadataTreeIntoParts(node)],
    },
  };
}

export function isActionCostAvailableInScope(
  dependencies: SearchServiceDependencies,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): boolean {
  if (!category) {
    return false;
  }

  return (
    dependencies.listFilterValues({
      field: "actionCost",
      category,
      ...(subcategory ? { subcategory } : {}),
    }).values.length > 0
  );
}

export function buildMetadataNodeForQueryFieldSelection(
  field: Pf2eTerminalFacetField,
  policy: Pf2eTerminalFilterValuePolicy<string>,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  const normalizedPolicy = normalizeQueryFieldPolicy(field, policy, fieldSemanticsByName);
  if (!normalizedPolicy) {
    return null;
  }

  const fieldSemantics = fieldSemanticsByName.get(field);
  if (!fieldSemantics) {
    return null;
  }

  if (fieldSemantics.fieldType === "set") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAny",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.all.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "includesAll",
        values: normalizedPolicy.all,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataSetField,
        op: "excludesAny",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "enumString") {
    const clauses: MetadataFilterNode[] = [];
    if (normalizedPolicy.any.length === 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "eq",
        value: normalizedPolicy.any[0]!,
      });
    } else if (normalizedPolicy.any.length > 1) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "in",
        values: normalizedPolicy.any,
      });
    }
    if (normalizedPolicy.exclude.length > 0) {
      clauses.push({
        field: field as MetadataEnumStringField,
        op: "notIn",
        values: normalizedPolicy.exclude,
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  if (fieldSemantics.fieldType === "boolean") {
    const clauses: MetadataFilterNode[] = [];
    for (const value of normalizedPolicy.any) {
      clauses.push({
        field: field as MetadataBooleanField,
        op: "eq",
        value: value === "true",
      });
    }
    for (const value of normalizedPolicy.exclude) {
      clauses.push({
        not: {
          field: field as MetadataBooleanField,
          op: "eq",
          value: value === "true",
        },
      });
    }
    return clauses.length === 0 ? null : clauses.length === 1 ? clauses[0]! : { and: clauses };
  }

  return null;
}

function buildMetadataNodeForFacet(
  facet: LegacyFacetSelection,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): MetadataFilterNode | null {
  return buildMetadataNodeForQueryFieldSelection(facet.field, facet.policy, fieldSemanticsByName);
}

function buildRootQueryPartsFromLegacyFilters(
  query: Pf2eTerminalSearchQuery,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
  actionCostAvailable: boolean,
): Pf2eTerminalQueryPart[] {
  const parts: Pf2eTerminalQueryPart[] = [];
  const legacyFilters = query.filters as Pf2eTerminalSearchFilters & {
    subcategory?: SearchSubcategory | null;
    levelMin?: number | null;
    levelMax?: number | null;
    rarity?: Pf2eTerminalFilterValuePolicy<string>;
    actionCost?: Pf2eTerminalFilterValuePolicy<number>;
    metadata?: MetadataFilterNode | null;
    facets?: LegacyFacetSelection[];
  };

  if (subcategory) {
    parts.push({
      kind: "subcategory",
      subcategory,
    });
  }

  if (legacyFilters.levelMin != null || legacyFilters.levelMax != null) {
    parts.push({
      kind: "levelRange",
      levelMin: legacyFilters.levelMin ?? null,
      levelMax: legacyFilters.levelMax ?? null,
    });
  }

  const rarityPolicy = normalizeStringPolicy(legacyFilters.rarity, fieldSemanticsByName.get("rarity")?.valueOrdering);
  if (hasStringPolicy(rarityPolicy)) {
    parts.push({
      kind: "rarityPolicy",
      policy: {
        any: rarityPolicy.any,
        all: [],
        exclude: rarityPolicy.exclude,
      },
    });
  }

  const actionCostPolicy = normalizeNumberPolicy(legacyFilters.actionCost);
  if (actionCostAvailable && hasNumberPolicy(actionCostPolicy)) {
    parts.push({
      kind: "actionCostPolicy",
      policy: {
        any: actionCostPolicy.any,
        all: [],
        exclude: actionCostPolicy.exclude,
      },
    });
  }

  for (const facet of legacyFilters.facets ?? []) {
    const facetNode = buildMetadataNodeForFacet(facet, fieldSemanticsByName);
    if (!facetNode) {
      continue;
    }
    parts.push(...metadataFilterNodeToRootQueryParts(facetNode));
  }

  if (legacyFilters.metadata) {
    parts.push(...metadataFilterNodeToRootQueryParts(legacyFilters.metadata));
  }

  return parts;
}

function normalizeRootQueryParts(
  parts: readonly Pf2eTerminalQueryPart[],
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
  category: SearchCategory | null,
  actionCostAvailable: boolean,
): Pf2eTerminalQueryPart[] {
  let subcategoryPart: Extract<Pf2eTerminalQueryPart, { kind: "subcategory" }> | null = null;
  let levelRangePart: Extract<Pf2eTerminalQueryPart, { kind: "levelRange" }> | null = null;
  let rarityPolicyPart: Extract<Pf2eTerminalQueryPart, { kind: "rarityPolicy" }> | null = null;
  let actionCostPolicyPart: Extract<Pf2eTerminalQueryPart, { kind: "actionCostPolicy" }> | null = null;
  const metadataParts: Pf2eTerminalQueryPart[] = [];

  for (const part of parts) {
    switch (part.kind) {
      case "subcategory": {
        if (!category) {
          continue;
        }
        const normalizedSubcategory = normalizeSearchSubcategory(part.subcategory);
        if (!normalizedSubcategory || !CATEGORY_SUBCATEGORY_MAP[category].includes(normalizedSubcategory)) {
          continue;
        }
        subcategoryPart = {
          kind: "subcategory",
          subcategory: normalizedSubcategory,
        };
        continue;
      }
      case "levelRange": {
        const levelMin = part.levelMin ?? null;
        const levelMax = part.levelMax ?? null;
        if (levelMin === null && levelMax === null) {
          continue;
        }
        levelRangePart = {
          kind: "levelRange",
          levelMin: levelMin !== null && levelMax !== null ? Math.min(levelMin, levelMax) : levelMin,
          levelMax: levelMin !== null && levelMax !== null ? Math.max(levelMin, levelMax) : levelMax,
        };
        continue;
      }
      case "rarityPolicy": {
        const normalizedPolicy = normalizeStringPolicy(part.policy, fieldSemanticsByName.get("rarity")?.valueOrdering);
        if (!hasStringPolicy(normalizedPolicy)) {
          continue;
        }
        rarityPolicyPart = {
          kind: "rarityPolicy",
          policy: {
            any: normalizedPolicy.any,
            all: [],
            exclude: normalizedPolicy.exclude,
          },
        };
        continue;
      }
      case "actionCostPolicy": {
        if (!actionCostAvailable) {
          continue;
        }
        const normalizedPolicy = normalizeNumberPolicy(part.policy);
        if (!hasNumberPolicy(normalizedPolicy)) {
          continue;
        }
        actionCostPolicyPart = {
          kind: "actionCostPolicy",
          policy: {
            any: normalizedPolicy.any,
            all: [],
            exclude: normalizedPolicy.exclude,
          },
        };
        continue;
      }
      case "metadataPredicate":
      case "metadataGroup":
      case "metadataNot": {
        if (isMetadataQueryPart(part)) {
          metadataParts.push(part);
        }
      }
    }
  }

  return [
    ...(subcategoryPart ? [subcategoryPart] : []),
    ...(levelRangePart ? [levelRangePart] : []),
    ...(rarityPolicyPart ? [rarityPolicyPart] : []),
    ...(actionCostPolicyPart ? [actionCostPolicyPart] : []),
    ...metadataParts,
  ];
}

export function normalizeSearchQuery(
  query: Pf2eTerminalSearchQuery,
  dependencies: SearchServiceDependencies,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  const category = normalizeSearchCategory(query.filters.category) ?? null;
  const legacyFilters = query.filters as Pf2eTerminalSearchFilters & {
    subcategory?: SearchSubcategory | null;
  };
  const currentParts = Array.isArray(query.filters.parts) ? query.filters.parts : [];
  const partSubcategory =
    category && currentParts.length > 0
      ? normalizeSearchSubcategory(currentParts.find((part) => part.kind === "subcategory")?.subcategory ?? null)
      : null;
  const legacySubcategory =
    category && legacyFilters.subcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(legacyFilters.subcategory)
      ? legacyFilters.subcategory
      : null;
  const scopedSubcategory =
    category && partSubcategory && CATEGORY_SUBCATEGORY_MAP[category].includes(partSubcategory)
      ? partSubcategory
      : legacySubcategory;
  const actionCostAvailable = isActionCostAvailableInScope(dependencies, category, scopedSubcategory);
  const nextParts = normalizeRootQueryParts(
    currentParts.length > 0
      ? currentParts
      : buildRootQueryPartsFromLegacyFilters(
          query,
          fieldSemanticsByName,
          category,
          scopedSubcategory,
          actionCostAvailable,
      ),
    fieldSemanticsByName,
    category,
    actionCostAvailable,
  );

  return {
    ...query,
    queryText: query.queryText.trim(),
    sourceLabel: query.sourceLabel?.trim() || null,
    filters: {
      category,
      parts: nextParts,
    },
  };
}
