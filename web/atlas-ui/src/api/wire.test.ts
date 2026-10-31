import { expectTypeOf } from "vitest";
import type { Wire } from "./wire";

type FutureResponse = {
  required: bigint;
  optional?: bigint;
  rows: Array<{ kind: "count"; value: bigint } | { kind: "text"; value: string }>;
  nullable: bigint | null;
  tuple: [bigint, number];
  numericOverride: number;
  details: unknown;
};

it("requires conversion of future required, optional and nested union members", () => {
  const input: Wire<FutureResponse> = {
    required: 1,
    optional: 0,
    rows: [
      { kind: "count", value: -2 },
      { kind: "text", value: "ready" },
    ],
    nullable: null,
    tuple: [3, 0.5],
    numericOverride: 4,
    details: { opaque: 5 },
  };
  // @ts-expect-error A required bigint cannot be copied from numeric JSON.
  const required: Pick<FutureResponse, "required"> = { required: input.required };
  // @ts-expect-error Optional bigint also requires conversion when present.
  const optional: Pick<FutureResponse, "optional"> = { optional: input.optional };
  // @ts-expect-error Arrays and tagged union branches retain their bigint obligation.
  const rows: Pick<FutureResponse, "rows"> = { rows: input.rows };
  // @ts-expect-error Tuple members retain independent number/bigint representations.
  const tuple: Pick<FutureResponse, "tuple"> = { tuple: input.tuple };
  expect([required, optional, rows, tuple]).toHaveLength(4);

  const { optional: optionalValue, ...rest } = input;
  const converted: FutureResponse = {
    ...rest,
    required: BigInt(input.required),
    ...(optionalValue === undefined ? {} : { optional: BigInt(optionalValue) }),
    rows: input.rows.map((row) =>
      row.kind === "count" ? { ...row, value: BigInt(row.value) } : row,
    ),
    nullable: input.nullable === null ? null : BigInt(input.nullable),
    tuple: [BigInt(input.tuple[0]), input.tuple[1]],
  };
  expect(converted.required).toBe(1n);
  expect(converted.optional).toBe(0n);
  expect(converted.rows[0]).toEqual({ kind: "count", value: -2n });
  expect(converted.numericOverride).toBe(4);
  expect(converted.nullable).toBeNull();
  expect(converted.details).toBe(input.details);
});

it("preserves optional absence, null, numeric overrides and opaque unknown data", () => {
  type Controls = {
    optional?: bigint;
    nullable: bigint | null;
    numeric: number;
    details: unknown;
  };
  const input: Wire<Controls> = { nullable: null, numeric: 0.1, details: undefined };
  expect(input).not.toHaveProperty("optional");
  expectTypeOf(input.nullable).toEqualTypeOf<number | null>();
  expectTypeOf(input.numeric).toEqualTypeOf<number>();
  expectTypeOf(input.details).toEqualTypeOf<unknown>();
  expect(input.nullable).toBeNull();
});
