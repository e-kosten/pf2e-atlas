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
import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataFieldNameByType,
  MetadataFieldType,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "../../domain/metadata-field-types.js";

type MetadataFieldNameByPredicateType<FieldType extends MetadataFieldType> = MetadataFieldNameByType<FieldType>;

export type MetadataSetOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["set"]>;
export type MetadataEnumStringOperator = MetadataPredicateOperator<
  (typeof METADATA_FIELD_PREDICATE_VARIANTS)["enumString"]
>;
export type MetadataTextStringOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["text"]>;
export type MetadataNumberOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["number"]>;
export type MetadataBooleanOperator = MetadataPredicateOperator<(typeof METADATA_FIELD_PREDICATE_VARIANTS)["boolean"]>;

export type MetadataSetPredicate = MetadataFieldPredicateFromType<MetadataFieldNameByPredicateType<"set">, "set">;
export type MetadataEnumStringPredicate = MetadataFieldPredicateFromType<
  MetadataFieldNameByPredicateType<"enumString">,
  "enumString"
>;
export type MetadataTextStringPredicate = MetadataFieldPredicateFromType<
  MetadataFieldNameByPredicateType<"text">,
  "text"
>;
export type MetadataNumberPredicate = MetadataFieldPredicateFromType<
  MetadataFieldNameByPredicateType<"number">,
  "number"
>;
export type MetadataBooleanPredicate = MetadataFieldPredicateFromType<
  MetadataFieldNameByPredicateType<"boolean">,
  "boolean"
>;

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

export type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
};
