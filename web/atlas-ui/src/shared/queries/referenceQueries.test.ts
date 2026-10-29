import { QueryClient, type QueryKey, type QueryMeta } from "@tanstack/react-query";
import { AtlasApiError } from "../../api/atlasApi";
import { conditionLifecycleQuery } from "../../features/encounters/conditionLifecycleQuery";
import { persistentReviewQuery } from "../../features/encounters/usePersistentReview";
import { recordDetailQueryOptions } from "../records/recordDetailQuery";
import {
  similarityPartsQueryOptions,
  similarityTraitSourceQueryOptions,
  traitSimilarPreviewQueryOptions,
} from "../search/similarityQueries";
import { similarityPartsFixture } from "../../test/similarityFixtures";
import {
  isArtifactGenerationChanged,
  retireReferenceQueries,
} from "./referenceQueries";

function client() {
  return new QueryClient({ defaultOptions: { queries: { retry: false } } });
}
function cache(client: QueryClient, options: { queryKey: QueryKey; meta?: QueryMeta }) {
  client.getQueryCache().build(client, options);
  client.setQueryData(options.queryKey, { cached: true });
  return options.queryKey;
}

describe("reference query retirement", () => {
  it("retires production raw, inventory, trait and contextual query definitions at unknown and concrete generations", async () => {
    const queryClient = client();
    const seed = { kind: "record" as const, record_key: "hazards:brown-mold" };
    const inventories = [undefined, null, "generation-a"].map((generation) =>
      cache(queryClient, similarityPartsQueryOptions(seed, "bge-base", generation)),
    );
    const traitSource = cache(
      queryClient,
      similarityTraitSourceQueryOptions("trait:fire", null),
    );
    const traitSimilar = cache(
      queryClient,
      traitSimilarPreviewQueryOptions({
        criteria: {
          kind: "similar_traits",
          seed_trait_key: "trait:fire",
          filter: null,
          similarity: { method: "baseline" },
        },
        search_profile: "bge-base",
        expected_artifact_generation: "generation-a",
        page: { number: 1, size: 5 },
      }),
    );
    const raw = cache(queryClient, recordDetailQueryOptions("spells:heal"));
    const contextual = cache(
      queryClient,
      conditionLifecycleQuery("encounter-a", "participant-a", 4n, null),
    );
    const persistent = cache(
      queryClient,
      persistentReviewQuery("encounter-a", "participant-a", 4n),
    );
    const savedKeys = [
      ["saved-list", "list-a"],
      ["encounter", "encounter-a"],
      ["encounter-turn-review", "encounter-a", "4"],
      ["condition-consequences", "encounter-a", "4"],
    ];
    for (const key of savedKeys) queryClient.setQueryData(key, { saved: true });
    await retireReferenceQueries(queryClient);
    for (const key of [
      ...inventories,
      traitSource,
      traitSimilar,
      raw,
      contextual,
      persistent,
    ])
      expect(queryClient.getQueryState(key)).toBeUndefined();
    for (const key of savedKeys)
      expect(queryClient.getQueryData(key)).toEqual({ saved: true });
    queryClient.clear();
  });

  it("cancels an imperative production inventory read before removal and fences its late completion", async () => {
    const queryClient = client();
    const options = similarityPartsQueryOptions(
      { kind: "record", record_key: "hazards:brown-mold" },
      "bge-base",
      null,
    );
    let finish!: (value: ReturnType<typeof similarityPartsFixture>) => void;
    let signal!: AbortSignal;
    const pending = queryClient
      .fetchQuery({
        ...options,
        queryFn: (context) => {
          signal = context.signal;
          return new Promise<ReturnType<typeof similarityPartsFixture>>((resolve) => {
            finish = resolve;
          });
        },
      })
      .catch(() => undefined);
    const removed = vi.fn();
    const unsubscribe = queryClient.getQueryCache().subscribe((event) => {
      if (event.type === "removed") {
        expect(signal.aborted).toBe(true);
        removed();
      }
    });
    await retireReferenceQueries(queryClient);
    expect(removed).toHaveBeenCalledOnce();
    finish(similarityPartsFixture("generation-a"));
    await pending;
    expect(queryClient.getQueryState(options.queryKey)).toBeUndefined();
    unsubscribe();
    queryClient.clear();
  });

  it("keeps reference queries created by a new session while an earlier retirement awaits cancellation", async () => {
    const queryClient = client();
    const seed = { kind: "record" as const, record_key: "hazards:brown-mold" };
    const retired = cache(
      queryClient,
      similarityPartsQueryOptions(seed, "bge-base", null),
    );
    let release!: () => void;
    const cancellation = new Promise<void>((resolve) => {
      release = resolve;
    });
    const cancel = queryClient.cancelQueries.bind(queryClient);
    vi.spyOn(queryClient, "cancelQueries").mockImplementationOnce(async (filters) => {
      await cancel(filters);
      await cancellation;
    });
    const retirement = retireReferenceQueries(queryClient);
    const reopened = cache(
      queryClient,
      similarityPartsQueryOptions(seed, "bge-base", null, 1),
    );
    release();
    await retirement;
    expect(queryClient.getQueryState(retired)).toBeUndefined();
    expect(queryClient.getQueryData(reopened)).toEqual({ cached: true });
    queryClient.clear();
  });

  it("requires the typed generation error rather than treating every conflict as refreshable", () => {
    expect(
      isArtifactGenerationChanged(
        new AtlasApiError(409, "changed", {
          code: "artifact_generation_changed",
          message: "changed",
        }),
      ),
    ).toBe(true);
    expect(
      isArtifactGenerationChanged(
        new AtlasApiError(409, "stale participant", {
          code: "encounter_participant_conflict",
          message: "stale participant",
        }),
      ),
    ).toBe(false);
    expect(isArtifactGenerationChanged(new Error("artifact_generation_changed"))).toBe(
      false,
    );
  });
});
