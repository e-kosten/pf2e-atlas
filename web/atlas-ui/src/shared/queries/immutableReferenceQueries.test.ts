import { QueryClient } from "@tanstack/react-query";
import { AtlasApiError } from "../../api/atlasApi";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "./immutableReferenceQueries";

describe("immutable reference retirement", () => {
  it("cancels old reads before removing reference data and retains mutable aggregates", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const immutable = [
      "results",
      "search-picker",
      "filter-discovery",
      "saved-list-filter-discovery",
      "filter-vocabulary",
      "record-detail",
      "embedded-reading",
      "related",
      "record-reference-preview",
      "similar-preview",
      "encounter-record-detail",
    ];
    const mutable = [
      "saved-list",
      "saved-lists",
      "encounter",
      "encounters",
      "encounter-condition-definitions",
      "readiness",
    ];
    for (const family of [...immutable, ...mutable])
      client.setQueryData([family, "sample"], { name: family });
    let finish!: (value: string) => void;
    let signal!: AbortSignal;
    const old = client
      .fetchQuery({
        queryKey: ["record-detail", "late"],
        queryFn: (context) => {
          signal = context.signal;
          return new Promise<string>((resolve) => {
            finish = resolve;
          });
        },
      })
      .catch(() => undefined);
    await retireImmutableReferenceQueries(client);
    expect(signal.aborted).toBe(true);
    for (const family of immutable)
      expect(client.getQueryData([family, "sample"])).toBeUndefined();
    for (const family of mutable)
      expect(client.getQueryData([family, "sample"])).toEqual({ name: family });
    finish("old detail");
    await old;
    expect(client.getQueryData(["record-detail", "late"])).toBeUndefined();
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
