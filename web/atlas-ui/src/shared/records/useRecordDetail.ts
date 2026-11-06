import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery } from "@tanstack/react-query";
import { recordDetailQueryOptions } from "./recordDetailQuery";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
  generation?: string | null,
) {
  const options = recordDetailQueryOptions(recordKey, selection, generation);
  const query = useQuery(options);
  const retirement = useReferenceQueryRetirement(options.queryKey);
  const data = useRetainedRecord(
    JSON.stringify([
      recordReadingIdentity(recordKey, selection),
      generation,
      retirement,
    ]),
    query.data,
    query.error,
  );
  return { ...query, data };
}
