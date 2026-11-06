import { queryOptions } from "@tanstack/react-query";
import {
  AtlasApiError,
  getSimilarityParts,
  getTraitDetail,
  searchPage,
} from "../../api/atlasApi";
import type { SearchResultsRequest, SimilaritySeed } from "../../generated/atlas";
import { referenceQueryMeta } from "../queries/referenceQueries";
import {
  assertSearchIdentity,
  assertSimilarityIdentity,
  assertSimilarityInventoryIdentity,
} from "./searchIdentity";

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

export function similarityTraitSourceQueryOptions(
  traitKey: string,
  generation: string | null,
) {
  return queryOptions({
    queryKey: ["similarity-trait-source", traitKey, generation],
    meta: referenceQueryMeta,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const detail = await getTraitDetail(
        {
          trait_key: traitKey,
          expected_artifact_generation: generation ?? null,
        },
        signal,
      );
      if (generation && detail.artifact_generation !== generation)
        throw new AtlasApiError(409, "The reference collection changed.", {
          code: "artifact_generation_changed",
          message: "The reference collection changed.",
          recoverable_action: "refresh_results",
        });
      return detail;
    },
  });
}

export function traitSimilarPreviewQueryOptions(
  request: SearchResultsRequest & {
    criteria: Extract<SearchResultsRequest["criteria"], { kind: "similar_traits" }>;
  },
) {
  return queryOptions({
    queryKey: ["trait-similar-preview", request],
    meta: referenceQueryMeta,
    staleTime: Infinity,
    queryFn: async ({ signal }) => {
      const page = await searchPage(request, signal);
      assertSearchIdentity(
        page,
        request.search_profile,
        request.expected_artifact_generation,
        request.sort,
      );
      assertSimilarityIdentity(
        page,
        request.criteria.similarity,
        request.criteria.source_sections,
        request.criteria.search_content,
      );
      return page;
    },
  });
}
