export const SEARCH_FILTER_OPERATOR_VOCABULARY = {
  EQUALITY: {
    EQ: "eq",
    NOT_EQ: "notEq",
  },
  ORDERING: {
    GT: "gt",
    GTE: "gte",
    LT: "lt",
    LTE: "lte",
  },
  RANGE: {
    BETWEEN: "between",
  },
  NULL: {
    IS_NULL: "isNull",
    IS_NOT_NULL: "isNotNull",
  },
  COLLECTION: {
    INCLUDES: "includes",
  },
  TEXT: {
    CONTAINS: "contains",
    NOT_CONTAINS: "notContains",
  },
} as const;

export const SEARCH_EQUALITY_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY);
export const SEARCH_ORDERING_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.ORDERING);
export const SEARCH_RANGE_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.RANGE);
export const SEARCH_NULL_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.NULL);
export const SEARCH_COLLECTION_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.COLLECTION);
export const SEARCH_TEXT_OPERATORS = Object.values(SEARCH_FILTER_OPERATOR_VOCABULARY.TEXT);

export type EqualityOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.EQUALITY];
export type OrderingOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.ORDERING)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.ORDERING];
export type RangeOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.RANGE)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.RANGE];
export type NullOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.NULL)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.NULL];
export type CollectionOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.COLLECTION)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.COLLECTION];
export type TextOperator = (typeof SEARCH_FILTER_OPERATOR_VOCABULARY.TEXT)[keyof typeof SEARCH_FILTER_OPERATOR_VOCABULARY.TEXT];

export type ScalarOperator = EqualityOperator | OrderingOperator | RangeOperator | NullOperator;
export type NumericOperator = EqualityOperator | OrderingOperator | RangeOperator;
export type NumericMetricOperator = EqualityOperator | OrderingOperator;
export type MetricOperator = EqualityOperator | OrderingOperator;
