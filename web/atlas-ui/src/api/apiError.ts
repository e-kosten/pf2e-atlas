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
