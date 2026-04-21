import {
  categorySupportsSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import type { SearchFilters } from "../../domain/search-types.js";
import type { NormalizedSearchFilters } from "../contracts.js";
import { hasStructuredFilterSignal, resolveSearchMode } from "../ranking.js";
import { normalizeMetadataFilterNode } from "./metadata.js";
import { normalizeSearchScope } from "./scope.js";

export type SearchFilterContext = "list" | "search";

export function validateSearchFilters(filters: NormalizedSearchFilters, context: SearchFilterContext): void {
  const mode = resolveSearchMode(filters, context);

  if (context === "list" && filters.searchProfile) {
    throw new Error("searchProfile is only supported for pf2e_search.");
  }

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (context === "list" && filters.query) {
    throw new Error("query is only supported for pf2e_search.");
  }

  if (context === "list" && filters.excludeQuery) {
    throw new Error("excludeQuery is only supported for pf2e_search.");
  }

  if (mode === "structured" && filters.query) {
    throw new Error("query requires a themed search profile such as balanced or concept.");
  }

  if (
    context === "search" &&
    !filters.query?.trim() &&
    !filters.nameQuery?.trim() &&
    !hasStructuredFilterSignal(filters)
  ) {
    throw new Error("pf2e_search requires search text and/or at least one structured filter.");
  }

  if (filters.linksTo !== undefined && filters.linksTo.length === 0) {
    throw new Error("linksTo must contain at least one record key.");
  }

  if (filters.excludeLinksTo !== undefined && filters.excludeLinksTo.length === 0) {
    throw new Error("excludeLinksTo must contain at least one record key.");
  }

  if (filters.linksToMode && (!filters.linksTo || filters.linksTo.length === 0)) {
    throw new Error("linksToMode requires linksTo.");
  }

  if (filters.scopes && filters.scopes.length > 0 && (filters.category || filters.subcategory)) {
    throw new Error("scopes can't be combined with top-level category or subcategory filters.");
  }

  if (filters.category && filters.subcategory && !categorySupportsSubcategory(filters.category, filters.subcategory)) {
    throw new Error(`Subcategory "${filters.subcategory}" does not belong to category "${filters.category}".`);
  }

  if (filters.scopes) {
    for (const scope of filters.scopes) {
      for (const subcategory of scope.subcategories ?? []) {
        if (!categorySupportsSubcategory(scope.category, subcategory)) {
          throw new Error(`Subcategory "${subcategory}" does not belong to category "${scope.category}".`);
        }
      }
    }
  }
}

function normalizeRecordKeyFilter(values: string[] | undefined): string[] | undefined {
  if (values === undefined) {
    return undefined;
  }

  const normalized = values.map((value) => value.trim()).filter((value) => value.length > 0);
  return [...new Set(normalized)];
}

export function normalizeSearchFilters(
  filters: SearchFilters,
  resolvePackName: (packValue: string) => string | undefined,
): NormalizedSearchFilters {
  const normalizedCategory = filters.category !== undefined ? normalizeSearchCategory(filters.category) : null;
  if (filters.category !== undefined && !normalizedCategory) {
    throw new Error(getSearchCategoryErrorMessage(String(filters.category)));
  }

  const normalizedSubcategory =
    filters.subcategory !== undefined ? normalizeSearchSubcategory(filters.subcategory) : null;
  if (filters.subcategory !== undefined && !normalizedSubcategory) {
    throw new Error(getSearchSubcategoryErrorMessage(String(filters.subcategory)));
  }

  const normalizedScopes = filters.scopes?.map((scope) => normalizeSearchScope(scope));

  return {
    ...filters,
    pack: filters.pack ? (resolvePackName(filters.pack) ?? filters.pack) : filters.pack,
    linksTo: normalizeRecordKeyFilter(filters.linksTo),
    excludeLinksTo: normalizeRecordKeyFilter(filters.excludeLinksTo),
    category: normalizedCategory ?? undefined,
    subcategory: normalizedSubcategory ?? undefined,
    metadata: filters.metadata ? normalizeMetadataFilterNode(filters.metadata) : undefined,
    scopes: normalizedScopes,
  };
}
