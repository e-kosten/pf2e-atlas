import { ACTOR_METRIC_NUMERIC_OPERATORS, ACTOR_METRIC_SCALAR_OPERATORS } from "./actor-metrics.js";
import { METADATA_FIELD_KIND_OPERATORS, type MetadataFieldType } from "./metadata-field-registry.js";

export type MetadataPredicatePayloadKind = "string" | "stringArray" | "number" | "numberRange" | "boolean";

export type MetadataPredicatePayloadShapeMap = {
  string: { value: string };
  stringArray: { values: string[] };
  number: { value: number };
  numberRange: { min: number; max: number };
  boolean: { value: boolean };
};

export interface MetadataPredicateVariantSpec<
  Operator extends string = string,
  Payload extends MetadataPredicatePayloadKind = MetadataPredicatePayloadKind,
> {
  op: Operator;
  payload: Payload;
  exposeInSchema: boolean;
}

function definePredicateVariant<const Operator extends string, const Payload extends MetadataPredicatePayloadKind>(
  op: Operator,
  payload: Payload,
  options?: { exposeInSchema?: boolean },
): MetadataPredicateVariantSpec<Operator, Payload> {
  return {
    op,
    payload,
    exposeInSchema: options?.exposeInSchema ?? true,
  };
}

function definePredicateVariants<
  const Operators extends readonly string[],
  const Payload extends MetadataPredicatePayloadKind,
>(
  operators: Operators,
  payload: Payload,
  options?: { exposeInSchema?: boolean | ((operator: Operators[number]) => boolean) },
): {
  [Index in keyof Operators]: MetadataPredicateVariantSpec<Operators[Index] & string, Payload>;
} {
  return operators.map((op) =>
    definePredicateVariant(
      op,
      payload,
      typeof options?.exposeInSchema === "function"
        ? { exposeInSchema: options.exposeInSchema(op) }
        : { exposeInSchema: options?.exposeInSchema },
    ),
  ) as {
    [Index in keyof Operators]: MetadataPredicateVariantSpec<Operators[Index] & string, Payload>;
  };
}

const METADATA_ENUM_STRING_EQ_OPERATOR = ["eq"] as const;
const METADATA_ENUM_STRING_MULTI_VALUE_OPERATORS = ["in", "notIn"] as const;
const METADATA_NUMBER_VALUE_OPERATORS = ["eq", "gte", "lte"] as const;
const METADATA_NUMBER_RANGE_OPERATOR = ["between"] as const;

export const METADATA_FIELD_PREDICATE_VARIANTS = {
  set: definePredicateVariants(METADATA_FIELD_KIND_OPERATORS.set, "stringArray"),
  enumString: [
    ...definePredicateVariants(METADATA_ENUM_STRING_EQ_OPERATOR, "string"),
    ...definePredicateVariants(METADATA_ENUM_STRING_MULTI_VALUE_OPERATORS, "stringArray"),
  ],
  text: definePredicateVariants(METADATA_FIELD_KIND_OPERATORS.text, "string", {
    exposeInSchema: (operator) => operator === "contains" || operator === "notContains",
  }),
  number: [
    ...definePredicateVariants(METADATA_NUMBER_VALUE_OPERATORS, "number"),
    ...definePredicateVariants(METADATA_NUMBER_RANGE_OPERATOR, "numberRange"),
  ],
  boolean: definePredicateVariants(METADATA_FIELD_KIND_OPERATORS.boolean, "boolean"),
} as const satisfies Record<MetadataFieldType, readonly MetadataPredicateVariantSpec[]>;

type MetadataFieldPredicateVariantsByType = typeof METADATA_FIELD_PREDICATE_VARIANTS;

export type MetadataPredicateOperator<Variants extends readonly MetadataPredicateVariantSpec[]> =
  Variants[number]["op"];

export type MetadataPredicateFromVariant<
  Field extends string,
  Variant extends MetadataPredicateVariantSpec,
> = Variant extends MetadataPredicateVariantSpec
  ? { field: Field; op: Variant["op"] } & MetadataPredicatePayloadShapeMap[Variant["payload"]]
  : never;

export type MetadataPredicateFromVariants<
  Field extends string,
  Variants extends readonly MetadataPredicateVariantSpec[],
> = MetadataPredicateFromVariant<Field, Variants[number]>;

export type MetadataFieldPredicateFromType<
  Field extends string,
  FieldType extends keyof MetadataFieldPredicateVariantsByType,
> = MetadataPredicateFromVariants<Field, MetadataFieldPredicateVariantsByType[FieldType]>;

const METADATA_FIELD_PREDICATE_VARIANT_LOOKUP = new Map<MetadataFieldType, Map<string, MetadataPredicateVariantSpec>>(
  Object.entries(METADATA_FIELD_PREDICATE_VARIANTS).map(([fieldType, variants]) => [
    fieldType as MetadataFieldType,
    new Map(variants.map((variant) => [variant.op, variant])),
  ]),
);

export function getMetadataFieldPredicateVariant(
  fieldType: MetadataFieldType,
  op: string,
): MetadataPredicateVariantSpec | null {
  return METADATA_FIELD_PREDICATE_VARIANT_LOOKUP.get(fieldType)?.get(op) ?? null;
}

export interface MetadataMetricValuePredicateSpec<Field extends string = string, MetricKey extends string = string> {
  field: Field;
  metricKey: MetricKey;
  numericOperators: readonly string[];
  scalarOperators: readonly string[];
}

export interface MetadataMetricComparePredicateSpec<
  Field extends string = string,
  LeftMetricKey extends string = string,
  RightMetricKey extends string = string,
> {
  field: Field;
  leftMetricKey: LeftMetricKey;
  rightMetricKey: RightMetricKey;
  operators: readonly string[];
}

type StringKeyedRecord<Key extends string> = {
  [Property in Key]: string;
};

export type MetricValuePredicateFromSpec<Spec extends MetadataMetricValuePredicateSpec> =
  | ({ field: Spec["field"]; op: Spec["numericOperators"][number]; value: number } & StringKeyedRecord<
      Spec["metricKey"]
    >)
  | ({
      field: Spec["field"];
      op: Spec["scalarOperators"][number];
      value: string | boolean;
    } & StringKeyedRecord<Spec["metricKey"]>);

export type MetricComparePredicateFromSpec<Spec extends MetadataMetricComparePredicateSpec> = {
  field: Spec["field"];
  op: Spec["operators"][number];
} & StringKeyedRecord<Spec["leftMetricKey"]> &
  StringKeyedRecord<Spec["rightMetricKey"]>;

export const ACTOR_METRIC_PREDICATE_SPEC = {
  field: "actorMetric",
  metricKey: "metric",
  numericOperators: ACTOR_METRIC_NUMERIC_OPERATORS,
  scalarOperators: ACTOR_METRIC_SCALAR_OPERATORS,
} as const satisfies MetadataMetricValuePredicateSpec<"actorMetric", "metric">;

export const ACTOR_METRIC_COMPARE_PREDICATE_SPEC = {
  field: "actorMetricCompare",
  leftMetricKey: "leftMetric",
  rightMetricKey: "rightMetric",
  operators: ACTOR_METRIC_NUMERIC_OPERATORS,
} as const satisfies MetadataMetricComparePredicateSpec<"actorMetricCompare", "leftMetric", "rightMetric">;

export const ITEM_METRIC_PREDICATE_SPEC = {
  field: "itemMetric",
  metricKey: "metric",
  numericOperators: ACTOR_METRIC_NUMERIC_OPERATORS,
  scalarOperators: ACTOR_METRIC_SCALAR_OPERATORS,
} as const satisfies MetadataMetricValuePredicateSpec<"itemMetric", "metric">;

export const ITEM_METRIC_COMPARE_PREDICATE_SPEC = {
  field: "itemMetricCompare",
  leftMetricKey: "leftMetric",
  rightMetricKey: "rightMetric",
  operators: ACTOR_METRIC_NUMERIC_OPERATORS,
} as const satisfies MetadataMetricComparePredicateSpec<"itemMetricCompare", "leftMetric", "rightMetric">;

export type MetricValueOperatorKind = "numeric" | "scalar";

export function getMetricValuePredicateOperatorKind(
  spec: MetadataMetricValuePredicateSpec,
  op: string,
  metricType: "number" | "text" | "boolean",
): MetricValueOperatorKind | null {
  if (metricType === "number") {
    return spec.numericOperators.includes(op) ? "numeric" : null;
  }

  return spec.scalarOperators.includes(op) ? "scalar" : null;
}
