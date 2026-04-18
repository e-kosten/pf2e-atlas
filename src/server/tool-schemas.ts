import * as z from "zod/v4";

import { ACTOR_METRIC_NUMERIC_OPERATORS, ACTOR_METRIC_SCALAR_OPERATORS } from "../domain/actor-metrics.js";
import {
  VALID_SEARCH_CATEGORY_LIST,
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import {
  FILTER_VALUE_FIELDS,
  METADATA_BOOLEAN_FIELDS,
  METADATA_ENUM_STRING_FIELDS,
  METADATA_NUMBER_FIELDS,
  METADATA_SET_FIELDS,
  METADATA_TEXT_STRING_FIELDS,
  FilterValueField,
  MetadataFilterNode,
  SearchCategory,
  SearchScope,
  SearchSubcategory,
} from "../types.js";

export const CATEGORY_HINT_DESCRIPTION = `Optional top-level category hint. Canonical values: ${VALID_SEARCH_CATEGORY_LIST}. Legacy plural aliases are also accepted.`;

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

export const searchScopeSchema: z.ZodType<SearchScope> = z
  .object({
    category: searchCategorySchema.describe(CATEGORY_HINT_DESCRIPTION),
    subcategories: z
      .array(searchSubcategorySchema)
      .min(1)
      .optional()
      .describe(
        "Optional narrower families inside this scope. Leave unset to include all records in the scope's category.",
      ),
  })
  .transform((scope) => ({
    category: scope.category,
    subcategories: scope.subcategories?.filter((subcategory, index, values) => values.indexOf(subcategory) === index),
  }));

export const searchProfileSchema = z.enum(["lexical", "balanced", "concept"]);
export const sourceCategorySchema = z.enum(["core", "rules", "adventure", "unknown"]);
export const spellKindSchema = z.enum(["focus", "ritual", "cantrip"]);
export const linksToModeSchema = z.enum(["any", "all"]);
export const recordKeyArraySchema = z.array(z.string().trim().min(1)).min(1);
const metadataSetFieldSchema = z.enum(METADATA_SET_FIELDS);
const metadataEnumStringFieldSchema = z.enum(METADATA_ENUM_STRING_FIELDS);
const metadataTextStringFieldSchema = z.enum(METADATA_TEXT_STRING_FIELDS);
const metadataNumberFieldSchema = z.enum(METADATA_NUMBER_FIELDS);
const metadataBooleanFieldSchema = z.enum(METADATA_BOOLEAN_FIELDS);

const metadataSetPredicateSchema = z
  .object({
    field: metadataSetFieldSchema,
    op: z.enum(["includesAny", "includesAll", "excludesAny"]),
    values: z.array(z.string()).min(1),
  })
  .strict();

const metadataEnumStringPredicateSchema = z.union([
  z
    .object({
      field: metadataEnumStringFieldSchema,
      op: z.literal("eq"),
      value: z.string(),
    })
    .strict(),
  z
    .object({
      field: metadataEnumStringFieldSchema,
      op: z.enum(["in", "notIn"]),
      values: z.array(z.string()).min(1),
    })
    .strict(),
]);

const metadataTextStringPredicateSchema = z
  .object({
    field: metadataTextStringFieldSchema,
    op: z.enum(["contains", "notContains"]),
    value: z.string(),
  })
  .strict();

const metadataNumberPredicateSchema = z.union([
  z
    .object({
      field: metadataNumberFieldSchema,
      op: z.enum(["eq", "gte", "lte"]),
      value: z.number(),
    })
    .strict(),
  z
    .object({
      field: metadataNumberFieldSchema,
      op: z.literal("between"),
      min: z.number(),
      max: z.number(),
    })
    .strict(),
]);

const metadataBooleanPredicateSchema = z
  .object({
    field: metadataBooleanFieldSchema,
    op: z.literal("eq"),
    value: z.boolean(),
  })
  .strict();

const actorMetricPredicateSchema = z.union([
  z
    .object({
      field: z.literal("actorMetric"),
      metric: z.string(),
      op: z.enum(ACTOR_METRIC_NUMERIC_OPERATORS),
      value: z.number(),
    })
    .strict(),
  z
    .object({
      field: z.literal("actorMetric"),
      metric: z.string(),
      op: z.enum(ACTOR_METRIC_SCALAR_OPERATORS),
      value: z.union([z.string(), z.boolean()]),
    })
    .strict(),
]);

const actorMetricComparePredicateSchema = z
  .object({
    field: z.literal("actorMetricCompare"),
    leftMetric: z.string(),
    op: z.enum(ACTOR_METRIC_NUMERIC_OPERATORS),
    rightMetric: z.string(),
  })
  .strict();

const itemMetricPredicateSchema = z.union([
  z
    .object({
      field: z.literal("itemMetric"),
      metric: z.string(),
      op: z.enum(ACTOR_METRIC_NUMERIC_OPERATORS),
      value: z.number(),
    })
    .strict(),
  z
    .object({
      field: z.literal("itemMetric"),
      metric: z.string(),
      op: z.enum(ACTOR_METRIC_SCALAR_OPERATORS),
      value: z.union([z.string(), z.boolean()]),
    })
    .strict(),
]);

const itemMetricComparePredicateSchema = z
  .object({
    field: z.literal("itemMetricCompare"),
    leftMetric: z.string(),
    op: z.enum(ACTOR_METRIC_NUMERIC_OPERATORS),
    rightMetric: z.string(),
  })
  .strict();

export const metadataFilterSchema: z.ZodType<MetadataFilterNode> = z.lazy(() =>
  z.union([
    metadataSetPredicateSchema,
    metadataEnumStringPredicateSchema,
    metadataTextStringPredicateSchema,
    metadataNumberPredicateSchema,
    metadataBooleanPredicateSchema,
    actorMetricPredicateSchema,
    actorMetricComparePredicateSchema,
    itemMetricPredicateSchema,
    itemMetricComparePredicateSchema,
    z
      .object({
        and: z.array(metadataFilterSchema).min(2),
      })
      .strict(),
    z
      .object({
        or: z.array(metadataFilterSchema).min(2),
      })
      .strict(),
    z
      .object({
        not: metadataFilterSchema,
      })
      .strict(),
  ]),
);

export const filterValueFieldSchema: z.ZodType<FilterValueField> = z.enum(FILTER_VALUE_FIELDS);
