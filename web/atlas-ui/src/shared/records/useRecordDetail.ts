import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";
import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery } from "@tanstack/react-query";
import { recordDetailQueryOptions } from "./recordDetailQuery";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
) {
  const options = recordDetailQueryOptions(recordKey, selection);
  const query = useQuery(options);
  const retirement = useReferenceQueryRetirement(options.queryKey);
  const data = useRetainedRecord(
    JSON.stringify([recordReadingIdentity(recordKey, selection), retirement]),
    query.data,
    query.error,
  );
  return { ...query, data };
}
