import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery } from "@tanstack/react-query";
import { recordDetailQueryOptions } from "./recordDetailQuery";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
  readingContext?: import("../../generated/atlas").RecordReadingContextView,
  generation?: string | null,
) {
  const options = recordDetailQueryOptions(
    recordKey,
    selection,
    generation,
    readingContext,
  );
  const query = useQuery(options);
  const retirement = useReferenceQueryRetirement(options.queryKey);
  const data = useRetainedRecord(
    JSON.stringify([
      recordReadingIdentity(recordKey, selection),
      readingContext,
      generation,
      retirement,
    ]),
    query.data,
    query.error,
  );
  return { ...query, data };
}
