import { useEffect, useEffectEvent } from "react";
import {
  type QueryClient,
  queryOptions,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import {
  AtlasApiError,
  getSimilarityParts,
  getSimilaritySource,
} from "../../api/atlasApi";
import { sameSimilaritySeed } from "./similaritySource";
import type { SimilaritySeed, SimilaritySourceView } from "../../generated/atlas";
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

type KnownSource = { title: string; generation: string; parentTitle?: string };

function sourceKey(seed: SimilaritySeed | null, generation?: string | null) {
  return ["similarity-source", seed, generation ?? null] as const;
}

function knownView(seed: SimilaritySeed, known: KnownSource): SimilaritySourceView {
  return {
    source: {
      seed,
      title: known.title,
      ...(known.parentTitle && seed.embedded_path?.length
        ? { parent_title: known.parentTitle }
        : {}),
    },
    artifact_generation: known.generation,
  };
}

function rememberSource(client: QueryClient, view: SimilaritySourceView) {
  // Both a fresh Search and one bound to this generation can reuse the reader's identity.
  for (const generation of [null, view.artifact_generation]) {
    const queryKey = sourceKey(view.source.seed, generation);
    client.getQueryCache().build(client, { queryKey, meta: referenceQueryMeta });
    client.setQueryData(queryKey, view);
  }
}

export function useRememberSimilaritySource(
  seed: SimilaritySeed | null,
  known?: KnownSource,
) {
  const client = useQueryClient();
  const remember = useEffectEvent(() => {
    if (seed && known) rememberSource(client, knownView(seed, known));
  });
  const address = JSON.stringify(seed);
  useEffect(() => {
    remember();
  }, [client, address, known?.title, known?.parentTitle, known?.generation]);
}

export function useSimilaritySource(
  seed: SimilaritySeed | null,
  generation?: string | null,
) {
  const client = useQueryClient();
  return useQuery({
    queryKey: sourceKey(seed, generation),
    enabled: Boolean(seed),
    meta: referenceQueryMeta,
    staleTime: Infinity,
    initialData: () => {
      const known = client.getQueryData<SimilaritySourceView>(sourceKey(seed, null));
      return generation && known?.artifact_generation === generation
        ? known
        : undefined;
    },
    queryFn: async ({ signal }) => {
      const result = await getSimilaritySource(
        { seed: seed!, expected_artifact_generation: generation ?? null },
        signal,
      );
      if (!sameSimilaritySeed(result.source.seed, seed))
        throw new Error("The comparison title does not match this source.");
      if (generation && result.artifact_generation !== generation)
        throw new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        });
      return result;
    },
  });
}
