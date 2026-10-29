import type { Query, QueryClient } from "@tanstack/react-query";
import { AtlasApiError } from "../../api/atlasApi";

export const referenceQueryMeta = { reference: true } as const;

export function isReferenceQuery(query: Query): boolean {
  return query.meta?.reference === true;
}

export async function retireReferenceQueries(client: QueryClient) {
  const retiring = new Set(
    client.getQueryCache().findAll({ predicate: isReferenceQuery }),
  );
  const filters = { predicate: (query: Query) => retiring.has(query) };
  await client.cancelQueries(filters);
  client.removeQueries(filters);
}

export function isArtifactGenerationChanged(error: unknown): boolean {
  return (
    error instanceof AtlasApiError &&
    error.appError?.code === "artifact_generation_changed"
  );
}
