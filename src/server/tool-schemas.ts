import * as z from "zod/v4";

import {
  VALID_SEARCH_CATEGORY_LIST,
  getCategoryForSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import {
  ACTOR_METRIC_COMPARE_PREDICATE_SPEC,
  ACTOR_METRIC_PREDICATE_SPEC,
  ITEM_METRIC_COMPARE_PREDICATE_SPEC,
  ITEM_METRIC_PREDICATE_SPEC,
  METADATA_FIELD_PREDICATE_VARIANTS,
  type MetadataMetricComparePredicateSpec,
  type MetadataMetricValuePredicateSpec,
  type MetadataPredicatePayloadKind,
  type MetadataPredicateVariantSpec,
} from "../domain/metadata-predicate-spec.js";
import type { MetadataFilterNode } from "../domain/metadata-filter-types.js";
import type { MetadataAtomicPredicate } from "../domain/search-filter-metadata.js";
import type { BrowseSortSpec, SearchFilterNode } from "../domain/search-request-types.js";
import {
  METADATA_BOOLEAN_FIELDS,
  METADATA_ENUM_STRING_FIELDS,
  METADATA_NUMBER_FIELDS,
  METADATA_SET_FIELDS,
  METADATA_TEXT_STRING_FIELDS,
} from "../domain/metadata-field-types.js";
import { FILTER_VALUE_FIELDS } from "../domain/search-types.js";
import type {
  FilterValueField,
  SearchCategory,
  SearchScope,
  SearchSubcategory,
} from "../domain/search-types.js";
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

const METADATA_PREDICATE_PAYLOAD_SCHEMAS = {
  string: z.object({ value: z.string() }),
  stringArray: z.object({ values: z.array(z.string()).min(1) }),
  number: z.object({ value: z.number() }),
  numberRange: z.object({ min: z.number(), max: z.number() }),
  boolean: z.object({ value: z.boolean() }),
} as const satisfies Record<MetadataPredicatePayloadKind, z.ZodObject<z.ZodRawShape>>;

function buildStrictObjectSchema(shape: z.ZodRawShape): z.ZodObject<z.ZodRawShape> {
  return z.object(shape).strict();
}

function buildVariantPredicateSchema(
  fieldSchema: z.ZodTypeAny,
  variant: MetadataPredicateVariantSpec,
): z.ZodObject<z.ZodRawShape> {
  return buildStrictObjectSchema({
    field: fieldSchema,
    op: z.literal(variant.op),
    ...METADATA_PREDICATE_PAYLOAD_SCHEMAS[variant.payload].shape,
  });
}

function buildMetadataFieldPredicateSchema(
  fieldSchema: z.ZodTypeAny,
  variants: readonly MetadataPredicateVariantSpec[],
): z.ZodTypeAny {
  const visibleVariants = variants.filter((variant) => variant.exposeInSchema);
  const variantSchemas = visibleVariants.map((variant) => buildVariantPredicateSchema(fieldSchema, variant));
  const [firstSchema, secondSchema, ...restSchemas] = variantSchemas;
  if (!firstSchema) {
    throw new Error("Metadata predicate schema requires at least one visible variant.");
  }

  if (!secondSchema) {
    return firstSchema;
  }

  return z.discriminatedUnion("op", [firstSchema, secondSchema, ...restSchemas]);
}

function buildMetricValuePredicateSchema(spec: MetadataMetricValuePredicateSpec): z.ZodTypeAny {
  return z.union([
    buildStrictObjectSchema({
      field: z.literal(spec.field),
      [spec.metricKey]: z.string(),
      op: z.enum(spec.numericOperators as [string, ...string[]]),
      value: z.number(),
    }),
    buildStrictObjectSchema({
      field: z.literal(spec.field),
      [spec.metricKey]: z.string(),
      op: z.enum(spec.scalarOperators as [string, ...string[]]),
      value: z.union([z.string(), z.boolean()]),
    }),
  ]);
}

function buildMetricComparePredicateSchema(spec: MetadataMetricComparePredicateSpec): z.ZodTypeAny {
  return buildStrictObjectSchema({
    field: z.literal(spec.field),
    [spec.leftMetricKey]: z.string(),
    op: z.enum(spec.operators as [string, ...string[]]),
    [spec.rightMetricKey]: z.string(),
  });
}

const metadataSetPredicateSchema = buildMetadataFieldPredicateSchema(
  metadataSetFieldSchema,
  METADATA_FIELD_PREDICATE_VARIANTS.set,
);
const metadataEnumStringPredicateSchema = buildMetadataFieldPredicateSchema(
  metadataEnumStringFieldSchema,
  METADATA_FIELD_PREDICATE_VARIANTS.enumString,
);
const metadataTextStringPredicateSchema = buildMetadataFieldPredicateSchema(
  metadataTextStringFieldSchema,
  METADATA_FIELD_PREDICATE_VARIANTS.text,
);
const metadataNumberPredicateSchema = buildMetadataFieldPredicateSchema(
  metadataNumberFieldSchema,
  METADATA_FIELD_PREDICATE_VARIANTS.number,
);
const metadataBooleanPredicateSchema = buildMetadataFieldPredicateSchema(
  metadataBooleanFieldSchema,
  METADATA_FIELD_PREDICATE_VARIANTS.boolean,
);

const actorMetricPredicateSchema = buildMetricValuePredicateSchema(ACTOR_METRIC_PREDICATE_SPEC);
const actorMetricComparePredicateSchema = buildMetricComparePredicateSchema(ACTOR_METRIC_COMPARE_PREDICATE_SPEC);
const itemMetricPredicateSchema = buildMetricValuePredicateSchema(ITEM_METRIC_PREDICATE_SPEC);
const itemMetricComparePredicateSchema = buildMetricComparePredicateSchema(ITEM_METRIC_COMPARE_PREDICATE_SPEC);

export const metadataFilterSchema: z.ZodType<MetadataFilterNode> = z.lazy(
  () =>
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
    ]) as z.ZodType<MetadataFilterNode>,
);

export const filterValueFieldSchema: z.ZodType<FilterValueField> = z.enum(FILTER_VALUE_FIELDS);

const searchCollectionPredicateSchema: z.ZodType<MetadataAtomicPredicate> = buildStrictObjectSchema({
  field: metadataSetFieldSchema,
  op: z.enum(["includes", "isNull", "isNotNull"]),
  value: z.string().optional(),
}) as unknown as z.ZodType<MetadataAtomicPredicate>;

const searchEnumPredicateSchema: z.ZodType<MetadataAtomicPredicate> = buildStrictObjectSchema({
  field: metadataEnumStringFieldSchema,
  op: z.enum(["eq", "notEq", "isNull", "isNotNull"]),
  value: z.string().optional(),
}) as unknown as z.ZodType<MetadataAtomicPredicate>;

const searchTextPredicateSchema: z.ZodType<MetadataAtomicPredicate> = buildStrictObjectSchema({
  field: metadataTextStringFieldSchema,
  op: z.enum(["eq", "notEq", "contains", "notContains", "isNull", "isNotNull"]),
  value: z.string().optional(),
}) as unknown as z.ZodType<MetadataAtomicPredicate>;

const searchNumberPredicateSchema: z.ZodType<MetadataAtomicPredicate> = z.union([
  buildStrictObjectSchema({
    field: metadataNumberFieldSchema,
    op: z.enum(["eq", "notEq", "gt", "gte", "lt", "lte"]),
    value: z.number(),
  }),
  buildStrictObjectSchema({
    field: metadataNumberFieldSchema,
    op: z.literal("between"),
    min: z.number(),
    max: z.number(),
  }),
  buildStrictObjectSchema({
    field: metadataNumberFieldSchema,
    op: z.enum(["isNull", "isNotNull"]),
  }),
]) as unknown as z.ZodType<MetadataAtomicPredicate>;

const searchBooleanPredicateSchema: z.ZodType<MetadataAtomicPredicate> = z.union([
  buildStrictObjectSchema({
    field: metadataBooleanFieldSchema,
    op: z.enum(["eq", "notEq"]),
    value: z.boolean(),
  }),
  buildStrictObjectSchema({
    field: metadataBooleanFieldSchema,
    op: z.enum(["isNull", "isNotNull"]),
  }),
]) as unknown as z.ZodType<MetadataAtomicPredicate>;

export const metadataAtomicPredicateSchema: z.ZodType<MetadataAtomicPredicate> = z.union([
  searchCollectionPredicateSchema,
  searchEnumPredicateSchema,
  searchTextPredicateSchema,
  searchNumberPredicateSchema,
  searchBooleanPredicateSchema,
]);

export const browseSortSchema = z.union([
  buildStrictObjectSchema({
    kind: z.enum(["alphabetical", "levelAsc", "levelDesc"]),
  }),
  buildStrictObjectSchema({
    kind: z.literal("random"),
    seed: z.number().int().optional(),
  }),
]) as unknown as z.ZodType<BrowseSortSpec>;

const searchScopeSubcategoryMatchSchema = z.union([
  buildStrictObjectSchema({ kind: z.literal("any") }),
  buildStrictObjectSchema({ kind: z.literal("eq"), value: searchSubcategorySchema }),
  buildStrictObjectSchema({ kind: z.literal("isNull") }),
  buildStrictObjectSchema({ kind: z.literal("isNotNull") }),
]);

const searchNumericMatchSchema = z.union([
  buildStrictObjectSchema({ kind: z.literal("eq"), value: z.number() }),
  buildStrictObjectSchema({ kind: z.literal("gte"), value: z.number() }),
  buildStrictObjectSchema({ kind: z.literal("lte"), value: z.number() }),
  buildStrictObjectSchema({ kind: z.literal("between"), min: z.number(), max: z.number() }),
]);

const searchNullableNumericMatchSchema = z.union([
  searchNumericMatchSchema,
  buildStrictObjectSchema({ kind: z.literal("isNull") }),
  buildStrictObjectSchema({ kind: z.literal("isNotNull") }),
]);

const searchNullableStringMatchSchema = z.union([
  buildStrictObjectSchema({ kind: z.literal("eq"), value: z.string() }),
  buildStrictObjectSchema({ kind: z.literal("isNull") }),
  buildStrictObjectSchema({ kind: z.literal("isNotNull") }),
]);

export const searchFilterSchema: z.ZodType<SearchFilterNode> = z.lazy(
  () =>
    z.union([
      buildStrictObjectSchema({
        kind: z.literal("pack"),
        value: z.string().trim().min(1),
      }),
      buildStrictObjectSchema({
        kind: z.literal("scope"),
        category: searchCategorySchema,
        subcategory: searchScopeSubcategoryMatchSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("level"),
        match: searchNumericMatchSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("price"),
        match: searchNumericMatchSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("rarity"),
        match: searchNullableStringMatchSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("actionCost"),
        match: searchNullableNumericMatchSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("linksTo"),
        target: z.string().trim().min(1),
      }),
      buildStrictObjectSchema({
        kind: z.literal("metadataPredicate"),
        predicate: metadataAtomicPredicateSchema,
      }),
      buildStrictObjectSchema({
        kind: z.literal("metric"),
        metric: z.string().trim().min(1),
        op: z.enum(["eq", "notEq", "gt", "gte", "lt", "lte"]),
        value: z.union([z.string(), z.number(), z.boolean()]),
      }),
      buildStrictObjectSchema({
        kind: z.literal("metricCompare"),
        leftMetric: z.string().trim().min(1),
        op: z.enum(["eq", "notEq", "gt", "gte", "lt", "lte"]),
        rightMetric: z.string().trim().min(1),
      }),
      buildStrictObjectSchema({
        kind: z.literal("anyOf"),
        children: z.array(searchFilterSchema).min(2),
      }),
      buildStrictObjectSchema({
        kind: z.literal("allOf"),
        children: z.array(searchFilterSchema).min(2),
      }),
      buildStrictObjectSchema({
        kind: z.literal("not"),
        child: searchFilterSchema,
      }),
    ]) as unknown as z.ZodType<SearchFilterNode>,
);

export const listRecordsToolInputSchema = buildStrictObjectSchema({
  mode: z
    .literal("browse")
    .describe('Canonical request mode for browse/list flows. This tool only accepts `mode: "browse"`.'),
  filter: searchFilterSchema
    .optional()
    .describe(
      "Canonical structured filter tree. Use atomic leaves plus anyOf/allOf/not for scope, links, ranges, metadata predicates, and metrics.",
    ),
  sort: browseSortSchema
    .optional()
    .describe(
      "Optional browse sort. Use alphabetical, levelAsc, or levelDesc for deterministic ordering, or random with an optional seed for stable shuffled paging.",
    ),
  offset: z.number().int().optional().describe("Pagination offset."),
  limit: z.number().int().optional().describe("Pagination limit, max 100."),
});

export const searchToolInputSchema = buildStrictObjectSchema({
  mode: z
    .literal("search")
    .describe('Canonical request mode for ranked retrieval. This tool only accepts `mode: "search"`.'),
  search: buildStrictObjectSchema({
    query: z
      .string()
      .trim()
      .min(1)
      .describe(
        "Required search text. Prefer one short natural-language phrase or sentence with 1-3 concrete anchor terms. Use pf2e_list_records with mode:\"browse\" when you only need structural filtering and paging.",
      ),
    exclude: z
      .string()
      .trim()
      .min(1)
      .optional()
      .describe(
        "Optional free-text exclusion terms. Remove ranked-search results whose indexed search text mentions these normalized terms.",
      ),
    profile: searchProfileSchema
      .optional()
      .describe(
        "User-facing retrieval profile. lexical is lexical-first, balanced is the default hybrid profile for broad themed search, and concept is semantic-forward hybrid search.",
      ),
  }).describe("Canonical search branch. `search.query` is required on this tool; `search.exclude` and `search.profile` are optional."),
  explain: z
    .boolean()
    .optional()
    .describe("Include score breakdowns and query-analysis details in the response. Valid only on search mode."),
  filter: searchFilterSchema
    .optional()
    .describe(
      "Canonical structured filter tree. Use atomic leaves plus anyOf/allOf/not for scope, links, ranges, metadata predicates, and metrics.",
    ),
  offset: z.number().int().optional().describe("Pagination offset."),
  limit: z.number().int().optional().describe("Pagination limit, max 100."),
});
