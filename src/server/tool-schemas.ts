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
