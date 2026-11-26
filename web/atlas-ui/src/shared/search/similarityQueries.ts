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
import { appError } from "../feedback/errorMessages";
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

function cacheSource(
  client: QueryClient,
  view: SimilaritySourceView,
  generation: string | null,
) {
  const queryKey = sourceKey(view.source.seed, generation);
  client.getQueryCache().build(client, { queryKey, meta: referenceQueryMeta });
  client.setQueryData(queryKey, view);
}

export function rememberRankedSimilaritySource(
  client: QueryClient,
  view: SimilaritySourceView,
  lookupGeneration: string | null,
) {
  const lookup = client.getQueryState(sourceKey(view.source.seed, lookupGeneration));
  if (!lookup?.error) return;
  const key = sourceKey(view.source.seed, view.artifact_generation);
  const pinned = client.getQueryState(key);
  if (pinned?.status === "success") return;
  for (const state of [lookup, pinned]) {
    const code = appError(state?.error)?.code;
    if (
      code === "record_not_found" ||
      code === "similarity_source_stale" ||
      code === "artifact_generation_changed"
    )
      return;
  }
  // A successful ranking acknowledges this generation only. Keep the unpinned
  // lookup independent so refresh can resolve the title in the next collection.
  cacheSource(client, view, view.artifact_generation);
}

export function useRememberSimilaritySource(
  seed: SimilaritySeed | null,
  known?: KnownSource,
) {
  const client = useQueryClient();
  const remember = useEffectEvent(() => {
    if (seed && known) {
      const view = knownView(seed, known);
      // Both a fresh Search and one bound to this generation can reuse the reader's identity.
      cacheSource(client, view, null);
      cacheSource(client, view, view.artifact_generation);
    }
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
