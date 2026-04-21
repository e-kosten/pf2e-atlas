import {
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../../domain/categories.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { SearchCategory, SearchScope, SearchSubcategory } from "../../domain/search-types.js";
import { normalizeText, uniqueSorted } from "../../shared/utils.js";
import type { NormalizedSearchFilters, NormalizedSearchScope } from "../contracts.js";
import { recordMatchesMetadataFilter } from "./metadata.js";

export function normalizeSearchScope(scope: SearchScope): NormalizedSearchScope {
  const category = normalizeSearchCategory(scope.category);
  if (!category) {
    throw new Error(getSearchCategoryErrorMessage(String(scope.category)));
  }

  const subcategories = scope.subcategories?.map((subcategory) => {
    const canonicalSubcategory = normalizeSearchSubcategory(subcategory);
    if (!canonicalSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(subcategory)));
    }
    return canonicalSubcategory;
  });

  const uniqueSubcategories = subcategories ? uniqueSorted(subcategories) : undefined;

  return {
    category,
    subcategories: uniqueSubcategories && uniqueSubcategories.length > 0 ? uniqueSubcategories : undefined,
  };
}

export function resolveEffectiveCategory(
  filters: Pick<NormalizedSearchFilters, "category" | "subcategory" | "scopes">,
): SearchCategory | null {
  if (filters.scopes && filters.scopes.length > 0) {
    return null;
  }

  const inferredCategoryFromSubcategory =
    !filters.category && filters.subcategory ? getCategoryForSubcategory(filters.subcategory) : null;
  return filters.category ?? inferredCategoryFromSubcategory;
}

function recordMatchesScope(record: NormalizedRecord, scope: NormalizedSearchScope): boolean {
  if (record.category !== scope.category) {
    return false;
  }

  if (!scope.subcategories || scope.subcategories.length === 0) {
    return true;
  }

  return record.subcategory !== null && scope.subcategories.includes(record.subcategory);
}

export function recordMatchesFilters(record: NormalizedRecord, filters: NormalizedSearchFilters): boolean {
  if (filters.pack) {
    const normalizedPack = normalizeText(filters.pack);
    if (normalizeText(record.packName) !== normalizedPack && normalizeText(record.packLabel) !== normalizedPack) {
      return false;
    }
  }

  if (filters.scopes && filters.scopes.length > 0) {
    if (!filters.scopes.some((scope) => recordMatchesScope(record, scope))) {
      return false;
    }
  } else {
    const effectiveCategory = resolveEffectiveCategory(filters);
    if (effectiveCategory && record.category !== effectiveCategory) {
      return false;
    }
    if (filters.subcategory && record.subcategory !== filters.subcategory) {
      return false;
    }
  }
  if (filters.levelMin !== undefined && (record.level === null || record.level < filters.levelMin)) {
    return false;
  }
  if (filters.levelMax !== undefined && (record.level === null || record.level > filters.levelMax)) {
    return false;
  }
  if (filters.rarity && normalizeText(record.rarity ?? "") !== normalizeText(filters.rarity)) {
    return false;
  }
  if (filters.priceMin !== undefined && (record.priceCp === null || record.priceCp < filters.priceMin)) {
    return false;
  }
  if (filters.priceMax !== undefined && (record.priceCp === null || record.priceCp > filters.priceMax)) {
    return false;
  }
  if (filters.actionCost !== undefined && record.actionCost !== filters.actionCost) {
    return false;
  }
  if (filters.metadata && !recordMatchesMetadataFilter(record, filters.metadata)) {
    return false;
  }

  return true;
}

export type { NormalizedSearchScope, SearchSubcategory };
