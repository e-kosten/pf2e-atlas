import type { SearchFilterDiscoveryContext } from "../../domain/search-field-domains.js";

export function buildDiscoveryCacheKey(parts: ReadonlyArray<string | null | undefined>): string {
  return parts.map((part) => part ?? "").join("|");
}

export function buildDiscoveryContextCacheKey(context: SearchFilterDiscoveryContext): string {
  return buildDiscoveryCacheKey([
    context.applicability.mode,
    context.applicability.pack,
    ...context.applicability.scopes.flatMap((scope) => [scope.category, scope.subcategory ?? "all"]),
    JSON.stringify(context.request),
  ]);
}
