import type { AppError } from "../generated/atlas";
import {
  API_CONTRACT,
  API_CONTRACT_HEADER,
  checkResponseCompatibility,
  getCompatibilityFailure,
} from "./compatibility";
import { AtlasConnectionError } from "./availability";
import { AtlasApiError } from "./apiError";

const API_BASE = import.meta.env.VITE_ATLAS_API_BASE ?? "";

// Non-GET callers must choose failure semantics. Actions include transient dice
// commands as well as saved mutations; intent never grants automatic retry.
export type AtlasRequestInit = Omit<RequestInit, "method"> &
  (
    | { method?: "GET"; intent?: "read" }
    | { method: "POST" | "PATCH" | "DELETE"; intent: "read" | "action" }
  );

async function atlasFetch<T>(path: string, init: AtlasRequestInit = {}): Promise<T> {
  const failure = getCompatibilityFailure();
  if (failure && path !== "/api/compatibility") throw failure;
  const headers = new Headers(init.headers);
  headers.set("Content-Type", "application/json");
  headers.set(API_CONTRACT_HEADER, API_CONTRACT);
  const { intent, ...request } = init;
  const write = intent === "action";
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, { ...request, headers });
  } catch (cause) {
    if (
      init.signal?.aborted ||
      (cause instanceof DOMException && cause.name === "AbortError")
    )
      throw cause;
    throw new AtlasConnectionError(write, cause);
  }
  // Preserve genuine proxy/service failures while rejecting old successful APIs
  // before attempting to interpret their DTOs. Old servers return 404 here.
  if (
    response.headers.has(API_CONTRACT_HEADER) ||
    response.ok ||
    response.status === 409 ||
    (path === "/api/compatibility" && response.status === 404)
  ) {
    checkResponseCompatibility(response);
  }
  let text: string;
  try {
    text = await response.text();
  } catch (cause) {
    if (init.signal?.aborted) throw cause;
    throw new AtlasConnectionError(write, cause);
  }
  const parsed = parseJsonResponse(text);
  const payload = parsed.ok ? parsed.value : undefined;

  if (!response.ok) {
    const appError = isAppError(payload) ? payload : undefined;
    if (!appError && [502, 503, 504].includes(response.status))
      throw new AtlasConnectionError(write, new Error(`HTTP ${response.status}`));
    throw new AtlasApiError(
      response.status,
      appError?.message ?? fallbackErrorMessage(response, text, parsed),
      appError,
    );
  }

  if (!parsed.ok) {
    throw new AtlasApiError(
      response.status,
      fallbackErrorMessage(response, text, parsed),
    );
  }

  validateResponseNumbers(payload);
  return payload as T;
}

function validateResponseNumbers(value: unknown): void {
  if (
    typeof value === "number" &&
    (!Number.isFinite(value) ||
      (Number.isInteger(value) && !Number.isSafeInteger(value)))
  )
    throw new AtlasApiError(
      200,
      "Invalid numeric field: response exceeds JSON safe numeric range",
    );
  if (Array.isArray(value)) value.forEach(validateResponseNumbers);
  else if (isRecord(value)) Object.values(value).forEach(validateResponseNumbers);
}

function parseJsonResponse(
  text: string,
): { ok: true; value: unknown } | { ok: false; error?: unknown } {
  if (text.length === 0) {
    return { ok: true, value: undefined };
  }
  try {
    return { ok: true, value: JSON.parse(text) };
  } catch (error) {
    return { ok: false, error };
  }
}

function fallbackErrorMessage(
  response: Response,
  text: string,
  parsed: { ok: true; value: unknown } | { ok: false; error?: unknown },
): string {
  if (!parsed.ok && text.trim().length > 0) {
    return text;
  }
  if (!parsed.ok) {
    return "Invalid JSON response";
  }
  return response.statusText || `HTTP ${response.status}`;
}

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isAppError(value: unknown): value is AppError {
  return (
    typeof value === "object" && value !== null && "code" in value && "message" in value
  );
}

export async function fetchGenerated<T>(
  path: string,
  { method, intent, ...options }: RequestInit & { intent?: "read" | "action" } = {},
): Promise<T> {
  if (method === undefined || method === "GET") {
    return atlasFetch(path, options);
  }
  if ((method === "POST" || method === "PATCH" || method === "DELETE") && intent) {
    return atlasFetch(path, { ...options, method, intent });
  }
  throw new AtlasApiError(
    0,
    "Unsupported API request method or missing request intent",
  );
}
