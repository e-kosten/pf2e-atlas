import type {
  MetadataBooleanField,
  MetadataEnumStringField,
  MetadataNumberField,
  MetadataSetField,
  MetadataTextStringField,
} from "./metadata-field-types.js";
import type {
  CollectionOperator,
  EqualityOperator,
  MetricOperator,
  NullOperator,
  NumericMetricOperator,
  NumericOperator,
  TextOperator,
} from "./search-filter-operators.js";

type MetadataCollectionPredicate = {
  field: MetadataSetField;
  op: CollectionOperator | NullOperator;
  value?: string;
};

type MetadataEnumPredicate = {
  field: MetadataEnumStringField;
  op: EqualityOperator | NullOperator;
  value?: string;
};

type MetadataTextPredicate = {
  field: MetadataTextStringField;
  op: EqualityOperator | TextOperator | NullOperator;
  value?: string;
};

type MetadataNumberPredicate = {
  field: MetadataNumberField;
  op: NumericOperator | NullOperator;
  value?: number;
  min?: number;
  max?: number;
};

type MetadataBooleanPredicate = {
  field: MetadataBooleanField;
  op: EqualityOperator | NullOperator;
  value?: boolean;
};

export type MetadataAtomicPredicate =
  | MetadataCollectionPredicate
  | MetadataEnumPredicate
  | MetadataTextPredicate
  | MetadataNumberPredicate
  | MetadataBooleanPredicate;

export type SearchMetricPredicate = {
  metric: string;
  op: MetricOperator;
  value: string | number | boolean;
};

export type SearchMetricComparePredicate = {
  leftMetric: string;
  op: NumericMetricOperator;
  rightMetric: string;
};
