import { AtlasApiError } from "./apiError";

/** Parsed JSON representation; this type does not validate incoming data. */
export type Wire<T> = T extends bigint
  ? number
  : T extends object
    ? { [K in keyof T]: Wire<T[K]> }
    : T;

export function toBigInt(value: unknown): bigint {
  if (typeof value === "bigint") return value;
  if (typeof value === "number" && Number.isSafeInteger(value)) return BigInt(value);
  if (typeof value === "string" && /^-?\d+$/.test(value)) return BigInt(value);
  throw new AtlasApiError(200, "Invalid numeric field in API response");
}
