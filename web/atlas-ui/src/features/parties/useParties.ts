import { useQuery, type QueryClient } from "@tanstack/react-query";
import { getParties } from "../../api/atlasApi";
import type { PartyIndexView, PartyView } from "../../generated/atlas";

export function useParties(enabled = true) {
  return useQuery({
    queryKey: ["parties"],
    queryFn: getParties,
    enabled,
    retry: false,
  });
}

export async function cacheParty(client: QueryClient, saved: PartyView) {
  const queryKey = ["parties"];
  await client.cancelQueries({ queryKey, exact: true });
  client.setQueryData<PartyIndexView>(
    queryKey,
    (index) =>
      index && {
        ...index,
        parties: index.parties.map((party) =>
          party.party_key === saved.party_key && party.revision <= saved.revision
            ? saved
            : party,
        ),
      },
  );
}
