import {
  METADATA_FIELD_REGISTRY,
  type MetadataFieldName,
  type MetadataFieldType,
} from "./metadata-field-registry.js";
import type {
  ActorMetricNumericOperator,
  ActorMetricScalarOperator,
} from "./actor-metrics.js";
import type {
  ItemMetricNumericOperator,
  ItemMetricScalarOperator,
} from "./item-metrics.js";

type MetadataFieldNameByType<FieldType extends MetadataFieldType> =
  Extract<(typeof METADATA_FIELD_REGISTRY)[number], { fieldType: FieldType }>["field"];

function fieldNamesForType<FieldType extends MetadataFieldType>(
  fieldType: FieldType,
): MetadataFieldNameByType<FieldType>[] {
  return METADATA_FIELD_REGISTRY
    .filter((entry): entry is Extract<(typeof METADATA_FIELD_REGISTRY)[number], { fieldType: FieldType }> => entry.fieldType === fieldType)
    .map((entry) => entry.field) as MetadataFieldNameByType<FieldType>[];
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

export type MetadataSetOperator = "includesAny" | "includesAll" | "excludesAny";
export type MetadataEnumStringOperator = "eq" | "in" | "notIn";
export type MetadataTextStringOperator = "contains" | "notContains";
export type MetadataNumberOperator = "eq" | "gte" | "lte" | "between";
export type MetadataBooleanOperator = "eq";

export type MetadataSetPredicate = {
  field: MetadataSetField;
  op: MetadataSetOperator;
  values: string[];
};

export type MetadataEnumStringPredicate =
  | {
    field: MetadataEnumStringField;
    op: "eq";
    value: string;
  }
  | {
    field: MetadataEnumStringField;
    op: "in" | "notIn";
    values: string[];
  };

export type MetadataTextStringPredicate = {
  field: MetadataTextStringField;
  op: MetadataTextStringOperator;
  value: string;
};

export type MetadataNumberPredicate =
  | {
    field: MetadataNumberField;
    op: "eq" | "gte" | "lte";
    value: number;
  }
  | {
    field: MetadataNumberField;
    op: "between";
    min: number;
    max: number;
  };

export type MetadataBooleanPredicate = {
  field: MetadataBooleanField;
  op: MetadataBooleanOperator;
  value: boolean;
};

export type ActorMetricPredicate =
  | {
    field: "actorMetric";
    metric: string;
    op: ActorMetricNumericOperator;
    value: number;
  }
  | {
    field: "actorMetric";
    metric: string;
    op: ActorMetricScalarOperator;
    value: string | boolean;
  };

export type ActorMetricComparePredicate = {
  field: "actorMetricCompare";
  leftMetric: string;
  op: ActorMetricNumericOperator;
  rightMetric: string;
};

export type ItemMetricPredicate =
  | {
    field: "itemMetric";
    metric: string;
    op: ItemMetricNumericOperator;
    value: number;
  }
  | {
    field: "itemMetric";
    metric: string;
    op: ItemMetricScalarOperator;
    value: string | boolean;
  };

export type ItemMetricComparePredicate = {
  field: "itemMetricCompare";
  leftMetric: string;
  op: ItemMetricNumericOperator;
  rightMetric: string;
};

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
