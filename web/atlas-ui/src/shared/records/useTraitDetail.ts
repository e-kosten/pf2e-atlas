import { referenceQueryMeta } from "../queries/referenceQueries";
import { useQuery } from "@tanstack/react-query";
import { getTraitDetail } from "../../api/atlasApi";
import { useRetainedRecord } from "./useRetainedRecord";
import { useReferenceQueryRetirement } from "../queries/useReferenceQueryRetirement";

export function useTraitDetail(traitKey: string) {
  const queryKey = ["trait-detail", traitKey];
  const query = useQuery({
    meta: referenceQueryMeta,
    queryKey,
    queryFn: ({ signal }) =>
      getTraitDetail(
        { trait_key: traitKey, expected_artifact_generation: null },
        signal,
      ),
  });
  const retirement = useReferenceQueryRetirement(queryKey);
  const data = useRetainedRecord(
    JSON.stringify([traitKey, retirement]),
    query.data,
    query.error,
  );
  return { ...query, data };
}
