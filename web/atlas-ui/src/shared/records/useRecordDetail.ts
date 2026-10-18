import { recordReadingIdentity, useRetainedRecord } from "./useRetainedRecord";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { recordDetailQueryOptions } from "./recordDetailQuery";

export function useRecordDetail(
  recordKey: string | null,
  selection?: import("../../generated/atlas").RecordNavigationView,
) {
  const client = useQueryClient();
  const options = recordDetailQueryOptions(recordKey, selection);
  const query = useQuery(options);
  const sourceQuery = client.getQueryCache().find(options);
  const [retirement, setRetirement] = useState(0);
  // Removing an immutable query retires its source generation. Rank changes
  // may retain display data, but an explicitly retired query must not.
  useEffect(
    () =>
      client.getQueryCache().subscribe((event) => {
        if (event.type === "removed" && event.query === sourceQuery)
          setRetirement((epoch) => epoch + 1);
      }),
    [client, sourceQuery],
  );
  const data = useRetainedRecord(
    JSON.stringify([recordReadingIdentity(recordKey, selection), retirement]),
    query.data,
    query.error,
  );
  return { ...query, data };
}
