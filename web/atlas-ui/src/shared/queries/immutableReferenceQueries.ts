import type { Query, QueryClient } from "@tanstack/react-query";
import { AtlasApiError } from "../../api/atlasApi";

/** Reference-derived queries can be retired without discarding mutable state. */
const IMMUTABLE_REFERENCE_FAMILIES = new Set([
  "results",
  "search-picker",
  "filter-discovery",
  "saved-list-filter-discovery",
  "filter-vocabulary",
  "record-detail",
  "owned-reading",
  "related",
  "similar-preview",
  "encounter-record-detail",
]);

export function isImmutableReferenceQuery(query: Query): boolean {
  return IMMUTABLE_REFERENCE_FAMILIES.has(String(query.queryKey[0]));
}

export async function retireImmutableReferenceQueries(client: QueryClient) {
  const filters = { predicate: isImmutableReferenceQuery };
  await client.cancelQueries(filters);
  client.removeQueries(filters);
}

export function isArtifactGenerationChanged(error: unknown): boolean {
  return (
    error instanceof AtlasApiError &&
    error.appError?.code === "artifact_generation_changed"
  );
}
