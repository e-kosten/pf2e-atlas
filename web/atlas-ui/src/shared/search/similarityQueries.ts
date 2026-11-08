import { queryOptions } from "@tanstack/react-query";
import { getSimilarityParts } from "../../api/atlasApi";
import type { SimilaritySeed } from "../../generated/atlas";
import { referenceQueryMeta } from "../queries/referenceQueries";
import { assertSimilarityInventoryIdentity } from "./searchIdentity";

export function similarityPartsQueryOptions(
  seed: SimilaritySeed | null,
  profile: string | null | undefined,
  generation: string | null | undefined,
  reload = 0,
) {
  return queryOptions({
    queryKey: ["similarity-parts", seed, profile, generation, reload],
    meta: referenceQueryMeta,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const inventory = await getSimilarityParts(
        {
          seed: seed!,
          search_profile: profile ?? undefined,
          expected_artifact_generation: generation ?? null,
        },
        signal,
      );
      assertSimilarityInventoryIdentity(inventory, seed!, profile!, generation);
      return inventory;
    },
  });
}
