import { AtlasCompatibilityError } from "./compatibility";
import type { AppError } from "../generated/atlas";

export class AtlasApiError extends Error {
  readonly appError?: AppError;
  readonly status: number;

  constructor(status: number, message: string, appError?: AppError) {
    super(message);
    this.name = "AtlasApiError";
    this.status = status;
    this.appError = appError;
  }
}

/** An action's response can be lost after its write commits. */
export function isUnconfirmedMutation(error: unknown): boolean {
  if (error instanceof AtlasCompatibilityError) return false;
  return !(
    error instanceof AtlasApiError &&
    error.status >= 400 &&
    error.status < 500 &&
    error.status !== 408
  );
}
