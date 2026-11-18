import { useQuery, type QueryClient } from "@tanstack/react-query";
import { getCharacter, getCharacters } from "../../api/atlasApi";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";

export function useCharacters(enabled = true) {
  return useQuery({
    queryKey: ["characters"],
    queryFn: ({ signal }) => getCharacters(signal),
    enabled,
  });
}

export function useCharacter(characterKey: string) {
  return useQuery({
    queryKey: ["character", characterKey],
    queryFn: ({ signal }) => getCharacter(characterKey, signal),
    meta: referenceQueryMeta,
  });
}

export async function refreshCharacters(client: QueryClient) {
  await Promise.all(
    [
      "characters",
      "character",
      "parties",
      "pc-reference",
      "encounters",
      "encounter",
    ].map((key) => client.invalidateQueries({ queryKey: [key] })),
  );
}
