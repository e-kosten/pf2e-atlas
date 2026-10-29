import { useState } from "react";
import type { RecordNavigationView } from "../../generated/atlas";
import { isArtifactGenerationChanged } from "../queries/referenceQueries";
import { recordNavigationIdentity } from "./recordNavigationIdentity";

// Rank is a view parameter, not a different source document. Keep every other
// address dimension: embedded copies and source generations must never borrow data.
export function recordReadingIdentity(
  recordKey: string | null,
  selection?: RecordNavigationView,
) {
  return recordNavigationIdentity(
    recordKey,
    selection ? { ...selection, spell_rank: undefined } : undefined,
  );
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
