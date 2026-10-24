import { queryOptions, useQuery } from "@tanstack/react-query";
import { getEncounterPersistentReview } from "../../api/atlasApi";
export function persistentReviewQuery(
  encounterRef: string,
  participantKey: string,
  revision: bigint,
  boundaryKey: string | null = null,
) {
  return queryOptions({
    queryKey: [
      "persistent-review",
      encounterRef,
      participantKey,
      revision.toString(),
      boundaryKey,
    ],
    // The advance preloads this revision before opening; avoid a duplicate mount read.
    staleTime: 1_000,
    retry: false,
    retryOnMount: false,
    queryFn: () =>
      getEncounterPersistentReview(encounterRef, participantKey, boundaryKey),
    placeholderData: (previous, query) =>
      query?.queryKey[1] === encounterRef &&
      query.queryKey[2] === participantKey &&
      query.queryKey[4] === boundaryKey
        ? previous
        : undefined,
  });
}

export function usePersistentReview(
  encounterRef: string,
  participantKey: string,
  revision: bigint,
  boundaryKey: string | null = null,
) {
  return useQuery(
    persistentReviewQuery(encounterRef, participantKey, revision, boundaryKey),
  );
}
