import type { SearchFilterDiscoveryApplicability } from "../../domain/search-field-domains.js";
import type { SearchRequestMode } from "../../domain/search-request-types.js";
import type { SearchCategory, SearchSubcategory } from "../../domain/search-types.js";

export function createScopedSearchDiscoveryApplicability(
  mode: SearchRequestMode,
  category: SearchCategory | null,
  subcategory: SearchSubcategory | null,
  options: { pack?: string } = {},
): SearchFilterDiscoveryApplicability {
  return {
    mode,
    ...(options.pack ? { pack: options.pack } : {}),
    scopes: category ? [{ category, ...(subcategory ? { subcategory } : {}) }] : [],
  };
}
