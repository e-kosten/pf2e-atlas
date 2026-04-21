import {
  CATEGORY_SUBCATEGORY_MAP,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import type { MetadataFieldSemantics } from "../../search/filters/semantics.js";
import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";
import {
  createDefaultQuery,
  normalizeSearchQuery,
  splitMetadataTreeIntoParts,
} from "./query-state.js";
import type {
  Pf2eTerminalFacetField,
  Pf2eTerminalSearchQuery,
  SearchServiceDependencies,
} from "./service-types.js";

function normalizeScopedSubcategory(
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
): SearchSubcategory | null {
  if (!category || !subcategory) {
    return null;
  }
  return CATEGORY_SUBCATEGORY_MAP[category].includes(subcategory) ? subcategory : null;
}

export function createSearchQueryFromOntologyQuery(
  query: OntologyNodeQuery,
  dependencies: SearchServiceDependencies,
  fieldSemanticsByName: Map<Pf2eTerminalFacetField, MetadataFieldSemantics>,
): Pf2eTerminalSearchQuery {
  const defaultQuery = createDefaultQuery();
  const category = normalizeSearchCategory(query.filters.category) ?? null;
  const subcategory = normalizeScopedSubcategory(category, normalizeSearchSubcategory(query.filters.subcategory) ?? null);
  const parts: Pf2eTerminalSearchQuery["filters"]["parts"] = [];

  if (subcategory) {
    parts.push({ kind: "subcategory", subcategory });
  }
  if (query.filters.levelMin !== undefined || query.filters.levelMax !== undefined) {
    parts.push({
      kind: "levelRange",
      levelMin: query.filters.levelMin ?? null,
      levelMax: query.filters.levelMax ?? null,
    });
  }
  if (query.filters.rarity) {
    parts.push({
      kind: "rarityPolicy",
      policy: {
        any: [query.filters.rarity],
        all: [],
        exclude: [],
      },
    });
  }
  if (query.filters.actionCost !== undefined) {
    parts.push({
      kind: "actionCostPolicy",
      policy: {
        any: [query.filters.actionCost],
        all: [],
        exclude: [],
      },
    });
  }
  parts.push(...splitMetadataTreeIntoParts(query.filters.metadata ?? null));

  return normalizeSearchQuery(
    {
      ...defaultQuery,
      mode: query.kind === "lookup" ? "lookup" : query.kind === "search" ? "search" : "browse",
      limit: query.filters.limit ?? defaultQuery.limit,
      queryText: query.filters.query ?? query.filters.nameQuery ?? "",
      searchProfile: query.filters.searchProfile ?? defaultQuery.searchProfile,
      sourceLabel: query.label ?? null,
      filters: {
        ...defaultQuery.filters,
        category,
        parts,
      },
    },
    dependencies,
    fieldSemanticsByName,
  );
}
