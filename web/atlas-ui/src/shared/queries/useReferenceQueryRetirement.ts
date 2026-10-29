import { useEffect, useState } from "react";
import { hashKey, useQueryClient, type QueryKey } from "@tanstack/react-query";

export function useReferenceQueryRetirement(queryKey: QueryKey) {
  const client = useQueryClient();
  const queryHash = hashKey(queryKey);
  const sourceQuery = client.getQueryCache().get(queryHash);
  const [observed, setObserved] = useState({ queryHash, sourceQuery, epoch: 0 });
  if (observed.queryHash !== queryHash || observed.sourceQuery !== sourceQuery)
    setObserved({
      queryHash,
      sourceQuery,
      epoch:
        observed.epoch +
        Number(observed.queryHash === queryHash && observed.sourceQuery !== undefined),
    });
  useEffect(
    () =>
      client.getQueryCache().subscribe((event) => {
        if (event.type === "removed" && event.query === sourceQuery)
          setObserved((current) => ({
            ...current,
            sourceQuery: undefined,
            epoch: current.epoch + 1,
          }));
      }),
    [client, sourceQuery],
  );
  return observed.epoch;
}
