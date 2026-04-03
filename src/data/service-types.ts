import type { SearchCategory, SearchFilters, SearchSubcategory } from "../types.js";

export type SqlValue = string | number | bigint | Uint8Array | Buffer | null;

export type NormalizedSearchScope = {
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
};

export type NormalizedSearchFilters = Omit<SearchFilters, "category" | "subcategory" | "scopes"> & {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  scopes?: NormalizedSearchScope[];
};
