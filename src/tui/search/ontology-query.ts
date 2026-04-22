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
  const request = query.request;
  const category = normalizeSearchCategory(request.category) ?? null;
  const parts = request.parts ?? [];

  return normalizeSearchQuery(
    {
      ...defaultQuery,
      mode: request.intent,
      limit: request.limit ?? defaultQuery.limit,
      queryText: request.text ?? "",
      searchProfile: request.searchProfile ?? defaultQuery.searchProfile,
      sourceLabel: query.label ?? null,
      filters: {
        ...defaultQuery.filters,
        category,
        parts: parts.map((part) =>
          part.kind === "subcategory"
            ? { kind: "subcategory", subcategory: normalizeScopedSubcategory(category, normalizeSearchSubcategory(part.subcategory) ?? null) ?? part.subcategory }
            : part,
        ),
      },
    },
    dependencies,
    fieldSemanticsByName,
  );
}
