import type { QueryLiteral } from "../../generated/atlas";

// Shared structural checks for the predicate and its exact source-selection wire.
export function invalidFilterUrl(): never {
  throw new Error("Invalid filter URL");
}

export function urlTuple(value: unknown, min: number, max = min): unknown[] {
  if (!Array.isArray(value) || value.length < min || value.length > max)
    invalidFilterUrl();
  return value;
}

export function urlString(value: unknown): string {
  if (typeof value !== "string") invalidFilterUrl();
  return value;
}

export function urlNullableString(value: unknown): string | null {
  return value == null ? null : urlString(value);
}

export function urlInteger(value: unknown): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value < 0)
    invalidFilterUrl();
  return value;
}

export function urlLiteral(value: unknown): QueryLiteral {
  if (
    typeof value !== "string" &&
    typeof value !== "boolean" &&
    !(typeof value === "number" && Number.isFinite(value))
  )
    invalidFilterUrl();
  return value;
}
