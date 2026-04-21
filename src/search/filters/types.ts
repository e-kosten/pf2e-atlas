import {
  ACTOR_METRIC_COMPARE_PREDICATE_SPEC,
  ACTOR_METRIC_PREDICATE_SPEC,
  ITEM_METRIC_COMPARE_PREDICATE_SPEC,
  ITEM_METRIC_PREDICATE_SPEC,
  METADATA_FIELD_PREDICATE_VARIANTS,
  type MetadataFieldPredicateFromType,
  type MetadataPredicateOperator,
  type MetricComparePredicateFromSpec,
  type MetricValuePredicateFromSpec,
} from "../../domain/metadata-predicate-spec.js";
import { METADATA_FIELD_REGISTRY, type MetadataFieldType } from "./registry.js";

type MetadataFieldNameByType<FieldType extends MetadataFieldType> = Extract<
  (typeof METADATA_FIELD_REGISTRY)[number],
  { fieldType: FieldType }
>["field"];

function fieldNamesForType<FieldType extends MetadataFieldType>(
  fieldType: FieldType,
): MetadataFieldNameByType<FieldType>[] {
  return METADATA_FIELD_REGISTRY.filter(
    (entry): entry is Extract<(typeof METADATA_FIELD_REGISTRY)[number], { fieldType: FieldType }> =>
      entry.fieldType === fieldType,
  ).map((entry) => entry.field) as MetadataFieldNameByType<FieldType>[];
}

export const METADATA_SET_FIELDS = fieldNamesForType("set");
export type MetadataSetField = MetadataFieldNameByType<"set">;

export const METADATA_ENUM_STRING_FIELDS = fieldNamesForType("enumString");
export type MetadataEnumStringField = MetadataFieldNameByType<"enumString">;

export const METADATA_TEXT_STRING_FIELDS = fieldNamesForType("text");
export type MetadataTextStringField = MetadataFieldNameByType<"text">;

export const METADATA_NUMBER_FIELDS = fieldNamesForType("number");
export type MetadataNumberField = MetadataFieldNameByType<"number">;

export const METADATA_BOOLEAN_FIELDS = fieldNamesForType("boolean");
export type MetadataBooleanField = MetadataFieldNameByType<"boolean">;

export type MetadataSetOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["set"]>;
export type MetadataEnumStringOperator = MetadataPredicateOperator<
  (typeof METADATA_FIELD_PREDICATE_VARIANTS)["enumString"]
>;
export type MetadataTextStringOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["text"]>;
export type MetadataNumberOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["number"]>;
export type MetadataBooleanOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["boolean"]>;

export type MetadataSetPredicate = MetadataFieldPredicateFromType<MetadataSetField, "set">;
export type MetadataEnumStringPredicate = MetadataFieldPredicateFromType<MetadataEnumStringField, "enumString">;
export type MetadataTextStringPredicate = MetadataFieldPredicateFromType<MetadataTextStringField, "text">;
export type MetadataNumberPredicate = MetadataFieldPredicateFromType<MetadataNumberField, "number">;
export type MetadataBooleanPredicate = MetadataFieldPredicateFromType<MetadataBooleanField, "boolean">;

export type ActorMetricPredicate = MetricValuePredicateFromSpec<typeof ACTOR_METRIC_PREDICATE_SPEC>;
export type ActorMetricComparePredicate = MetricComparePredicateFromSpec<typeof ACTOR_METRIC_COMPARE_PREDICATE_SPEC>;
export type ItemMetricPredicate = MetricValuePredicateFromSpec<typeof ITEM_METRIC_PREDICATE_SPEC>;
export type ItemMetricComparePredicate = MetricComparePredicateFromSpec<typeof ITEM_METRIC_COMPARE_PREDICATE_SPEC>;

export type MetadataPredicate =
  | MetadataSetPredicate
  | MetadataEnumStringPredicate
  | MetadataTextStringPredicate
  | MetadataNumberPredicate
  | MetadataBooleanPredicate
  | ActorMetricPredicate
  | ActorMetricComparePredicate
  | ItemMetricPredicate
  | ItemMetricComparePredicate;

export type MetadataFilterNode =
  | MetadataPredicate
  | { and: MetadataFilterNode[] }
  | { or: MetadataFilterNode[] }
  | { not: MetadataFilterNode };
