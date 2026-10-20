import { useQuery } from "@tanstack/react-query";
import { getEncounterPersistentReview } from "../../api/atlasApi";
export function usePersistentReview(
  encounterRef: string,
  participantKey: string,
  revision: bigint,
  boundaryKey: string | null = null,
) {
  return useQuery({
    queryKey: [
      "persistent-review",
      encounterRef,
      participantKey,
      revision.toString(),
      boundaryKey,
    ],
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
