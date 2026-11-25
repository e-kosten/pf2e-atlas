import { useQuery, type QueryClient } from "@tanstack/react-query";
import { getParties, getParty } from "../../api/atlasApi";
import type { PartyDirectoryView, PartyView } from "../../generated/atlas";

export function useParties(enabled = true) {
  return useQuery({
    queryKey: ["parties"],
    queryFn: getParties,
    enabled,
    retry: false,
  });
}

export function useParty(key: string, enabled: boolean) {
  return useQuery({
    queryKey: ["parties", "roster", key],
    queryFn: ({ signal }) => getParty(key, signal),
    enabled,
    retry: false,
  });
}

export async function cacheParty(client: QueryClient, saved: PartyView) {
  const queryKey = ["parties"];
  const rosterKey = ["parties", "roster", saved.party_key];
  await Promise.all([
    client.cancelQueries({ queryKey, exact: true }),
    client.cancelQueries({ queryKey: rosterKey, exact: true }),
  ]);
  client.setQueryData<PartyView>(rosterKey, (current) =>
    current && current.revision > saved.revision ? current : saved,
  );
  client.setQueryData<PartyDirectoryView>(
    queryKey,
    (index) =>
      index && {
        ...index,
        parties: index.parties.map((party) =>
          party.party_key === saved.party_key && party.revision <= saved.revision
            ? {
                party_key: saved.party_key,
                name: saved.name,
                revision: saved.revision,
                member_count: saved.members.length,
              }
            : party,
        ),
      },
  );
  await client.invalidateQueries({ queryKey: ["parties", "rosters"] });
}
