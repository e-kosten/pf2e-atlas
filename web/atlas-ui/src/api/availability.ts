import type { AppError } from "../generated/atlas";
/** A transport failure says nothing about whether an in-flight write committed. */
export class AtlasConnectionError extends Error {
  constructor(
    readonly write: boolean,
    readonly cause: unknown,
  ) {
    super("Unable to connect to Atlas");
    this.name = "AtlasConnectionError";
  }
}

const referenceCodes = new Set([
  "artifact_incompatible",
  "artifact_not_ready",
  "setup_required",
  "index_unavailable",
]);
export function isReferenceFailure(error: unknown): error is { appError: AppError } {
  if (!error || typeof error !== "object" || !("appError" in error)) return false;
  const value = error.appError;
  return (
    !!value &&
    typeof value === "object" &&
    "code" in value &&
    typeof value.code === "string" &&
    referenceCodes.has(value.code)
  );
}
export type AtlasAvailabilityFailure = AtlasConnectionError | { appError: AppError };
let failure: AtlasAvailabilityFailure | null = null;
const listeners = new Set<() => void>();
export const getAvailabilityFailure = () => failure;
export function subscribeAvailabilityFailures(listener: () => void) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
// QueryCache reports after bounded read retries, MutationCache after a failed write.
export function reportAvailabilityFailure(error: unknown) {
  if (!(error instanceof AtlasConnectionError) && !isReferenceFailure(error)) return;
  // Later failed reads must not erase an unconfirmed write outcome.
  if (
    failure instanceof AtlasConnectionError &&
    failure.write &&
    error instanceof AtlasConnectionError &&
    !error.write
  )
    return;
  failure = error;
  for (const listener of listeners) listener();
}
export function clearAvailabilityFailure() {
  failure = null;
  for (const listener of listeners) listener();
}
