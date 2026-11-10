import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useId, useState } from "react";
import { getEncounter } from "../../api/atlasApi";

/** Resolve a mounted route once and observe only its canonical encounter thereafter. */
export function useEncounterQuery(slug: string) {
  const queryClient = useQueryClient();
  const navigation = useId();
  const [encounterKey, setEncounterKey] = useState<string>();
  const resolution = useQuery({
    queryKey: ["encounter-route", slug, navigation],
    queryFn: async ({ signal }) => {
      const view = await getEncounter(slug);
      signal.throwIfAborted();
      const key = view.encounter.encounter_key;
      queryClient.setQueryData(["encounter", key], view);
      return key;
    },
    enabled: encounterKey === undefined,
    gcTime: 0,
  });

  const key = encounterKey ?? resolution.data;
  if (key !== encounterKey) setEncounterKey(key);

  const query = useCanonicalEncounter(key);
  const active = key === undefined ? resolution : query;
  return {
    data: key === undefined ? undefined : query.data,
    error: active.error,
    isFetching: active.isFetching,
    isLoading: active.isLoading,
    isPending: active.isPending,
    refetch: active.refetch,
  };
}

export function useCanonicalEncounter(key: string | undefined) {
  return useQuery({
    queryKey: ["encounter", key],
    queryFn: () => getEncounter(key!),
    enabled: key !== undefined,
    // Route resolution just loaded the canonical snapshot.
    staleTime: 20_000,
  });
}
