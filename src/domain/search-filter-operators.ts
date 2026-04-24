export const SEARCH_EQUALITY_OPERATORS = ["eq", "notEq"] as const;
export const SEARCH_ORDERING_OPERATORS = ["gt", "gte", "lt", "lte"] as const;
export const SEARCH_RANGE_OPERATORS = ["between"] as const;
export const SEARCH_NULL_OPERATORS = ["isNull", "isNotNull"] as const;
export const SEARCH_COLLECTION_OPERATORS = ["includes"] as const;
export const SEARCH_TEXT_OPERATORS = ["contains", "notContains"] as const;

export type EqualityOperator = (typeof SEARCH_EQUALITY_OPERATORS)[number];
export type OrderingOperator = (typeof SEARCH_ORDERING_OPERATORS)[number];
export type RangeOperator = (typeof SEARCH_RANGE_OPERATORS)[number];
export type NullOperator = (typeof SEARCH_NULL_OPERATORS)[number];
export type CollectionOperator = (typeof SEARCH_COLLECTION_OPERATORS)[number];
export type TextOperator = (typeof SEARCH_TEXT_OPERATORS)[number];

export type ScalarOperator = EqualityOperator | OrderingOperator | RangeOperator | NullOperator;
export type NumericOperator = EqualityOperator | OrderingOperator | RangeOperator;
export type NumericMetricOperator = EqualityOperator | OrderingOperator;
export type MetricOperator = EqualityOperator | OrderingOperator;
