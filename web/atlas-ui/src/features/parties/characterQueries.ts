import {
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
  type QueryKey,
} from "@tanstack/react-query";
import { getCharacter, getCharacters, savePcAbilityLink } from "../../api/atlasApi";
import type { PcReferenceView } from "../../generated/atlas";
import type { AbilityLinkChange } from "./PcAbilityBrowser";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";

export function useCharacters(enabled = true) {
  return useQuery({
    queryKey: ["characters"],
    queryFn: ({ signal }) => getCharacters(signal),
    enabled,
  });
}

export const characterQueryKey = (characterKey: string) =>
  ["character", characterKey] as const;

export function useCharacter(characterKey: string) {
  return useQuery({
    queryKey: characterQueryKey(characterKey),
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

export function useSavedAbilityLink(
  character: PcReferenceView | null | undefined,
  queryKey: QueryKey,
) {
  const client = useQueryClient();
  return useMutation({
    retry: false,
    mutationFn: ({ abilityKey, selection }: AbilityLinkChange) => {
      if (!character) throw new Error("Character is unavailable.");
      return savePcAbilityLink(character.character_key, abilityKey, {
        expected_character_revision: character.character_revision,
        selection,
      });
    },
    onSuccess: async (saved) => {
      client.setQueryData(queryKey, saved);
      await refreshCharacters(client);
    },
    onError: () => void client.invalidateQueries({ queryKey, exact: true }),
  });
}
