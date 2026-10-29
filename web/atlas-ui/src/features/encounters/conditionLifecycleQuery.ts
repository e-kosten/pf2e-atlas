import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { queryOptions } from "@tanstack/react-query";
import { getEncounterConditionLifecycleActions } from "../../api/atlasApi";

export function conditionLifecycleQuery(
  encounterRef: string,
  participantKey: string,
  revision: bigint,
  boundaryKey: string | null,
) {
  return queryOptions({
    meta: referenceQueryMeta,
    queryKey: [
      "condition-lifecycle-actions",
      encounterRef,
      participantKey,
      revision.toString(),
      boundaryKey,
    ],
    queryFn: () =>
      getEncounterConditionLifecycleActions(encounterRef, participantKey, boundaryKey),
    staleTime: 1_000,
    retry: false,
    retryOnMount: false,
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participantKey &&
      query.queryKey[4] === boundaryKey
        ? previous
        : undefined,
  });
}
