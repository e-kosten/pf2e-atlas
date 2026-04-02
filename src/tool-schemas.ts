import * as z from "zod/v4";

import {
  CATEGORY_SUBCATEGORY_MAP,
  SEARCH_CATEGORIES,
  VALID_SEARCH_CATEGORY_LIST,
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "./categories.js";
import { FilterValueField, SearchCategory, SearchScope, SearchSubcategory } from "./types.js";

export const CATEGORY_HINT_DESCRIPTION =
  `Optional top-level category hint. Canonical values: ${VALID_SEARCH_CATEGORY_LIST}. Legacy plural aliases are also accepted.`;

export const SUBCATEGORY_HINT_DESCRIPTION =
  "Optional narrower hint within a category. Usually leave unset unless you want a specific family such as action, condition, haunt, consumable, or archetype. Legacy plural aliases are also accepted.";

export const SCOPES_HINT_DESCRIPTION =
  "Optional advanced multi-family filter. Use this when you need multiple category/subcategory pairs, such as feat/archetype plus rule/action.";

export const searchCategorySchema: z.ZodType<SearchCategory> = z.string().transform((value, ctx) => {
  const canonicalCategory = normalizeSearchCategory(value);
  if (canonicalCategory) {
    return canonicalCategory;
  }

  const subcategoryMatch = normalizeSearchSubcategory(value);
  const inferredCategory = subcategoryMatch ? getCategoryForSubcategory(subcategoryMatch) : null;
  ctx.addIssue({
    code: "custom",
    message: inferredCategory
      ? `Unknown top-level category "${value}". Try category:"${inferredCategory}", subcategory:"${subcategoryMatch}".`
      : getSearchCategoryErrorMessage(value),
  });
  return z.NEVER;
});

export const searchSubcategorySchema: z.ZodType<SearchSubcategory> = z.string().transform((value, ctx) => {
  const canonicalSubcategory = normalizeSearchSubcategory(value);
  if (canonicalSubcategory) {
    return canonicalSubcategory;
  }

  ctx.addIssue({
    code: "custom",
    message: getSearchSubcategoryErrorMessage(value),
  });
  return z.NEVER;
});

export const searchScopeSchema: z.ZodType<SearchScope> = z.object({
  category: searchCategorySchema.describe(CATEGORY_HINT_DESCRIPTION),
  subcategories: z.array(searchSubcategorySchema).min(1).optional().describe(
    "Optional narrower families inside this scope. Leave unset to include all records in the scope's category.",
  ),
}).transform((scope) => ({
  category: scope.category,
  subcategories: scope.subcategories?.filter((subcategory, index, values) => values.indexOf(subcategory) === index),
}));

export const searchProfileSchema = z.enum(["lexical", "balanced", "concept"]);
export const sourceCategorySchema = z.enum(["core", "rules", "adventure", "unknown"]);
export const spellKindSchema = z.enum(["focus", "ritual", "cantrip"]);
export const filterValueFieldSchema: z.ZodType<FilterValueField> = z.enum([
  "traits",
  "derivedTags",
  "rarity",
  "size",
  "publicationTitle",
  "traditions",
  "spellKinds",
  "sources",
  "categories",
  "subcategories",
  "packs",
]);
