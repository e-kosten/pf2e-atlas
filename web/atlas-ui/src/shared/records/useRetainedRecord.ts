import { useEffect, useState } from "react";
import { useQueryClient, type QueryKey } from "@tanstack/react-query";
import type { RecordNavigationView } from "../../generated/atlas";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

// Rank is a view parameter, not a different source document. Keep every other
// address dimension: embedded copies and source generations must never borrow data.
export function recordReadingIdentity(
  recordKey: string | null,
  selection?: RecordNavigationView,
) {
  return JSON.stringify([
    recordKey,
    selection?.embedded_path ?? [],
    selection?.field ?? null,
    selection?.passage ?? null,
    selection?.source_fingerprint ?? null,
  ]);
}

export function useRetainedRecord<T>(
  key: string,
  data: T | undefined,
  error: unknown,
): T | undefined {
  const [retained, setRetained] = useState<{ key: string; data: T | undefined }>({
    key,
    data,
  });
  const invalid = isArtifactGenerationChanged(error);
  if (
    retained.key !== key ||
    (invalid
      ? retained.data !== undefined
      : data !== undefined && retained.data !== data)
  ) {
    setRetained({ key, data: invalid ? undefined : data });
  }
  return invalid
    ? undefined
    : (data ?? (retained.key === key ? retained.data : undefined));
}

/** An explicitly retired immutable query cannot lend retained display data. */
export function useReferenceQueryRetirement(queryKey: QueryKey) {
  const client = useQueryClient();
  const sourceQuery = client.getQueryCache().find({ queryKey, exact: true });
  const [epoch, setEpoch] = useState(0);
  useEffect(
    () =>
      client.getQueryCache().subscribe((event) => {
        if (event.type === "removed" && event.query === sourceQuery)
          setEpoch((value) => value + 1);
      }),
    [client, sourceQuery],
  );
  return epoch;
}
