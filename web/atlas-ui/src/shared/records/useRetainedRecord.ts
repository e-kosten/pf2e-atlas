import { useState } from "react";
import type { RecordNavigationView } from "../../generated/atlas";
import { isArtifactGenerationChanged } from "../queries/immutableReferenceQueries";

// Rank is a view parameter, not a different source document. Keep every other
// address dimension: owned copies and source generations must never borrow data.
export function recordReadingIdentity(
  recordKey: string | null,
  selection?: RecordNavigationView,
) {
  return JSON.stringify([
    recordKey,
    selection?.owners ?? [],
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
